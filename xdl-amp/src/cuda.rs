//! CUDA backend for NVIDIA GPU acceleration
//!
//! This backend provides hardware-accelerated compute operations using NVIDIA CUDA.
//! It supports all standard element-wise operations, matrix multiplication (GEMM),
//! and reduction operations.

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::{GpuError, Result};

#[cfg(feature = "cuda")]
use cudarc::driver::{CudaDevice as CudaDeviceHandle, CudaSlice, LaunchAsync, LaunchConfig};
#[cfg(feature = "cuda")]
use std::sync::Arc;

/// CUDA kernel source code for all operations
#[cfg(feature = "cuda")]
const CUDA_KERNELS: &str = r#"
// Element-wise binary operations
extern "C" __global__ void add_f32(const float* a, const float* b, float* c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        c[idx] = a[idx] + b[idx];
    }
}

extern "C" __global__ void mul_f32(const float* a, const float* b, float* c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        c[idx] = a[idx] * b[idx];
    }
}

extern "C" __global__ void sub_f32(const float* a, const float* b, float* c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        c[idx] = a[idx] - b[idx];
    }
}

extern "C" __global__ void div_f32(const float* a, const float* b, float* c, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        c[idx] = a[idx] / b[idx];
    }
}

// Element-wise unary operations
extern "C" __global__ void sin_f32(const float* x, float* y, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        y[idx] = sinf(x[idx]);
    }
}

extern "C" __global__ void cos_f32(const float* x, float* y, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        y[idx] = cosf(x[idx]);
    }
}

extern "C" __global__ void exp_f32(const float* x, float* y, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        y[idx] = expf(x[idx]);
    }
}

extern "C" __global__ void log_f32(const float* x, float* y, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        y[idx] = logf(x[idx]);
    }
}

extern "C" __global__ void sqrt_f32(const float* x, float* y, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        y[idx] = sqrtf(x[idx]);
    }
}

extern "C" __global__ void pow_f32(const float* x, float p, float* y, int n) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        y[idx] = powf(x[idx], p);
    }
}

// Reduction operations
extern "C" __global__ void sum_reduce_f32(const float* x, float* partial, int n) {
    extern __shared__ float sdata[];

    int tid = threadIdx.x;
    int idx = blockIdx.x * blockDim.x * 2 + threadIdx.x;

    // Load and perform first reduction during load
    float sum = 0.0f;
    if (idx < n) sum = x[idx];
    if (idx + blockDim.x < n) sum += x[idx + blockDim.x];
    sdata[tid] = sum;
    __syncthreads();

    // Reduction in shared memory
    for (int s = blockDim.x / 2; s > 0; s >>= 1) {
        if (tid < s) {
            sdata[tid] += sdata[tid + s];
        }
        __syncthreads();
    }

    if (tid == 0) partial[blockIdx.x] = sdata[0];
}

extern "C" __global__ void max_reduce_f32(const float* x, float* partial, int n) {
    extern __shared__ float sdata[];

    int tid = threadIdx.x;
    int idx = blockIdx.x * blockDim.x * 2 + threadIdx.x;

    float val = -3.402823466e+38f; // -FLT_MAX
    if (idx < n) val = x[idx];
    if (idx + blockDim.x < n) val = fmaxf(val, x[idx + blockDim.x]);
    sdata[tid] = val;
    __syncthreads();

    for (int s = blockDim.x / 2; s > 0; s >>= 1) {
        if (tid < s) {
            sdata[tid] = fmaxf(sdata[tid], sdata[tid + s]);
        }
        __syncthreads();
    }

    if (tid == 0) partial[blockIdx.x] = sdata[0];
}

extern "C" __global__ void min_reduce_f32(const float* x, float* partial, int n) {
    extern __shared__ float sdata[];

    int tid = threadIdx.x;
    int idx = blockIdx.x * blockDim.x * 2 + threadIdx.x;

    float val = 3.402823466e+38f; // FLT_MAX
    if (idx < n) val = x[idx];
    if (idx + blockDim.x < n) val = fminf(val, x[idx + blockDim.x]);
    sdata[tid] = val;
    __syncthreads();

    for (int s = blockDim.x / 2; s > 0; s >>= 1) {
        if (tid < s) {
            sdata[tid] = fminf(sdata[tid], sdata[tid + s]);
        }
        __syncthreads();
    }

    if (tid == 0) partial[blockIdx.x] = sdata[0];
}

