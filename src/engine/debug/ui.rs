use crate::engine::renderer::{Frame, Renderer, SolidRect};
use glam::{Mat4, Vec2};

// TODO: Rename this file to `inspector.rs` once additional inspector elements
// are added as this file is expected to grow into the main inspector UI.
pub struct Slider {
    pub position: Vec2,
    pub size: Vec2,
    pub min: f32,
    pub max: f32,
    pub value: f32,
    dragging: bool,
}

impl Slider {
    pub fn new(position: Vec2, size: Vec2, min: f32, max: f32, value: f32) -> Self {
        Self {
            position,
            size,
            min,
            max,
            value,
            dragging: false,
        }
    }

    /// Call once per frame while the slider is visible. Returns true
    /// if value changed this call, so the caller knows when to act
    /// on it (rather than re-applying an unchanged value every frame).
    pub fn update(&mut self, mouse_pos: Vec2, mouse_down: bool) -> bool {
        let half_size = self.size * 0.5;
        let over_track = mouse_pos.x >= self.position.x - half_size.x
            && mouse_pos.x <= self.position.x + half_size.x
            && mouse_pos.y >= self.position.y - half_size.y
            && mouse_pos.y <= self.position.y + half_size.y;

        if mouse_down && (self.dragging || over_track) {
            self.dragging = true;
            let t = ((mouse_pos.x - (self.position.x - half_size.x)) / self.size.x).clamp(0.0, 1.0);
            let new_value = self.min + t * (self.max - self.min);
            let changed = new_value != self.value;
            self.value = new_value;
            return changed;
        }
        self.dragging = false;
        false
    }

    /// Produces the track + handle as SolidRects, ready for
    /// render_solid_rects — same primitive every other UI element
    /// (the dialogue panel, debug backgrounds) already uses.
    pub fn build_rects(&self) -> Vec<SolidRect> {
        let handle_t = (self.value - self.min) / (self.max - self.min);
        let handle_x = self.position.x - self.size.x * 0.5 + handle_t * self.size.x;

        vec![
            SolidRect {
                position: self.position,
                size: self.size,
                fill_color: [0.3, 0.3, 0.3, 0.9],
                border_color: [0.8, 0.8, 0.8, 1.0],
                border_thickness_px: 2.0,
            },
            SolidRect {
                position: Vec2::new(handle_x, self.position.y),
                size: Vec2::new(8.0, self.size.y + 6.0),
                fill_color: [1.0, 1.0, 1.0, 1.0],
                border_color: [1.0, 1.0, 1.0, 1.0],
                border_thickness_px: 0.0,
            },
        ]
    }

    /// Draws the slider on the screen using the given renderer and frame.
    pub fn draw(&self, renderer: &mut Renderer, frame: &Frame) {
        let rects = self.build_rects();
        let projection = renderer.screen_projection();
        renderer.render_solid_rects(frame, &rects, projection, Mat4::IDENTITY);
    }
}
