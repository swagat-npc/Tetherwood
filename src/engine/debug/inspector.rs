use crate::engine::entity::Rect;
use crate::engine::renderer::{Frame, Renderer, SolidRect, text, tile};
use glam::{Mat4, Vec2};

const INSPECTOR_PADDING: f32 = 6.0;
const INSPECTOR_SECTION_PADDING: f32 = 12.0;
const INSPECTOR_BORDER_THICKNESS: f32 = 4.0;
const SECTION_BORDER_THICKNESS: f32 = 1.5;

pub enum PaintMode {
    Place,
    Remove,
}

pub enum SectionWidget {
    Slider(Slider),
    TilePalette(TilePalette),
}

pub struct InspectorSection {
    pub title: String,
    offset: Vec2,
    half_size: Vec2,
    pub widgets: Vec<SectionWidget>,
}

pub struct InspectorState {
    pub sections: Vec<InspectorSection>,
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
    const VOLUME_SLIDER_SIZE: Vec2 = Vec2::new(120.0, 12.0);
    const TILE_CELL_SIZE: f32 = 64.0;
    const TILE_CELL_PADDING: f32 = 6.0;
    const TILE_THUMBNAIL_SIZE: f32 = Self::TILE_CELL_SIZE - 2.0 * Self::TILE_CELL_PADDING;
    const TILE_CELL_GAP: f32 = 8.0;

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

    pub fn toggle(&mut self) {
        self.is_hidden = !self.is_hidden;
        if let Some(state) = &mut self.state {
            for section in state.sections.iter_mut() {
                for widget in section.widgets.iter_mut() {
                    if let SectionWidget::Slider(slider) = widget {
                        slider.cancel_drag();
                    }
                }
            }
        }
    }

    /// Full panel bounds, resolved fresh (position animates) - used to
    /// gate world-clicks (paint mode) against clicks meant for the UI.
    pub fn bounds(&self) -> Rect {
        Rect {
            center: self.position,
            half_size: self.size * 0.5,
        }
    }

    /// Finds the first TilePalette widget in any section, by type, not
    /// by section title - stays correct even if section names change.
    pub fn selected_tile(&self) -> Option<(i32, i32)> {
        let state = self.state.as_ref()?;
        for section in &state.sections {
            for widget in &section.widgets {
                if let SectionWidget::TilePalette(p) = widget {
                    return Some(p.selected);
                }
            }
        }
        None
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
        let content_width = self.get_panel_content_width();

        let tile_palette = TilePalette::new(Self::TILE_CELL_SIZE, Self::TILE_CELL_GAP, (0, 0));

        let section_heights = [
            Self::VOLUME_SLIDER_SIZE.y + 2.0 * INSPECTOR_SECTION_PADDING,
            tile_palette.required_height(content_width),
        ];

        let (volume_offset, volume_half_size) = self.compute_section_offset(0, &section_heights);
        let (tileset_offset, tileset_half_size) = self.compute_section_offset(1, &section_heights);

        self.state = Some(InspectorState {
            sections: vec![
                InspectorSection {
                    title: "Audio".to_string(),
                    offset: volume_offset,
                    half_size: volume_half_size,
                    widgets: vec![SectionWidget::Slider(Self::populate_volume_slider(
                        volume_offset,
                        volume_half_size,
                    ))],
                },
                InspectorSection {
                    title: "Tileset".to_string(),
                    offset: tileset_offset,
                    half_size: tileset_half_size,
                    widgets: vec![SectionWidget::TilePalette(tile_palette)],
                },
            ],
        });
    }

