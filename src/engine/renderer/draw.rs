use super::gpu::{Frame, Renderer};
use super::mesh::{SolidRect, build_solid_rect_mesh, build_text_mesh, model_matrix};
use crate::engine::entity;
use crate::engine::scene::Scene;
use wgpu::util::DeviceExt;

impl Renderer {
    pub fn render_scene(&mut self, frame: &Frame, scene: &Scene, show_colliders: bool) {
        let projection = self.screen_projection();
        let screen_center = self.screen_size() * 0.5;

        // camera_position is set per-frame by the caller based on scene.camera_mode (ADR-041).
        let camera_view =
            glam::Mat4::from_translation((screen_center - self.camera_position).extend(0.0));

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

        let mut debug_rects: Vec<SolidRect> = Vec::new();
        if show_colliders {
            const WALL_FILL: [f32; 4] = [1.0, 0.0, 0.0, 0.15];
            const WALL_BORDER: [f32; 4] = [0.7, 0.0, 0.0, 0.9];
            const ENTITY_FILL: [f32; 4] = [0.0, 0.4, 1.0, 0.15];
            const ENTITY_BORDER: [f32; 4] = [0.0, 0.2, 0.8, 0.9];
            const TRIGGER_FILL: [f32; 4] = [0.0, 1.0, 0.0, 0.15];
            const TRIGGER_BORDER: [f32; 4] = [0.0, 0.7, 0.0, 0.9];
            const DIALOGUE_FILL: [f32; 4] = [1.0, 1.0, 0.0, 0.15];
            const DIALOGUE_BORDER: [f32; 4] = [0.7, 0.7, 0.0, 0.9];
            const INTERACT_FILL: [f32; 4] = [1.0, 0.0, 1.0, 0.15];
            const INTERACT_BORDER: [f32; 4] = [0.7, 0.0, 0.7, 0.9];

            push_center_marker(&mut debug_rects, glam::Vec2::ZERO, 1.0);

            for wall in &scene.walls {
                debug_rects.push(SolidRect {
                    position: wall.rect.center,
                    size: wall.rect.half_size * 2.0,
                    fill_color: WALL_FILL,
                    border_color: WALL_BORDER,
                    border_thickness_px: 3.0,
                });
                push_center_marker(&mut debug_rects, wall.rect.center, 1.0);
            }
            for entity in &scene.entities {
                if let Some(collider) = &entity.collider {
                    debug_rects.push(SolidRect {
                        position: entity.position + collider.rect.center,
                        size: collider.rect.half_size * 2.0,
                        fill_color: ENTITY_FILL,
                        border_color: ENTITY_BORDER,
                        border_thickness_px: 3.0,
                    });
                    push_center_marker(
                        &mut debug_rects,
                        entity.position + collider.rect.center,
                        1.0,
                    );
                }
                if entity.texture_id.is_some() {
                    push_facing_marker(&mut debug_rects, entity.position, entity.facing, 1.0);
                }
            }
            for trigger in &scene.triggers {
                if !trigger.active {
                    continue;
                }
                let (fill_color, border_color) = match trigger.kind {
                    TriggerKind::Warp { .. } => (TRIGGER_FILL, TRIGGER_BORDER),
                    TriggerKind::Dialogue { .. } => (DIALOGUE_FILL, DIALOGUE_BORDER),
                    TriggerKind::Toggle { .. } => (INTERACT_FILL, INTERACT_BORDER),
                };

                debug_rects.push(SolidRect {
                    position: trigger.rect.center,
                    size: trigger.rect.half_size * 2.0,
                    fill_color,
                    border_color,
                    border_thickness_px: 3.0,
                });
                push_center_marker(&mut debug_rects, trigger.rect.center, 1.0);
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
            let model = model_matrix(*position, draw_size);
            let transform = projection * camera_view * model;
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
                        r: 0.0,
                        g: 1.0,
                        b: 1.0,
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

        if !debug_rects.is_empty() {
            self.render_solid_rects(frame, &debug_rects, projection, camera_view);
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
        let (vertices, indices) = build_solid_rect_mesh(rects);
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

    pub fn render_dialogue_panel(
        &mut self,
        frame: &Frame,
        register: Option<&crate::game::dialogue::Register>,
    ) {
        const MARGIN: f32 = 20.0;
        const NARRATOR_BORDER: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // Neutral gray border for narrator
        const MONOLOGUE_BORDER: [f32; 4] = [0.5, 0.5, 1.0, 1.0]; // Blue border for inner monologue

        let border_color = match register {
            Some(crate::game::dialogue::Register::InnerMonologue) => MONOLOGUE_BORDER,
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

    pub fn render_text(&mut self, frame: &Frame, glyphs: &[crate::engine::text::PositionedGlyph]) {
        // Screen-space only — no camera_view term, so text stays fixed
        // to the window regardless of camera position or mode (HUD).
        if glyphs.is_empty() {
            return;
        }

        let (vertices, indices) = build_text_mesh(glyphs);
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

    pub fn render_text_with_bg(
        &mut self,
        frame: &Frame,
        glyphs: &[crate::engine::text::PositionedGlyph],
    ) {
        self.render_text_bg(
            frame,
            glyphs,
            [0.0, 0.0, 0.0, 0.85],
            None,
            0.0,
            crate::engine::text::DEBUG_TEXT_PADDING,
        );
        self.render_text(frame, glyphs);
    }

    pub fn render_text_bg(
        &mut self,
        frame: &Frame,
        glyphs: &[crate::engine::text::PositionedGlyph],
        fill_color: [f32; 4],
        border_color: Option<[f32; 4]>,
        border_thickness_px: f32,
        padding: f32,
    ) {
        let (position, size) = crate::engine::text::combined_glyph_info(glyphs, padding);

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
