use glam::Mat4;

use crate::engine::debug::{DebugSettings, overlay};
use crate::engine::renderer::{Frame, Renderer, mesh};
use crate::engine::scene::Scene;

impl Renderer {
    pub fn draw_debug_geometry(
        &mut self,
        frame: &Frame,
        scene: &Scene,
        debug: &DebugSettings,
        is_isometric: bool,
        multiplying_factor: f32,
    ) {
        let projection = self.world_projection();
        let iso_projection = self.isometric_projection();

        let mut debug_rects = Vec::new();
        if debug.show_colliders {
            debug_rects.extend(overlay::build_debug_rects(scene));
        }
        if debug.show_debug_renderer && debug.show_grid {
            let half_screen = (self.screen_size() * 0.5) / self.zoom;
            let visible_min = self.smoothed_camera - half_screen;
            let visible_max = self.smoothed_camera + half_screen;
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
            Mat4::from_translation((-self.shear(self.smoothed_camera)).extend(0.0)) * iso_projection
        } else {
            Mat4::from_translation((-self.smoothed_camera).extend(0.0))
        };

        if !debug_rects.is_empty() {
            self.render_solid_rects(frame, &debug_rects, projection, debug_view);
        }
    }
}
