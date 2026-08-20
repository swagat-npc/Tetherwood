use crate::engine::app::AppState;
use crate::engine::renderer::{Frame, text};
use glam::Vec2;

impl AppState {
    pub fn draw_ui(&mut self, frame: &Frame) {
        // HUD::Display Dialogue Panel
        if let Some(dialogue) = &self.dialogue {
            self.renderer
                .render_dialogue_panel(frame, dialogue.current_register());
            let text_pos = self.renderer.dialogue_text_position();
            let max_width = self.renderer.dialogue_text_max_width();

            let colored_chars: Vec<(char, [f32; 4])> = dialogue
                .visible_chars()
                .into_iter()
                .map(|rc| (rc.ch, rc.color))
                .collect();

            let wrapped_lines =
                text::wrap_colored_text(&colored_chars, max_width, text::DIALOGUE_TEXT_SCALE);

            let line_height = text::GLYPH_SIZE.y * text::DIALOGUE_TEXT_SCALE + 4.0;
            for (i, line_chars) in wrapped_lines.iter().enumerate() {
                let line_origin = text_pos + Vec2::new(0.0, i as f32 * line_height);
                let glyphs = text::layout_colored_text_scaled(
                    line_chars,
                    line_origin,
                    text::DIALOGUE_TEXT_SCALE,
                );
                self.renderer.render_text(frame, &glyphs);
            }

            if dialogue.caret_visible() {
                let caret_pos = self.renderer.dialogue_caret_position();
                let caret_glyphs = text::layout_colored_text_scaled(
                    &[('▼', [1.0, 1.0, 1.0, 1.0])],
                    caret_pos,
                    text::DIALOGUE_TEXT_SCALE + 2.0,
                );
                self.renderer.render_text(frame, &caret_glyphs);
            }
        }
    }
}
