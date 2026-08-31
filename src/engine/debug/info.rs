use super::notifications::Notification;
use crate::engine::{
    entity::Entity,
    renderer::{
        Frame, Renderer,
        text::{self, GLYPH_SIZE},
        tile,
    },
};
use glam::Vec2;

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
    is_isometric: bool,
) {
    let screen_pos = Vec2::new(
        screen_mouse_position.0 as f32,
        screen_mouse_position.1 as f32,
    );
    let world_pos = renderer.screen_to_world(screen_pos, is_isometric);
    let authoring_pos = world_pos / multiplying_factor;
    let cell_pos = tile::cell_at_position(world_pos, multiplying_factor);

    let mouse_pos_text = format!("World Pos: {:.0}, {:.0}", authoring_pos.x, authoring_pos.y);
    let mouse_cell_text = format!("Cell Pos: {:.0}, {:.0}", cell_pos.0, cell_pos.1);

    let screen_size = renderer.screen_size();
    let mouse_text_pos = Vec2::new(text::DEBUG_TEXT_PADDING, screen_size.y - 25.0);
    let mouse_cell_pos = Vec2::new(
        text::DEBUG_TEXT_PADDING,
        screen_size.y - 25.0 - GLYPH_SIZE.y - text::DEBUG_TEXT_PADDING,
    );

    let mut glyphs =
        text::layout_text_scaled(&mouse_pos_text, mouse_text_pos, text::DEBUG_TEXT_SCALE);
    glyphs.extend(text::layout_text_scaled(
        &mouse_cell_text,
        mouse_cell_pos,
        text::DEBUG_TEXT_SCALE,
    ));
    renderer.render_text_with_bg(frame, &glyphs);
}

pub fn draw_player_position(
    renderer: &mut Renderer,
    frame: &Frame,
    multiplying_factor: f32,
    player: &Entity,
) {
    let player_pos = player.position / multiplying_factor;

    let player_pos_text = format!("Player Pos: {:.0}, {:.0}", player_pos.x, player_pos.y);

    let screen_size = renderer.screen_size();
    let player_text_pos = Vec2::new(
        text::DEBUG_TEXT_PADDING,
        screen_size.y
            - 25.0 // world pos text
            - GLYPH_SIZE.y
            - text::DEBUG_TEXT_PADDING // mouse cell pos text
            - GLYPH_SIZE.y
            - text::DEBUG_TEXT_PADDING * 2.0 - 4.0, // player pos text
    );

    let glyphs =
        text::layout_text_scaled(&player_pos_text, player_text_pos, text::DEBUG_TEXT_SCALE);

    renderer.render_text_with_bg(frame, &glyphs);
}
