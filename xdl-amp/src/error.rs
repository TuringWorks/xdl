//! Error types for GPU operations

use thiserror::Error;

pub type Result<T> = std::result::Result<T, GpuError>;

#[derive(Error, Debug)]
pub enum GpuError {
    #[error("GPU device not found")]
    DeviceNotFound,

    #[error("Unsupported backend: {0}")]
    UnsupportedBackend(String),

    #[error("Failed to create GPU buffer: {0}")]
    BufferCreationFailed(String),

    #[error("Failed to compile shader/kernel: {0}")]
    CompilationFailed(String),

    #[error("Failed to execute GPU operation: {0}")]
    ExecutionFailed(String),

    #[error("Buffer size mismatch: expected {expected}, got {actual}")]
    BufferSizeMismatch { expected: usize, actual: usize },

    #[error("Invalid buffer access")]
    InvalidBufferAccess,

    #[error("Out of GPU memory")]
    OutOfMemory,

    #[error("Platform-specific error: {0}")]
    PlatformError(String),

    #[cfg(target_os = "macos")]
    #[error("Metal error: {0}")]
    MetalError(String),

    #[cfg(target_os = "windows")]
    #[error("DirectX error: {0}")]
    DirectXError(String),

    #[cfg(feature = "cuda")]
    #[error("CUDA error: {0}")]
    CudaError(String),

    #[cfg(feature = "opencl")]
    #[error("OpenCL error: {0}")]
    OpenCLError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Metal doesn't expose MTLError directly, errors are handled via Result types

#[cfg(feature = "opencl")]
impl From<ocl::Error> for GpuError {
    fn from(err: ocl::Error) -> Self {
        GpuError::OpenCLError(err.to_string())
    }
}
