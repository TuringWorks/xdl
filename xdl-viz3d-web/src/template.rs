//! HTML template generation for volume visualization

use base64::{engine::general_purpose, Engine as _};

/// Generate complete HTML page with embedded volume data and WebGPU renderer
pub fn generate_volume_viewer(
    volume_data: &[f32],
    dimensions: [usize; 3],
    colormap: &str,
    title: &str,
) -> String {
    // Encode volume data as base64
    let data_bytes: Vec<u8> = volume_data.iter().flat_map(|&f| f.to_le_bytes()).collect();
    let data_base64 = general_purpose::STANDARD.encode(&data_bytes);

    // Get colormap colors
    let colormap_data = get_colormap_data(colormap);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a1a;
            color: #fff;
            overflow: hidden;
        }}
        #canvas {{
            display: block;
            width: 100vw;
            height: 100vh;
        }}
        #controls {{
            position: absolute;
            top: 20px;
            right: 20px;
            background: rgba(0, 0, 0, 0.7);
            padding: 15px;
            border-radius: 8px;
            backdrop-filter: blur(10px);
        }}
        #info {{
            position: absolute;
            top: 20px;
            left: 20px;
            background: rgba(0, 0, 0, 0.7);
            padding: 15px;
            border-radius: 8px;
            backdrop-filter: blur(10px);
            font-size: 12px;
        }}
        button, select {{
            background: #007bff;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            margin: 5px 0;
            width: 100%;
        }}
        button:hover, select:hover {{
            background: #0056b3;
        }}
        .control-group {{
            margin: 10px 0;
        }}
        label {{
            display: block;
            margin-bottom: 5px;
            font-size: 12px;
        }}
        input[type="range"] {{
            width: 100%;
        }}
        #error {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            background: rgba(220, 53, 69, 0.9);
            padding: 20px;
            border-radius: 8px;
            display: none;
        }}
    </style>
</head>
<body>
    <canvas id="canvas"></canvas>
    <div id="info">
        <strong>{title}</strong><br>
        Volume: {dimx}×{dimy}×{dimz}<br>
        <span id="fps">FPS: --</span><br>
        <span id="status">Initializing...</span>
    </div>
    <div id="controls">
        <div class="control-group">
            <button id="reset">Reset Camera</button>
        </div>
        <div class="control-group">
            <label>Colormap:</label>
            <select id="colormap">
                <option value="VIRIDIS">Viridis</option>
                <option value="RAINBOW">Rainbow</option>
                <option value="PLASMA">Plasma</option>
                <option value="INFERNO">Inferno</option>
                <option value="TURBO">Turbo</option>
                <option value="GRAYSCALE">Grayscale</option>
            </select>
        </div>
        <div class="control-group">
            <label>Density Threshold: <span id="thresholdValue">0.01</span></label>
            <input type="range" id="threshold" min="0" max="0.1" step="0.001" value="0.01">
        </div>
        <div class="control-group">
            <label>Opacity: <span id="opacityValue">0.15</span></label>
            <input type="range" id="opacity" min="0.05" max="0.5" step="0.01" value="0.15">
        </div>
    </div>
    <div id="error"></div>

    <script type="module">
        // Volume data (embedded)
        const volumeDataBase64 = '{data_base64}';
        const dimensions = [{dimx}, {dimy}, {dimz}];
        // Decode asynchronously to avoid stack overflow
        const volumeData = await base64ToFloat32Array(volumeDataBase64);

        // Colormap data
        const colormapData = {colormap_json};
        let currentColormap = '{colormap}';

        // Camera initial positions - volume centered at origin
        const cameraPos = [0.0, 0.0, 2.0];  // Look at volume from front
        const cameraTarget = [0.0, 0.0, 0.0];  // Center of volume at origin
        const cameraUp = [0.0, 1.0, 0.0];

        // WebGPU shader (optimized ray marching from xdl-viz3d)
        const shaderCode = `{shader_code}`;

        // Helper function to decode base64 to Float32Array
        // Uses optimized approach to avoid stack overflow on large volumes
        async function base64ToFloat32Array(base64) {{
            console.log('Decoding base64 data, length:', base64.length);

            // Split base64 string into smaller chunks and decode incrementally
            const chunkSize = 1024 * 1024; // 1MB chunks
            const chunks = [];
            const numChunks = Math.ceil(base64.length / chunkSize);

            console.log('Processing', numChunks, 'chunks...');
            for (let i = 0; i < base64.length; i += chunkSize) {{
                const chunkNum = Math.floor(i / chunkSize) + 1;
                console.log('Decoding chunk', chunkNum, 'of', numChunks);

                const chunk = base64.substring(i, Math.min(i + chunkSize, base64.length));
                const dataUrl = 'data:application/octet-stream;base64,' + chunk;
                const response = await fetch(dataUrl);
                const arrayBuffer = await response.arrayBuffer();
                chunks.push(new Uint8Array(arrayBuffer));
            }}

            console.log('Concatenating chunks...');
            // Concatenate all chunks
            const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
            const result = new Uint8Array(totalLength);
            let offset = 0;
            for (const chunk of chunks) {{
                result.set(chunk, offset);
                offset += chunk.length;
            }}

            console.log('Base64 decoding complete, total bytes:', totalLength);
            return new Float32Array(result.buffer);
        }}

        // Initialize and catch errors
        (async () => {{
            try {{
                {webgpu_code}
                console.log('WebGPU initialization complete');
            }} catch (err) {{
                console.error('WebGPU Error:', err);
                document.getElementById('error').style.display = 'block';
                document.getElementById('error').textContent = 'Error: ' + err.message;
                document.getElementById('status').textContent = 'Error: ' + err.message;
            }}
        }})();
    </script>
