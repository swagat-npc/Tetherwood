use crate::engine::app::input::InputState;
use glam::Vec2;
use winit::keyboard::KeyCode;

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
pub fn resolve_movement(input: &InputState, is_isometric: bool) -> Vec2 {
    let w = input.is_held(KeyCode::KeyW);
    let s = input.is_held(KeyCode::KeyS);
    let a = input.is_held(KeyCode::KeyA);
    let d = input.is_held(KeyCode::KeyD);

    if is_isometric {
        resolve_isometric_movement(w, a, s, d)
    } else {
        resolve_flat_movement(w, a, s, d)
    }
}

fn resolve_flat_movement(w: bool, a: bool, s: bool, d: bool) -> Vec2 {
    let mut dir = Vec2::ZERO;
    if w {
        dir.y -= 1.0;
    }
    if s {
        dir.y += 1.0;
    }
    if a {
        dir.x -= 1.0;
    }
    if d {
        dir.x += 1.0;
    }
    if dir == Vec2::ZERO {
        Vec2::ZERO
    } else {
        dir.normalize()
    }
}

/// Isometric control scheme: a single key looks screen-cardinal
/// (matches straight up/down/left/right on screen) and moves along a
/// world-space 45deg diagonal; two keys together look grid-diagonal
/// on screen (matching the isometric tile edges, for diagonal sprite
/// art) and move along a single world axis. Not the pure mathematical
/// inverse of the projection for the two-key case - a deliberate
/// control-scheme/art choice, hence a hand-authored table rather than
/// one formula covering all 8 cases.
fn resolve_isometric_movement(w: bool, a: bool, s: bool, d: bool) -> Vec2 {
    match (w, a, s, d) {
        (true, false, false, true) => Vec2::new(0.0, -1.0), // W+D
        (true, true, false, false) => Vec2::new(-1.0, 0.0), // W+A
        (false, false, true, true) => Vec2::new(1.0, 0.0),  // S+D
        (false, true, true, false) => Vec2::new(0.0, 1.0),  // S+A
        (true, false, false, false) => Vec2::new(-0.7071068, -0.7071068), // W
        (false, false, true, false) => Vec2::new(0.7071068, 0.7071068), // S
        (false, true, false, false) => Vec2::new(-0.7071068, 0.7071068), // A
        (false, false, false, true) => Vec2::new(0.7071068, -0.7071068), // D
        _ => Vec2::ZERO, // no keys, or conflicting opposite pairs (W+S, A+D)
    }
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