    /// Builds the slider with a fixed offset from Inspector.position,
    /// same relative-position pattern as InspectorSection — the slider's
    /// actual screen position gets synced fresh each frame in draw(),
    /// via Slider::set_position, rather than trusting a value baked in
    /// once at construction time.
    fn populate_volume_slider(section_offset: Vec2, section_half_size: Vec2) -> Slider {
        let slider_offset = section_offset - section_half_size
            + Vec2::new(
                INSPECTOR_SECTION_PADDING + Self::VOLUME_SLIDER_SIZE.x * 0.5,
                INSPECTOR_SECTION_PADDING + Self::VOLUME_SLIDER_SIZE.y * 0.5,
            );
        Slider::new(slider_offset, Self::VOLUME_SLIDER_SIZE, -40.0, 0.0, -24.0)
    }

    // Draw order:
    // FIRST backgrounds+slider first (solid, unchanged),
    // THEN thumbnails (textured, painted on top),
    // THEN the selection highlight (solid, painted
    // on top of thumbnails so its border is visible),
    // THEN text - same layering logic as the main frame's
    // own layer order (tiles, then entities, then debug).
    pub fn draw(&mut self, renderer: &mut Renderer, frame: &Frame, is_isometric: bool) {
        if let Some(state) = &mut self.state {
            for section in state.sections.iter_mut() {
                for widget in section.widgets.iter_mut() {
                    if let SectionWidget::Slider(slider) = widget {
                        slider.sync_position(self.position);
                    }
                }
            }
        }

        let mut rects = self.build_panel();
        rects.extend(self.build_sections());

        let mut thumbnail_entries = Vec::new();
        let mut highlight_rects = Vec::new();

        if let Some(state) = &self.state {
            for section in state.sections.iter() {
                let bounds = self.section_bounds(section);
                let section_top_left = bounds.center - bounds.half_size;
                let content_width = bounds.half_size.x * 2.0;

                for widget in section.widgets.iter() {
                    match widget {
                        SectionWidget::Slider(slider) => rects.extend(slider.build_rects()),
                        SectionWidget::TilePalette(palette) => {
                            thumbnail_entries
                                .extend(palette.thumbnail_entries(section_top_left, content_width));
                            highlight_rects.extend(
                                palette.build_highlight_rect(section_top_left, content_width),
                            );
                        }
                    }
                }
            }
        }

        let projection = renderer.screen_projection();
        renderer.render_solid_rects(frame, &rects, projection, Mat4::IDENTITY);
        renderer.render_ui_tiles(
            frame,
            &thumbnail_entries,
            Self::TILE_THUMBNAIL_SIZE,
            is_isometric,
            projection,
        );
        renderer.render_solid_rects(frame, &highlight_rects, projection, Mat4::IDENTITY);
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
    fn compute_section_offset(&self, index: usize, section_heights: &[f32]) -> (Vec2, Vec2) {
        let section_gap = 10.0;
        let section_size = section_heights[index];
        let content_width = self.get_panel_content_width();
        let start_offset = Vec2::new(
            -(content_width / 2.0),
            -(self.size.y / 2.0) + self.border_thickness_px + INSPECTOR_PADDING,
        );
        let y_before: f32 =
            section_heights[..index].iter().sum::<f32>() + section_gap * index as f32;
        let center_offset = Vec2::new(
            start_offset.x + content_width * 0.5,
            start_offset.y + y_before + section_size * 0.5,
        );
        (center_offset, Vec2::new(content_width, section_size) * 0.5)
    }

    pub fn resolve_section_bounds(position: Vec2, section: &InspectorSection) -> Rect {
        Rect {
            center: position + section.offset,
            half_size: section.half_size,
        }
    }

    /// A section's real, current on-screen bounds — recomputed fresh
    /// from Inspector.position every time it's needed, so it always
    /// follows the panel's current (possibly animating) position rather
    /// than a stale, construction-time snapshot.
    fn section_bounds(&self, section: &InspectorSection) -> Rect {
        Self::resolve_section_bounds(self.position, section)
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
    offset: Vec2,
    pub position: Vec2,
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

pub struct TilePalette {
    cell_size: f32,
    gap: f32,
    selected: (i32, i32),
}

impl TilePalette {
    pub fn new(cell_size: f32, gap: f32, default_selected: (i32, i32)) -> Self {
        Self {
            cell_size,
            gap,
            selected: default_selected,
        }
    }

    /// Sorted for a stable on-screen order — tile_names() is a HashMap,
    /// whose iteration order is unspecified (the same reason tile
    /// depth-sorting couldn't trust it, per Phase 30).
    fn tile_list() -> Vec<(&'static str, (i32, i32))> {
        let mut tiles: Vec<_> = tile::tile_names().into_iter().collect();
        tiles.sort_by_key(|(name, _)| *name);
        tiles
    }

    fn column_length(&self, content_width: f32) -> usize {
        (((content_width + self.gap) / (self.cell_size + self.gap)).floor() as usize).max(1)
    }

    pub fn required_height(&self, content_width: f32) -> f32 {
        let no_of_cols = self.column_length(content_width);
        let rows = (Self::tile_list().len() as f32 / no_of_cols as f32)
            .ceil()
            .max(1.0);
        2.0 * INSPECTOR_SECTION_PADDING + (rows - 1.0) * self.gap + rows * self.cell_size
    }

    fn cell_top_left(&self, index: usize, cols: usize, section_top_left: Vec2) -> Vec2 {
        let col = (index % cols) as f32;
        let row = (index / cols) as f32;
        section_top_left
            + Vec2::splat(INSPECTOR_SECTION_PADDING)
            + Vec2::new(col, row) * (self.cell_size + self.gap)
    }

    /// Center position + atlas cell for every tile, ready to hand to
    /// the renderer as textured-quad draw entries.
    pub fn thumbnail_entries(
        &self,
        section_top_left: Vec2,
        content_width: f32,
    ) -> Vec<(Vec2, (i32, i32))> {
        let tiles = Self::tile_list();
        let cols = self.column_length(content_width);
        tiles
            .iter()
            .enumerate()
            .map(|(i, (_, cell))| {
                let top_left = self.cell_top_left(i, cols, section_top_left);
                (top_left + Vec2::splat(self.cell_size * 0.5), *cell)
            })
            .collect()
    }

    pub fn build_highlight_rect(
        &self,
        section_top_left: Vec2,
        content_width: f32,
    ) -> Option<SolidRect> {
        let tiles = Self::tile_list();
        let cols = self.column_length(content_width);
        let index = tiles.iter().position(|(_, cell)| *cell == self.selected)?;
        let top_left = self.cell_top_left(index, cols, section_top_left);
        Some(SolidRect {
            position: top_left + Vec2::splat(self.cell_size * 0.5),
            size: Vec2::splat(self.cell_size),
            fill_color: [0.0, 1.0, 0.0, 0.0],
            border_color: [0.0, 1.0, 0.0, 0.7],
            border_thickness_px: 2.0,
        })
    }

    /// section_top_left: this widget's drawing-area top-left corner,
    /// resolved fresh from the section's current bounds — same
    /// "supplied each call, never stored" convention as Slider::sync_position.
    pub fn handle_click(
        &mut self,
        mouse_pos: Vec2,
        section_top_left: Vec2,
        content_width: f32,
    ) -> bool {
        let tiles = Self::tile_list();
        let cols = self.column_length(content_width);
        let local = mouse_pos - section_top_left - Vec2::splat(INSPECTOR_SECTION_PADDING);
        if local.x < 0.0 || local.y < 0.0 {
            return false;
        }
        let col = (local.x / (self.cell_size + self.gap)).floor() as i32;
        let row = (local.y / (self.cell_size + self.gap)).floor() as i32;
        if col < 0 || col as usize >= cols {
            return false;
        }
        let index = row as usize * cols + col as usize;
        match tiles.get(index) {
            Some((_, cell)) if *cell != self.selected => {
                self.selected = *cell;
                true
            }
            _ => false,
        }
    }
}
