use std::collections::HashMap;

use crate::engine::entity::Rect;
use crate::engine::renderer::text::{GLYPH_SIZE, PositionedTTFGlyph, TTFGlyph};
use crate::engine::renderer::tile::PaintMode;
use crate::engine::renderer::{Frame, Renderer, SolidRect, text, tile};
use glam::{Mat4, Vec2};

const INSPECTOR_PADDING: f32 = 6.0;
const INSPECTOR_SECTION_PADDING: f32 = 12.0;
const INSPECTOR_BORDER_THICKNESS: f32 = 4.0;
const SECTION_BORDER_THICKNESS: f32 = 1.5;
const WIDGET_GAP: f32 = 8.0;
const TITLE_BAR_HEIGHT: f32 = GLYPH_SIZE.y + 5.0 * 2.0;
const SCROLLBAR_WIDTH: f32 = 8.0;
const SCROLLBAR_GAP: f32 = 4.0;

const VOLUME_SLIDER_SIZE: Vec2 = Vec2::new(120.0, 12.0);
const BUTTON_SIZE: Vec2 = Vec2::new(80.0, 30.0);
const BUTTON_GAP: f32 = 8.0;
const TILE_CELL_SIZE: f32 = 64.0;
const TILE_CELL_PADDING: f32 = 6.0;
const TILE_THUMBNAIL_SIZE: f32 = TILE_CELL_SIZE - 2.0 * TILE_CELL_PADDING;
const TILE_CELL_GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy)]
pub enum TilesetAction {
    None,
    SetMode(PaintMode),
    Save,
    Revert,
    ClearAll,
}

pub enum SectionWidget {
    Slider(Slider),
    TilePalette(TilePalette),
    TilesetControls(TilesetControls),
    HotkeyList(HotkeyList),
}

impl SectionWidget {
    fn required_height(&self, content_width: f32, font: &HashMap<char, TTFGlyph>) -> f32 {
        match self {
            SectionWidget::Slider(s) => s.required_height(),
            SectionWidget::TilePalette(p) => p.required_height(content_width),
            SectionWidget::TilesetControls(c) => c.required_height(),
            SectionWidget::HotkeyList(h) => h.required_height(content_width, font),
        }
    }
}

pub struct InspectorSection {
    pub title: String,
    offset: Vec2,
    half_size: Vec2,
    pub widgets: Vec<SectionWidget>,
}

impl InspectorSection {
    fn required_height(
        widgets: &[SectionWidget],
        content_width: f32,
        font: &HashMap<char, TTFGlyph>,
    ) -> f32 {
        let sum: f32 = widgets
            .iter()
            .map(|w| w.required_height(content_width, font))
            .sum();
        TITLE_BAR_HEIGHT
            + sum
            + WIDGET_GAP * widgets.len().saturating_sub(1) as f32
            + INSPECTOR_SECTION_PADDING
    }
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
    pub scroll_offset: f32,
    scrollbar: Scrollbar,
    pub state: Option<InspectorState>,
}

/// Resolves every widget's own (top_left, height) within a section,
/// stacked top-to-bottom. Recomputed fresh every call (click handling,
/// drawing) - never cached, consistent with section_bounds() and
/// every other layout value in this file.
pub fn stack_widgets(
    widgets: &[SectionWidget],
    section_top_left: Vec2,
    content_width: f32,
    font: &HashMap<char, TTFGlyph>,
) -> Vec<(Vec2, f32)> {
    let mut cursor_y = section_top_left.y + TITLE_BAR_HEIGHT;
    widgets
        .iter()
        .map(|w| {
            let height = w.required_height(content_width, font);
            let top_left = Vec2::new(section_top_left.x, cursor_y);
            cursor_y += height + WIDGET_GAP;
            (top_left, height)
        })
        .collect()
}

impl Inspector {
    pub fn new(screen_size: Vec2, font: &HashMap<char, TTFGlyph>) -> Self {
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
            scroll_offset: 0.0,
            scrollbar: Scrollbar::new(),
            state: None,
        };

        inspector.populate_inspector(font);
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

    /// The panel's inner content area only - inset from the full panel
    /// bounds by border + padding on every side, same inset
    /// get_panel_content_width already applies for width. Used for
    /// scissoring scrolled content, distinct from bounds() (the full
    /// panel, used for "is the mouse over this panel at all" gating).
    fn content_bounds(&self) -> Rect {
        let inset = self.border_thickness_px + INSPECTOR_PADDING;
        Rect {
            center: self.position,
            half_size: Vec2::new(self.get_panel_content_width(), self.size.y - 2.0 * inset) * 0.5,
        }
    }

