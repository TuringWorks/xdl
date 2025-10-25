#include <metal_stdlib>
using namespace metal;

// Element-wise addition: c = a + b
kernel void add_f32(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device float* c [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    c[id] = a[id] + b[id];
}

// Element-wise multiplication: c = a * b
kernel void mul_f32(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device float* c [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    c[id] = a[id] * b[id];
}

// Element-wise subtraction: c = a - b
kernel void sub_f32(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device float* c [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    c[id] = a[id] - b[id];
}

// Element-wise division: c = a / b
kernel void div_f32(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device float* c [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    c[id] = a[id] / b[id];
}

// Sine: y = sin(x)
kernel void sin_f32(
    device const float* x [[buffer(0)]],
    device float* y [[buffer(1)]],
    uint id [[thread_position_in_grid]]
) {
    y[id] = sin(x[id]);
}

// Cosine: y = cos(x)
kernel void cos_f32(
    device const float* x [[buffer(0)]],
    device float* y [[buffer(1)]],
    uint id [[thread_position_in_grid]]
) {
    y[id] = cos(x[id]);
}

// Exponential: y = exp(x)
kernel void exp_f32(
    device const float* x [[buffer(0)]],
    device float* y [[buffer(1)]],
    uint id [[thread_position_in_grid]]
) {
    y[id] = exp(x[id]);
}

// Natural logarithm: y = log(x)
kernel void log_f32(
    device const float* x [[buffer(0)]],
    device float* y [[buffer(1)]],
    uint id [[thread_position_in_grid]]
) {
    y[id] = log(x[id]);
}

// Square root: y = sqrt(x)
kernel void sqrt_f32(
    device const float* x [[buffer(0)]],
    device float* y [[buffer(1)]],
    uint id [[thread_position_in_grid]]
) {
    y[id] = sqrt(x[id]);
}

// Power: y = x^p
kernel void pow_f32(
    device const float* x [[buffer(0)]],
    device const float& p [[buffer(1)]],
    device float* y [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    y[id] = pow(x[id], p);
}
