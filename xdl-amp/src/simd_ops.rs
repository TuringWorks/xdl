//! SIMD-optimized CPU operations for XDL-AMP
//!
//! Uses the `wide` crate for portable SIMD (SSE/AVX on x86, NEON on ARM)
//! and `rayon` for parallel execution on large arrays.

use rayon::prelude::*;
use wide::{f32x8, CmpGt, CmpLt};

/// Threshold for switching to parallel execution (elements)
const PARALLEL_THRESHOLD: usize = 100_000;

/// SIMD lane width for f32
const SIMD_WIDTH: usize = 8;

// ============================================================================
// Element-wise Binary Operations (SIMD + Parallel)
// ============================================================================

/// SIMD-optimized element-wise addition
pub fn add_f32(a: &[f32], b: &[f32], c: &mut [f32]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), c.len());

    if a.len() >= PARALLEL_THRESHOLD {
        add_f32_parallel(a, b, c);
    } else {
        add_f32_simd(a, b, c);
    }
}

fn add_f32_simd(a: &[f32], b: &[f32], c: &mut [f32]) {
    let len = a.len();
    let chunks = len / SIMD_WIDTH;

    // SIMD main loop
    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let va = f32x8::new(a[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vb = f32x8::new(b[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vc = va + vb;
        let result: [f32; 8] = vc.into();
        c[offset..offset + SIMD_WIDTH].copy_from_slice(&result);
    }

    // Scalar remainder
    for i in (chunks * SIMD_WIDTH)..len {
        c[i] = a[i] + b[i];
    }
}

fn add_f32_parallel(a: &[f32], b: &[f32], c: &mut [f32]) {
    const CHUNK_SIZE: usize = 8192;
    c.par_chunks_mut(CHUNK_SIZE)
        .enumerate()
        .for_each(|(chunk_idx, c_chunk)| {
            let offset = chunk_idx * CHUNK_SIZE;
            let a_chunk = &a[offset..offset + c_chunk.len()];
            let b_chunk = &b[offset..offset + c_chunk.len()];
            add_f32_simd(a_chunk, b_chunk, c_chunk);
        });
}

/// SIMD-optimized element-wise subtraction
pub fn sub_f32(a: &[f32], b: &[f32], c: &mut [f32]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), c.len());

    if a.len() >= PARALLEL_THRESHOLD {
        sub_f32_parallel(a, b, c);
    } else {
        sub_f32_simd(a, b, c);
    }
}

fn sub_f32_simd(a: &[f32], b: &[f32], c: &mut [f32]) {
    let len = a.len();
    let chunks = len / SIMD_WIDTH;

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let va = f32x8::new(a[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vb = f32x8::new(b[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vc = va - vb;
        let result: [f32; 8] = vc.into();
        c[offset..offset + SIMD_WIDTH].copy_from_slice(&result);
    }

    for i in (chunks * SIMD_WIDTH)..len {
        c[i] = a[i] - b[i];
    }
}

fn sub_f32_parallel(a: &[f32], b: &[f32], c: &mut [f32]) {
    const CHUNK_SIZE: usize = 8192;
    c.par_chunks_mut(CHUNK_SIZE)
        .enumerate()
        .for_each(|(chunk_idx, c_chunk)| {
            let offset = chunk_idx * CHUNK_SIZE;
            let a_chunk = &a[offset..offset + c_chunk.len()];
            let b_chunk = &b[offset..offset + c_chunk.len()];
            sub_f32_simd(a_chunk, b_chunk, c_chunk);
        });
}

/// SIMD-optimized element-wise multiplication
pub fn mul_f32(a: &[f32], b: &[f32], c: &mut [f32]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), c.len());

    if a.len() >= PARALLEL_THRESHOLD {
        mul_f32_parallel(a, b, c);
    } else {
        mul_f32_simd(a, b, c);
    }
}

fn mul_f32_simd(a: &[f32], b: &[f32], c: &mut [f32]) {
    let len = a.len();
    let chunks = len / SIMD_WIDTH;

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let va = f32x8::new(a[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vb = f32x8::new(b[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vc = va * vb;
        let result: [f32; 8] = vc.into();
        c[offset..offset + SIMD_WIDTH].copy_from_slice(&result);
    }

    for i in (chunks * SIMD_WIDTH)..len {
        c[i] = a[i] * b[i];
    }
}

fn mul_f32_parallel(a: &[f32], b: &[f32], c: &mut [f32]) {
    const CHUNK_SIZE: usize = 8192;
    c.par_chunks_mut(CHUNK_SIZE)
        .enumerate()
        .for_each(|(chunk_idx, c_chunk)| {
            let offset = chunk_idx * CHUNK_SIZE;
            let a_chunk = &a[offset..offset + c_chunk.len()];
            let b_chunk = &b[offset..offset + c_chunk.len()];
            mul_f32_simd(a_chunk, b_chunk, c_chunk);
        });
}

/// SIMD-optimized element-wise division
pub fn div_f32(a: &[f32], b: &[f32], c: &mut [f32]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), c.len());

    if a.len() >= PARALLEL_THRESHOLD {
        div_f32_parallel(a, b, c);
    } else {
        div_f32_simd(a, b, c);
    }
}

fn div_f32_simd(a: &[f32], b: &[f32], c: &mut [f32]) {
    let len = a.len();
    let chunks = len / SIMD_WIDTH;

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let va = f32x8::new(a[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vb = f32x8::new(b[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vc = va / vb;
        let result: [f32; 8] = vc.into();
        c[offset..offset + SIMD_WIDTH].copy_from_slice(&result);
    }

    for i in (chunks * SIMD_WIDTH)..len {
        c[i] = a[i] / b[i];
    }
}

fn div_f32_parallel(a: &[f32], b: &[f32], c: &mut [f32]) {
    const CHUNK_SIZE: usize = 8192;
    c.par_chunks_mut(CHUNK_SIZE)
        .enumerate()
        .for_each(|(chunk_idx, c_chunk)| {
            let offset = chunk_idx * CHUNK_SIZE;
            let a_chunk = &a[offset..offset + c_chunk.len()];
            let b_chunk = &b[offset..offset + c_chunk.len()];
            div_f32_simd(a_chunk, b_chunk, c_chunk);
        });
}

// ============================================================================
// Element-wise Unary Operations (SIMD + Parallel)
// ============================================================================

/// SIMD-optimized element-wise sqrt
pub fn sqrt_f32(x: &[f32], y: &mut [f32]) {
    debug_assert_eq!(x.len(), y.len());

    if x.len() >= PARALLEL_THRESHOLD {
        sqrt_f32_parallel(x, y);
    } else {
        sqrt_f32_simd(x, y);
    }
}

fn sqrt_f32_simd(x: &[f32], y: &mut [f32]) {
    let len = x.len();
    let chunks = len / SIMD_WIDTH;

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let vx = f32x8::new(x[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vy = vx.sqrt();
        let result: [f32; 8] = vy.into();
        y[offset..offset + SIMD_WIDTH].copy_from_slice(&result);
    }

    for i in (chunks * SIMD_WIDTH)..len {
        y[i] = x[i].sqrt();
    }
}

fn sqrt_f32_parallel(x: &[f32], y: &mut [f32]) {
    const CHUNK_SIZE: usize = 8192;
    y.par_chunks_mut(CHUNK_SIZE)
        .enumerate()
        .for_each(|(chunk_idx, y_chunk)| {
            let offset = chunk_idx * CHUNK_SIZE;
            let x_chunk = &x[offset..offset + y_chunk.len()];
            sqrt_f32_simd(x_chunk, y_chunk);
        });
}

/// Element-wise sine (parallel, scalar - SIMD sin not in wide)
pub fn sin_f32(x: &[f32], y: &mut [f32]) {
    debug_assert_eq!(x.len(), y.len());

    if x.len() >= PARALLEL_THRESHOLD {
        x.par_iter()
            .zip(y.par_iter_mut())
            .for_each(|(xi, yi)| *yi = xi.sin());
    } else {
        for i in 0..x.len() {
            y[i] = x[i].sin();
        }
    }
}

/// Element-wise cosine (parallel, scalar - SIMD cos not in wide)
pub fn cos_f32(x: &[f32], y: &mut [f32]) {
    debug_assert_eq!(x.len(), y.len());

    if x.len() >= PARALLEL_THRESHOLD {
        x.par_iter()
            .zip(y.par_iter_mut())
            .for_each(|(xi, yi)| *yi = xi.cos());
    } else {
        for i in 0..x.len() {
            y[i] = x[i].cos();
        }
    }
}

/// Element-wise exp (parallel, scalar)
pub fn exp_f32(x: &[f32], y: &mut [f32]) {
    debug_assert_eq!(x.len(), y.len());

    if x.len() >= PARALLEL_THRESHOLD {
        x.par_iter()
            .zip(y.par_iter_mut())
            .for_each(|(xi, yi)| *yi = xi.exp());
    } else {
        for i in 0..x.len() {
            y[i] = x[i].exp();
        }
    }
}

/// Element-wise log (parallel, scalar)
pub fn log_f32(x: &[f32], y: &mut [f32]) {
    debug_assert_eq!(x.len(), y.len());

    if x.len() >= PARALLEL_THRESHOLD {
        x.par_iter()
            .zip(y.par_iter_mut())
            .for_each(|(xi, yi)| *yi = xi.ln());
    } else {
        for i in 0..x.len() {
            y[i] = x[i].ln();
        }
    }
}

/// Element-wise pow (parallel, scalar)
pub fn pow_f32(x: &[f32], p: f32, y: &mut [f32]) {
    debug_assert_eq!(x.len(), y.len());

    if x.len() >= PARALLEL_THRESHOLD {
        x.par_iter()
            .zip(y.par_iter_mut())
            .for_each(|(xi, yi)| *yi = xi.powf(p));
    } else {
        for i in 0..x.len() {
            y[i] = x[i].powf(p);
        }
    }
}

// ============================================================================
// Reduction Operations (SIMD + Parallel)
// ============================================================================

/// SIMD-optimized sum reduction
pub fn sum_f32(x: &[f32]) -> f32 {
    if x.len() >= PARALLEL_THRESHOLD {
        sum_f32_parallel(x)
    } else {
        sum_f32_simd(x)
    }
}

fn sum_f32_simd(x: &[f32]) -> f32 {
    let len = x.len();
    let chunks = len / SIMD_WIDTH;

    let mut acc = f32x8::ZERO;

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let vx = f32x8::new(x[offset..offset + SIMD_WIDTH].try_into().unwrap());
        acc += vx;
    }

    // Horizontal sum of SIMD register
    let arr: [f32; 8] = acc.into();
    let mut sum: f32 = arr.iter().sum();

    // Add remainder
    for val in x.iter().skip(chunks * SIMD_WIDTH) {
        sum += val;
    }

    sum
}

fn sum_f32_parallel(x: &[f32]) -> f32 {
    const CHUNK_SIZE: usize = 8192;
    x.par_chunks(CHUNK_SIZE).map(sum_f32_simd).sum()
}

/// SIMD-optimized max reduction
pub fn max_f32(x: &[f32]) -> f32 {
    if x.is_empty() {
        return f32::NEG_INFINITY;
    }

    if x.len() >= PARALLEL_THRESHOLD {
        max_f32_parallel(x)
    } else {
        max_f32_simd(x)
    }
}

fn max_f32_simd(x: &[f32]) -> f32 {
    let len = x.len();
    let chunks = len / SIMD_WIDTH;

    let mut acc = f32x8::splat(f32::NEG_INFINITY);

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let vx = f32x8::new(x[offset..offset + SIMD_WIDTH].try_into().unwrap());
        // SIMD max using comparison and blend
        let mask = vx.cmp_gt(acc);
        acc = mask.blend(vx, acc);
    }

    // Horizontal max of SIMD register
    let arr: [f32; 8] = acc.into();
    let mut max_val = arr.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    // Check remainder
    for val in x.iter().skip(chunks * SIMD_WIDTH) {
        max_val = max_val.max(*val);
    }

    max_val
}

fn max_f32_parallel(x: &[f32]) -> f32 {
    const CHUNK_SIZE: usize = 8192;
    x.par_chunks(CHUNK_SIZE)
        .map(max_f32_simd)
        .reduce(|| f32::NEG_INFINITY, f32::max)
}

/// SIMD-optimized min reduction
pub fn min_f32(x: &[f32]) -> f32 {
    if x.is_empty() {
        return f32::INFINITY;
    }

    if x.len() >= PARALLEL_THRESHOLD {
        min_f32_parallel(x)
    } else {
        min_f32_simd(x)
    }
}

fn min_f32_simd(x: &[f32]) -> f32 {
    let len = x.len();
    let chunks = len / SIMD_WIDTH;

    let mut acc = f32x8::splat(f32::INFINITY);

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let vx = f32x8::new(x[offset..offset + SIMD_WIDTH].try_into().unwrap());
        // SIMD min using comparison and blend
        let mask = vx.cmp_lt(acc);
        acc = mask.blend(vx, acc);
    }

    // Horizontal min of SIMD register
    let arr: [f32; 8] = acc.into();
    let mut min_val = arr.iter().cloned().fold(f32::INFINITY, f32::min);

    // Check remainder
    for val in x.iter().skip(chunks * SIMD_WIDTH) {
        min_val = min_val.min(*val);
    }

    min_val
}

fn min_f32_parallel(x: &[f32]) -> f32 {
    const CHUNK_SIZE: usize = 8192;
    x.par_chunks(CHUNK_SIZE)
        .map(min_f32_simd)
        .reduce(|| f32::INFINITY, f32::min)
}

/// Median: returns the middle value of a sorted array
/// For even-length arrays, returns the average of the two middle values
pub fn median_f32(x: &[f32]) -> f32 {
    if x.is_empty() {
        return f32::NAN;
    }

    if x.len() == 1 {
        return x[0];
    }

    // Clone and sort (median requires sorting)
    let sorted: Vec<f32> = if x.len() >= PARALLEL_THRESHOLD {
        // Parallel sort for large arrays
        let mut v = x.to_vec();
        v.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        v
    } else {
        let mut v = x.to_vec();
        v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        v
    };

    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        // Even length: average of two middle values
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        // Odd length: middle value
        sorted[mid]
    }
}

/// Variance: returns the population variance of elements
/// Variance = sum((x - mean)^2) / n
pub fn variance_f32(x: &[f32]) -> f32 {
    if x.is_empty() {
        return f32::NAN;
    }

    if x.len() == 1 {
        return 0.0;
    }

    let n = x.len() as f32;
    let mean = sum_f32(x) / n;

    if x.len() >= PARALLEL_THRESHOLD {
        variance_f32_parallel(x, mean)
    } else {
        variance_f32_simd(x, mean)
    }
}

fn variance_f32_simd(x: &[f32], mean: f32) -> f32 {
    let len = x.len();
    let n = len as f32;
    let chunks = len / SIMD_WIDTH;

    let vmean = f32x8::splat(mean);
    let mut acc = f32x8::ZERO;

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let vx = f32x8::new(x[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let diff = vx - vmean;
        acc += diff * diff; // (x - mean)^2
    }

    // Horizontal sum of SIMD register
    let arr: [f32; 8] = acc.into();
    let mut sum_sq: f32 = arr.iter().sum();

    // Add remainder
    for val in x.iter().skip(chunks * SIMD_WIDTH) {
        let diff = val - mean;
        sum_sq += diff * diff;
    }

    sum_sq / n
}

fn variance_f32_parallel(x: &[f32], mean: f32) -> f32 {
    const CHUNK_SIZE: usize = 8192;
    let sum_sq: f32 = x
        .par_chunks(CHUNK_SIZE)
        .map(|chunk| {
            let chunks = chunk.len() / SIMD_WIDTH;
            let vmean = f32x8::splat(mean);
            let mut acc = f32x8::ZERO;

            for i in 0..chunks {
                let offset = i * SIMD_WIDTH;
                let vx = f32x8::new(chunk[offset..offset + SIMD_WIDTH].try_into().unwrap());
                let diff = vx - vmean;
                acc += diff * diff;
            }

            let arr: [f32; 8] = acc.into();
            let mut partial: f32 = arr.iter().sum();

            for val in chunk.iter().skip(chunks * SIMD_WIDTH) {
                let diff = val - mean;
                partial += diff * diff;
            }

            partial
        })
        .sum();

    sum_sq / (x.len() as f32)
}

/// Standard deviation: returns the population standard deviation
/// Stddev = sqrt(variance)
pub fn stddev_f32(x: &[f32]) -> f32 {
    variance_f32(x).sqrt()
}

// ============================================================================
// Matrix Multiplication (using matrixmultiply crate)
// ============================================================================

/// Cache-efficient matrix multiplication using matrixmultiply crate
///
/// Computes C = A * B where:
/// - A is m x k (row-major)
/// - B is k x n (row-major)
/// - C is m x n (row-major)
pub fn matmul_f32(a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize) {
    debug_assert_eq!(a.len(), m * k);
    debug_assert_eq!(b.len(), k * n);
    debug_assert_eq!(c.len(), m * n);

    // Initialize C to zero
    c.iter_mut().for_each(|x| *x = 0.0);

    // Use matrixmultiply's GEMM (General Matrix Multiply)
    // sgemm computes: C = beta*C + alpha*A*B
    unsafe {
        matrixmultiply::sgemm(
            m,   // rows of A and C
            k,   // cols of A, rows of B
            n,   // cols of B and C
            1.0, // alpha
            a.as_ptr(),
            k as isize, // row stride of A (distance between rows)
            1,          // col stride of A (distance between columns)
            b.as_ptr(),
            n as isize, // row stride of B
            1,          // col stride of B
            0.0,        // beta (we initialized C to zero)
            c.as_mut_ptr(),
            n as isize, // row stride of C
            1,          // col stride of C
        );
    }
}

/// Parallel matrix multiplication for very large matrices
pub fn matmul_f32_parallel(a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize) {
    debug_assert_eq!(a.len(), m * k);
    debug_assert_eq!(b.len(), k * n);
    debug_assert_eq!(c.len(), m * n);

    // For very large matrices, parallelize over rows of the output
    const ROW_CHUNK: usize = 64;

    if m >= ROW_CHUNK * 4 {
        c.par_chunks_mut(n * ROW_CHUNK)
            .enumerate()
            .for_each(|(chunk_idx, c_chunk)| {
                let row_start = chunk_idx * ROW_CHUNK;
                let rows = c_chunk.len() / n;
                let a_chunk = &a[row_start * k..(row_start + rows) * k];

                // Initialize chunk to zero
                c_chunk.iter_mut().for_each(|x| *x = 0.0);

                unsafe {
                    matrixmultiply::sgemm(
                        rows,
                        k,
                        n,
                        1.0,
                        a_chunk.as_ptr(),
                        k as isize,
                        1,
                        b.as_ptr(),
                        n as isize,
                        1,
                        0.0,
                        c_chunk.as_mut_ptr(),
                        n as isize,
                        1,
                    );
                }
            });
    } else {
        // Fall back to single-threaded for smaller matrices
        matmul_f32(a, b, c, m, n, k);
    }
}

// ============================================================================
// Fused Operations (for performance)
// ============================================================================

/// Fused multiply-add: c = a * b + d (SIMD)
pub fn fma_f32(a: &[f32], b: &[f32], d: &[f32], c: &mut [f32]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), d.len());
    debug_assert_eq!(a.len(), c.len());

    let len = a.len();

    if len >= PARALLEL_THRESHOLD {
        const CHUNK_SIZE: usize = 8192;
        c.par_chunks_mut(CHUNK_SIZE)
            .enumerate()
            .for_each(|(chunk_idx, c_chunk)| {
                let offset = chunk_idx * CHUNK_SIZE;
                let a_chunk = &a[offset..offset + c_chunk.len()];
                let b_chunk = &b[offset..offset + c_chunk.len()];
                let d_chunk = &d[offset..offset + c_chunk.len()];
                fma_f32_simd(a_chunk, b_chunk, d_chunk, c_chunk);
            });
    } else {
        fma_f32_simd(a, b, d, c);
    }
}

fn fma_f32_simd(a: &[f32], b: &[f32], d: &[f32], c: &mut [f32]) {
    let len = a.len();
    let chunks = len / SIMD_WIDTH;

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let va = f32x8::new(a[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vb = f32x8::new(b[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vd = f32x8::new(d[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vc = va.mul_add(vb, vd); // a * b + d
        let result: [f32; 8] = vc.into();
        c[offset..offset + SIMD_WIDTH].copy_from_slice(&result);
    }

    for i in (chunks * SIMD_WIDTH)..len {
        c[i] = a[i].mul_add(b[i], d[i]);
    }
}

/// Scale and add: c = alpha * a + b (SIMD)
pub fn axpy_f32(alpha: f32, a: &[f32], b: &[f32], c: &mut [f32]) {
    debug_assert_eq!(a.len(), b.len());
    debug_assert_eq!(a.len(), c.len());

    let len = a.len();
    let chunks = len / SIMD_WIDTH;
    let valpha = f32x8::splat(alpha);

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let va = f32x8::new(a[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vb = f32x8::new(b[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vc = valpha * va + vb;
        let result: [f32; 8] = vc.into();
        c[offset..offset + SIMD_WIDTH].copy_from_slice(&result);
    }

    for i in (chunks * SIMD_WIDTH)..len {
        c[i] = alpha * a[i] + b[i];
    }
}

/// Dot product (SIMD + parallel)
pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    if a.len() >= PARALLEL_THRESHOLD {
        dot_f32_parallel(a, b)
    } else {
        dot_f32_simd(a, b)
    }
}

fn dot_f32_simd(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let chunks = len / SIMD_WIDTH;

    let mut acc = f32x8::ZERO;

    for i in 0..chunks {
        let offset = i * SIMD_WIDTH;
        let va = f32x8::new(a[offset..offset + SIMD_WIDTH].try_into().unwrap());
        let vb = f32x8::new(b[offset..offset + SIMD_WIDTH].try_into().unwrap());
        acc = va.mul_add(vb, acc);
    }

    // Horizontal sum
    let arr: [f32; 8] = acc.into();
    let mut sum: f32 = arr.iter().sum();

    // Remainder
    for i in (chunks * SIMD_WIDTH)..len {
        sum += a[i] * b[i];
    }

    sum
}

fn dot_f32_parallel(a: &[f32], b: &[f32]) -> f32 {
    const CHUNK_SIZE: usize = 8192;
    a.par_chunks(CHUNK_SIZE)
        .zip(b.par_chunks(CHUNK_SIZE))
        .map(|(a_chunk, b_chunk)| dot_f32_simd(a_chunk, b_chunk))
        .sum()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_f32() {
        let a = vec![1.0f32; 1000];
        let b = vec![2.0f32; 1000];
        let mut c = vec![0.0f32; 1000];

        add_f32(&a, &b, &mut c);

        assert!(c.iter().all(|&x| (x - 3.0).abs() < 1e-6));
    }

    #[test]
    fn test_mul_f32() {
        let a = vec![2.0f32; 1000];
        let b = vec![3.0f32; 1000];
        let mut c = vec![0.0f32; 1000];

        mul_f32(&a, &b, &mut c);

        assert!(c.iter().all(|&x| (x - 6.0).abs() < 1e-6));
    }

    #[test]
    fn test_sum_f32() {
        let x = vec![1.0f32; 1000];
        let sum = sum_f32(&x);
        assert!((sum - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn test_max_f32() {
        let mut x = vec![1.0f32; 1000];
        x[500] = 999.0;
        let max_val = max_f32(&x);
        assert!((max_val - 999.0).abs() < 1e-6);
    }

    #[test]
    fn test_min_f32() {
        let mut x = vec![10.0f32; 1000];
        x[500] = -5.0;
        let min_val = min_f32(&x);
        assert!((min_val - (-5.0)).abs() < 1e-6);
    }

    #[test]
    fn test_matmul_f32() {
        // 2x3 * 3x2 = 2x2
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 2x3
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 3x2
        let mut c = vec![0.0f32; 4]; // 2x2

        matmul_f32(&a, &b, &mut c, 2, 2, 3);

        // Expected: [[22, 28], [49, 64]]
        assert!((c[0] - 22.0).abs() < 1e-5);
        assert!((c[1] - 28.0).abs() < 1e-5);
        assert!((c[2] - 49.0).abs() < 1e-5);
        assert!((c[3] - 64.0).abs() < 1e-5);
    }

    #[test]
    fn test_dot_f32() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let result = dot_f32(&a, &b);
        // 1*5 + 2*6 + 3*7 + 4*8 = 5 + 12 + 21 + 32 = 70
        assert!((result - 70.0).abs() < 1e-5);
    }

    #[test]
    fn test_fma_f32() {
        let a = vec![2.0f32; 100];
        let b = vec![3.0f32; 100];
        let d = vec![1.0f32; 100];
        let mut c = vec![0.0f32; 100];

        fma_f32(&a, &b, &d, &mut c);

        // 2 * 3 + 1 = 7
        assert!(c.iter().all(|&x| (x - 7.0).abs() < 1e-6));
    }

    #[test]
    fn test_parallel_large_array() {
        // Test with array larger than PARALLEL_THRESHOLD
        let n = 200_000;
        let a = vec![1.0f32; n];
        let b = vec![2.0f32; n];
        let mut c = vec![0.0f32; n];

        add_f32(&a, &b, &mut c);

        assert!(c.iter().all(|&x| (x - 3.0).abs() < 1e-6));

        let sum = sum_f32(&c);
        assert!((sum - 3.0 * n as f32).abs() < 1.0);
    }

    #[test]
    fn test_median_f32_odd() {
        // Odd-length array
        let x = vec![3.0, 1.0, 4.0, 1.0, 5.0];
        let median = median_f32(&x);
        // Sorted: [1, 1, 3, 4, 5], median = 3
        assert!((median - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_median_f32_even() {
        // Even-length array
        let x = vec![3.0, 1.0, 4.0, 2.0];
        let median = median_f32(&x);
        // Sorted: [1, 2, 3, 4], median = (2 + 3) / 2 = 2.5
        assert!((median - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_variance_f32() {
        // Simple variance test
        let x = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let var = variance_f32(&x);
        // Mean = 40/8 = 5
        // Variance = ((2-5)^2 + (4-5)^2 + (4-5)^2 + (4-5)^2 + (5-5)^2 + (5-5)^2 + (7-5)^2 + (9-5)^2) / 8
        //          = (9 + 1 + 1 + 1 + 0 + 0 + 4 + 16) / 8 = 32 / 8 = 4
        assert!((var - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_stddev_f32() {
        // Stddev = sqrt(variance)
        let x = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let std = stddev_f32(&x);
        // Variance = 4, Stddev = 2
        assert!((std - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_variance_single_element() {
        let x = vec![42.0];
        let var = variance_f32(&x);
        assert!((var - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_median_single_element() {
        let x = vec![42.0];
        let median = median_f32(&x);
        assert!((median - 42.0).abs() < 1e-6);
    }
}
