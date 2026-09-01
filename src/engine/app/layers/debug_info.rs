use crate::engine::app::AppState;
use crate::engine::debug::inspector::{self, Inspector, PaintMode, SectionWidget, TilesetAction};
use crate::engine::renderer::{Frame, tile};
use crate::engine::{debug, entity};
use glam::Vec2;

impl AppState {
    pub fn update_debug_ui(&mut self) {
        let mouse_pressed_this_frame = self.left_mouse_pressed;
        self.left_mouse_pressed = false;

        let screen_mouse = Vec2::new(
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
        let over_panel = entity::point_in_rect(screen_mouse, &self.inspector.bounds());
        let is_paint_active = self.debug.show_tile_editor;
        let mut tileset_action = TilesetAction::None;

        {
            let Some(inspector_state) = &mut self.inspector.state else {
                return;
            };
            for section in inspector_state.sections.iter_mut() {
                let bounds = Inspector::resolve_section_bounds(inspector_position, section);
                let section_top_left = bounds.center - bounds.half_size;
                let content_width = bounds.half_size.x * 2.0;
                let slots = inspector::stack_widgets(
                    &section.widgets,
                    section_top_left,
                    content_width,
                    &self.renderer.ttf_glyphs,
                );
                // NOTE: stack_widgets is private in inspector.rs today - either
                // make it pub(crate) or add a small pub wrapper; see note below.

                for (widget, (widget_top_left, _height)) in
                    section.widgets.iter_mut().zip(slots.iter())
                {
                    match widget {
                        SectionWidget::Slider(slider) => {
                            if slider.update(screen_mouse, self.left_mouse_down) {
                                self.blip_volume = slider.value;
                            }
                        }
                        SectionWidget::TilePalette(palette) => {
                            if mouse_pressed_this_frame {
                                palette.handle_click(screen_mouse, *widget_top_left, content_width);
                            }
                        }
                        SectionWidget::TilesetControls(controls) => {
                            if mouse_pressed_this_frame {
                                let action = controls.handle_click(
                                    screen_mouse,
                                    *widget_top_left,
                                    is_paint_active,
                                );
                                if !matches!(action, TilesetAction::None) {
                                    tileset_action = action;
                                }
                            }
                        }
                        SectionWidget::HotkeyList(_) => {}
                    }
                }
            }
        }

        match tileset_action {
            TilesetAction::SetMode(mode) => self.paint_mode = mode,
            TilesetAction::Save => self.save_paint_session(),
            TilesetAction::Clear => self.reset_paint_session(),
            TilesetAction::None => {}
        }

        // World-click paint/erase: only when paint mode is active, the
        // panel is settled (not mid-animation), the click isn't meant
        // for the UI, and this is a genuine press (not a held button).
        if is_paint_active && self.inspector.is_settled() && !over_panel && mouse_pressed_this_frame
        {
            let world_pos = self
                .renderer
                .screen_to_world(screen_mouse, self.is_isometric);
            let cell = tile::cell_at_position(world_pos, self.multiplying_factor);

            if let Some(session) = &mut self.paint_session {
                match self.paint_mode {
                    PaintMode::Place => {
                        if let Some(selected) = self.inspector.selected_tile() {
                            session.set(cell, selected);
                        }
                    }
                    PaintMode::Remove => session.remove(cell),
                }
            }
        }
    }

    pub fn draw_debug_info(&mut self, frame: &Frame) {
        // DEBUG::Notifications Text
        debug::info::draw_notifications(&mut self.renderer, frame, &mut self.notifications);

        if self.debug.show_debug_renderer {
            self.inspector.draw(
                &mut self.renderer,
                frame,
                self.is_isometric,
                &self.paint_mode,
                self.debug.show_tile_editor,
            );

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

                // DEBUG::Player Position
                debug::info::draw_player_position(
                    &mut self.renderer,
                    frame,
                    self.multiplying_factor,
                    &self.scene.entities[self.scene.player_index],
                );
            }
        }
    }
}