// Tiled matrix multiplication (GEMM)
// C[M,N] = A[M,K] * B[K,N]
#define TILE_SIZE 16

extern "C" __global__ void matmul_f32(const float* A, const float* B, float* C, int M, int N, int K) {
    __shared__ float tileA[TILE_SIZE][TILE_SIZE];
    __shared__ float tileB[TILE_SIZE][TILE_SIZE];

    int row = blockIdx.y * TILE_SIZE + threadIdx.y;
    int col = blockIdx.x * TILE_SIZE + threadIdx.x;

    float sum = 0.0f;

    int numTiles = (K + TILE_SIZE - 1) / TILE_SIZE;

    for (int t = 0; t < numTiles; t++) {
        // Load tile from A
        int aCol = t * TILE_SIZE + threadIdx.x;
        if (row < M && aCol < K) {
            tileA[threadIdx.y][threadIdx.x] = A[row * K + aCol];
        } else {
            tileA[threadIdx.y][threadIdx.x] = 0.0f;
        }

        // Load tile from B
        int bRow = t * TILE_SIZE + threadIdx.y;
        if (bRow < K && col < N) {
            tileB[threadIdx.y][threadIdx.x] = B[bRow * N + col];
        } else {
            tileB[threadIdx.y][threadIdx.x] = 0.0f;
        }

        __syncthreads();

        // Compute partial dot product for this tile
        for (int k = 0; k < TILE_SIZE; k++) {
            sum += tileA[threadIdx.y][k] * tileB[k][threadIdx.x];
        }

        __syncthreads();
    }

    // Write result
    if (row < M && col < N) {
        C[row * N + col] = sum;
    }
}
"#;

/// CUDA GPU buffer wrapper
#[cfg(feature = "cuda")]
#[derive(Debug)]
pub struct CudaBuffer {
    data: CudaSlice<u8>,
    size: usize,
    device: Arc<CudaDeviceHandle>,
}

#[cfg(not(feature = "cuda"))]
#[derive(Debug)]
pub struct CudaBuffer {
    size: usize,
}

#[cfg(feature = "cuda")]
impl GpuBuffer for CudaBuffer {
    fn size(&self) -> usize {
        self.size
    }

    fn read_to_slice(&self, dst: &mut [u8]) -> Result<()> {
        if dst.len() != self.size {
            return Err(GpuError::BufferSizeMismatch {
                expected: self.size,
                actual: dst.len(),
            });
        }

        self.device
            .dtoh_sync_copy_into(&self.data, dst)
            .map_err(|e| GpuError::CudaError(format!("Failed to copy data from device: {}", e)))?;

        Ok(())
    }

    fn write_from_slice(&mut self, src: &[u8]) -> Result<()> {
        if src.len() != self.size {
            return Err(GpuError::BufferSizeMismatch {
                expected: self.size,
                actual: src.len(),
            });
        }

        self.device
            .htod_sync_copy_into(src, &mut self.data)
            .map_err(|e| GpuError::CudaError(format!("Failed to copy data to device: {}", e)))?;

        Ok(())
    }
}

#[cfg(not(feature = "cuda"))]
impl GpuBuffer for CudaBuffer {
    fn size(&self) -> usize {
        self.size
    }

    fn read_to_slice(&self, _dst: &mut [u8]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn write_from_slice(&mut self, _src: &[u8]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }
}

/// CUDA GPU device
#[cfg(feature = "cuda")]
pub struct CudaDevice {
    device: Arc<CudaDeviceHandle>,
    kernels_loaded: bool,
}

#[cfg(not(feature = "cuda"))]
pub struct CudaDevice {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(feature = "cuda")]
impl std::fmt::Debug for CudaDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CudaDevice")
            .field("kernels_loaded", &self.kernels_loaded)
            .finish()
    }
}

#[cfg(not(feature = "cuda"))]
impl std::fmt::Debug for CudaDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CudaDevice").finish()
    }
}

