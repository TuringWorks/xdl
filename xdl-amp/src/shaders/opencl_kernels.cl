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
