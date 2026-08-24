use glam::Vec2;

use crate::engine::app::AppState;
use crate::engine::debug;
use crate::engine::debug::inspector::{Inspector, SectionWidget};
use crate::engine::renderer::Frame;

impl AppState {
    pub fn update_debug_ui(&mut self) {
        let mouse_pressed_this_frame = self.left_mouse_pressed;
        self.left_mouse_pressed = false;

        let world_mouse = Vec2::new(
            self.screen_mouse_position.0 as f32,
            self.screen_mouse_position.1 as f32,
        );
        if !(self.debug.show_debug_renderer && self.inspector.is_settled()) {
            return;
        }

        // Copy the position out BEFORE taking a mutable borrow of
        // inspector.state below - Vec2 is Copy, so this is a cheap
        // snapshot, not aliasing.
        let inspector_position = self.inspector.position;

        let Some(inspector_state) = &mut self.inspector.state else {
            return;
        };

        for section in inspector_state.sections.iter_mut() {
            let bounds = Inspector::resolve_section_bounds(inspector_position, section);
            let section_top_left = bounds.center - bounds.half_size;
            let content_width = bounds.half_size.x * 2.0;

            for widget in section.widgets.iter_mut() {
                match widget {
                    SectionWidget::Slider(slider) => {
                        if slider.update(world_mouse, self.left_mouse_down) {
                            self.blip_volume = slider.value;
                        }
                    }
                    SectionWidget::TilePalette(palette) => {
                        if mouse_pressed_this_frame {
                            palette.handle_click(world_mouse, section_top_left, content_width);
                        }
                    }
                }
            }
        }
    }

    pub fn draw_debug_info(&mut self, frame: &Frame) {
        // DEBUG::Notifications Text
        debug::info::draw_notifications(&mut self.renderer, frame, &mut self.notifications);

        if self.debug.show_debug_renderer {
            self.inspector
                .draw(&mut self.renderer, frame, self.is_isometric);

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
