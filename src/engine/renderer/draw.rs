use super::gpu::{Frame, Renderer};
use super::mesh::{self, SolidRect};
use super::text;
use crate::engine::debug::{DebugSettings, overlay};
use crate::engine::entity;
use crate::engine::scene::Scene;
use crate::game::dialogue::Register;
use wgpu::util::DeviceExt;

impl Renderer {
    pub fn render_scene(
        &mut self,
        frame: &Frame,
        scene: &Scene,
        debug: &DebugSettings,
        is_isometric: bool,
        multiplying_factor: f32,
    ) {
        self.draw_background_and_entities(frame, scene, is_isometric);

        let projection = self.screen_projection();
        let iso_projection = self.isometric_projection();
        let screen_center = self.screen_size() * 0.5;
        let shear = |p: glam::Vec2| iso_projection.transform_point3(p.extend(0.0)).truncate();

        let mut debug_rects = Vec::new();
        if debug.show_colliders {
            debug_rects.extend(overlay::build_debug_rects(scene));
        }

        if debug.show_debug_renderer && debug.show_grid {
            let half_screen = self.screen_size() * 0.5;
            let visible_min = self.camera_position - half_screen;
            let visible_max = self.camera_position + half_screen;

            debug_rects.extend(mesh::build_grid_lines_mesh(
                scene,
                visible_min,
                visible_max,
                debug.grid_display_cell_size * multiplying_factor,
            ));
            if debug.show_occupied_cells {
                debug_rects.extend(mesh::build_occupied_cells_mesh(scene));
            }
            if debug.show_player_neighbours {
                debug_rects.extend(mesh::build_player_neighborhood_mesh(scene));
            }
        }

        // Debug overlay is procedural geometry, not art. A grid cell IS a
        // flat world-space square, and it should genuinely look like a
        // diamond from this angle. So debug rects get the full shear baked
        // into their view matrix, deforming the whole shape.
        let debug_view = if is_isometric {
            glam::Mat4::from_translation((screen_center - shear(self.camera_position)).extend(0.0))
                * iso_projection
        } else {
            glam::Mat4::from_translation((screen_center - self.camera_position).extend(0.0))
        };

        if !debug_rects.is_empty() {
            self.render_solid_rects(frame, &debug_rects, projection, debug_view);
        }
    }

    pub fn draw_background_and_entities(
        &mut self,
        frame: &Frame,
        scene: &Scene,
        is_isometric: bool,
    ) {
        let projection = self.screen_projection();
        let iso_projection = self.isometric_projection();
        let screen_center = self.screen_size() * 0.5;

        // Shears a single world-space point into iso space - used for
        // camera offset and, per-entity, for anchor position. Never
        // applied to a whole shape here; that's the debug_view's job below.
        let shear = |p: glam::Vec2| iso_projection.transform_point3(p.extend(0.0)).truncate();

        // Camera_position is set per-frame by the caller based on scene.camera_mode (ADR-041).
        // For isometric mode, camera_position is sheared first so it lands in the same space
        // as each entity's sheared anchor - shape is never touched by this matrix.
        let sprite_camera_view = if is_isometric {
            glam::Mat4::from_translation((screen_center - shear(self.camera_position)).extend(0.0))
        } else {
            glam::Mat4::from_translation((screen_center - self.camera_position).extend(0.0))
        };

        // y-sort: entities drawn in ascending order of baseline
        // (bottom edge = position.y + half height), so entities with a
        // lower baseline draw first and correctly end up behind
        // entities with a higher one.
        let mut order: Vec<usize> = (0..scene.entities.len()).collect();
        order.sort_by(|&a, &b| {
            let baseline_a = scene.entities[a].position.y + scene.entities[a].size.y / 2.0;
            let baseline_b = scene.entities[b].position.y + scene.entities[b].size.y / 2.0;
            baseline_a.partial_cmp(&baseline_b).unwrap()
        });

        // Full draw list: background first (always behind, never
        // y-sorted), then entities in sorted order. Each entry is
        // (bind group index, position, size).
        let mut draws: Vec<(usize, glam::Vec2, glam::Vec2, entity::Direction)> = Vec::new();
        for bg in &scene.background {
            draws.push((bg.texture.0, bg.position, bg.size, entity::Direction::Down));
        }

        for &idx in &order {
            let entity = &scene.entities[idx];
            if let Some(texture_id) = entity.texture_id {
                draws.push((texture_id.0, entity.position, entity.size, entity.facing));
            }
        }

        // Each draw gets its own encoder and its own submit. See the
        // explanation above the code block in this message: repeatedly
        // calling write_buffer on the same buffer before a single
        // shared submit() would let every draw see only the *last*
        // written transform. Submitting per-draw guarantees each
        // write_buffer lands before its own draw executes. The first
        // draw clears the screen; every draw after it loads (paints
        // over) what's already there instead of erasing it.
        for (i, (bind_group_index, position, size, facing)) in draws.iter().enumerate() {
            let mut draw_size = *size;
            if *facing == entity::Direction::Left {
                draw_size.x = -draw_size.x;
            }
            // Shear the anchor only - isometric art is expected to already
            // look correct from that angle; this only decides placement.
            let effective_position = if is_isometric {
                shear(*position)
            } else {
                *position
            };
            let model = mesh::model_matrix(effective_position, draw_size);
            let transform = projection * sprite_camera_view * model;
            self.queue.write_buffer(
                &self.transform_buffer,
                0,
                bytemuck::cast_slice(&transform.to_cols_array()),
            );

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("draw encoder"),
                });