impl CudaDevice {
    /// Create a new CUDA device
    #[cfg(feature = "cuda")]
    pub fn new() -> Result<Self> {
        // CudaDeviceHandle::new() returns Arc<CudaDevice>
        let device = CudaDeviceHandle::new(0)
            .map_err(|e| GpuError::CudaError(format!("Failed to initialize CUDA device: {}", e)))?;

        // Compile and load kernels
        let ptx = cudarc::nvrtc::compile_ptx(CUDA_KERNELS).map_err(|e| {
            GpuError::CompilationFailed(format!("Failed to compile CUDA kernels: {}", e))
        })?;

        device
            .load_ptx(
                ptx,
                "xdl_kernels",
                &[
                    "add_f32",
                    "mul_f32",
                    "sub_f32",
                    "div_f32",
                    "sin_f32",
                    "cos_f32",
                    "exp_f32",
                    "log_f32",
                    "sqrt_f32",
                    "pow_f32",
                    "sum_reduce_f32",
                    "max_reduce_f32",
                    "min_reduce_f32",
                    "matmul_f32",
                ],
            )
            .map_err(|e| GpuError::CudaError(format!("Failed to load PTX module: {}", e)))?;

        Ok(Self {
            device,
            kernels_loaded: true,
        })
    }

    #[cfg(not(feature = "cuda"))]
    pub fn new() -> Result<Self> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    /// Check if CUDA is available
    #[cfg(feature = "cuda")]
    pub fn is_available() -> bool {
        CudaDeviceHandle::new(0).is_ok()
    }

    #[cfg(not(feature = "cuda"))]
    pub fn is_available() -> bool {
        false
    }

    #[cfg(feature = "cuda")]
    fn launch_config(n: usize) -> LaunchConfig {
        let block_size = 256;
        let grid_size = (n + block_size - 1) / block_size;
        LaunchConfig {
            block_dim: (block_size as u32, 1, 1),
            grid_dim: (grid_size as u32, 1, 1),
            shared_mem_bytes: 0,
        }
    }

    #[cfg(feature = "cuda")]
    fn launch_config_2d(m: usize, n: usize) -> LaunchConfig {
        let tile_size = 16;
        let grid_x = (n + tile_size - 1) / tile_size;
        let grid_y = (m + tile_size - 1) / tile_size;
        LaunchConfig {
            block_dim: (tile_size as u32, tile_size as u32, 1),
            grid_dim: (grid_x as u32, grid_y as u32, 1),
            shared_mem_bytes: 0,
        }
    }

    #[cfg(feature = "cuda")]
    fn launch_config_reduce(n: usize) -> (LaunchConfig, usize) {
        let block_size = 256;
        let grid_size = (n + block_size * 2 - 1) / (block_size * 2);
        let config = LaunchConfig {
            block_dim: (block_size as u32, 1, 1),
            grid_dim: (grid_size as u32, 1, 1),
            shared_mem_bytes: (block_size * std::mem::size_of::<f32>()) as u32,
        };
        (config, grid_size)
    }
}

#[cfg(feature = "cuda")]
impl GpuDevice for CudaDevice {
    fn name(&self) -> &str {
        "CUDA"
    }

    fn create_buffer(&self, size: usize) -> Result<Box<dyn GpuBuffer>> {
        let data = self.device.alloc_zeros::<u8>(size).map_err(|e| {
            GpuError::BufferCreationFailed(format!("Failed to allocate CUDA buffer: {}", e))
        })?;

        Ok(Box::new(CudaBuffer {
            data,
            size,
            device: Arc::clone(&self.device),
        }))
    }

    fn create_buffer_with_data(&self, data: &[u8]) -> Result<Box<dyn GpuBuffer>> {
        let gpu_data = self.device.htod_sync_copy(data).map_err(|e| {
            GpuError::BufferCreationFailed(format!("Failed to create CUDA buffer with data: {}", e))
        })?;

        Ok(Box::new(CudaBuffer {
            data: gpu_data,
            size: data.len(),
            device: Arc::clone(&self.device),
        }))
    }

