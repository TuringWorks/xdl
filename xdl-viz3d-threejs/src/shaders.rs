//! GLSL shaders for Three.js volume rendering

/// Vertex shader for volume raycasting
pub fn vertex_shader() -> &'static str {
    r#"
varying vec3 vPosition;
varying vec3 vNormal;

void main() {
    vPosition = position;
    vNormal = normal;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
"#
}

/// Fragment shader for volume raycasting
pub fn fragment_shader() -> &'static str {
    r#"
uniform sampler3D u_volume;
uniform sampler2D u_colormap;
uniform float u_threshold;
uniform float u_opacity;
uniform vec3 u_volumeDims;
uniform vec3 u_cameraPos;

varying vec3 vPosition;
varying vec3 vNormal;

// Ray-box intersection
vec2 intersectBox(vec3 orig, vec3 dir) {
    const vec3 box_min = vec3(-0.5);
    const vec3 box_max = vec3(0.5);
    vec3 inv_dir = 1.0 / dir;
    vec3 tmin_tmp = (box_min - orig) * inv_dir;
    vec3 tmax_tmp = (box_max - orig) * inv_dir;
    vec3 tmin = min(tmin_tmp, tmax_tmp);
    vec3 tmax = max(tmin_tmp, tmax_tmp);
    float t0 = max(tmin.x, max(tmin.y, tmin.z));
    float t1 = min(tmax.x, min(tmax.y, tmax.z));
    return vec2(t0, t1);
}

void main() {
    // Ray direction from camera to fragment
    vec3 rayOrigin = u_cameraPos;
    vec3 rayDir = normalize(vPosition - u_cameraPos);

    // Intersect ray with bounding box
    vec2 t = intersectBox(rayOrigin, rayDir);
    if (t.x > t.y || t.y < 0.0) {
        discard;
    }

    // Ray marching parameters
    float stepSize = 0.01;
    int maxSteps = 256;
    vec4 color = vec4(0.0);

    // Start ray marching
    float t_current = max(t.x, 0.0);
    for (int i = 0; i < maxSteps; i++) {
        if (t_current > t.y) break;

        vec3 pos = rayOrigin + rayDir * t_current;

        // Convert to texture coordinates [0, 1]
        vec3 texCoord = pos + 0.5;

        // Sample volume
        float density = texture(u_volume, texCoord).r;

        // Apply threshold
        if (density > u_threshold) {
            // Lookup color from colormap
            vec4 sampleColor = texture2D(u_colormap, vec2(density, 0.5));
            sampleColor.a *= u_opacity;

            // Front-to-back compositing
            color.rgb += (1.0 - color.a) * sampleColor.rgb * sampleColor.a;
            color.a += (1.0 - color.a) * sampleColor.a;

            // Early ray termination
            if (color.a >= 0.95) break;
        }

        t_current += stepSize;
    }

    gl_FragColor = color;
}
"#
}

/// Get shader uniforms as JSON
pub fn get_uniforms_json(
    volume_texture: &str,
    colormap_texture: &str,
    threshold: f32,
    opacity: f32,
    volume_dims: [usize; 3],
) -> String {
    format!(
        r#"{{
    "u_volume": {{ "value": {} }},
    "u_colormap": {{ "value": {} }},
    "u_threshold": {{ "value": {} }},
    "u_opacity": {{ "value": {} }},
    "u_volumeDims": {{ "value": [{}, {}, {}] }},
    "u_cameraPos": {{ "value": [0.0, 0.0, 2.0] }}
}}"#,
        volume_texture,
        colormap_texture,
        threshold,
        opacity,
        volume_dims[0],
        volume_dims[1],
        volume_dims[2]
    )
}
