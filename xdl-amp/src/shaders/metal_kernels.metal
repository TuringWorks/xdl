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

// Tiled matrix multiplication: C = A * B
// A: M x K, B: K x N, C: M x N
kernel void matmul_f32(
    device const float* a [[buffer(0)]],
    device const float* b [[buffer(1)]],
    device float* c [[buffer(2)]],
    constant uint& M [[buffer(3)]],
    constant uint& N [[buffer(4)]],
    constant uint& K [[buffer(5)]],
    uint2 gid [[thread_position_in_grid]],
    uint2 lid [[thread_position_in_threadgroup]],
    uint2 tgid [[threadgroup_position_in_grid]]
) {
    // Tile size
    constexpr uint TILE_SIZE = 16;

    // Shared memory for tiles
    threadgroup float tileA[TILE_SIZE][TILE_SIZE];
    threadgroup float tileB[TILE_SIZE][TILE_SIZE];

    uint row = gid.y;
    uint col = gid.x;
    uint localRow = lid.y;
    uint localCol = lid.x;

    // Bounds check
    if (row >= M || col >= N) {
        return;
    }

    float sum = 0.0;

    // Number of tiles
    uint numTiles = (K + TILE_SIZE - 1) / TILE_SIZE;

    // Iterate over tiles
    for (uint t = 0; t < numTiles; t++) {
        // Load tile of A
        uint aCol = t * TILE_SIZE + localCol;
        if (row < M && aCol < K) {
            tileA[localRow][localCol] = a[row * K + aCol];
        } else {
            tileA[localRow][localCol] = 0.0;
        }

        // Load tile of B
        uint bRow = t * TILE_SIZE + localRow;
        if (bRow < K && col < N) {
            tileB[localRow][localCol] = b[bRow * N + col];
        } else {
            tileB[localRow][localCol] = 0.0;
        }

        // Synchronize
        threadgroup_barrier(mem_flags::mem_threadgroup);

        // Compute partial dot product
        for (uint k = 0; k < TILE_SIZE; k++) {
            sum += tileA[localRow][k] * tileB[k][localCol];
        }

        // Synchronize before next tile
        threadgroup_barrier(mem_flags::mem_threadgroup);
    }

    // Write result
    if (row < M && col < N) {
        c[row * N + col] = sum;
    }
}
