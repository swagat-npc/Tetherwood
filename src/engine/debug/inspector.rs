use crate::engine::entity::Rect;
use crate::engine::renderer::{Frame, Renderer, SolidRect};
use glam::{Mat4, Vec2};

const INSPECTOR_PADDING: f32 = 6.0;
const INSPECTOR_SECTION_PADDING: f32 = 12.0;
const INSPECTOR_BORDER_THICKNESS: f32 = 4.0;
const SECTION_BORDER_THICKNESS: f32 = 1.5;

pub struct InspectorSection {
    pub title: String,
    pub bounds: Rect,
}

pub struct InspectorState {
    pub sections: Vec<InspectorSection>,
    pub volume_slider: Slider,
}

pub struct Inspector {
    pub position: Vec2,
    size: Vec2,
    fill_color: [f32; 4],
    border_color: [f32; 4],
    border_thickness_px: f32,
    pub state: Option<InspectorState>,
}

impl Inspector {
    pub fn new(screen_size: Vec2) -> Self {
        let width = 250.0;
        let border_thickness_px = INSPECTOR_BORDER_THICKNESS;
        let mut inspector = Self {
            position: screen_size - Vec2::new(width / 2.0, screen_size.y / 2.0),
            size: Vec2::new(width, screen_size.y),
            fill_color: [0.4, 0.4, 0.4, 1.0],
            border_color: [0.2, 0.2, 0.2, 1.0],
            border_thickness_px,
            state: None,
        };

        inspector.populate_inspector();
        inspector
    }

    fn get_start_position(&self) -> Vec2 {
        Vec2::new(
            self.position.x - (self.get_panel_content_width() / 2.0),
            self.position.y - (self.size.y / 2.0) + self.border_thickness_px + INSPECTOR_PADDING,
        )
    }

    fn get_panel_content_width(&self) -> f32 {
        self.size.x - 2.0 * self.border_thickness_px - 2.0 * INSPECTOR_PADDING
    }

    fn populate_inspector(&mut self) {
        let volume_bounds = self.compute_section_bounds(0);
        let hotkey_bounds = self.compute_section_bounds(1);

        let state = InspectorState {
            sections: vec![
                InspectorSection {
                    title: "Volume: ".to_string(),
                    bounds: volume_bounds,
                },
                InspectorSection {
                    title: "Hotkeys: ".to_string(),
                    bounds: hotkey_bounds,
                },
            ],
            volume_slider: self.populate_volume_slider(volume_bounds),
        };

        self.state = Some(state);
    }

    fn populate_volume_slider(&mut self, section_bounds: Rect) -> Slider {
        let slider_size = Vec2::new(120.0, 12.0);
        Slider::new(
            section_bounds.center - section_bounds.half_size
                + Vec2::new(
                    INSPECTOR_SECTION_PADDING + slider_size.x * 0.5,
                    INSPECTOR_SECTION_PADDING + slider_size.y * 0.5,
                ),
            slider_size,
            -40.0,
            0.0,
            -24.0, // matches blip_volume's current default
        )
    }

    pub fn draw(&mut self, renderer: &mut Renderer, frame: &Frame) {
        let mut rects = self.build_panel();

        rects.extend(self.build_sections());
        if let Some(state) = &mut self.state {
            rects.extend(state.volume_slider.build_rects());
        }
        let projection = renderer.screen_projection();
        renderer.render_solid_rects(frame, &rects, projection, Mat4::IDENTITY);
    }

    fn build_panel(&self) -> Vec<SolidRect> {
        vec![SolidRect {
            position: self.position,
            size: self.size,
            fill_color: self.fill_color,
            border_color: self.border_color,
            border_thickness_px: self.border_thickness_px,
        }]
    }

    fn compute_section_bounds(&self, index: usize) -> Rect {
        let section_gap = 10.0;
        let section_size = 40.0;
        let start_pos = self.get_start_position();
        Rect {
            center: Vec2::new(
                start_pos.x + self.get_panel_content_width() * 0.5,
                start_pos.y + section_size * 0.5 + (section_gap + section_size) * index as f32,
            ),
            half_size: Vec2::new(self.get_panel_content_width(), section_size) * 0.5,
        }
    }

    fn build_sections(&self) -> Vec<SolidRect> {
        let Some(state) = &self.state else {
            return Vec::new();
        };
        state
            .sections
            .iter()
            .map(|section| SolidRect {
                position: section.bounds.center,
                size: section.bounds.half_size * 2.0,
                fill_color: [0.0, 0.0, 0.0, 0.3],
                border_color: [0.1, 0.1, 0.1, 1.0],
                border_thickness_px: SECTION_BORDER_THICKNESS,
            })
            .collect()
    }
}

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
}
