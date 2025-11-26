//! # XDL AMP (Accelerated Math Processing)
//!
//! Multi-backend GPU/ML acceleration for XDL:
//!
//! ## Apple Platforms (macOS, iOS)
//! - **Metal** - Low-level GPU compute
//! - **Metal Performance Shaders (MPS)** - Optimized operations
//! - **CoreML** - Neural Engine acceleration
//!
//! ## Windows
//! - **DirectX 12** - GPU compute shaders
//! - **DirectML** - ML acceleration on DirectX (any GPU)
//! - **CUDA/cuDNN** - NVIDIA GPUs
//! - **Vulkan** - Cross-platform GPU compute
//!
//! ## Linux
//! - **CUDA/cuDNN** - NVIDIA GPUs
//! - **ROCm** - AMD GPUs
//! - **Vulkan** - Cross-platform GPU compute
//! - **OpenCL** - Cross-platform fallback
//!
//! ## Cross-Platform
//! - **Vulkan** - Cross-platform GPU compute
//! - **ONNX Runtime** - ML model inference

pub mod backend;
pub mod error;
pub mod ops;

// Apple backends
#[cfg(target_os = "macos")]
pub mod metal;

#[cfg(all(target_os = "macos", feature = "mps"))]
pub mod mps;

#[cfg(all(target_os = "macos", feature = "coreml"))]
pub mod coreml;

// Windows backends
#[cfg(target_os = "windows")]
pub mod directx;

#[cfg(all(target_os = "windows", feature = "directml"))]
pub mod directml;

// Linux backends
#[cfg(target_os = "linux")]
pub mod opencl;

#[cfg(feature = "rocm")]
pub mod rocm;

// NVIDIA backends (cross-platform)
#[cfg(feature = "cuda")]
pub mod cuda;

#[cfg(feature = "cudnn")]
pub mod cudnn;

// Cross-platform backends
#[cfg(feature = "vulkan")]
pub mod vulkan;

#[cfg(feature = "onnx")]
pub mod onnx;

pub use backend::{GpuBackend, GpuBuffer, GpuDevice};
pub use error::{GpuError, Result};

use std::sync::Arc;

/// GPU compute context that automatically selects the best available backend
pub struct GpuContext {
    device: Arc<dyn GpuDevice>,
    backend_name: String,
}

impl GpuContext {
    /// Create a new GPU context with automatic backend selection
    pub fn new() -> Result<Self> {
        Self::with_preference(None)
    }

    /// Create a GPU context with a preferred backend
    pub fn with_preference(preference: Option<GpuBackend>) -> Result<Self> {
        let backend = preference.unwrap_or_else(Self::default_backend);

        match backend {
            // Apple backends
            #[cfg(target_os = "macos")]
            GpuBackend::Metal => {
                let device = metal::MetalDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "Metal".to_string(),
                })
            }

