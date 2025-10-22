//! GSL (GNU Scientific Library) interface

pub struct GslInterface;

impl GslInterface {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GslInterface {
    fn default() -> Self {
        Self::new()
    }
}
