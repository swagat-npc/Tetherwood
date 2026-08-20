use super::gpu::{Frame, Renderer};
use super::mesh::{self, SolidRect};
use super::text;
use crate::game::dialogue::Register;
use wgpu::util::DeviceExt;

impl Renderer {
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
