//! # XDL FFI
//!
//! Foreign Function Interface for integrating with external libraries

pub mod gsl;
pub mod hdf5;
pub mod netcdf;

pub struct FfiManager {
    // TODO: FFI management
}

impl FfiManager {
    pub fn new() -> Self {
        Self {
            // TODO: Initialize FFI manager
        }
    }
}

impl Default for FfiManager {
    fn default() -> Self {
        Self::new()
    }
}
