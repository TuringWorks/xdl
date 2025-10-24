//! Volume data structures and GPU upload

/// Volume data format
#[derive(Debug, Clone, Copy)]
pub enum VolumeFormat {
    F32,
    F16,
    U8,
}

/// 3D volume data container
pub struct VolumeData {
    pub data: Vec<f32>,
    pub dimensions: [usize; 3],
    pub format: VolumeFormat,
}

impl VolumeData {
    /// Create new volume from raw data
    pub fn new(data: Vec<f32>, dimensions: [usize; 3]) -> Self {
        Self {
            data,
            dimensions,
            format: VolumeFormat::F32,
        }
    }

    /// Get volume dimensions [width, height, depth]
    pub fn dimensions(&self) -> [usize; 3] {
        self.dimensions
    }

    /// Total number of voxels
    pub fn size(&self) -> usize {
        self.dimensions[0] * self.dimensions[1] * self.dimensions[2]
    }
}
