use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// Tracks which physical keys are currently held. Knows nothing about
/// what any key means — that's game::actions' job. Engine only ever
/// answers "is this KeyCode down right now."
#[derive(Default)]
pub struct InputState {
    held: HashSet<KeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn press(&mut self, code: KeyCode) {
        self.held.insert(code);
    }

    pub fn release(&mut self, code: KeyCode) {
        self.held.remove(&code);
    }

    pub fn is_held(&self, code: KeyCode) -> bool {
        self.held.contains(&code)
    }
}