</body>
</html>"#,
        title = title,
        dimx = dimensions[0],
        dimy = dimensions[1],
        dimz = dimensions[2],
        data_base64 = data_base64,
        colormap = colormap,
        colormap_json = colormap_data,
        shader_code = get_shader_code(),
        webgpu_code = get_webgpu_code(),
    )
}

fn get_colormap_data(name: &str) -> String {
    // Generate colormap LUT
    let colors = match name.to_uppercase().as_str() {
        "VIRIDIS" => generate_viridis(),
        "RAINBOW" => generate_rainbow(),
        "PLASMA" => generate_plasma(),
        "INFERNO" => generate_inferno(),
        "TURBO" => generate_turbo(),
        "GRAYSCALE" => generate_grayscale(),
        _ => generate_viridis(),
    };

    serde_json::to_string(&colors).unwrap()
}

fn generate_viridis() -> Vec<[u8; 4]> {
    // Simplified Viridis colormap (256 entries)
    (0..256)
        .map(|i| {
            let t = i as f32 / 255.0;
            [
                (68.0 + t * (59.0 - 68.0)) as u8,
                (1.0 + t * (217.0 - 1.0)) as u8,
                (84.0 + t * (134.0 - 84.0)) as u8,
                255,
            ]
        })
        .collect()
}

fn generate_rainbow() -> Vec<[u8; 4]> {
    use std::f32::consts::TAU;
    (0..256)
        .map(|i| {
            let t = i as f32 / 255.0;
            let r = (255.0 * (0.5 + 0.5 * (t * TAU).cos())) as u8;
            let g = (255.0 * (0.5 + 0.5 * (t * TAU + 2.0944).cos())) as u8;
            let b = (255.0 * (0.5 + 0.5 * (t * TAU + 4.18879).cos())) as u8;
            [r, g, b, 255]
        })
        .collect()
}

fn generate_plasma() -> Vec<[u8; 4]> {
    (0..256)
        .map(|i| {
            let t = i as f32 / 255.0;
            [
                (13.0 + t * (240.0 - 13.0)) as u8,
                (8.0 + t * (141.0 - 8.0)) as u8,
                (135.0 - t * 135.0) as u8,
                255,
            ]
        })
        .collect()
}

