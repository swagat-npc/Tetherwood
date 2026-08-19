use std::collections::HashMap;

pub struct ProgressionTracker {
    flags: HashMap<&'static str, bool>,
}

impl ProgressionTracker {
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
        }
    }

    pub fn set(&mut self, flag: &'static str, value: bool) {
        self.flags.insert(flag, value);
    }

    pub fn is_set(&self, flag: &'static str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }
}