            {
                let load_op = if i == 0 {
                    wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.15,
                        g: 0.15,
                        b: 0.15,
                        a: 1.0,
                    })
                } else {
                    wgpu::LoadOp::Load
                };

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("draw pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame.view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: load_op,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.bind_groups[*bind_group_index], &[]);
                render_pass.set_bind_group(1, &self.transform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    /// Draws a batch of solid-colored rects — used by both the F1 debug
    /// overlay (gated behind show_colliders) and permanent UI like the
    /// dialogue panel. Takes projection/view as parameters rather than
    /// assuming one, since callers need genuinely different transforms:
    /// debug rects are world-space (render_scene's projection*camera_view),
    /// the dialogue panel is screen-space (an identity view, matching
    /// render_text's ADR-058 convention).
    pub fn render_solid_rects(
        &mut self,
        frame: &Frame,
        rects: &[SolidRect],
        projection: glam::Mat4,
        view: glam::Mat4,
    ) {
        if rects.is_empty() {
            return;
        }
        let (vertices, indices) = mesh::build_solid_rect_mesh(rects);
        let solid_vertex_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("solid vertex buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let solid_index_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("solid index buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // Positions are already world-space (baked from each rect's real
        // center/size), so only projection*camera_view is needed here —
        // no per-rect model matrix, same simplification build_text_mesh
        // already applies for screen-space text.
        let transform = projection * view;
        self.queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(&transform.to_cols_array()),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("solid vertex encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("solid vertex pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.debug_pipeline);
            render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, solid_vertex_buffer.slice(..));
            render_pass.set_index_buffer(solid_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render_dialogue_panel(&mut self, frame: &Frame, register: Option<&Register>) {
        const MARGIN: f32 = 20.0;
        const NARRATOR_BORDER: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // Neutral gray border for narrator
        const MONOLOGUE_BORDER: [f32; 4] = [0.5, 0.5, 1.0, 1.0]; // Blue border for inner monologue

        let border_color = match register {
            Some(Register::InnerMonologue) => MONOLOGUE_BORDER,
            _ => NARRATOR_BORDER, // Narrator or not active lines
        };

        let screen = self.screen_size();
        let panel_height = screen.y / 3.0 - MARGIN;
        let panel = SolidRect {
            position: glam::Vec2::new(screen.x / 2.0, screen.y - panel_height / 2.0 - MARGIN),
            size: glam::Vec2::new(screen.x - MARGIN * 2.0, panel_height),
            fill_color: [0.0, 0.0, 0.0, 0.95], // near-opaque black, per your earlier "legible over noise" ask
            border_color,
            border_thickness_px: 10.0,
        };

        let projection = self.screen_projection();
        self.render_solid_rects(frame, &[panel], projection, glam::Mat4::IDENTITY);
    }

    pub fn render_text(&mut self, frame: &Frame, glyphs: &[text::PositionedGlyph]) {
        // Screen-space only — no camera_view term, so text stays fixed
        // to the window regardless of camera position or mode (HUD).
        if glyphs.is_empty() {
            return;
        }

        let (vertices, indices) = mesh::build_text_mesh(glyphs);
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("text vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("text index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        // Screen=space only (ADR-058) - no camera_view term, so text stays fixed
        // to the window regardless of camera position or mode (HUD).
        let projection = self.screen_projection();
        self.queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(&projection.to_cols_array()),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("text render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text draw pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // always paint over — scene already drew this frame
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.glyph_atlas_bind_group, &[]);
            render_pass.set_bind_group(1, &self.transform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render_text_with_bg(&mut self, frame: &Frame, glyphs: &[text::PositionedGlyph]) {
        self.render_text_bg(
            frame,
            glyphs,
            [0.0, 0.0, 0.0, 0.85],
            None,
            0.0,
            text::DEBUG_TEXT_PADDING,
        );
        self.render_text(frame, glyphs);
    }

    pub fn render_text_bg(
        &mut self,
        frame: &Frame,
        glyphs: &[text::PositionedGlyph],
        fill_color: [f32; 4],
        border_color: Option<[f32; 4]>,
        border_thickness_px: f32,
        padding: f32,
    ) {
        let (position, size) = text::combined_glyph_info(glyphs, padding);

        let bg = SolidRect {
            position,
            size,
            fill_color,
            border_color: border_color.unwrap_or(fill_color),
            border_thickness_px,
        };

        let projection = self.screen_projection();
        self.render_solid_rects(frame, &[bg], projection, glam::Mat4::IDENTITY);
    }
}