fn generate_inferno() -> Vec<[u8; 4]> {
    (0..256)
        .map(|i| {
            let t = i as f32 / 255.0;
            [
                (0.0 + t * 252.0) as u8,
                (0.0 + t * 147.0) as u8,
                (7.0 + t * 39.0) as u8,
                255,
            ]
        })
        .collect()
}

fn generate_turbo() -> Vec<[u8; 4]> {
    (0..256)
        .map(|i| {
            let t = i as f32 / 255.0;
            [
                (48.0 + t * (191.0 - 48.0)) as u8,
                (18.0 + t * (244.0 - 18.0)) as u8,
                (255.0 - t * 137.0) as u8,
                255,
            ]
        })
        .collect()
}

fn generate_grayscale() -> Vec<[u8; 4]> {
    (0..256).map(|i| [i as u8, i as u8, i as u8, 255]).collect()
}

fn get_shader_code() -> &'static str {
    r#"
// Simple volume ray marching shader for WebGPU
struct Uniforms {
    camera_pos: vec3f,
    _pad0: f32,
    camera_target: vec3f,
    _pad1: f32,
    camera_up: vec3f,
    _pad2: f32,
    dimensions: vec3f,
    _pad3: f32,
    aspect: f32,
    fov: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var volume_texture: texture_3d<f32>;
@group(0) @binding(2) var colormap_texture: texture_1d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((i & 1u) << 1u) - 1.0;
    let y = f32((i & 2u)) - 1.0;
    out.position = vec4f(x, y, 0.0, 1.0);
    out.uv = vec2f((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// Ray-box intersection
fn ray_box_intersection(origin: vec3f, dir: vec3f, box_min: vec3f, box_max: vec3f) -> vec2f {
    let inv_dir = 1.0 / dir;
    let t1 = (box_min - origin) * inv_dir;
    let t2 = (box_max - origin) * inv_dir;
    let tmin = min(t1, t2);
    let tmax = max(t1, t2);
    let t_near = max(max(tmin.x, tmin.y), tmin.z);
    let t_far = min(min(tmax.x, tmax.y), tmax.z);
    return vec2f(max(t_near, 0.0), max(t_far, 0.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // Compute ray direction from camera
    let forward = normalize(uniforms.camera_target - uniforms.camera_pos);
    let right = normalize(cross(forward, uniforms.camera_up));
    let up = cross(right, forward);

    let tan_fov = tan(uniforms.fov * 0.5);
    let ndc_x = (in.uv.x * 2.0 - 1.0) * uniforms.aspect * tan_fov;
    let ndc_y = (1.0 - in.uv.y * 2.0) * tan_fov;

    let ray_dir = normalize(forward + right * ndc_x + up * ndc_y);
    let ray_origin = uniforms.camera_pos;

    // Volume bounds centered at origin [-0.5, 0.5]^3 to match camera coordinate system
    let box_min = vec3f(-0.5, -0.5, -0.5);
    let box_max = vec3f(0.5, 0.5, 0.5);

    let t_range = ray_box_intersection(ray_origin, ray_dir, box_min, box_max);

    // If ray misses volume, show blue background
    if (t_range.x >= t_range.y) {
        return vec4f(0.1, 0.1, 0.3, 1.0);
    }

    // DEBUG: Visualize entry point to see if ray-box intersection is correct
    let entry_point = ray_origin + ray_dir * t_range.x;
    let entry_tex = entry_point + vec3f(0.5, 0.5, 0.5);
    // Uncomment to debug: return vec4f(entry_tex.x, entry_tex.y, entry_tex.z, 1.0);

    // Ray marching through volume
    var accumulated_color = vec4f(0.0);
    let step_size = 0.005;  // Smaller steps for smoother rendering
    let max_steps = 500;     // More steps for better quality

    var t = t_range.x + 0.001; // Slight offset to avoid artifacts at boundaries
    for (var step = 0; step < max_steps; step++) {
        if (t > t_range.y) {
            break;
        }
        // Disable early alpha termination for debugging
        // if (accumulated_color.a > 0.99) { break; }

        let pos = ray_origin + ray_dir * t;

        // Convert from world space [-0.5, 0.5] to texture coordinates [0, 1]
        let tex_coord = pos + vec3f(0.5, 0.5, 0.5);

        // Check bounds
        if (all(tex_coord >= vec3f(0.0)) && all(tex_coord <= vec3f(1.0))) {
            // Sample volume using textureLoad (integer coordinates)
            let voxel_coord = vec3i(tex_coord * uniforms.dimensions);

            // Bounds check for texture coordinates
            if (all(voxel_coord >= vec3i(0)) && all(voxel_coord < vec3i(uniforms.dimensions))) {
                let density = textureLoad(volume_texture, voxel_coord, 0).r;

                if (density > 0.02) {
                    // Sample colormap using textureLoad (no sampler required)
                    let colormap_index = clamp(i32(density * 255.0), 0, 255);
                    let color = textureLoad(colormap_texture, colormap_index, 0);

                    // Very low opacity transfer function for dense exponential volumes
                    // Use power of 3 to emphasize only high-density regions
                    let normalized_density = pow(density, 3.0);
                    let alpha = clamp(normalized_density * 0.015, 0.0, 0.15);

                    // Front-to-back alpha compositing
                    let src_color = vec4f(color.rgb, alpha);
                    accumulated_color = accumulated_color + src_color * (1.0 - accumulated_color.a);
                }
            }
        }

        t += step_size;
    }

    // DEBUG: Show accumulated alpha to diagnose early termination
    // return vec4f(accumulated_color.a, accumulated_color.a, accumulated_color.a, 1.0);

    // If no accumulation happened, show the volume was missed
    if (accumulated_color.a < 0.001) {
        return vec4f(0.0, 0.0, 0.0, 1.0);  // Black for empty rays
    }

    // Blend with background for better appearance
    let background = vec4f(0.05, 0.05, 0.1, 1.0);
    return vec4f(accumulated_color.rgb + background.rgb * (1.0 - accumulated_color.a), 1.0);
}
"#
}
fn get_webgpu_code() -> &'static str {
    r#"
console.log('=== WebGPU Initialization ===');
console.log('Volume dimensions:', dimensions);
console.log('Volume data length:', volumeData.length);
console.log('Volume data sample (first 10):', volumeData.slice(0, 10));

// Compute min/max without spread operator to avoid stack overflow
let minVal = volumeData[0];
let maxVal = volumeData[0];
for (let i = 1; i < volumeData.length; i++) {{
    if (volumeData[i] < minVal) minVal = volumeData[i];
    if (volumeData[i] > maxVal) maxVal = volumeData[i];
}}
console.log('Volume data min:', minVal);
console.log('Volume data max:', maxVal);

console.log('Camera initial:', cameraPos, cameraTarget, cameraUp);
console.log('Colormap data length:', colormapData.length);

const canvas = document.getElementById('canvas');
canvas.width = window.innerWidth;
canvas.height = window.innerHeight;

if (!navigator.gpu) {
    throw new Error('WebGPU not supported');
}

const adapter = await navigator.gpu.requestAdapter();
if (!adapter) {
    throw new Error('No GPU adapter found');
}

const device = await adapter.requestDevice();
const context = canvas.getContext('webgpu');
const format = navigator.gpu.getPreferredCanvasFormat();

context.configure({
    device: device,
    format: format,
});

console.log('Creating 3D volume texture...');

// Create 3D texture for volume data
console.log('Creating 3D texture with dimensions:', dimensions);
const volumeTexture = device.createTexture({
    size: [dimensions[0], dimensions[1], dimensions[2]],
    format: 'r32float',
    dimension: '3d',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
});
console.log('Volume texture created:', volumeTexture);

// Upload volume data to texture
console.log('Uploading volume data...');
const volumeArray = new Float32Array(volumeData);
console.log('Volume array buffer size:', volumeArray.buffer.byteLength, 'bytes');
device.queue.writeTexture(
    { texture: volumeTexture },
    volumeArray.buffer,
    { bytesPerRow: dimensions[0] * 4, rowsPerImage: dimensions[1] },
    [dimensions[0], dimensions[1], dimensions[2]]
);
console.log('Volume data uploaded successfully');

console.log('Creating colormap texture...');

// Create 1D texture for colormap
const colormapTexture = device.createTexture({
    size: [256, 1, 1],
    format: 'rgba8unorm',
    dimension: '1d',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
});

// Upload colormap data (already in correct format)
device.queue.writeTexture(
    { texture: colormapTexture },
    new Uint8Array(colormapData.flat()).buffer,
    { bytesPerRow: 256 * 4 },
    [256]
);

// Camera state
let camera = {
    position: [cameraPos[0], cameraPos[1], cameraPos[2]],
    target: [cameraTarget[0], cameraTarget[1], cameraTarget[2]],
    up: [cameraUp[0], cameraUp[1], cameraUp[2]],
    fov: 45.0,
    rotation: { x: 0, y: 0 },
    // Calculate distance from camera to target (not from origin)
    distance: Math.sqrt(
        (cameraPos[0] - cameraTarget[0])**2 +
        (cameraPos[1] - cameraTarget[1])**2 +
        (cameraPos[2] - cameraTarget[2])**2
    )
};

// Create uniform buffer
// Size: vec3f + pad (16) + vec3f + pad (16) + vec3f + pad (16) + vec3f + pad (16) + 2*f32 (8) = 72 bytes, round to 80
const uniformBuffer = device.createBuffer({
    size: 80,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
});

console.log('Loading ray marching shader...');

// Load the ray marching shader
console.log('Creating shader module...');
console.log('Shader code length:', shaderCode.length);
const shaderModule = device.createShaderModule({
    code: shaderCode
});
console.log('Shader module created:', shaderModule);

// Create bind group layout
const bindGroupLayout = device.createBindGroupLayout({
    entries: [
        { binding: 0, visibility: GPUShaderStage.FRAGMENT, buffer: { type: 'uniform' } },
        { binding: 1, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: 'unfilterable-float', viewDimension: '3d' } },
        { binding: 2, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: 'unfilterable-float', viewDimension: '1d' } },
    ],
});

// Create bind group
const bindGroup = device.createBindGroup({
    layout: bindGroupLayout,
    entries: [
        { binding: 0, resource: { buffer: uniformBuffer } },
        { binding: 1, resource: volumeTexture.createView() },
        { binding: 2, resource: colormapTexture.createView() },
    ],
});

// Create pipeline layout
const pipelineLayout = device.createPipelineLayout({
    bindGroupLayouts: [bindGroupLayout],
});

// Create render pipeline
const pipeline = device.createRenderPipeline({
    layout: pipelineLayout,
    vertex: {
        module: shaderModule,
        entryPoint: 'vs_main',
    },
    fragment: {
        module: shaderModule,
        entryPoint: 'fs_main',
        targets: [{ format: format }],
    },
});

console.log('Pipeline created successfully');

// Update uniforms
let frameCount = 0;
function updateUniforms() {
    // Calculate camera position from rotation
    const rad = camera.distance;
    const theta = camera.rotation.y;
    const phi = camera.rotation.x;

    camera.position = [
        camera.target[0] + rad * Math.sin(theta) * Math.cos(phi),
        camera.target[1] + rad * Math.sin(phi),
        camera.target[2] + rad * Math.cos(theta) * Math.cos(phi)
    ];

    // Pack uniforms to match shader layout
    const uniforms = new Float32Array(20); // 80 bytes / 4

    // Log camera state on first frame
    if (frameCount === 0) {
        console.log('=== First Frame Camera State ===');
        console.log('Camera position:', camera.position);
        console.log('Camera target:', camera.target);
        console.log('Camera distance:', camera.distance);
        console.log('Camera rotation:', camera.rotation);
        console.log('FOV:', camera.fov, 'degrees');
        console.log('Aspect ratio:', canvas.width / canvas.height);
        console.log('Uniforms buffer size:', uniforms.length * 4, 'bytes');

        // Count non-zero voxels
        let nonZeroCount = 0;
        let maxDensity = 0;
        for (let i = 0; i < volumeData.length; i++) {
            if (volumeData[i] > 0.01) nonZeroCount++;
            if (volumeData[i] > maxDensity) maxDensity = volumeData[i];
        }
        console.log('Non-zero voxels (>0.01):', nonZeroCount, 'of', volumeData.length);
        console.log('Max density in volume:', maxDensity);
    }

    // camera_pos (vec3f) + _pad0 (f32)
    uniforms[0] = camera.position[0];
    uniforms[1] = camera.position[1];
    uniforms[2] = camera.position[2];
    uniforms[3] = 0.0; // padding

    // camera_target (vec3f) + _pad1 (f32)
    uniforms[4] = camera.target[0];
    uniforms[5] = camera.target[1];
    uniforms[6] = camera.target[2];
    uniforms[7] = 0.0; // padding

    // camera_up (vec3f) + _pad2 (f32)
    uniforms[8] = camera.up[0];
    uniforms[9] = camera.up[1];
    uniforms[10] = camera.up[2];
    uniforms[11] = 0.0; // padding

    // dimensions (vec3f) + _pad3 (f32)
    uniforms[12] = dimensions[0];
    uniforms[13] = dimensions[1];
    uniforms[14] = dimensions[2];
    uniforms[15] = 0.0; // padding

    // aspect (f32)
    uniforms[16] = canvas.width / canvas.height;

    // fov (f32) - convert degrees to radians
    uniforms[17] = camera.fov * Math.PI / 180.0;

    device.queue.writeBuffer(uniformBuffer, 0, uniforms.buffer);
}

// Mouse interaction
let isDragging = false;
let lastX = 0, lastY = 0;

canvas.addEventListener('mousedown', (e) => {
    isDragging = true;
    lastX = e.clientX;
    lastY = e.clientY;
});

canvas.addEventListener('mousemove', (e) => {
    if (!isDragging) return;

    const dx = e.clientX - lastX;
    const dy = e.clientY - lastY;

    camera.rotation.y += dx * 0.01;
    camera.rotation.x += dy * 0.01;

    // Clamp phi
    camera.rotation.x = Math.max(-Math.PI/2 + 0.1, Math.min(Math.PI/2 - 0.1, camera.rotation.x));

    lastX = e.clientX;
    lastY = e.clientY;
});

canvas.addEventListener('mouseup', () => {
    isDragging = false;
});

canvas.addEventListener('wheel', (e) => {
    e.preventDefault();
    camera.distance *= (1 + e.deltaY * 0.001);
    camera.distance = Math.max(0.5, Math.min(10.0, camera.distance));
});

// Render loop
function render() {
    updateUniforms();

    const encoder = device.createCommandEncoder();
    const view = context.getCurrentTexture().createView();

    const renderPass = encoder.beginRenderPass({
        colorAttachments: [{
            view: view,
            clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            loadOp: 'clear',
            storeOp: 'store',
        }],
    });

    renderPass.setPipeline(pipeline);
    renderPass.setBindGroup(0, bindGroup);
    renderPass.draw(6); // Full-screen quad
    renderPass.end();

    device.queue.submit([encoder.finish()]);

    frameCount++;
    if (frameCount === 1) {
        console.log('First frame rendered successfully');
    }

    document.getElementById('status').textContent = `Rendering (frame ${frameCount})`;
    requestAnimationFrame(render);
}

render();
console.log('Volume rendering started');
"#
}
