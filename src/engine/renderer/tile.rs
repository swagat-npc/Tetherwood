use glam::Vec2;
use std::collections::HashMap;

pub const TILE_ATLAS_SIZE: Vec2 = Vec2::new(198.0, 352.0);
pub const TILE_PIXEL_SIZE: f32 = 32.0;
pub const FLAT_TILE_PIXEL_SIZE: Vec2 = Vec2::new(16.0, 16.0);
pub const TILE_WORLD_SIZE: Vec2 = Vec2::new(16.0, 16.0); // authoring-unit size when drawn/placed in a scene

pub const TILE_PITCH_X: f32 = 33.0; // 32px tile + 1px horizontal gap
pub const TILE_PITCH_Y: f32 = 32.0; // no vertical gap between a tile's iso/flat row-pair and the next

/// Human-readable names for atlas cells, authoring convenience only —
/// never touches the eventual save file (ADR-097's format stores raw
/// small integers/cells, not names). Sequential indices now - no
/// skipped columns, since gap columns were only ever a workaround for
/// faking a 64px pitch with 32px-multiple arithmetic.
pub fn tile_names() -> HashMap<&'static str, (i32, i32)> {
    HashMap::from([
        ("floor", (0, 0)),
        ("half-floor", (2, 0)),
        ("staircase-right", (2, 1)),
        ("staircase-left", (3, 1)),
    ])
}

/// Isometric UV lookup — each tile occupies raw atlas row `cell.1 * 2`.
fn tile_uv_iso(cell: (i32, i32)) -> (Vec2, Vec2) {
    let pixel_pos = Vec2::new(
        cell.0 as f32 * TILE_PITCH_X,
        (cell.1 * 2) as f32 * TILE_PITCH_Y,
    );
    (
        pixel_pos / TILE_ATLAS_SIZE,
        (pixel_pos + Vec2::splat(TILE_PIXEL_SIZE)) / TILE_ATLAS_SIZE,
    )
}

/// Flat UV lookup — derived from the iso cell, not a separate table.
/// By convention, every tile's flat 16x16 variant sits at raw atlas
/// row `cell.1 * 2 + 1`, centered within that 32px-tall row (an 8px
/// margin on every side, which also handles bleed prevention
/// automatically as long as this convention holds).
fn tile_uv_flat(cell: (i32, i32)) -> (Vec2, Vec2) {
    let block_origin = Vec2::new(
        cell.0 as f32 * TILE_PITCH_X,
        (cell.1 * 2 + 1) as f32 * TILE_PITCH_Y,
    );
    let inset = (Vec2::splat(TILE_PIXEL_SIZE) - FLAT_TILE_PIXEL_SIZE) * 0.5;
    let pixel_pos = block_origin + inset;
    (
        pixel_pos / TILE_ATLAS_SIZE,
        (pixel_pos + FLAT_TILE_PIXEL_SIZE) / TILE_ATLAS_SIZE,
    )
}

/// Dispatches to the iso or flat lookup — same cell coordinate either
/// way, since the flat variant's position is always derived from it.
pub fn tile_uv(cell: (i32, i32), is_isometric: bool) -> (Vec2, Vec2) {
    if is_isometric {
        tile_uv_iso(cell)
    } else {
        tile_uv_flat(cell)
    }
}

pub fn tile_world_position(cell: (i32, i32), multiplying_factor: f32, is_isometric: bool) -> Vec2 {
    let offset = if is_isometric { 1.0 } else { 0.5 };
    Vec2::new(
        (cell.0 as f32 + offset) * TILE_WORLD_SIZE.x,
        (cell.1 as f32 + offset) * TILE_WORLD_SIZE.y,
    ) * multiplying_factor
}

/// Inverse of tile_world_position: given a world-space point, which
/// cell contains it. Same offset constants (1.0 iso / 0.5 flat) as
/// the forward function, since this is that formula solved for `cell`.
pub fn cell_at_position(
    world_pos: Vec2,
    multiplying_factor: f32,
    is_isometric: bool,
) -> (i32, i32) {
    let offset = if is_isometric { 1.0 } else { 0.5 };
    let cell_f = (world_pos / multiplying_factor) / TILE_WORLD_SIZE - Vec2::splat(offset);
    (cell_f.x.floor() as i32, cell_f.y.floor() as i32)
}
