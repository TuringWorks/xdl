//! XDL Runtime System

pub mod gc;
pub mod memory;

pub struct RuntimeSystem {
    // TODO: Runtime implementation
}

impl RuntimeSystem {
    pub fn new() -> Self {
        Self {
            // TODO: Initialize runtime
        }
    }
}

impl Default for RuntimeSystem {
    fn default() -> Self {
        Self::new()
    }
}
