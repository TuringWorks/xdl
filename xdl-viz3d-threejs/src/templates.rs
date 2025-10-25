//! HTML templates for Three.js volume rendering

use crate::colormaps::generate_colormap;
use crate::shaders::{fragment_shader, vertex_shader};

/// Generate complete HTML for Three.js volume rendering
pub fn generate_volume_html(
    volume_data: &[f32],
    dims: [usize; 3],
    colormap: &str,
    title: &str,
    threshold: f32,
    opacity: f32,
) -> String {
    let [nx, ny, nz] = dims;

    // Generate colormap data
    let colormap_colors = generate_colormap(colormap);
    let colormap_json = serde_json::to_string(&colormap_colors).unwrap();

    // Convert volume data to JSON
    let volume_json = serde_json::to_string(volume_data).unwrap();

    // Get shaders
    let vert_shader = vertex_shader();
    let frag_shader = fragment_shader();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            overflow: hidden;
            background: #1a1a1a;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }}
        #canvas {{
            display: block;
            width: 100vw;
            height: 100vh;
        }}
        #info {{
            position: absolute;
            top: 10px;
            left: 10px;
            color: white;
            background: rgba(0, 0, 0, 0.7);
            padding: 10px;
            border-radius: 5px;
            font-size: 14px;
        }}
        #loading {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: white;
            font-size: 24px;
            text-align: center;
        }}
    </style>
    <script type="importmap">
    {{
        "imports": {{
            "three": "https://cdn.jsdelivr.net/npm/three@0.161.0/build/three.module.js",
            "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.161.0/examples/jsm/"
        }}
    }}
    </script>
</head>
<body>
    <div id="loading">Loading volume data...</div>
    <div id="info">
        <strong>{title}</strong><br>
        Volume: {nx}×{ny}×{nz}<br>
        Colormap: {colormap}<br>
        Controls: Left mouse = Rotate, Wheel = Zoom
    </div>
    <canvas id="canvas"></canvas>

    <script type="module">
        import * as THREE from 'three';
        import {{ OrbitControls }} from 'three/addons/controls/OrbitControls.js';
        import {{ GUI }} from 'three/addons/libs/lil-gui.module.min.js';

        // Hide loading message
        setTimeout(() => {{
            document.getElementById('loading').style.display = 'none';
        }}, 100);

        // Volume data
        const volumeData = new Float32Array({volume_json});
        const dims = [{nx}, {ny}, {nz}];

        // Colormap data
        const colormapColors = {colormap_json};

        // Scene setup
        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x1a1a1a);

        // Camera
        const camera = new THREE.PerspectiveCamera(
            45,
            window.innerWidth / window.innerHeight,
            0.1,
            1000
        );
        camera.position.set(0, 0, 2);

        // Renderer
        const canvas = document.getElementById('canvas');
        const renderer = new THREE.WebGLRenderer({{ canvas, antialias: true }});
        renderer.setSize(window.innerWidth, window.innerHeight);
        renderer.setPixelRatio(window.devicePixelRatio);

        // Controls
        const controls = new OrbitControls(camera, renderer.domElement);
        controls.enableDamping = true;
        controls.dampingFactor = 0.05;

        // Create 3D texture from volume data
        const texture = new THREE.Data3DTexture(volumeData, dims[0], dims[1], dims[2]);
        texture.format = THREE.RedFormat;
        texture.type = THREE.FloatType;
        texture.minFilter = THREE.LinearFilter;
        texture.magFilter = THREE.LinearFilter;
        texture.unpackAlignment = 1;
        texture.needsUpdate = true;

        // Create colormap texture
        const colormapData = new Uint8Array(colormapColors.length * 4);
        for (let i = 0; i < colormapColors.length; i++) {{
            colormapData[i * 4 + 0] = colormapColors[i][0] * 255;
            colormapData[i * 4 + 1] = colormapColors[i][1] * 255;
            colormapData[i * 4 + 2] = colormapColors[i][2] * 255;
            colormapData[i * 4 + 3] = 255;
        }}
        const colormapTexture = new THREE.DataTexture(
            colormapData,
            colormapColors.length,
            1,
            THREE.RGBAFormat
        );
        colormapTexture.needsUpdate = true;

        // Shader material
        const params = {{
            threshold: {threshold},
            opacity: {opacity},
        }};

        const material = new THREE.ShaderMaterial({{
            uniforms: {{
                u_volume: {{ value: texture }},
                u_colormap: {{ value: colormapTexture }},
                u_threshold: {{ value: params.threshold }},
                u_opacity: {{ value: params.opacity }},
                u_volumeDims: {{ value: new THREE.Vector3(dims[0], dims[1], dims[2]) }},
                u_cameraPos: {{ value: camera.position }},
            }},
            vertexShader: `{vert_shader}`,
            fragmentShader: `{frag_shader}`,
            side: THREE.BackSide,
            transparent: true,
        }});

        // Create mesh
        const geometry = new THREE.BoxGeometry(1, 1, 1);
        const mesh = new THREE.Mesh(geometry, material);
        scene.add(mesh);

        // GUI
        const gui = new GUI();
        gui.add(params, 'threshold', 0.0, 1.0, 0.01).name('Threshold').onChange((value) => {{
            material.uniforms.u_threshold.value = value;
        }});
        gui.add(params, 'opacity', 0.0, 1.0, 0.01).name('Opacity').onChange((value) => {{
            material.uniforms.u_opacity.value = value;
        }});

        // Lighting (for reference, not used in volume rendering)
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
        scene.add(ambientLight);

        // Animation loop
        function animate() {{
            requestAnimationFrame(animate);

            // Update camera position in shader
            material.uniforms.u_cameraPos.value.copy(camera.position);

            controls.update();
            renderer.render(scene, camera);
        }}

        // Handle window resize
        window.addEventListener('resize', () => {{
            camera.aspect = window.innerWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(window.innerWidth, window.innerHeight);
        }});

        // Start animation
        animate();

        console.log('XDL Volume Visualization initialized');
        console.log('Volume dimensions:', dims);
        console.log('Voxel count:', volumeData.length);
    </script>
</body>
</html>"#,
        title = title,
        nx = nx,
        ny = ny,
        nz = nz,
        colormap = colormap,
        volume_json = volume_json,
        colormap_json = colormap_json,
        vert_shader = vert_shader.replace('`', r"\`"),
        frag_shader = frag_shader.replace('`', r"\`"),
        threshold = threshold,
        opacity = opacity,
    )
}
