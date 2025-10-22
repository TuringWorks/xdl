//! HDF5 interface

pub struct Hdf5Interface;

impl Hdf5Interface {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Hdf5Interface {
    fn default() -> Self {
        Self::new()
    }
}