    fn scissor_rect(&self, screen_size: Vec2) -> (u32, u32, u32, u32) {
        let bounds = self.content_bounds();
        let x = (bounds.center.x - bounds.half_size.x).max(0.0);
        let y = (bounds.center.y - bounds.half_size.y).max(0.0);
        let width = (bounds.half_size.x * 2.0).min(screen_size.x - x);
        let height = (bounds.half_size.y * 2.0).min(screen_size.y - y);
        (
            x as u32,
            y as u32,
            width.max(0.0) as u32,
            height.max(0.0) as u32,
        )
    }

    /// Total height every section would need stacked with no scrolling
    /// at all - same accumulate-and-sum shape populate_inspector
    /// already uses to build section_heights, just summed once more
    /// for the max-scroll bound.
    fn total_content_height(&self, font: &HashMap<char, TTFGlyph>) -> f32 {
        let Some(state) = &self.state else { return 0.0 };
        let content_width = self.get_panel_content_width();
        let section_gap = 10.0;
        let sum: f32 = state
            .sections
            .iter()
            .map(|s| InspectorSection::required_height(&s.widgets, content_width, font))
            .sum();
        sum + section_gap * state.sections.len().saturating_sub(1) as f32
    }

    fn max_scroll(&self, font: &HashMap<char, TTFGlyph>) -> f32 {
        let visible = self.content_bounds().half_size.y * 2.0;
        (self.total_content_height(font) - visible).max(0.0)
    }

    pub fn scroll(&mut self, delta: f32, font: &HashMap<char, TTFGlyph>) {
        let max = self.max_scroll(font);
        self.scroll_offset = (self.scroll_offset - delta).clamp(0.0, max);
    }

    fn scrollbar_track_bounds(&self) -> Rect {
        let content = self.content_bounds();
        let track_x =
            content.center.x + content.half_size.x + SCROLLBAR_GAP + SCROLLBAR_WIDTH * 0.5;
        Rect {
            center: Vec2::new(track_x, content.center.y),
            half_size: Vec2::new(SCROLLBAR_WIDTH * 0.5, content.half_size.y),
        }
    }

