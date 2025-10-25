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

/// Fragment shader for volume raycasting with lighting
pub fn fragment_shader() -> &'static str {
    r#"
uniform sampler3D u_volume;
uniform sampler2D u_colormap;
uniform float u_threshold;
uniform float u_opacity;
uniform vec3 u_volumeDims;
uniform vec3 u_cameraPos;
uniform float u_stepSize;
uniform int u_maxSteps;
uniform bool u_enableLighting;
uniform vec3 u_lightDirection;
uniform float u_ambient;
uniform float u_diffuse;
uniform float u_specular;
uniform float u_shininess;

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

// Calculate gradient for normal estimation
vec3 calculateGradient(vec3 texCoord, float delta) {
    vec3 gradient;
    gradient.x = texture(u_volume, texCoord + vec3(delta, 0.0, 0.0)).r -
                 texture(u_volume, texCoord - vec3(delta, 0.0, 0.0)).r;
    gradient.y = texture(u_volume, texCoord + vec3(0.0, delta, 0.0)).r -
                 texture(u_volume, texCoord - vec3(0.0, delta, 0.0)).r;
    gradient.z = texture(u_volume, texCoord + vec3(0.0, 0.0, delta)).r -
                 texture(u_volume, texCoord - vec3(0.0, 0.0, delta)).r;
    return normalize(gradient);
}

// Phong lighting calculation
vec3 calculateLighting(vec3 normal, vec3 viewDir, vec3 baseColor) {
    // Ambient
    vec3 ambient = u_ambient * baseColor;

    // Diffuse
    float diff = max(dot(normal, u_lightDirection), 0.0);
    vec3 diffuse = u_diffuse * diff * baseColor;

    // Specular
    vec3 reflectDir = reflect(-u_lightDirection, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), u_shininess);
    vec3 specular = u_specular * spec * vec3(1.0);

    return ambient + diffuse + specular;
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

    // Ray marching parameters (now uniforms for dynamic control)
    vec4 color = vec4(0.0);

    // Start ray marching
    float t_current = max(t.x, 0.0);
    for (int i = 0; i < 512; i++) {
        if (i >= u_maxSteps) break;
        if (t_current > t.y) break;

        vec3 pos = rayOrigin + rayDir * t_current;

        // Convert to texture coordinates [0, 1]
        vec3 texCoord = pos + 0.5;

        // Sample volume
        float density = texture(u_volume, texCoord).r;

        // Apply threshold
        if (density > u_threshold) {
            // Lookup base color from colormap
            vec4 sampleColor = texture2D(u_colormap, vec2(density, 0.5));

            // Apply lighting if enabled
            if (u_enableLighting) {
                // Calculate gradient normal
                vec3 normal = calculateGradient(texCoord, 0.01);
                if (length(normal) > 0.01) {
                    vec3 litColor = calculateLighting(normal, -rayDir, sampleColor.rgb);
                    sampleColor.rgb = litColor;
                }
            }

            // Apply opacity
            sampleColor.a *= u_opacity;

            // Gradient-based opacity modulation (enhance edges)
            vec3 gradient = calculateGradient(texCoord, 0.01);
            float gradientMag = length(gradient);
            sampleColor.a *= (1.0 + gradientMag * 2.0);
            sampleColor.a = clamp(sampleColor.a, 0.0, 1.0);

            // Front-to-back compositing
            color.rgb += (1.0 - color.a) * sampleColor.rgb * sampleColor.a;
            color.a += (1.0 - color.a) * sampleColor.a;

            // Early ray termination
            if (color.a >= 0.95) break;
        }

        t_current += u_stepSize;
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
