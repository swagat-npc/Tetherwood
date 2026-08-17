use glam::Vec2;
use winit::keyboard::KeyCode;

use crate::engine::platform::input::InputState;

/// Discrete, press-triggered things a player can do. Movement doesn't
/// belong here — it's continuous held-state, not a single press event
/// (see resolve_movement). Engine knows nothing about this enum; a
/// different game only needs to edit this file to change what E/Space
/// mean (ADR-035).
pub enum Action {
    Interact,
    AdvanceOrSkip,
}

/// Held-state movement axes → a single vector. Same WASD checks the
/// code already had, just named and relocated — this is the concrete
/// binding table ADR-035 deferred building an abstraction around.
pub fn resolve_movement(input: &InputState) -> Vec2 {
    let mut movement = Vec2::ZERO;
    if input.is_held(KeyCode::KeyW) {
        movement.y -= 1.0;
    }
    if input.is_held(KeyCode::KeyS) {
        movement.y += 1.0;
    }
    if input.is_held(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if input.is_held(KeyCode::KeyD) {
        movement.x += 1.0;
    }
    movement
}

/// Resolves one key press into at most one Action. `KeyE` alone is
/// ambiguous — it starts an interaction with no dialogue active, or
/// advances/skips one that already is — so the caller's current state,
/// not just the key, decides which action applies. Same dispatch the
/// original inline handler already had.
pub fn resolve_key_press(code: KeyCode, dialogue_active: bool) -> Option<Action> {
    match code {
        KeyCode::KeyE | KeyCode::Space if dialogue_active => Some(Action::AdvanceOrSkip),
        KeyCode::KeyE => Some(Action::Interact),
        _ => None,
    }
}
