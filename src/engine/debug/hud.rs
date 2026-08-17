use super::notifications::Notification;
use crate::engine::renderer::{Frame, Renderer, SolidRect, text};
use glam::{Mat4, Vec2};

pub fn draw_notifications(
    renderer: &mut Renderer,
    frame: &Frame,
    notifications: &mut Vec<Notification>,
) {
    if !notifications.is_empty() {
        let screen_size = renderer.screen_size();
        let notification_height =
            text::GLYPH_SIZE.y * text::DEBUG_TEXT_SCALE + text::DEBUG_TEXT_PADDING * 2.0;
        let notification_gap = 10.0;
        for (i, notification) in notifications.iter().enumerate() {
            let origin = text::centered_text_origin(
                &notification.message,
                screen_size.x * 0.5,
                screen_size.y - 25.0 - i as f32 * (notification_height + notification_gap),
                text::DEBUG_TEXT_SCALE,
            );
            let glyphs =
                text::layout_text_scaled(&notification.message, origin, text::DEBUG_TEXT_SCALE);
            renderer.render_text_with_bg(frame, &glyphs);
        }
        notifications.retain(|n| !n.expired());
    }
}

pub fn draw_fps_counter(renderer: &mut Renderer, frame: &Frame, smoothed_fps: f32) {
    let fps_text = format!("FPS: {:.0}", smoothed_fps);
    let glyphs = text::layout_text_scaled(&fps_text, Vec2::new(10.0, 10.0), text::DEBUG_TEXT_SCALE);
    renderer.render_text_with_bg(frame, &glyphs);
}

pub fn draw_mouse_position(
    renderer: &mut Renderer,
    frame: &Frame,
    screen_mouse_position: (f64, f64),
    multiplying_factor: f32,
) {
    let screen_pos = Vec2::new(
        screen_mouse_position.0 as f32,
        screen_mouse_position.1 as f32,
    );
    let world_pos = renderer.screen_to_world(screen_pos);
    let authoring_pos = world_pos / multiplying_factor;
    let mouse_text = format!("Mouse Pos: {:.0}, {:.0}", authoring_pos.x, authoring_pos.y);
    let screen_size = renderer.screen_size();
    let mouse_text_pos = Vec2::new(text::DEBUG_TEXT_PADDING, screen_size.y - 25.0);
    let glyphs = text::layout_text_scaled(&mouse_text, mouse_text_pos, text::DEBUG_TEXT_SCALE);
    renderer.render_text_with_bg(frame, &glyphs);
}

pub fn draw_slider(renderer: &mut Renderer, frame: &Frame, rects: &[SolidRect]) {
    let projection = renderer.screen_projection();
    renderer.render_solid_rects(frame, rects, projection, Mat4::IDENTITY);
}
