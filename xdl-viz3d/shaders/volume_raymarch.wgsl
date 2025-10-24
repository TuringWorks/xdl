// Volume Ray Marching Shader
// WebGPU Shading Language (WGSL)

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,  // Camera position in world space
};

struct VolumeParams {
    dimensions: vec4<f32>,  // x, y, z, unused
    data_min: f32,
    data_max: f32,
    step_size: f32,
    max_steps: u32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> params: VolumeParams;
@group(0) @binding(2) var volume_texture: texture_3d<f32>;
@group(0) @binding(3) var volume_sampler: sampler;
@group(0) @binding(4) var colormap_texture: texture_1d<f32>;
@group(0) @binding(5) var colormap_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) ray_dir: vec3<f32>,
};

// Vertex shader - creates fullscreen quad
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Generate fullscreen quad
    let x = f32((vertex_index & 1u) << 1u) - 1.0;
    let y = f32((vertex_index & 2u)) - 1.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    // Ray direction will be computed in fragment shader
    // Pass screen coordinates to fragment shader
    out.ray_dir = vec3<f32>(x, y, 0.0); // Pass screen coords
    out.world_pos = camera.view_pos.xyz;

    return out;
}

// Ray-box intersection
fn ray_box_intersection(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec2<f32> {
    let box_min = vec3<f32>(-0.5, -0.5, -0.5);
    let box_max = vec3<f32>(0.5, 0.5, 0.5);

    let inv_dir = 1.0 / ray_dir;
    let t1 = (box_min - ray_origin) * inv_dir;
    let t2 = (box_max - ray_origin) * inv_dir;

    let tmin = min(t1, t2);
    let tmax = max(t1, t2);

    let t_near = max(max(tmin.x, tmin.y), tmin.z);
    let t_far = min(min(tmax.x, tmax.y), tmax.z);

    return vec2<f32>(max(t_near, 0.0), max(t_far, 0.0));
}

// Sample volume with trilinear interpolation
fn sample_volume(pos: vec3<f32>) -> f32 {
    // Convert from [-0.5, 0.5] to [0, 1] texture coordinates
    let tex_coord = pos + 0.5;

    // Clamp to valid range
    if (any(tex_coord < vec3<f32>(0.0)) || any(tex_coord > vec3<f32>(1.0))) {
        return 0.0;
    }

    return textureSample(volume_texture, volume_sampler, tex_coord).r;
}

// Map density value to color using colormap
fn apply_colormap(value: f32) -> vec4<f32> {
    // Normalize value to [0, 1] range
    let normalized = clamp((value - params.data_min) / (params.data_max - params.data_min), 0.0, 1.0);
    return textureSample(colormap_texture, colormap_sampler, normalized);
}

// Compute density gradient for shading (optimized - use central differences)
fn compute_gradient(pos: vec3<f32>, delta: f32) -> vec3<f32> {
    // Convert to texture coordinates
    let tex_coord = pos + 0.5;

    // Sample only once per direction using textureLoad for better performance
    let dx = sample_volume(pos + vec3<f32>(delta, 0.0, 0.0)) -
             sample_volume(pos - vec3<f32>(delta, 0.0, 0.0));
    let dy = sample_volume(pos + vec3<f32>(0.0, delta, 0.0)) -
             sample_volume(pos - vec3<f32>(0.0, delta, 0.0));
    let dz = sample_volume(pos + vec3<f32>(0.0, 0.0, delta)) -
             sample_volume(pos - vec3<f32>(0.0, 0.0, delta));

    let grad = vec3<f32>(dx, dy, dz);
    let len = length(grad);

    // Only normalize if gradient is significant
    if (len > 0.001) {
        return grad / len;
    } else {
        return vec3<f32>(0.0, 1.0, 0.0); // Default normal
    }
}

// Fragment shader - ray marching
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray_origin = in.world_pos;

    // Compute ray direction in world space
    // Screen coords passed from vertex shader
    let x = in.ray_dir.x;
    let y = in.ray_dir.y;

    let aspect = 1280.0 / 720.0;
    let fov = 0.785398;
    let tan_half_fov = tan(fov * 0.5);

    // Compute world space ray by aiming from camera to volume center + screen offset
    let volume_center = vec3<f32>(0.0, 0.0, 0.0); // Volume center
    let forward = normalize(volume_center - ray_origin);
    let right = normalize(cross(forward, vec3<f32>(0.0, 1.0, 0.0)));
    let up = cross(right, forward);

    let ray_offset = right * (x * aspect * tan_half_fov) + up * (y * tan_half_fov);
    let ray_dir = normalize(forward + ray_offset);

    // Compute ray-box intersection
    let t_range = ray_box_intersection(ray_origin, ray_dir);
    let t_start = t_range.x;
    let t_end = t_range.y;

    // Early exit if ray misses the volume
    if (t_start >= t_end) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Ray marching with adaptive step size
    var color = vec4<f32>(0.0);
    var t = t_start;
    let base_step = params.step_size;

    // Precompute light direction
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 2.0));

    for (var i = 0u; i < params.max_steps; i = i + 1u) {
        // Early termination
        if (t > t_end || color.a > 0.98) {
            break;
        }

        let pos = ray_origin + ray_dir * t;
        let density = sample_volume(pos);

        // Adaptive step size - larger steps in empty space
        var step = base_step;
        if (density < 0.005) {
            step = base_step * 2.0;  // Skip faster through empty regions
        }

        if (density > 0.01) {
            // Apply colormap
            let sample_color = apply_colormap(density);

            // Simplified shading - only compute gradient for visible samples
            var lighting = 1.0;
            if (density > 0.1) {  // Only shade denser regions
                let gradient = compute_gradient(pos, base_step * 2.0);  // Larger delta for performance
                lighting = max(dot(gradient, light_dir), 0.3) + 0.2;  // Ambient + diffuse
            }

            // Opacity based on density with better transfer function
            let alpha = clamp(density * 0.15, 0.0, 0.5);

            // Front-to-back compositing
            let src = vec4<f32>(sample_color.rgb * lighting, alpha);
            color = color + src * (1.0 - color.a);
        }

        t = t + step;
    }

    return color;
}
