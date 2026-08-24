use glam::Vec2;

use crate::engine::app::AppState;
use crate::engine::debug;
use crate::engine::renderer::Frame;

impl AppState {
    pub fn update_debug_ui(&mut self) {
        // INSPECTOR:: Volume slider
        let world_mouse = Vec2::new(
            self.screen_mouse_position.0 as f32,
            self.screen_mouse_position.1 as f32,
        );
        if self.debug.show_debug_renderer && self.inspector.is_settled() {
            if let Some(inspector_state) = &mut self.inspector.state {
                if inspector_state
                    .volume_slider
                    .update(world_mouse, self.left_mouse_down)
                {
                    self.blip_volume = inspector_state.volume_slider.value;
                }
            }
        }
    }

    pub fn draw_debug_info(&mut self, frame: &Frame) {
        // DEBUG::Notifications Text
        debug::info::draw_notifications(&mut self.renderer, frame, &mut self.notifications);

        if self.debug.show_debug_renderer {
            self.inspector.draw(&mut self.renderer, frame);

            if self.debug.show_debug_info {
                // DEBUG::FPS Counter
                debug::info::draw_fps_counter(&mut self.renderer, frame, self.smoothed_fps);

                // DEBUG::Mouse Position
                debug::info::draw_mouse_position(
                    &mut self.renderer,
                    frame,
                    self.screen_mouse_position,
                    self.multiplying_factor,
                    self.is_isometric,
                );
            }
        }
    }
}