    fn add_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let n = a.len();
        let dev_a = self
            .device
            .htod_sync_copy(a)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let dev_b = self
            .device
            .htod_sync_copy(b)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_c = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "add_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel add_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(
                    Self::launch_config(n),
                    (&dev_a, &dev_b, &mut dev_c, n as i32),
                )
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_c, c)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn mul_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let n = a.len();
        let dev_a = self
            .device
            .htod_sync_copy(a)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let dev_b = self
            .device
            .htod_sync_copy(b)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_c = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "mul_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel mul_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(
                    Self::launch_config(n),
                    (&dev_a, &dev_b, &mut dev_c, n as i32),
                )
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_c, c)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn sub_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let n = a.len();
        let dev_a = self
            .device
            .htod_sync_copy(a)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let dev_b = self
            .device
            .htod_sync_copy(b)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_c = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "sub_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel sub_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(
                    Self::launch_config(n),
                    (&dev_a, &dev_b, &mut dev_c, n as i32),
                )
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_c, c)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn div_f32(&self, a: &[f32], b: &[f32], c: &mut [f32]) -> Result<()> {
        let n = a.len();
        let dev_a = self
            .device
            .htod_sync_copy(a)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let dev_b = self
            .device
            .htod_sync_copy(b)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_c = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "div_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel div_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(
                    Self::launch_config(n),
                    (&dev_a, &dev_b, &mut dev_c, n as i32),
                )
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_c, c)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn matmul_f32(
        &self,
        a: &[f32],
        b: &[f32],
        c: &mut [f32],
        m: usize,
        n: usize,
        k: usize,
    ) -> Result<()> {
        let dev_a = self
            .device
            .htod_sync_copy(a)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let dev_b = self
            .device
            .htod_sync_copy(b)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_c = self
            .device
            .alloc_zeros::<f32>(m * n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "matmul_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel matmul_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(
                    Self::launch_config_2d(m, n),
                    (&dev_a, &dev_b, &mut dev_c, m as i32, n as i32, k as i32),
                )
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_c, c)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn sin_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let n = x.len();
        let dev_x = self
            .device
            .htod_sync_copy(x)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_y = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "sin_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel sin_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(Self::launch_config(n), (&dev_x, &mut dev_y, n as i32))
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_y, y)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn cos_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let n = x.len();
        let dev_x = self
            .device
            .htod_sync_copy(x)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_y = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "cos_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel cos_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(Self::launch_config(n), (&dev_x, &mut dev_y, n as i32))
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_y, y)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn exp_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let n = x.len();
        let dev_x = self
            .device
            .htod_sync_copy(x)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_y = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "exp_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel exp_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(Self::launch_config(n), (&dev_x, &mut dev_y, n as i32))
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_y, y)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn log_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let n = x.len();
        let dev_x = self
            .device
            .htod_sync_copy(x)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_y = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "log_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel log_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(Self::launch_config(n), (&dev_x, &mut dev_y, n as i32))
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_y, y)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn sqrt_f32(&self, x: &[f32], y: &mut [f32]) -> Result<()> {
        let n = x.len();
        let dev_x = self
            .device
            .htod_sync_copy(x)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_y = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "sqrt_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel sqrt_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(Self::launch_config(n), (&dev_x, &mut dev_y, n as i32))
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_y, y)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn pow_f32(&self, x: &[f32], p: f32, y: &mut [f32]) -> Result<()> {
        let n = x.len();
        let dev_x = self
            .device
            .htod_sync_copy(x)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;
        let mut dev_y = self
            .device
            .alloc_zeros::<f32>(n)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "pow_f32")
            .ok_or_else(|| GpuError::ExecutionFailed("Kernel pow_f32 not found".to_string()))?;

        unsafe {
            kernel
                .launch(Self::launch_config(n), (&dev_x, p, &mut dev_y, n as i32))
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        self.device
            .dtoh_sync_copy_into(&dev_y, y)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(())
    }

    fn sum_f32(&self, x: &[f32]) -> Result<f32> {
        let n = x.len();
        if n == 0 {
            return Ok(0.0);
        }

        let dev_x = self
            .device
            .htod_sync_copy(x)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let (config, grid_size) = Self::launch_config_reduce(n);
        let mut dev_partial = self
            .device
            .alloc_zeros::<f32>(grid_size)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "sum_reduce_f32")
            .ok_or_else(|| {
                GpuError::ExecutionFailed("Kernel sum_reduce_f32 not found".to_string())
            })?;

        unsafe {
            kernel
                .launch(config, (&dev_x, &mut dev_partial, n as i32))
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        // Copy partial results back and sum on CPU
        let mut partial = vec![0.0f32; grid_size];
        self.device
            .dtoh_sync_copy_into(&dev_partial, &mut partial)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(partial.iter().sum())
    }

    fn max_f32(&self, x: &[f32]) -> Result<f32> {
        let n = x.len();
        if n == 0 {
            return Err(GpuError::ExecutionFailed("Empty array".to_string()));
        }

        let dev_x = self
            .device
            .htod_sync_copy(x)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let (config, grid_size) = Self::launch_config_reduce(n);
        let mut dev_partial = self
            .device
            .alloc_zeros::<f32>(grid_size)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "max_reduce_f32")
            .ok_or_else(|| {
                GpuError::ExecutionFailed("Kernel max_reduce_f32 not found".to_string())
            })?;

        unsafe {
            kernel
                .launch(config, (&dev_x, &mut dev_partial, n as i32))
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        let mut partial = vec![0.0f32; grid_size];
        self.device
            .dtoh_sync_copy_into(&dev_partial, &mut partial)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(partial.iter().cloned().fold(f32::NEG_INFINITY, f32::max))
    }

    fn min_f32(&self, x: &[f32]) -> Result<f32> {
        let n = x.len();
        if n == 0 {
            return Err(GpuError::ExecutionFailed("Empty array".to_string()));
        }

        let dev_x = self
            .device
            .htod_sync_copy(x)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let (config, grid_size) = Self::launch_config_reduce(n);
        let mut dev_partial = self
            .device
            .alloc_zeros::<f32>(grid_size)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        let kernel = self
            .device
            .get_func("xdl_kernels", "min_reduce_f32")
            .ok_or_else(|| {
                GpuError::ExecutionFailed("Kernel min_reduce_f32 not found".to_string())
            })?;

        unsafe {
            kernel
                .launch(config, (&dev_x, &mut dev_partial, n as i32))
                .map_err(|e| GpuError::ExecutionFailed(format!("Kernel launch failed: {}", e)))?;
        }

        let mut partial = vec![0.0f32; grid_size];
        self.device
            .dtoh_sync_copy_into(&dev_partial, &mut partial)
            .map_err(|e| GpuError::CudaError(e.to_string()))?;

        Ok(partial.iter().cloned().fold(f32::INFINITY, f32::min))
    }

    fn synchronize(&self) -> Result<()> {
        self.device
            .synchronize()
            .map_err(|e| GpuError::CudaError(format!("Failed to synchronize: {}", e)))?;
        Ok(())
    }
}

