use crate::engine::entity::Rect;
use crate::engine::renderer::{Frame, Renderer, SolidRect, text};
use glam::{Mat4, Vec2};

const INSPECTOR_PADDING: f32 = 6.0;
const INSPECTOR_SECTION_PADDING: f32 = 12.0;
const INSPECTOR_BORDER_THICKNESS: f32 = 4.0;
const SECTION_BORDER_THICKNESS: f32 = 1.5;

pub struct InspectorSection {
    pub title: String,
    offset: Vec2,
    half_size: Vec2,
}

pub struct InspectorState {
    pub sections: Vec<InspectorSection>,
    pub volume_slider: Slider,
}

pub struct Inspector {
    pub position: Vec2,
    visible_position: Vec2,
    pub is_hidden: bool,
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
        let visible_position = screen_size - Vec2::new(width / 2.0, screen_size.y / 2.0);

        let mut inspector = Self {
            position: screen_size - Vec2::new(width / 2.0, screen_size.y / 2.0),
            visible_position,
            is_hidden: false,
            size: Vec2::new(width, screen_size.y),
            fill_color: [0.4, 0.4, 0.4, 1.0],
            border_color: [0.2, 0.2, 0.2, 1.0],
            border_thickness_px,
            state: None,
        };

        inspector.populate_inspector();
        inspector
    }

    pub fn toggle(&mut self) {
        self.is_hidden = !self.is_hidden;
        if let Some(state) = &mut self.state {
            state.volume_slider.cancel_drag();
        }
    }

    /// Call once per frame, before drawing. Eases `position` toward
    /// visible_position, or fully off-screen to the right if hidden -
    /// same lerp-toward-target pattern as Renderer::update_smoothed_camera.
    pub fn update(&mut self) {
        let target = self.target_position();
        self.position = self.position.lerp(target, 0.15);
    }

    pub fn is_settled(&self) -> bool {
        let target = self.target_position();
        (self.position - target).length() < 0.5 // small epsilon — "close enough" to done animating
    }

    fn get_panel_content_width(&self) -> f32 {
        self.size.x - 2.0 * self.border_thickness_px - 2.0 * INSPECTOR_PADDING
    }

    fn target_position(&self) -> Vec2 {
        if self.is_hidden {
            self.visible_position + Vec2::new(self.size.x, 0.0)
        } else {
            self.visible_position
        }
    }

    /// Recomputes visible_position/size and every section's offset
    /// from a new screen size — same layout math as new(), callable
    /// again so a window resize can keep the panel correctly anchored
    /// to the right edge instead of drifting.
    pub fn recompute_layout(&mut self, screen_size: Vec2) {
        let width = 250.0;
        self.visible_position = screen_size - Vec2::new(width / 2.0, screen_size.y / 2.0);
        self.size = Vec2::new(width, screen_size.y);
        self.populate_inspector();

        // Resize is a sudden, discrete event, not a user-initiated
        // toggle - snap position immediately rather than letting the
        // usual lerp animate through it, which read as floaty/wrong.
        self.position = self.target_position();
    }

    fn populate_inspector(&mut self) {
        let (volume_offset, volume_half_size) = self.compute_section_offset(0);
        let (hotkey_offset, hotkey_half_size) = self.compute_section_offset(1);

        let state = InspectorState {
            sections: vec![
                InspectorSection {
                    title: "Volume: ".to_string(),
                    offset: volume_offset,
                    half_size: volume_half_size,
                },
                InspectorSection {
                    title: "Hotkeys: ".to_string(),
                    offset: hotkey_offset,
                    half_size: hotkey_half_size,
                },
            ],
            volume_slider: Self::populate_volume_slider(volume_offset, volume_half_size),
        };

        self.state = Some(state);
    }

    /// Builds the slider with a fixed offset from Inspector.position,
    /// same relative-position pattern as InspectorSection — the slider's
    /// actual screen position gets synced fresh each frame in draw(),
    /// via Slider::set_position, rather than trusting a value baked in
    /// once at construction time.
    fn populate_volume_slider(section_offset: Vec2, section_half_size: Vec2) -> Slider {
        let slider_size = Vec2::new(120.0, 12.0);
        let slider_offset = section_offset - section_half_size
            + Vec2::new(
                INSPECTOR_SECTION_PADDING + slider_size.x * 0.5,
                INSPECTOR_SECTION_PADDING + slider_size.y * 0.5,
            );
        Slider::new(slider_offset, slider_size, -40.0, 0.0, -24.0)
    }

    pub fn draw(&mut self, renderer: &mut Renderer, frame: &Frame) {
        if let Some(state) = &mut self.state {
            state.volume_slider.sync_position(self.position);
        }

        let mut rects = self.build_panel();
        rects.extend(self.build_sections());
        if let Some(state) = &self.state {
            rects.extend(state.volume_slider.build_rects());
        }
        let projection = renderer.screen_projection();
        renderer.render_solid_rects(frame, &rects, projection, Mat4::IDENTITY);
        self.draw_section_titles(renderer, frame);
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

    /// The section's fixed offset from Inspector.position, computed once
    /// at construction — never changes, since it's purely a layout fact
    /// about the panel's internal structure.
    fn compute_section_offset(&self, index: usize) -> (Vec2, Vec2) {
        let section_gap = 10.0;
        let section_size = 40.0;
        let start_offset = Vec2::new(
            -(self.get_panel_content_width() / 2.0),
            -(self.size.y / 2.0) + self.border_thickness_px + INSPECTOR_PADDING,
        );
        let center_offset = Vec2::new(
            start_offset.x + self.get_panel_content_width() * 0.5,
            start_offset.y + section_size * 0.5 + (section_gap + section_size) * index as f32,
        );
        (
            center_offset,
            Vec2::new(self.get_panel_content_width(), section_size) * 0.5,
        )
    }

    /// A section's real, current on-screen bounds — recomputed fresh
    /// from Inspector.position every time it's needed, so it always
    /// follows the panel's current (possibly animating) position rather
    /// than a stale, construction-time snapshot.
    fn section_bounds(&self, section: &InspectorSection) -> Rect {
        Rect {
            center: self.position + section.offset,
            half_size: section.half_size,
        }
    }

    fn build_sections(&self) -> Vec<SolidRect> {
        let Some(state) = &self.state else {
            return Vec::new();
        };
        state
            .sections
            .iter()
            .map(|section| {
                let bounds = self.section_bounds(section);
                SolidRect {
                    position: bounds.center,
                    size: bounds.half_size * 2.0,
                    fill_color: [0.0, 0.0, 0.0, 0.3],
                    border_color: [0.1, 0.1, 0.1, 1.0],
                    border_thickness_px: SECTION_BORDER_THICKNESS,
                }
            })
            .collect()
    }

    fn draw_section_titles(&self, renderer: &mut Renderer, frame: &Frame) {
        let Some(state) = &self.state else { return };
        for section in &state.sections {
            let bounds = self.section_bounds(section);
            let origin = Vec2::new(
                bounds.center.x - bounds.half_size.x + 8.0,
                bounds.center.y - bounds.half_size.y + 4.0,
            );
            let glyphs = text::layout_ttf_text(
                &section.title,
                &renderer.ttf_glyphs,
                origin,
                1.0,
                [1.0, 1.0, 1.0, 1.0],
            );
            renderer.render_ttf_text(frame, &glyphs);
        }
    }
}

pub struct Slider {
    offset: Vec2,       // fixed, relative to whatever panel owns this slider
    pub position: Vec2, // synced each frame from panel_position + offset
    pub size: Vec2,
    pub min: f32,
    pub max: f32,
    pub value: f32,
    dragging: bool,
}

impl Slider {
    pub fn new(offset: Vec2, size: Vec2, min: f32, max: f32, value: f32) -> Self {
        Self {
            offset,
            position: offset,
            size,
            min,
            max,
            value,
            dragging: false,
        }
    }

    /// Call once per frame, before update/build_rects, with the
    /// current absolute position of whatever panel this slider lives
    /// in - keeps position following the panel without needing the
    /// slider to know anything about panels/animation itself.
    pub fn sync_position(&mut self, panel_position: Vec2) {
        self.position = panel_position + self.offset;
    }

    pub fn cancel_drag(&mut self) {
        self.dragging = false;
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
