use glam::Vec2;
use std::collections::HashMap;

pub const TILE_SIZE: Vec2 = Vec2::new(32.0, 32.0);
pub const TILE_ATLAS_SIZE: Vec2 = Vec2::new(352.0, 352.0);

/// Human-readable names for atlas cells, authoring convenience only —
/// never touches the eventual save file (ADR-097's format stores raw
/// small integers/cells, not names).
pub fn tile_names() -> HashMap<&'static str, (i32, i32)> {
    HashMap::from([
        ("floor", (0, 0)),
        ("half-floor", (4, 0)),
        ("staircase-right", (4, 2)),
        ("staircase-left", (6, 2)),
    ])
}

/// UV-space (0..1) top-left and bottom-right corners for a tile's
/// atlas cell, given in 32px-pixel-grid coordinates (col, row) — same
/// role as text::glyph_uv, applied to the tile atlas instead of the
/// font atlas.
pub fn tile_uv(cell: (i32, i32)) -> (Vec2, Vec2) {
    let pixel_pos = Vec2::new(cell.0 as f32 * TILE_SIZE.x, cell.1 as f32 * TILE_SIZE.y);
    (
        pixel_pos / TILE_ATLAS_SIZE,
        (pixel_pos + TILE_SIZE) / TILE_ATLAS_SIZE,
    )
}

pub fn tile_world_position(cell: (i32, i32), multiplying_factor: f32) -> Vec2 {
    Vec2::new(cell.0 as f32 * TILE_SIZE.x, cell.1 as f32 * TILE_SIZE.y) * multiplying_factor
}