#[cfg(not(feature = "cuda"))]
impl GpuDevice for CudaDevice {
    fn name(&self) -> &str {
        "CUDA"
    }

    fn create_buffer(&self, _size: usize) -> Result<Box<dyn GpuBuffer>> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn create_buffer_with_data(&self, _data: &[u8]) -> Result<Box<dyn GpuBuffer>> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn add_f32(&self, _a: &[f32], _b: &[f32], _c: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn mul_f32(&self, _a: &[f32], _b: &[f32], _c: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn sub_f32(&self, _a: &[f32], _b: &[f32], _c: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn div_f32(&self, _a: &[f32], _b: &[f32], _c: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn matmul_f32(
        &self,
        _a: &[f32],
        _b: &[f32],
        _c: &mut [f32],
        _m: usize,
        _n: usize,
        _k: usize,
    ) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn sin_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn cos_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn exp_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn log_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn sqrt_f32(&self, _x: &[f32], _y: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn pow_f32(&self, _x: &[f32], _p: f32, _y: &mut [f32]) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn sum_f32(&self, _x: &[f32]) -> Result<f32> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn max_f32(&self, _x: &[f32]) -> Result<f32> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn min_f32(&self, _x: &[f32]) -> Result<f32> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }

    fn synchronize(&self) -> Result<()> {
        Err(GpuError::UnsupportedBackend("CUDA not enabled".to_string()))
    }
}
