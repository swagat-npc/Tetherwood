pub mod info;
pub mod notifications;
pub mod overlay;
pub mod ui;

use crate::engine::grid;

pub struct DebugSettings {
    pub show_debug_info: bool,
    pub show_debug_renderer: bool,
    pub show_grid: bool,
    pub show_colliders: bool,
    pub show_player_neighbours: bool,
    pub show_occupied_cells: bool,
    pub grid_display_cell_size: f32,
    pub enable_player_collider: bool,
}

impl DebugSettings {
    pub fn new() -> Self {
        // DEBUG: All flags are supposed to be false by default, set to true for debugging
        Self {
            show_debug_info: false,
            show_debug_renderer: true,
            show_colliders: true,
            show_grid: true,
            show_player_neighbours: false,
            show_occupied_cells: false,
            grid_display_cell_size: grid::CELL_SIZE,
            enable_player_collider: true,
        }
    }

    pub fn toggle_debug_info(&mut self) -> &'static str {
        self.show_debug_info = !self.show_debug_info;
        if self.show_debug_info {
            "Debug Info: ON"
        } else {
            "Debug Info: OFF"
        }
    }

    pub fn toggle_debug_renderer(&mut self) -> &'static str {
        self.show_debug_renderer = !self.show_debug_renderer;
        if self.show_debug_renderer {
            "Debug Renderer: ON"
        } else {
            "Debug Renderer: OFF"
        }
    }

    pub fn toggle_colliders(&mut self) -> &'static str {
        self.show_colliders = !self.show_colliders;
        if self.show_colliders {
            "Colliders: ON"
        } else {
            "Colliders: OFF"
        }
    }

    pub fn toggle_player_collider(&mut self) -> &'static str {
        self.enable_player_collider = !self.enable_player_collider;
        if self.enable_player_collider {
            "Player Collider: ON"
        } else {
            "Player Collider: OFF"
        }
    }

    pub fn toggle_grid(&mut self) -> &'static str {
        self.show_grid = !self.show_grid;
        if self.show_grid {
            "Grid: ON"
        } else {
            "Grid: OFF"
        }
    }

    pub fn toggle_player_neighbours(&mut self) -> &'static str {
        self.show_player_neighbours = !self.show_player_neighbours;
        if self.show_player_neighbours {
            "Grid - Player Neighbours: ON"
        } else {
            "Grid - Player Neighbours: OFF"
        }
    }

    pub fn toggle_occupied_cells(&mut self) -> &'static str {
        self.show_occupied_cells = !self.show_occupied_cells;
        if self.show_occupied_cells {
            "Grid - Occupied Cells: ON"
        } else {
            "Grid - Occupied Cells: OFF"
        }
    }

    pub fn increase_grid_cell_size(&mut self) -> String {
        self.grid_display_cell_size = (self.grid_display_cell_size + 2.0).min(64.0);
        format!("Grid display cell size: {}", self.grid_display_cell_size)
    }

    pub fn decrease_grid_cell_size(&mut self) -> String {
        self.grid_display_cell_size = (self.grid_display_cell_size - 2.0).max(2.0);
        format!("Grid display cell size: {}", self.grid_display_cell_size)
    }
}
