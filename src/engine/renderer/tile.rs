use glam::Vec2;
use std::collections::HashMap;

use crate::engine::{grid::CELL_SIZE, renderer::Renderer};

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
        ("flat-floor", (4, 0)),
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
    let world_pos = world_at_cell(cell, multiplying_factor);
    // This offset is to render the tile's top on this cell
    // instead of the tile's bottom
    let placement_offset = if is_isometric { 0.0 } else { 0.5 };
    world_pos + Vec2::splat(placement_offset) * TILE_WORLD_SIZE * multiplying_factor
}

// Gets the pixel size from grid size
pub fn grid_to_pixel(grid_size: (f32, f32)) -> Vec2 {
    Vec2::new(
        grid_size.0 * TILE_WORLD_SIZE.x,
        grid_size.1 * TILE_WORLD_SIZE.y,
    )
}

pub fn world_at_cell(cell: (i32, i32), multiplying_factor: f32) -> Vec2 {
    Vec2::new(
        (cell.0 as f32) * TILE_WORLD_SIZE.x,
        (cell.1 as f32) * TILE_WORLD_SIZE.y,
    ) * multiplying_factor
}

/// Inverse of tile_world_position: given a world-space point, which
/// cell contains it. Same offset constants (1.0 iso / 0.5 flat) as
/// the forward function, since this is that formula solved for `cell`.
pub fn cell_at_position(world_pos: Vec2, multiplying_factor: f32) -> (i32, i32) {
    let cell_f = (world_pos / multiplying_factor) / TILE_WORLD_SIZE;
    (cell_f.x.floor() as i32, cell_f.y.floor() as i32)
}

/// Height (in grid cells) of a footprint's flat "floor" shape — the
/// larger of its two grid dimensions. Used both to size a block's
/// render_size (debug.rs) and to derive how much of an entity's
/// render height is "extra Z" beyond its footprint (spawn_entity's
/// isometric anchor correction).
pub fn isometric_footprint_base_height(base_grid_x: f32, base_grid_y: f32) -> f32 {
    base_grid_x.max(base_grid_y)
}

/// Extra isometric "lip" padding a footprint of this grid-unit shape
/// needs so a cube-shaped block's top face renders flush — square
/// footprints need a full cell, odd-sum rectangular footprints need
/// half a cell, even-sum rectangular footprints need none.
pub fn isometric_footprint_padding(base_grid_x: f32, base_grid_y: f32) -> f32 {
    if base_grid_x == base_grid_y {
        CELL_SIZE
    } else if (base_grid_x + base_grid_y) % 2.0 != 0.0 {
        CELL_SIZE * 0.5
    } else {
        0.0
    }
}

pub fn tile_entry(
    cell: (i32, i32),
    atlas_cell: (i32, i32),
    multiplying_factor: f32,
    is_isometric: bool,
    renderer: &Renderer,
) -> (Vec2, (i32, i32), f32) {
    let depth = (cell.0 + cell.1) as f32;
    let world_pos = tile_world_position(cell, multiplying_factor, is_isometric);
    let effective_pos = if is_isometric {
        renderer.shear(world_pos)
    } else {
        world_pos
    };

    (effective_pos, atlas_cell, depth)
}
