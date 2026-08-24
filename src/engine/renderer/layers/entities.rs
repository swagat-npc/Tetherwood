use crate::engine::entity;
use crate::engine::renderer::{Frame, Renderer, mesh};
use crate::engine::scene::Scene;

impl Renderer {
    pub fn draw_background_and_entities(
        &mut self,
        frame: &Frame,
        scene: &Scene,
        is_isometric: bool,
    ) {
        let projection = self.world_projection();
        let sprite_camera_view = self.sprite_camera_view(is_isometric);

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

        // Main draw list: background, then every non-overlay entity in
        // sorted order. Overlay entities (prompt icons, etc.) are excluded
        // here - drawn in a second pass below, always after, so they can
        // never be occluded by a normal entity regardless of y-sort.
        let mut draws: Vec<(usize, glam::Vec2, glam::Vec2, entity::Direction)> = Vec::new();
        for bg in &scene.background {
            draws.push((bg.texture.0, bg.position, bg.size, entity::Direction::Down));
        }
        for &idx in &order {
            let entity = &scene.entities[idx];
            if entity.is_overlay_layer {
                continue;
            }
            if let Some(texture_id) = entity.texture_id {
                draws.push((texture_id.0, entity.position, entity.size, entity.facing));
            }
        }

        // Second pass: overlay entities. Copy-pasted from the loop above,
        // deliberately, for now - always LoadOp::Load (something already
        // cleared above, even if `draws` was somehow empty this frame -
        // background always pushes at least one entry). Extracting the
        // shared submission logic is a planned follow-up, not done yet.
        let mut overlay_draws: Vec<(usize, glam::Vec2, glam::Vec2, entity::Direction)> = Vec::new();
        for &idx in &order {
            let entity = &scene.entities[idx];
            if !entity.is_overlay_layer {
                continue;
            }
            if let Some(texture_id) = entity.texture_id {
                overlay_draws.push((texture_id.0, entity.position, entity.size, entity.facing));
            }
        }

        for (bind_group_index, position, size, facing) in draws.iter() {
            let mut draw_size = *size;
            if *facing == entity::Direction::Left {
                draw_size.x = -draw_size.x;
            }
            let effective_position = if is_isometric {
                self.shear(*position)
            } else {
                *position
            };
            self.submit_sprite_draw(
                frame,
                projection,
                sprite_camera_view,
                *bind_group_index,
                effective_position,
                draw_size,
            );
        }

        for (bind_group_index, position, size, facing) in overlay_draws.iter() {
            let mut draw_size = *size;
            if *facing == entity::Direction::Left {
                draw_size.x = -draw_size.x;
            }
            let effective_position = if is_isometric {
                self.shear(*position)
            } else {
                *position
            };
            self.submit_sprite_draw(
                frame,
                projection,
                sprite_camera_view,
                *bind_group_index,
                effective_position,
                draw_size,
            );
        }
    }

    /// Submits one sprite draw: writes its transform, then renders it in
    /// its own encoder/submit (see the note on why each draw gets its own
    /// submission - shared write_buffer + deferred submit would let every
    /// draw see only the last-written transform). `is_first_draw_this_frame`
    /// controls Clear vs. Load - only the very first draw of the whole
    /// frame should clear; everything after, in either pass, loads.
    fn submit_sprite_draw(
        &mut self,
        frame: &Frame,
        projection: glam::Mat4,
        sprite_camera_view: glam::Mat4,
        bind_group_index: usize,
        position: glam::Vec2,
        size: glam::Vec2,
    ) {
        let model = mesh::model_matrix(position, size);
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("draw pass"),
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_groups[bind_group_index], &[]);
            render_pass.set_bind_group(1, &self.transform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
