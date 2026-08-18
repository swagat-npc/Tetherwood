use crate::engine::entity::{EntityId, Rect};
use crate::engine::scene::WallId;
use glam::Vec2;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionHandle {
    Wall(WallId),
    Entity(EntityId),
}

pub const CELL_SIZE: f32 = 64.0;

pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<CollisionHandle>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Adds a collider's occupied cell(s) to the grid, filed under `handle`.
    /// A rect wider than one cell is filed under every cell it touches.
    /// Querying any of those cells later will find it.
    pub fn insert(&mut self, rect: &Rect, handle: CollisionHandle) {
        for cell in self.cells_for_rect(rect) {
            self.cells.entry(cell).or_insert_with(Vec::new).push(handle);
        }
    }

    /// The set of cells a rect's bounding box spans. A wall wider than
    /// one cell needs to be found from any of them, not just its center.
    fn cells_for_rect(&self, rect: &Rect) -> Vec<(i32, i32)> {
        let min = rect.center - rect.half_size;
        let max = rect.center + rect.half_size;

        let min_cell = self.cell_at_position(min);
        let max_cell = self.cell_at_position(max);

        let mut result = Vec::new();
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                result.push((cx, cy));
            }
        }
        result
    }

    /// Return which cell a world position falls into.
    /// The grid's basic unit of "where," everything else builds on this.
    pub fn cell_at_position(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    /// Every cell coordinate within `radius` of `cell`, occupied or not.
    /// Pure geometry, no lookup. Debug visualization uses this directly to
    /// draw a highlight even over empty space; collision queries use it as
    /// a first step before checking which of those cells actually hold anything.
    pub fn neighboring_cells(&self, cell: (i32, i32), radius: i32) -> Vec<(i32, i32)> {
        let mut result = Vec::new();
        for cx in (cell.0 - radius)..=(cell.0 + radius) {
            for cy in (cell.1 - radius)..=(cell.1 + radius) {
                result.push((cx, cy));
            }
        }
        result
    }

    /// Everything actually stored in `cell`'s neighborhood. This is the
    /// collision-relevant narrowing of `neighboring_cells`, skipping any that are empty.
    fn collision_handles_around_cell(
        &self,
        cell: (i32, i32),
        radius: i32,
    ) -> HashSet<CollisionHandle> {
        let mut results = HashSet::new();
        for c in self.neighboring_cells(cell, radius) {
            if let Some(handles) = self.cells.get(&c) {
                results.extend(handles.iter().copied());
            }
        }
        results
    }

    /// Entry point for collision checks: nearby walls/entities around a
    /// world position, without the caller needing to think in cells at all.
    pub fn collision_handles_around_position(
        &self,
        pos: Vec2,
        radius: i32,
    ) -> HashSet<CollisionHandle> {
        self.collision_handles_around_cell(self.cell_at_position(pos), radius)
    }

    /// Cells with at least one collider filed under them right now. Reads
    /// the grid's current state, doesn't rebuild or rescan anything. Used by
    /// the debug overlay to show which cells are actually doing work.
    pub fn occupied_cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.cells.keys().copied()
    }

    /// The grid's tuning knob. Bigger cells mean fewer, cheaper lookups but
    /// coarser candidate lists; smaller cells mean the opposite. Tune by feel.
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }
}