    pub fn update_scrollbar(
        &mut self,
        mouse_pos: Vec2,
        mouse_down: bool,
        font: &HashMap<char, TTFGlyph>,
    ) {
        let track = self.scrollbar_track_bounds();
        let max_scroll = self.max_scroll(font);
        let total = self.total_content_height(font);
        if let Some(new_offset) = self.scrollbar.update(
            track,
            mouse_pos,
            mouse_down,
            self.scroll_offset,
            max_scroll,
            total,
        ) {
            self.scroll_offset = new_offset;
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

    // Content width now reserves the gutter unconditionally,
    // whether or not scrolling is currently needed. This avoids a layout
    // feedback loop (needing a scrollbar depends on total content height,
    // which depends on content width) and keeps every existing widget's
    // layout math (TilePalette's column_length, etc.) correct automatically,
    // since they all already recompute fresh from content_width each call.
    fn get_panel_content_width(&self) -> f32 {
        self.size.x
            - 2.0 * self.border_thickness_px
            - 2.0 * INSPECTOR_PADDING
            - SCROLLBAR_WIDTH
            - SCROLLBAR_GAP
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
    pub fn recompute_layout(&mut self, screen_size: Vec2, font: &HashMap<char, TTFGlyph>) {
        let width = 250.0;
        self.visible_position = screen_size - Vec2::new(width / 2.0, screen_size.y / 2.0);
        self.size = Vec2::new(width, screen_size.y);
        self.populate_inspector(font);

        // Resize is a sudden, discrete event, not a user-initiated
        // toggle - snap position immediately rather than letting the
        // usual lerp animate through it, which read as floaty/wrong.
        self.position = self.target_position();
    }

    fn populate_inspector(&mut self, font: &HashMap<char, TTFGlyph>) {
        let content_width = self.get_panel_content_width();

        // Slider needs a real offset derived from section geometry,
        // which isn't known until AFTER heights/offsets are computed -
        // built with a placeholder here, corrected below once section
        // geometry exists. Only Slider has this chicken-and-egg
        // problem; TilePalette/TilesetControls never store an offset,
        // so they need no such fixup.
        let volume_widgets = vec![SectionWidget::Slider(Slider::new(
            Vec2::ZERO,
            VOLUME_SLIDER_SIZE,
            -40.0,
            0.0,
            -24.0,
        ))];

        let tileset_widgets = vec![
            SectionWidget::TilePalette(TilePalette::new(TILE_CELL_SIZE, TILE_CELL_GAP, (0, 0))),
            SectionWidget::TilesetControls(TilesetControls {
                mode_buttons: vec![
                    Button {
                        id: "place",
                        size: BUTTON_SIZE,
                        label: "Place".to_string(),
                    },
                    Button {
                        id: "remove",
                        size: BUTTON_SIZE + Vec2::new(20.0, 0.0),
                        label: "Remove".to_string(),
                    },
                ],
                action_buttons: vec![
                    Button {
                        id: "save",
                        size: BUTTON_SIZE,
                        label: "Save".to_string(),
                    },
                    Button {
                        id: "revert",
                        size: BUTTON_SIZE,
                        label: "Revert".to_string(),
                    },
                ],
                danger_buttons: vec![Button {
                    id: "clear_all",
                    size: BUTTON_SIZE + Vec2::new(20.0, 0.0),
                    label: "Clear All".to_string(),
                }],
            }),
        ];

        let hotkey_lines = SectionWidget::HotkeyList(HotkeyList);

        let section_heights = [
            InspectorSection::required_height(&volume_widgets, content_width, font),
            InspectorSection::required_height(&tileset_widgets, content_width, font),
            SectionWidget::required_height(&hotkey_lines, content_width, font),
        ];

        let (volume_offset, volume_half_size) = self.compute_section_offset(0, &section_heights);
        let (tileset_offset, tileset_half_size) = self.compute_section_offset(1, &section_heights);
        let (hotkeys_offset, hotkeys_half_size) = self.compute_section_offset(2, &section_heights);

        let mut volume_widgets = volume_widgets;
        if let Some(SectionWidget::Slider(slider)) = volume_widgets.get_mut(0) {
            *slider = Self::populate_volume_slider();
        }

        self.state = Some(InspectorState {
            sections: vec![
                InspectorSection {
                    title: "Audio".to_string(),
                    offset: volume_offset,
                    half_size: volume_half_size,
                    widgets: volume_widgets,
                },
                InspectorSection {
                    title: "Tileset".to_string(),
                    offset: tileset_offset,
                    half_size: tileset_half_size,
                    widgets: tileset_widgets,
                },
                InspectorSection {
                    title: "Hotkeys".to_string(),
                    offset: hotkeys_offset,
                    half_size: hotkeys_half_size,
                    widgets: vec![SectionWidget::HotkeyList(HotkeyList)],
                },
            ],
        });
    }

    /// Builds the slider with a fixed offset from Inspector.position,
    /// same relative-position pattern as InspectorSection — the slider's
    /// actual screen position gets synced fresh each frame in draw(),
    /// via Slider::set_position, rather than trusting a value baked in
    /// once at construction time.
    fn populate_volume_slider() -> Slider {
        let slider_offset = Vec2::new(
            INSPECTOR_SECTION_PADDING + VOLUME_SLIDER_SIZE.x * 0.5,
            INSPECTOR_SECTION_PADDING + VOLUME_SLIDER_SIZE.y * 0.5,
        );
        Slider::new(slider_offset, VOLUME_SLIDER_SIZE, -40.0, 0.0, -24.0)
    }

    // Draw order:
    // FIRST backgrounds+slider first (solid, unchanged),
    // THEN thumbnails (textured, painted on top),
    // THEN the selection highlight (solid, painted
    // on top of thumbnails so its border is visible),
    // THEN text - same layering logic as the main frame's
    // own layer order (tiles, then entities, then debug).
    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        frame: &Frame,
        is_isometric: bool,
        paint_mode: &PaintMode,
        is_paint_active: bool,
    ) {
        let mut panel_rects = self.build_panel();
        let mut section_rects = self.build_sections();

        let mut thumbnail_entries = Vec::new();
        let mut highlight_rects = Vec::new();
        let mut label_glyphs = Vec::new();

        if let Some(state) = &self.state {
            for section in state.sections.iter() {
                let bounds = self.section_bounds(section);
                let section_top_left = bounds.center - bounds.half_size;
                let content_width = bounds.half_size.x * 2.0;
                let slots = stack_widgets(
                    &section.widgets,
                    section_top_left,
                    content_width,
                    &renderer.ttf_glyphs,
                );

                for (widget, (widget_top_left, _height)) in section.widgets.iter().zip(slots.iter())
                {
                    match widget {
                        SectionWidget::Slider(slider) => {
                            section_rects.extend(slider.build_rects(*widget_top_left))
                        }
                        SectionWidget::TilePalette(palette) => {
                            thumbnail_entries
                                .extend(palette.thumbnail_entries(*widget_top_left, content_width));
                            highlight_rects.extend(
                                palette.build_tile_highlight_rect(*widget_top_left, content_width),
                            );
                        }
                        SectionWidget::TilesetControls(controls) => {
                            let (button_rects, button_glyphs) = controls.build(
                                *widget_top_left,
                                paint_mode,
                                is_paint_active,
                                &renderer.ttf_glyphs,
                            );
                            section_rects.extend(button_rects);
                            label_glyphs.extend(button_glyphs);
                        }
                        SectionWidget::HotkeyList(list) => {
                            label_glyphs.extend(list.build_label_glyphs(
                                *widget_top_left,
                                content_width,
                                &renderer.ttf_glyphs,
                            ));
                        }
                    }
                }
            }
        }

        let projection = renderer.screen_projection();
        let screen_size = renderer.screen_size();
        // Everything section-derived (sections themselves, slider, thumbnails,
        // highlight, text): scissored to the panel's inner content area, so
        // scrolled-past content is cleanly cut off at the panel's edge rather
        // than bleeding past the border.
        let scissor = Some(self.scissor_rect(screen_size));

        panel_rects.extend(self.scrollbar.build_rects(
            self.scrollbar_track_bounds(),
            self.scroll_offset,
            self.max_scroll(&renderer.ttf_glyphs),
            self.total_content_height(&renderer.ttf_glyphs),
        ));

        // Panel border/background: always full, never clipped by scroll.
        renderer.render_solid_rects(frame, &panel_rects, projection, Mat4::IDENTITY, None);

        renderer.render_solid_rects(frame, &section_rects, projection, Mat4::IDENTITY, scissor);
        renderer.render_ui_tiles(
            frame,
            &thumbnail_entries,
            TILE_THUMBNAIL_SIZE,
            is_isometric,
            projection,
            scissor,
        );
        renderer.render_solid_rects(frame, &highlight_rects, projection, Mat4::IDENTITY, scissor);
        self.draw_section_titles(renderer, frame, &label_glyphs, scissor);
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

    pub fn resolve_section_bounds(
        position: Vec2,
        section: &InspectorSection,
        scroll_offset: f32,
    ) -> Rect {
        Rect {
            center: position + section.offset - Vec2::new(0.0, scroll_offset),
            half_size: section.half_size,
        }
    }

    /// A section's real, current on-screen bounds — recomputed fresh
    /// from Inspector.position every time it's needed, so it always
    /// follows the panel's current (possibly animating) position rather
    /// than a stale, construction-time snapshot.
    fn section_bounds(&self, section: &InspectorSection) -> Rect {
        Self::resolve_section_bounds(self.position, section, self.scroll_offset)
    }

    // build_sections — one title-bar rect per section, added alongside
    // the existing body rect, both drawn into the scissored batch.
    fn build_sections(&self) -> Vec<SolidRect> {
        let Some(state) = &self.state else {
            return Vec::new();
        };
        let mut rects = Vec::new();
        for section in &state.sections {
            let bounds = self.section_bounds(section);
            rects.push(SolidRect {
                position: bounds.center,
                size: bounds.half_size * 2.0,
                fill_color: [0.0, 0.0, 0.0, 0.3],
                border_color: [0.1, 0.1, 0.1, 1.0],
                border_thickness_px: SECTION_BORDER_THICKNESS,
            });
            let title_bar_top = bounds.center.y - bounds.half_size.y;
            rects.push(SolidRect {
                position: Vec2::new(bounds.center.x, title_bar_top + TITLE_BAR_HEIGHT * 0.5),
                size: Vec2::new(bounds.half_size.x * 2.0, TITLE_BAR_HEIGHT),
                fill_color: [0.15, 0.15, 0.15, 0.7], // distinct from section body — tune by eye
                border_color: [0.1, 0.1, 0.1, 1.0],
                border_thickness_px: 0.0,
            });
        }
        rects
    }

    fn draw_section_titles(
        &self,
        renderer: &mut Renderer,
        frame: &Frame,
        label_glyphs: &[PositionedTTFGlyph],
        scissor: Option<(u32, u32, u32, u32)>,
    ) {
        let Some(state) = &self.state else { return };

        let mut combined_glyphs = Vec::new();
        combined_glyphs.extend(label_glyphs);
        for section in &state.sections {
            let bounds = self.section_bounds(section);
            let origin = Vec2::new(
                bounds.center.x - bounds.half_size.x + 8.0,
                bounds.center.y - bounds.half_size.y + TITLE_BAR_HEIGHT
                    - (GLYPH_SIZE.y - 2.0) * 0.5,
            );
            let (glyphs, _bounds) = text::layout_ttf_text(
                &section.title,
                &renderer.ttf_glyphs,
                origin,
                1.0,
                [1.0, 1.0, 1.0, 1.0],
            );
            combined_glyphs.extend(&glyphs);
        }
        renderer.render_ttf_text(frame, &combined_glyphs, scissor);
    }
}

pub struct Slider {
    offset: Vec2,
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
            size,
            min,
            max,
            value,
            dragging: false,
        }
    }

    fn required_height(&self) -> f32 {
        INSPECTOR_SECTION_PADDING + self.size.y
    }

    pub fn cancel_drag(&mut self) {
        self.dragging = false;
    }

    /// Call once per frame while the slider is visible. Returns true
    /// if value changed this call, so the caller knows when to act
    /// on it (rather than re-applying an unchanged value every frame).
    pub fn update(&mut self, widget_top_left: Vec2, mouse_pos: Vec2, mouse_down: bool) -> bool {
        let position = widget_top_left + self.offset;

        let half_size = self.size * 0.5;
        let over_track = mouse_pos.x >= position.x - half_size.x
            && mouse_pos.x <= position.x + half_size.x
            && mouse_pos.y >= position.y - half_size.y
            && mouse_pos.y <= position.y + half_size.y;

        if mouse_down && (self.dragging || over_track) {
            self.dragging = true;
            let t = ((mouse_pos.x - (position.x - half_size.x)) / self.size.x).clamp(0.0, 1.0);
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
    pub fn build_rects(&self, widget_top_left: Vec2) -> Vec<SolidRect> {
        let position = widget_top_left + self.offset;
        let handle_t = (self.value - self.min) / (self.max - self.min);
        let handle_x = position.x - self.size.x * 0.5 + handle_t * self.size.x;

        vec![
            SolidRect {
                position: position,
                size: self.size,
                fill_color: [0.3, 0.3, 0.3, 0.9],
                border_color: [0.8, 0.8, 0.8, 1.0],
                border_thickness_px: 2.0,
            },
            SolidRect {
                position: Vec2::new(handle_x, position.y),
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
        INSPECTOR_SECTION_PADDING + (rows - 1.0) * self.gap + rows * self.cell_size
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

    pub fn build_tile_highlight_rect(
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

pub struct Button {
    id: &'static str,
    size: Vec2,
    label: String,
}

impl Button {
    const DEFAULT_BG: [f32; 4] = [0.2, 0.2, 0.2, 0.9];
    const DEFAULT_BORDER: [f32; 4] = [0.6, 0.6, 0.6, 1.0];
    const SELECTED_BG: [f32; 4] = [0.0, 0.5, 0.0, 0.9];
    const SELECTED_BORDER: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    const DISABLED_BG: [f32; 4] = [0.15, 0.15, 0.15, 0.6];
    const DISABLED_BORDER: [f32; 4] = [0.3, 0.3, 0.3, 0.6];
    const DANGER_BG: [f32; 4] = [0.5, 0.1, 0.1, 0.9];
    const DANGER_BORDER: [f32; 4] = [1.0, 0.3, 0.3, 1.0];

    fn colors(
        is_selected: bool,
        is_disabled: bool,
        is_danger: bool,
    ) -> ([f32; 4], [f32; 4], [f32; 4]) {
        const DEFAULT_TEXT: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const SELECTED_TEXT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const DISABLED_TEXT: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
        match (is_disabled, is_danger, is_selected) {
            (true, _, _) => (Self::DISABLED_BG, Self::DISABLED_BORDER, DISABLED_TEXT),
            (false, true, _) => (Self::DANGER_BG, Self::DANGER_BORDER, SELECTED_TEXT),
            (false, false, true) => (Self::SELECTED_BG, Self::SELECTED_BORDER, SELECTED_TEXT),
            (false, false, false) => (Self::DEFAULT_BG, Self::DEFAULT_BORDER, DEFAULT_TEXT),
        }
    }

    fn contains(&self, top_left: Vec2, point: Vec2) -> bool {
        point.x >= top_left.x
            && point.x <= top_left.x + self.size.x
            && point.y >= top_left.y
            && point.y <= top_left.y + self.size.y
    }

    /// Builds this button's rect and its centered label glyphs together -
    /// one call per button, since both derive from the same
    /// is_selected/is_disabled state and the same section_top_left.
    fn build(
        &self,
        top_left: Vec2,
        is_selected: bool,
        is_disabled: bool,
        is_danger: bool,
        font: &HashMap<char, TTFGlyph>,
    ) -> (SolidRect, Vec<PositionedTTFGlyph>) {
        let (fill_color, border_color, text_color) =
            Self::colors(is_selected, is_disabled, is_danger);
        let rect = SolidRect {
            position: top_left + self.size * 0.5,
            size: self.size,
            fill_color,
            border_color,
            border_thickness_px: 1.5,
        };
        let glyphs = self.build_label_glyphs(top_left, text_color, font);
        (rect, glyphs)
    }

    /// Lays out this button's label, then re-centers every glyph
    /// inside the button's actual rect - two steps because layout
    /// needs *some* origin to measure from before the real centered
    /// origin is known.
    fn build_label_glyphs(
        &self,
        top_left: Vec2,
        text_color: [f32; 4],
        font: &HashMap<char, TTFGlyph>,
    ) -> Vec<PositionedTTFGlyph> {
        let button_center = top_left + self.size * 0.5;
        let (mut glyphs, bounds) =
            text::layout_ttf_text(&self.label, font, Vec2::ZERO, 1.0, text_color);

        let bounds_center = bounds.min + Vec2::new(bounds.width(), bounds.height()) * 0.5;
        let shift = button_center - bounds_center;
        for glyph in &mut glyphs {
            glyph.position += shift;
        }
        glyphs
    }
}

pub struct TilesetControls {
    mode_buttons: Vec<Button>,
    action_buttons: Vec<Button>,
    danger_buttons: Vec<Button>,
}

impl TilesetControls {
    /// Lays out rows top-to-bottom, and within each row, buttons
    /// left-to-right — purely from each button's own size and
    /// BUTTON_GAP. Adding, removing, or resizing a button never
    /// requires touching anyone else's position; same "derive, don't
    /// hardcode" idea as stack_widgets, one level further in (buttons
    /// within a group, rather than widgets within a section).
    fn layout_offsets(rows: &[&[Button]]) -> Vec<Vec<Vec2>> {
        let mut cursor_y = INSPECTOR_SECTION_PADDING;
        rows.iter()
            .map(|row| {
                let mut cursor_x = INSPECTOR_SECTION_PADDING;
                let row_height = row.iter().map(|b| b.size.y).fold(0.0, f32::max);
                let offsets = row
                    .iter()
                    .map(|b| {
                        let offset = Vec2::new(cursor_x, cursor_y);
                        cursor_x += b.size.x + BUTTON_GAP;
                        offset
                    })
                    .collect();
                cursor_y += row_height + BUTTON_GAP;
                offsets
            })
            .collect()
    }

    /// Same row-shape as layout_offsets, so height and actual position
    /// can never disagree — same principle as InspectorSection's own
    /// required_height/stack_widgets pairing.
    fn required_height(&self) -> f32 {
        let rows = [
            self.mode_buttons.as_slice(),
            self.action_buttons.as_slice(),
            self.danger_buttons.as_slice(),
        ];
        let row_heights: f32 = rows
            .iter()
            .map(|row| row.iter().map(|b| b.size.y).fold(0.0, f32::max))
            .sum();
        INSPECTOR_SECTION_PADDING + row_heights + BUTTON_GAP * (rows.len() - 1) as f32
    }

    /// is_active: whether paint mode (show_tile_editor) is currently
    /// on - Save/Clear are meaningless with no session grid, so they
    /// render disabled otherwise. current_mode: AppState's live paint
    /// mode, for radio-highlight comparison. Neither is stored on this
    /// struct - both are supplied fresh, same as TilePalette never
    /// storing is_isometric.
    pub fn build(
        &self,
        section_top_left: Vec2,
        current_mode: &PaintMode,
        is_active: bool,
        font: &HashMap<char, TTFGlyph>,
    ) -> (Vec<SolidRect>, Vec<PositionedTTFGlyph>) {
        let offsets = Self::layout_offsets(&[
            &self.mode_buttons,
            &self.action_buttons,
            &self.danger_buttons,
        ]);
        let mut rects = Vec::new();
        let mut glyphs = Vec::new();

        for (button, offset) in self.mode_buttons.iter().zip(&offsets[0]) {
            let is_selected = is_active
                && matches!(
                    (button.id, current_mode),
                    ("place", PaintMode::Place) | ("remove", PaintMode::Remove)
                );
            let (rect, label_glyphs) = button.build(
                section_top_left + *offset,
                is_selected,
                !is_active,
                false,
                font,
            );
            rects.push(rect);
            glyphs.extend(label_glyphs);
        }
        for (button, offset) in self.action_buttons.iter().zip(&offsets[1]) {
            let (rect, label_glyphs) =
                button.build(section_top_left + *offset, false, !is_active, false, font);
            rects.push(rect);
            glyphs.extend(label_glyphs);
        }
        for (button, offset) in self.danger_buttons.iter().zip(&offsets[2]) {
            let (rect, label_glyphs) =
                button.build(section_top_left + *offset, false, !is_active, true, font); // is_danger = true
            rects.push(rect);
            glyphs.extend(label_glyphs);
        }
        (rects, glyphs)
    }

    pub fn handle_click(
        &self,
        mouse_pos: Vec2,
        section_top_left: Vec2,
        is_active: bool,
    ) -> TilesetAction {
        if !is_active {
            return TilesetAction::None;
        }
        let offsets = Self::layout_offsets(&[
            &self.mode_buttons,
            &self.action_buttons,
            &self.danger_buttons,
        ]);

        for (button, offset) in self.mode_buttons.iter().zip(&offsets[0]) {
            if button.contains(section_top_left + *offset, mouse_pos) {
                return match button.id {
                    "place" => TilesetAction::SetMode(PaintMode::Place),
                    "remove" => TilesetAction::SetMode(PaintMode::Remove),
                    _ => TilesetAction::None,
                };
            }
        }
        for (button, offset) in self.action_buttons.iter().zip(&offsets[1]) {
            if button.contains(section_top_left + *offset, mouse_pos) {
                return match button.id {
                    "save" => TilesetAction::Save,
                    "revert" => TilesetAction::Revert,
                    _ => TilesetAction::None,
                };
            }
        }
        for (button, offset) in self.danger_buttons.iter().zip(&offsets[2]) {
            if button.contains(section_top_left + *offset, mouse_pos) {
                return match button.id {
                    "clear_all" => TilesetAction::ClearAll,
                    _ => TilesetAction::None,
                };
            }
        }
        TilesetAction::None
    }
}

const HOTKEYS: &[(&str, &str)] = &[
    ("WASD", "Move"),
    ("E", "Interact / Advance dialogue"),
    ("Space", "Advance dialogue"),
    ("F1", "Toggle debug info"),
    ("F2", "Toggle colliders"),
    ("F3", "Toggle debug renderer"),
    ("F4", "Toggle grid"),
    ("F5", "Toggle player neighbours"),
    ("F6", "Toggle occupied cells"),
    ("F8", "Toggle tile editor"),
    ("F10", "Toggle isometric mode"),
    ("F11", "Toggle player collider"),
    ("F12", "Toggle inspector"),
    ("Ctrl+R", "Reset scene"),
    ("Numpad 8/2", "Grid display cell size +/-"),
    ("Esc", "Quit"),
];
const HOTKEY_LINE_HEIGHT: f32 = 18.0;

pub struct HotkeyList;

fn wrap_text(text: &str, max_width: f32, font: &HashMap<char, TTFGlyph>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        let (_, bounds) = text::layout_ttf_text(&candidate, font, Vec2::ZERO, 1.0, [1.0; 4]);
        if bounds.width() > max_width && !current.is_empty() {
            lines.push(std::mem::replace(&mut current, word.to_string()));
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

impl HotkeyList {
    /// Same wrapping as before, but each line now carries how many
    /// leading bytes belong to "key:" (None for continuation lines
    /// that are pure wrapped description, with no key prefix at all).
    /// Computed once here, at the point the key is actually known -
    /// not re-derived later by matching text back against HOTKEYS.
    fn wrapped_entries(
        &self,
        content_width: f32,
        font: &HashMap<char, TTFGlyph>,
    ) -> Vec<(Option<usize>, String)> {
        let usable_width = content_width - 2.0 * INSPECTOR_SECTION_PADDING;
        HOTKEYS
            .iter()
            .flat_map(|(key, desc)| {
                let full = format!("{key}: {desc}");
                let key_prefix_len = key.len() + 1; // covers "key:" - all HOTKEYS entries are ASCII, byte len == char count
                wrap_text(&full, usable_width, font)
                    .into_iter()
                    .enumerate()
                    .map(move |(i, line)| (if i == 0 { Some(key_prefix_len) } else { None }, line))
            })
            .collect()
    }

    fn required_height(&self, content_width: f32, font: &HashMap<char, TTFGlyph>) -> f32 {
        self.wrapped_entries(content_width, font).len() as f32 * HOTKEY_LINE_HEIGHT
            + 4.0 * INSPECTOR_SECTION_PADDING
    }

    fn build_label_glyphs(
        &self,
        section_top_left: Vec2,
        content_width: f32,
        font: &HashMap<char, TTFGlyph>,
    ) -> Vec<PositionedTTFGlyph> {
        const KEY_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0]; // black
        const DESC_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // white

        let mut glyphs = Vec::new();
        for (i, (key_len, line)) in self.wrapped_entries(content_width, font).iter().enumerate() {
            let origin = section_top_left
                + Vec2::new(INSPECTOR_SECTION_PADDING, INSPECTOR_SECTION_PADDING * 2.0)
                + Vec2::new(0.0, i as f32 * HOTKEY_LINE_HEIGHT);

            let colored_chars = match key_len {
                Some(split) => text::flatten_colored_spans(&[
                    (&line[..*split], KEY_COLOR),
                    (&line[*split..], DESC_COLOR),
                ]),
                None => text::flatten_colored_spans(&[(line.as_str(), DESC_COLOR)]),
            };

            glyphs.extend(text::layout_colored_ttf_text(
                &colored_chars,
                font,
                origin,
                0.75,
            ));
        }
        glyphs
    }
}

pub struct Scrollbar {
    dragging: bool,
}

impl Scrollbar {
    pub fn new() -> Self {
        Self { dragging: false }
    }

    fn thumb_rect(
        &self,
        track_bounds: Rect,
        scroll_offset: f32,
        max_scroll: f32,
        total_content_height: f32,
    ) -> Option<Rect> {
        if max_scroll <= 0.0 {
            return None;
        }
        let track_height = track_bounds.half_size.y * 2.0;
        const MIN_THUMB_HEIGHT: f32 = 24.0;
        let thumb_height = (track_height * (track_height / total_content_height))
            .clamp(MIN_THUMB_HEIGHT, track_height);
        let travel = track_height - thumb_height;
        let t = (scroll_offset / max_scroll).clamp(0.0, 1.0);
        let thumb_top = (track_bounds.center.y - track_bounds.half_size.y) + t * travel;
        Some(Rect {
            center: Vec2::new(track_bounds.center.x, thumb_top + thumb_height * 0.5),
            half_size: Vec2::new(track_bounds.half_size.x, thumb_height * 0.5),
        })
    }

    pub fn build_rects(
        &self,
        track_bounds: Rect,
        scroll_offset: f32,
        max_scroll: f32,
        total_content_height: f32,
    ) -> Vec<SolidRect> {
        let mut rects = vec![SolidRect {
            position: track_bounds.center,
            size: track_bounds.half_size * 2.0,
            fill_color: [0.1, 0.1, 0.1, 0.6],
            border_color: [0.05, 0.05, 0.05, 1.0],
            border_thickness_px: 1.0,
        }];
        if let Some(thumb) = self.thumb_rect(
            track_bounds,
            scroll_offset,
            max_scroll,
            total_content_height,
        ) {
            rects.push(SolidRect {
                position: thumb.center,
                size: thumb.half_size * 2.0,
                fill_color: [0.6, 0.6, 0.6, 0.9],
                border_color: [0.8, 0.8, 0.8, 1.0],
                border_thickness_px: 1.0,
            });
        }
        rects
    }

    /// Returns the new scroll_offset if the thumb was dragged this
    /// call, None otherwise - caller (Inspector) applies it, same
    /// "widget reports, caller acts" pattern as Slider::update's bool
    /// and TilesetControls::handle_click's TilesetAction.
    pub fn update(
        &mut self,
        track_bounds: Rect,
        mouse_pos: Vec2,
        mouse_down: bool,
        scroll_offset: f32,
        max_scroll: f32,
        total_content_height: f32,
    ) -> Option<f32> {
        let Some(thumb) = self.thumb_rect(
            track_bounds,
            scroll_offset,
            max_scroll,
            total_content_height,
        ) else {
            self.dragging = false;
            return None;
        };
        let track_height = track_bounds.half_size.y * 2.0;
        let thumb_height = thumb.half_size.y * 2.0;

        let over_thumb = mouse_pos.x >= thumb.center.x - thumb.half_size.x
            && mouse_pos.x <= thumb.center.x + thumb.half_size.x
            && mouse_pos.y >= thumb.center.y - thumb.half_size.y
            && mouse_pos.y <= thumb.center.y + thumb.half_size.y;

        if mouse_down && (self.dragging || over_thumb) {
            self.dragging = true;
            let track_top = track_bounds.center.y - track_bounds.half_size.y;
            let travel = track_height - thumb_height;
            let desired_thumb_top = mouse_pos.y - track_top - thumb_height * 0.5;
            let t = if travel > 0.0 {
                (desired_thumb_top / travel).clamp(0.0, 1.0)
            } else {
                0.0
            };
            Some(t * max_scroll)
        } else {
            self.dragging = false;
            None
        }
    }
}