            #[cfg(all(target_os = "macos", feature = "mps"))]
            GpuBackend::MetalPerformanceShaders => {
                let device = mps::MPSDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "Metal Performance Shaders".to_string(),
                })
            }

            #[cfg(all(target_os = "macos", feature = "coreml"))]
            GpuBackend::CoreML => {
                let device = coreml::CoreMLDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "CoreML/ANE".to_string(),
                })
            }

            // Windows backends
            #[cfg(target_os = "windows")]
            GpuBackend::DirectX12 => {
                let device = directx::DirectXDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "DirectX 12".to_string(),
                })
            }

            #[cfg(all(target_os = "windows", feature = "directml"))]
            GpuBackend::DirectML => {
                let device = directml::DirectMLDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "DirectML".to_string(),
                })
            }

            // NVIDIA backends
            #[cfg(feature = "cuda")]
            GpuBackend::Cuda => {
                let device = cuda::CudaDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "CUDA".to_string(),
                })
            }

            #[cfg(feature = "cudnn")]
            GpuBackend::CuDNN => {
                let device = cudnn::CuDNNDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "cuDNN".to_string(),
                })
            }

            // AMD backends
            #[cfg(feature = "rocm")]
            GpuBackend::ROCm => {
                let device = rocm::ROCmDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "ROCm".to_string(),
                })
            }

            // Cross-platform backends
            #[cfg(feature = "opencl")]
            GpuBackend::OpenCL => {
                let device = opencl::OpenCLDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "OpenCL".to_string(),
                })
            }

            #[cfg(feature = "vulkan")]
            GpuBackend::Vulkan => {
                let device = vulkan::VulkanDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "Vulkan".to_string(),
                })
            }

            #[cfg(feature = "onnx")]
            GpuBackend::OnnxRuntime => {
                let device = onnx::OnnxDevice::new()?;
                Ok(Self {
                    device: Arc::new(device),
                    backend_name: "ONNX Runtime".to_string(),
                })
            }

            _ => Err(GpuError::UnsupportedBackend(format!(
                "Backend {:?} not available on this platform or not enabled",
                backend
            ))),
        }
    }

    /// Get the default backend for the current platform
    /// Priority order:
    /// - macOS: MPS > Metal > CoreML
    /// - Windows: cuDNN > CUDA > DirectML > DirectX12
    /// - Linux: cuDNN > CUDA > ROCm > OpenCL
    #[cfg(target_os = "macos")]
    fn default_backend() -> GpuBackend {
        // Prefer MPS for optimized operations on Apple Silicon
        #[cfg(feature = "mps")]
        if mps::MPSDevice::is_available() {
            return GpuBackend::MetalPerformanceShaders;
        }

        // Fallback to base Metal
        GpuBackend::Metal
    }

    #[cfg(target_os = "windows")]
    fn default_backend() -> GpuBackend {
        // Prefer cuDNN for ML workloads
        #[cfg(feature = "cudnn")]
        if cudnn::CuDNNDevice::is_available() {
            return GpuBackend::CuDNN;
        }

        // Then CUDA for NVIDIA GPUs
        #[cfg(feature = "cuda")]
        if cuda::CudaDevice::is_available() {
            return GpuBackend::Cuda;
        }

        // DirectML for ML on any GPU (NVIDIA, AMD, Intel)
        #[cfg(feature = "directml")]
        if directml::DirectMLDevice::is_available() {
            return GpuBackend::DirectML;
        }

        // Vulkan for cross-platform GPU compute
        #[cfg(feature = "vulkan")]
        if vulkan::VulkanDevice::is_available() {
            return GpuBackend::Vulkan;
        }

        // Default to DirectX 12
        GpuBackend::DirectX12
    }

    #[cfg(target_os = "linux")]
    fn default_backend() -> GpuBackend {
        // Prefer cuDNN for ML workloads
        #[cfg(feature = "cudnn")]
        if cudnn::CuDNNDevice::is_available() {
            return GpuBackend::CuDNN;
        }

        // Then CUDA for NVIDIA GPUs
        #[cfg(feature = "cuda")]
        if cuda::CudaDevice::is_available() {
            return GpuBackend::Cuda;
        }

        // ROCm for AMD GPUs
        #[cfg(feature = "rocm")]
        if rocm::ROCmDevice::is_available() {
            return GpuBackend::ROCm;
        }

        // Vulkan for cross-platform support
        #[cfg(feature = "vulkan")]
        if vulkan::VulkanDevice::is_available() {
            return GpuBackend::Vulkan;
        }

        // OpenCL fallback
        #[cfg(feature = "opencl")]
        return GpuBackend::OpenCL;

        #[cfg(not(any(feature = "cuda", feature = "rocm", feature = "opencl")))]
        panic!("No GPU backend available on Linux. Enable 'cuda', 'rocm', or 'opencl' feature.");
    }

    /// Get the name of the active backend
    pub fn backend_name(&self) -> &str {
        &self.backend_name
    }

    /// Get the device handle
    pub fn device(&self) -> &Arc<dyn GpuDevice> {
        &self.device
    }
}

impl Default for GpuContext {
    fn default() -> Self {
        Self::new().expect("Failed to create GPU context")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_context_creation() {
        let ctx = GpuContext::new();
        assert!(ctx.is_ok(), "Failed to create GPU context");
        if let Ok(ctx) = ctx {
            println!("Using GPU backend: {}", ctx.backend_name());
        }
    }
}
