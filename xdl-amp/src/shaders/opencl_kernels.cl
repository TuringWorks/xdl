// OpenCL compute kernels for GPU operations

// Element-wise addition: c = a + b
__kernel void add_f32(
    __global const float* a,
    __global const float* b,
    __global float* c
) {
    int id = get_global_id(0);
    c[id] = a[id] + b[id];
}

// Element-wise multiplication: c = a * b
__kernel void mul_f32(
    __global const float* a,
    __global const float* b,
    __global float* c
) {
    int id = get_global_id(0);
    c[id] = a[id] * b[id];
}

// Element-wise subtraction: c = a - b
__kernel void sub_f32(
    __global const float* a,
    __global const float* b,
    __global float* c
) {
    int id = get_global_id(0);
    c[id] = a[id] - b[id];
}

// Element-wise division: c = a / b
__kernel void div_f32(
    __global const float* a,
    __global const float* b,
    __global float* c
) {
    int id = get_global_id(0);
    c[id] = a[id] / b[id];
}

// Sine: y = sin(x)
__kernel void sin_f32(
    __global const float* x,
    __global float* y
) {
    int id = get_global_id(0);
    y[id] = sin(x[id]);
}

// Cosine: y = cos(x)
__kernel void cos_f32(
    __global const float* x,
    __global float* y
) {
    int id = get_global_id(0);
    y[id] = cos(x[id]);
}

// Exponential: y = exp(x)
__kernel void exp_f32(
    __global const float* x,
    __global float* y
) {
    int id = get_global_id(0);
    y[id] = exp(x[id]);
}

// Natural logarithm: y = log(x)
__kernel void log_f32(
    __global const float* x,
    __global float* y
) {
    int id = get_global_id(0);
    y[id] = log(x[id]);
}

// Square root: y = sqrt(x)
__kernel void sqrt_f32(
    __global const float* x,
    __global float* y
) {
    int id = get_global_id(0);
    y[id] = sqrt(x[id]);
}

// Power: y = x^p
__kernel void pow_f32(
    __global const float* x,
    float p,
    __global float* y
) {
    int id = get_global_id(0);
    y[id] = pow(x[id], p);
}

// Matrix multiplication: C = A * B
// A is M x K, B is K x N, C is M x N
__kernel void matmul_f32(
    __global const float* a,
    __global const float* b,
    __global float* c,
    uint M,
    uint N,
    uint K
) {
    int col = get_global_id(0);
    int row = get_global_id(1);

    if (row >= M || col >= N) return;

    float sum = 0.0f;
    for (uint i = 0; i < K; i++) {
        sum += a[row * K + i] * b[i * N + col];
    }
    c[row * N + col] = sum;
}

// Sum reduction with local memory
__kernel void sum_reduce_f32(
    __global const float* x,
    __global float* partial,
    uint n
) {
    __local float scratch[256];

    int gid = get_global_id(0);
    int lid = get_local_id(0);
    int group_id = get_group_id(0);
    int local_size = get_local_size(0);

    // Load data into local memory
    scratch[lid] = (gid < n) ? x[gid] : 0.0f;
    barrier(CLK_LOCAL_MEM_FENCE);

    // Parallel reduction in local memory
    for (int stride = local_size / 2; stride > 0; stride >>= 1) {
        if (lid < stride) {
            scratch[lid] += scratch[lid + stride];
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }

    // Write result for this work-group
    if (lid == 0) {
        partial[group_id] = scratch[0];
    }
}

// Max reduction with local memory
__kernel void max_reduce_f32(
    __global const float* x,
    __global float* partial,
    uint n
) {
    __local float scratch[256];

    int gid = get_global_id(0);
    int lid = get_local_id(0);
    int group_id = get_group_id(0);
    int local_size = get_local_size(0);

    // Load data into local memory
    scratch[lid] = (gid < n) ? x[gid] : -INFINITY;
    barrier(CLK_LOCAL_MEM_FENCE);

    // Parallel reduction in local memory
    for (int stride = local_size / 2; stride > 0; stride >>= 1) {
        if (lid < stride) {
            scratch[lid] = fmax(scratch[lid], scratch[lid + stride]);
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }

    // Write result for this work-group
    if (lid == 0) {
        partial[group_id] = scratch[0];
    }
}

// Min reduction with local memory
__kernel void min_reduce_f32(
    __global const float* x,
    __global float* partial,
    uint n
) {
    __local float scratch[256];

    int gid = get_global_id(0);
    int lid = get_local_id(0);
    int group_id = get_group_id(0);
    int local_size = get_local_size(0);

    // Load data into local memory
    scratch[lid] = (gid < n) ? x[gid] : INFINITY;
    barrier(CLK_LOCAL_MEM_FENCE);

    // Parallel reduction in local memory
    for (int stride = local_size / 2; stride > 0; stride >>= 1) {
        if (lid < stride) {
            scratch[lid] = fmin(scratch[lid], scratch[lid + stride]);
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }

    // Write result for this work-group
    if (lid == 0) {
        partial[group_id] = scratch[0];
    }
}
