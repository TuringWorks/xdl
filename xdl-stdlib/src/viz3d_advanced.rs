//! Advanced 3D Visualization Functions
//!
//! This module provides advanced 3D visualization capabilities:
//! - ISOSURFACE - Extract isosurfaces using marching cubes
//! - SHADE_VOLUME - Direct volume rendering
//! - PARTICLE_TRACE - Particle tracing in vector fields
//! - STREAMLINE - Streamline visualization

use std::collections::HashMap;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Helper to extract f64 from XdlValue
fn value_to_f64(v: &XdlValue) -> Option<f64> {
    match v {
        XdlValue::Float(f) => Some(*f as f64),
        XdlValue::Double(d) => Some(*d),
        XdlValue::Int(i) => Some(*i as f64),
        XdlValue::Long(l) => Some(*l as f64),
        XdlValue::Byte(b) => Some(*b as f64),
        _ => None,
    }
}

/// Extract 3D volume data from XdlValue
fn extract_volume_3d(value: &XdlValue) -> XdlResult<(Vec<f64>, [usize; 3])> {
    match value {
        XdlValue::MultiDimArray { data, shape } => {
            if shape.len() != 3 {
                return Err(XdlError::InvalidArgument(
                    "Expected 3D array for volume data".to_string(),
                ));
            }
            Ok((data.clone(), [shape[0], shape[1], shape[2]]))
        }
        XdlValue::Array(data) => {
            // Assume cubic if just 1D array
            let n = (data.len() as f64).cbrt().round() as usize;
            if n * n * n != data.len() {
                return Err(XdlError::InvalidArgument(
                    "1D array size must be a perfect cube, or use MultiDimArray".to_string(),
                ));
            }
            Ok((data.clone(), [n, n, n]))
        }
        _ => Err(XdlError::InvalidArgument(
            "Volume data must be a 3D array".to_string(),
        )),
    }
}

/// ISOSURFACE - Extract isosurface from 3D volume data
/// IDL syntax: ISOSURFACE, volume, isovalue, verts, polys [, /VERBOSE]
///
/// Uses marching cubes algorithm to extract triangulated surface
pub fn isosurface(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "ISOSURFACE: Expected volume and isovalue arguments".to_string(),
        ));
    }

    let (volume, dims) = extract_volume_3d(&args[0])?;
    let isovalue = value_to_f64(&args[1]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[1]),
        }
    })?;

    let verbose = keywords.contains_key("VERBOSE");

    if verbose {
        println!(
            "ISOSURFACE: Extracting surface at value {} from {}x{}x{} volume",
            isovalue, dims[0], dims[1], dims[2]
        );
    }

    // Marching cubes algorithm
    let (vertices, triangles) = marching_cubes(&volume, dims, isovalue);

    if verbose {
        println!(
            "ISOSURFACE: Extracted {} vertices, {} triangles",
            vertices.len() / 3,
            triangles.len() / 3
        );
    }

    // Return vertices and triangle indices as nested array
    let verts_value = XdlValue::Array(vertices);
    let polys_value = XdlValue::Array(triangles.iter().map(|&i| i as f64).collect());

    Ok(XdlValue::NestedArray(vec![verts_value, polys_value]))
}

/// SHADE_VOLUME - Direct volume rendering
/// IDL syntax: result = SHADE_VOLUME(volume [, OPACITY=opacity] [, /LOW])
///
/// Performs ray casting through the volume for direct rendering
pub fn shade_volume(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "SHADE_VOLUME: Expected volume argument".to_string(),
        ));
    }

    let (volume, dims) = extract_volume_3d(&args[0])?;

    // Get opacity scale
    let opacity = keywords
        .get("OPACITY")
        .and_then(|v| value_to_f64(v))
        .unwrap_or(1.0);

    // LOW keyword inverts the rendering (low values are opaque)
    let low_mode = keywords.contains_key("LOW");

    println!(
        "SHADE_VOLUME: Rendering {}x{}x{} volume (opacity={}, low={})",
        dims[0], dims[1], dims[2], opacity, low_mode
    );

    // Compute volume statistics for transfer function
    let (min_val, max_val) = volume.iter().fold((f64::MAX, f64::MIN), |(min, max), &v| {
        (min.min(v), max.max(v))
    });

    let range = max_val - min_val;
    if range < 1e-10 {
        println!("SHADE_VOLUME: Warning - volume has no variation");
    }

    // Generate a simple 2D projection (maximum intensity projection)
    let proj_size = dims[0].max(dims[1]);
    let mut projection = vec![0.0; proj_size * proj_size];

    // Maximum intensity projection along Z
    for y in 0..dims[1].min(proj_size) {
        for x in 0..dims[0].min(proj_size) {
            let mut max_intensity: f64 = 0.0;
            for z in 0..dims[2] {
                let idx = z * dims[0] * dims[1] + y * dims[0] + x;
                if idx < volume.len() {
                    let val = (volume[idx] - min_val) / range;
                    let val = if low_mode { 1.0 - val } else { val };
                    max_intensity = max_intensity.max(val * opacity);
                }
            }
            projection[y * proj_size + x] = max_intensity;
        }
    }

    println!(
        "SHADE_VOLUME: Generated {}x{} projection",
        proj_size, proj_size
    );

    // Return projection as 2D array
    Ok(XdlValue::MultiDimArray {
        data: projection,
        shape: vec![proj_size, proj_size],
    })
}

/// PARTICLE_TRACE - Trace particles through a vector field
/// IDL syntax: PARTICLE_TRACE, vx, vy, vz, seeds, output [, STEPS=n] [, DT=timestep]
///
/// Traces massless particles through a 3D vector field
pub fn particle_trace(
    args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::InvalidArgument(
            "PARTICLE_TRACE: Expected vx, vy, vz, seeds arguments".to_string(),
        ));
    }

    // Extract vector field components
    let (vx, dims) = extract_volume_3d(&args[0])?;
    let (vy, _) = extract_volume_3d(&args[1])?;
    let (vz, _) = extract_volume_3d(&args[2])?;

    // Extract seed points
    let seeds = match &args[3] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::NestedArray(arr) => {
            let mut flat = Vec::new();
            for v in arr {
                if let XdlValue::Array(a) = v {
                    flat.extend(a.iter().cloned());
                }
            }
            flat
        }
        _ => {
            return Err(XdlError::InvalidArgument(
                "Seeds must be an array of [x, y, z] coordinates".to_string(),
            ))
        }
    };

    // Get parameters
    let num_steps = keywords
        .get("STEPS")
        .and_then(|v| value_to_f64(v))
        .unwrap_or(100.0) as usize;
    let dt = keywords
        .get("DT")
        .and_then(|v| value_to_f64(v))
        .unwrap_or(0.1);

    let num_particles = seeds.len() / 3;
    println!(
        "PARTICLE_TRACE: Tracing {} particles for {} steps (dt={})",
        num_particles, num_steps, dt
    );
    println!(
        "  Vector field: {}x{}x{}",
        dims[0], dims[1], dims[2]
    );

    // Trace particles using RK4 integration
    let mut traces = Vec::new();

    for i in 0..num_particles {
        let mut pos = [seeds[i * 3], seeds[i * 3 + 1], seeds[i * 3 + 2]];
        let mut trace = vec![pos[0], pos[1], pos[2]];

        for _ in 0..num_steps {
            // Sample velocity at current position (trilinear interpolation)
            let vel = sample_vector_field(&vx, &vy, &vz, dims, pos);

            // RK4 integration
            let k1 = vel;
            let pos1 = [
                pos[0] + 0.5 * dt * k1[0],
                pos[1] + 0.5 * dt * k1[1],
                pos[2] + 0.5 * dt * k1[2],
            ];
            let k2 = sample_vector_field(&vx, &vy, &vz, dims, pos1);

            let pos2 = [
                pos[0] + 0.5 * dt * k2[0],
                pos[1] + 0.5 * dt * k2[1],
                pos[2] + 0.5 * dt * k2[2],
            ];
            let k3 = sample_vector_field(&vx, &vy, &vz, dims, pos2);

            let pos3 = [
                pos[0] + dt * k3[0],
                pos[1] + dt * k3[1],
                pos[2] + dt * k3[2],
            ];
            let k4 = sample_vector_field(&vx, &vy, &vz, dims, pos3);

            // Update position
            pos[0] += dt / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]);
            pos[1] += dt / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]);
            pos[2] += dt / 6.0 * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]);

            // Check bounds
            if pos[0] < 0.0
                || pos[0] >= dims[0] as f64
                || pos[1] < 0.0
                || pos[1] >= dims[1] as f64
                || pos[2] < 0.0
                || pos[2] >= dims[2] as f64
            {
                break;
            }

            trace.push(pos[0]);
            trace.push(pos[1]);
            trace.push(pos[2]);
        }

        traces.push(XdlValue::Array(trace));
    }

    println!("PARTICLE_TRACE: Generated {} particle traces", traces.len());

    Ok(XdlValue::NestedArray(traces))
}

/// STREAMLINE - Generate streamlines from vector field
/// IDL syntax: STREAMLINE, vx, vy, vz, seeds, output [, STEPS=n] [, DT=timestep] [, /BACKWARD]
///
/// Similar to PARTICLE_TRACE but optimized for visualization
pub fn streamline(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::InvalidArgument(
            "STREAMLINE: Expected vx, vy, vz, seeds arguments".to_string(),
        ));
    }

    // Use particle_trace as the core implementation
    let backward = keywords.contains_key("BACKWARD");

    // If backward, negate the vector field
    let (mut vx, dims) = extract_volume_3d(&args[0])?;
    let (mut vy, _) = extract_volume_3d(&args[1])?;
    let (mut vz, _) = extract_volume_3d(&args[2])?;

    if backward {
        vx.iter_mut().for_each(|v| *v = -*v);
        vy.iter_mut().for_each(|v| *v = -*v);
        vz.iter_mut().for_each(|v| *v = -*v);
    }

    // Extract seed points
    let seeds = match &args[3] {
        XdlValue::Array(arr) => arr.clone(),
        XdlValue::NestedArray(arr) => {
            let mut flat = Vec::new();
            for v in arr {
                if let XdlValue::Array(a) = v {
                    flat.extend(a.iter().cloned());
                }
            }
            flat
        }
        _ => {
            return Err(XdlError::InvalidArgument(
                "Seeds must be an array of [x, y, z] coordinates".to_string(),
            ))
        }
    };

    let num_steps = keywords
        .get("STEPS")
        .and_then(|v| value_to_f64(v))
        .unwrap_or(200.0) as usize;
    let dt = keywords
        .get("DT")
        .and_then(|v| value_to_f64(v))
        .unwrap_or(0.05);

    let num_seeds = seeds.len() / 3;
    println!(
        "STREAMLINE: Generating {} streamlines for {} steps (backward={})",
        num_seeds, num_steps, backward
    );

    // Generate streamlines with adaptive step size
    let mut streamlines = Vec::new();

    for i in 0..num_seeds {
        let mut pos = [seeds[i * 3], seeds[i * 3 + 1], seeds[i * 3 + 2]];
        let mut line = vec![pos[0], pos[1], pos[2]];
        let mut step_count = 0;

        while step_count < num_steps {
            let vel = sample_vector_field(&vx, &vy, &vz, dims, pos);
            let vel_mag = (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]).sqrt();

            if vel_mag < 1e-10 {
                break; // Stagnation point
            }

            // Adaptive step size based on velocity magnitude
            let adaptive_dt = dt / vel_mag.max(1.0);

            // Euler integration (faster than RK4 for visualization)
            pos[0] += adaptive_dt * vel[0];
            pos[1] += adaptive_dt * vel[1];
            pos[2] += adaptive_dt * vel[2];

            // Check bounds
            if pos[0] < 0.0
                || pos[0] >= dims[0] as f64
                || pos[1] < 0.0
                || pos[1] >= dims[1] as f64
                || pos[2] < 0.0
                || pos[2] >= dims[2] as f64
            {
                break;
            }

            line.push(pos[0]);
            line.push(pos[1]);
            line.push(pos[2]);
            step_count += 1;
        }

        streamlines.push(XdlValue::Array(line));
    }

    println!("STREAMLINE: Generated {} streamlines", streamlines.len());

    Ok(XdlValue::NestedArray(streamlines))
}

/// VOXEL_PROJ - Project voxel data to 2D image
/// IDL syntax: result = VOXEL_PROJ(volume [, XSIZE=xsize] [, YSIZE=ysize] [, /MAXIMUM] [, /AVERAGE])
pub fn voxel_proj(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "VOXEL_PROJ: Expected volume argument".to_string(),
        ));
    }

    let (volume, dims) = extract_volume_3d(&args[0])?;

    let xsize = keywords
        .get("XSIZE")
        .and_then(|v| value_to_f64(v))
        .unwrap_or(dims[0] as f64) as usize;
    let ysize = keywords
        .get("YSIZE")
        .and_then(|v| value_to_f64(v))
        .unwrap_or(dims[1] as f64) as usize;

    let use_max = keywords.contains_key("MAXIMUM");
    let use_avg = keywords.contains_key("AVERAGE");

    println!(
        "VOXEL_PROJ: Projecting {}x{}x{} volume to {}x{} image",
        dims[0], dims[1], dims[2], xsize, ysize
    );

    let mut projection = vec![0.0; xsize * ysize];

    for y in 0..ysize.min(dims[1]) {
        for x in 0..xsize.min(dims[0]) {
            let mut value: f64 = if use_max { f64::MIN } else { 0.0 };
            let mut count = 0;

            for z in 0..dims[2] {
                let idx = z * dims[0] * dims[1] + y * dims[0] + x;
                if idx < volume.len() {
                    if use_max {
                        value = value.max(volume[idx]);
                    } else {
                        value += volume[idx];
                    }
                    count += 1;
                }
            }

            if use_avg && count > 0 {
                value /= count as f64;
            }

            projection[y * xsize + x] = value;
        }
    }

    Ok(XdlValue::MultiDimArray {
        data: projection,
        shape: vec![xsize, ysize],
    })
}

/// POLYSHADE - Shade a 3D polygon mesh
/// IDL syntax: result = POLYSHADE(vertices, polygons [, /GOURAUD] [, SHADES=shades])
pub fn polyshade(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "POLYSHADE: Expected vertices and polygons arguments".to_string(),
        ));
    }

    let verts = match &args[0] {
        XdlValue::Array(arr) => arr.clone(),
        _ => {
            return Err(XdlError::InvalidArgument(
                "Vertices must be an array".to_string(),
            ))
        }
    };

    let polys = match &args[1] {
        XdlValue::Array(arr) => arr.iter().map(|&v| v as i32).collect::<Vec<_>>(),
        _ => {
            return Err(XdlError::InvalidArgument(
                "Polygons must be an array".to_string(),
            ))
        }
    };

    let num_verts = verts.len() / 3;
    let _gouraud = keywords.contains_key("GOURAUD");

    println!(
        "POLYSHADE: Shading mesh with {} vertices",
        num_verts
    );

    // Compute vertex normals for shading
    let mut normals = vec![0.0; verts.len()];
    let mut i = 0;
    while i < polys.len() {
        let n = polys[i] as usize;
        if n >= 3 && i + n < polys.len() {
            // Get triangle vertices
            let i0 = polys[i + 1] as usize;
            let i1 = polys[i + 2] as usize;
            let i2 = polys[i + 3] as usize;

            if i0 * 3 + 2 < verts.len() && i1 * 3 + 2 < verts.len() && i2 * 3 + 2 < verts.len() {
                // Compute face normal
                let v0 = [verts[i0 * 3], verts[i0 * 3 + 1], verts[i0 * 3 + 2]];
                let v1 = [verts[i1 * 3], verts[i1 * 3 + 1], verts[i1 * 3 + 2]];
                let v2 = [verts[i2 * 3], verts[i2 * 3 + 1], verts[i2 * 3 + 2]];

                let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
                let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

                let normal = [
                    e1[1] * e2[2] - e1[2] * e2[1],
                    e1[2] * e2[0] - e1[0] * e2[2],
                    e1[0] * e2[1] - e1[1] * e2[0],
                ];

                // Accumulate to vertex normals
                for &idx in &[i0, i1, i2] {
                    normals[idx * 3] += normal[0];
                    normals[idx * 3 + 1] += normal[1];
                    normals[idx * 3 + 2] += normal[2];
                }
            }
        }
        i += n as usize + 1;
    }

    // Normalize
    for i in 0..num_verts {
        let len = (normals[i * 3].powi(2)
            + normals[i * 3 + 1].powi(2)
            + normals[i * 3 + 2].powi(2))
        .sqrt();
        if len > 1e-10 {
            normals[i * 3] /= len;
            normals[i * 3 + 1] /= len;
            normals[i * 3 + 2] /= len;
        }
    }

    // Compute shading values (simple directional light)
    let light_dir = [0.577, 0.577, 0.577]; // Normalized (1,1,1)
    let mut shades = Vec::with_capacity(num_verts);

    for i in 0..num_verts {
        let dot = normals[i * 3] * light_dir[0]
            + normals[i * 3 + 1] * light_dir[1]
            + normals[i * 3 + 2] * light_dir[2];
        let shade = (dot.max(0.0) * 255.0) as f64;
        shades.push(shade);
    }

    Ok(XdlValue::Array(shades))
}

// ============================================================================
// Helper functions
// ============================================================================

/// Sample vector field at position using trilinear interpolation
fn sample_vector_field(
    vx: &[f64],
    vy: &[f64],
    vz: &[f64],
    dims: [usize; 3],
    pos: [f64; 3],
) -> [f64; 3] {
    let x = pos[0].max(0.0).min((dims[0] - 1) as f64);
    let y = pos[1].max(0.0).min((dims[1] - 1) as f64);
    let z = pos[2].max(0.0).min((dims[2] - 1) as f64);

    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let z0 = z.floor() as usize;
    let x1 = (x0 + 1).min(dims[0] - 1);
    let y1 = (y0 + 1).min(dims[1] - 1);
    let z1 = (z0 + 1).min(dims[2] - 1);

    let fx = x - x0 as f64;
    let fy = y - y0 as f64;
    let fz = z - z0 as f64;

    // Trilinear interpolation
    let idx = |ix: usize, iy: usize, iz: usize| iz * dims[0] * dims[1] + iy * dims[0] + ix;

    let interp = |field: &[f64]| {
        let c000 = field.get(idx(x0, y0, z0)).copied().unwrap_or(0.0);
        let c100 = field.get(idx(x1, y0, z0)).copied().unwrap_or(0.0);
        let c010 = field.get(idx(x0, y1, z0)).copied().unwrap_or(0.0);
        let c110 = field.get(idx(x1, y1, z0)).copied().unwrap_or(0.0);
        let c001 = field.get(idx(x0, y0, z1)).copied().unwrap_or(0.0);
        let c101 = field.get(idx(x1, y0, z1)).copied().unwrap_or(0.0);
        let c011 = field.get(idx(x0, y1, z1)).copied().unwrap_or(0.0);
        let c111 = field.get(idx(x1, y1, z1)).copied().unwrap_or(0.0);

        let c00 = c000 * (1.0 - fx) + c100 * fx;
        let c10 = c010 * (1.0 - fx) + c110 * fx;
        let c01 = c001 * (1.0 - fx) + c101 * fx;
        let c11 = c011 * (1.0 - fx) + c111 * fx;

        let c0 = c00 * (1.0 - fy) + c10 * fy;
        let c1 = c01 * (1.0 - fy) + c11 * fy;

        c0 * (1.0 - fz) + c1 * fz
    };

    [interp(vx), interp(vy), interp(vz)]
}

/// Marching cubes edge table
const EDGE_TABLE: [u16; 256] = [
    0x0, 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c, 0x80c, 0x905, 0xa0f, 0xb06, 0xc0a, 0xd03,
    0xe09, 0xf00, 0x190, 0x99, 0x393, 0x29a, 0x596, 0x49f, 0x795, 0x69c, 0x99c, 0x895, 0xb9f,
    0xa96, 0xd9a, 0xc93, 0xf99, 0xe90, 0x230, 0x339, 0x33, 0x13a, 0x636, 0x73f, 0x435, 0x53c,
    0xa3c, 0xb35, 0x83f, 0x936, 0xe3a, 0xf33, 0xc39, 0xd30, 0x3a0, 0x2a9, 0x1a3, 0xaa, 0x7a6,
    0x6af, 0x5a5, 0x4ac, 0xbac, 0xaa5, 0x9af, 0x8a6, 0xfaa, 0xea3, 0xda9, 0xca0, 0x460, 0x569,
    0x663, 0x76a, 0x66, 0x16f, 0x265, 0x36c, 0xc6c, 0xd65, 0xe6f, 0xf66, 0x86a, 0x963, 0xa69,
    0xb60, 0x5f0, 0x4f9, 0x7f3, 0x6fa, 0x1f6, 0xff, 0x3f5, 0x2fc, 0xdfc, 0xcf5, 0xfff, 0xef6,
    0x9fa, 0x8f3, 0xbf9, 0xaf0, 0x650, 0x759, 0x453, 0x55a, 0x256, 0x35f, 0x55, 0x15c, 0xe5c,
    0xf55, 0xc5f, 0xd56, 0xa5a, 0xb53, 0x859, 0x950, 0x7c0, 0x6c9, 0x5c3, 0x4ca, 0x3c6, 0x2cf,
    0x1c5, 0xcc, 0xfcc, 0xec5, 0xdcf, 0xcc6, 0xbca, 0xac3, 0x9c9, 0x8c0, 0x8c0, 0x9c9, 0xac3,
    0xbca, 0xcc6, 0xdcf, 0xec5, 0xfcc, 0xcc, 0x1c5, 0x2cf, 0x3c6, 0x4ca, 0x5c3, 0x6c9, 0x7c0,
    0x950, 0x859, 0xb53, 0xa5a, 0xd56, 0xc5f, 0xf55, 0xe5c, 0x15c, 0x55, 0x35f, 0x256, 0x55a,
    0x453, 0x759, 0x650, 0xaf0, 0xbf9, 0x8f3, 0x9fa, 0xef6, 0xfff, 0xcf5, 0xdfc, 0x2fc, 0x3f5,
    0xff, 0x1f6, 0x6fa, 0x7f3, 0x4f9, 0x5f0, 0xb60, 0xa69, 0x963, 0x86a, 0xf66, 0xe6f, 0xd65,
    0xc6c, 0x36c, 0x265, 0x16f, 0x66, 0x76a, 0x663, 0x569, 0x460, 0xca0, 0xda9, 0xea3, 0xfaa,
    0x8a6, 0x9af, 0xaa5, 0xbac, 0x4ac, 0x5a5, 0x6af, 0x7a6, 0xaa, 0x1a3, 0x2a9, 0x3a0, 0xd30,
    0xc39, 0xf33, 0xe3a, 0x936, 0x83f, 0xb35, 0xa3c, 0x53c, 0x435, 0x73f, 0x636, 0x13a, 0x33,
    0x339, 0x230, 0xe90, 0xf99, 0xc93, 0xd9a, 0xa96, 0xb9f, 0x895, 0x99c, 0x69c, 0x795, 0x49f,
    0x596, 0x29a, 0x393, 0x99, 0x190, 0xf00, 0xe09, 0xd03, 0xc0a, 0xb06, 0xa0f, 0x905, 0x80c,
    0x70c, 0x605, 0x50f, 0x406, 0x30a, 0x203, 0x109, 0x0,
];

/// Simplified marching cubes implementation
fn marching_cubes(volume: &[f64], dims: [usize; 3], isovalue: f64) -> (Vec<f64>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();

    let idx = |x: usize, y: usize, z: usize| z * dims[0] * dims[1] + y * dims[0] + x;

    for z in 0..dims[2].saturating_sub(1) {
        for y in 0..dims[1].saturating_sub(1) {
            for x in 0..dims[0].saturating_sub(1) {
                // Get cube corner values
                let v = [
                    volume[idx(x, y, z)],
                    volume[idx(x + 1, y, z)],
                    volume[idx(x + 1, y + 1, z)],
                    volume[idx(x, y + 1, z)],
                    volume[idx(x, y, z + 1)],
                    volume[idx(x + 1, y, z + 1)],
                    volume[idx(x + 1, y + 1, z + 1)],
                    volume[idx(x, y + 1, z + 1)],
                ];

                // Determine cube index
                let mut cube_idx = 0u8;
                for (i, &val) in v.iter().enumerate() {
                    if val < isovalue {
                        cube_idx |= 1 << i;
                    }
                }

                // Skip if entirely inside or outside
                if cube_idx == 0 || cube_idx == 255 {
                    continue;
                }

                // Get edge flags
                let edges = EDGE_TABLE[cube_idx as usize];
                if edges == 0 {
                    continue;
                }

                // Interpolate vertices on edges
                let mut vert_list = [[0.0f64; 3]; 12];

                if edges & 1 != 0 {
                    vert_list[0] = interp_vertex(x as f64, y as f64, z as f64, x as f64 + 1.0, y as f64, z as f64, v[0], v[1], isovalue);
                }
                if edges & 2 != 0 {
                    vert_list[1] = interp_vertex(x as f64 + 1.0, y as f64, z as f64, x as f64 + 1.0, y as f64 + 1.0, z as f64, v[1], v[2], isovalue);
                }
                if edges & 4 != 0 {
                    vert_list[2] = interp_vertex(x as f64 + 1.0, y as f64 + 1.0, z as f64, x as f64, y as f64 + 1.0, z as f64, v[2], v[3], isovalue);
                }
                if edges & 8 != 0 {
                    vert_list[3] = interp_vertex(x as f64, y as f64 + 1.0, z as f64, x as f64, y as f64, z as f64, v[3], v[0], isovalue);
                }
                if edges & 16 != 0 {
                    vert_list[4] = interp_vertex(x as f64, y as f64, z as f64 + 1.0, x as f64 + 1.0, y as f64, z as f64 + 1.0, v[4], v[5], isovalue);
                }
                if edges & 32 != 0 {
                    vert_list[5] = interp_vertex(x as f64 + 1.0, y as f64, z as f64 + 1.0, x as f64 + 1.0, y as f64 + 1.0, z as f64 + 1.0, v[5], v[6], isovalue);
                }
                if edges & 64 != 0 {
                    vert_list[6] = interp_vertex(x as f64 + 1.0, y as f64 + 1.0, z as f64 + 1.0, x as f64, y as f64 + 1.0, z as f64 + 1.0, v[6], v[7], isovalue);
                }
                if edges & 128 != 0 {
                    vert_list[7] = interp_vertex(x as f64, y as f64 + 1.0, z as f64 + 1.0, x as f64, y as f64, z as f64 + 1.0, v[7], v[4], isovalue);
                }
                if edges & 256 != 0 {
                    vert_list[8] = interp_vertex(x as f64, y as f64, z as f64, x as f64, y as f64, z as f64 + 1.0, v[0], v[4], isovalue);
                }
                if edges & 512 != 0 {
                    vert_list[9] = interp_vertex(x as f64 + 1.0, y as f64, z as f64, x as f64 + 1.0, y as f64, z as f64 + 1.0, v[1], v[5], isovalue);
                }
                if edges & 1024 != 0 {
                    vert_list[10] = interp_vertex(x as f64 + 1.0, y as f64 + 1.0, z as f64, x as f64 + 1.0, y as f64 + 1.0, z as f64 + 1.0, v[2], v[6], isovalue);
                }
                if edges & 2048 != 0 {
                    vert_list[11] = interp_vertex(x as f64, y as f64 + 1.0, z as f64, x as f64, y as f64 + 1.0, z as f64 + 1.0, v[3], v[7], isovalue);
                }

                // Create triangles using simplified triangle table lookup
                // (Using basic triangulation based on cube index)
                let base_idx = vertices.len() / 3;
                for vert in &vert_list {
                    if vert[0] != 0.0 || vert[1] != 0.0 || vert[2] != 0.0 {
                        vertices.push(vert[0]);
                        vertices.push(vert[1]);
                        vertices.push(vert[2]);
                    }
                }

                // Add basic triangles (simplified)
                let num_new_verts = (vertices.len() / 3) - base_idx;
                if num_new_verts >= 3 {
                    for i in 1..num_new_verts - 1 {
                        triangles.push(base_idx as u32);
                        triangles.push((base_idx + i) as u32);
                        triangles.push((base_idx + i + 1) as u32);
                    }
                }
            }
        }
    }

    (vertices, triangles)
}

/// Interpolate vertex position on edge
fn interp_vertex(
    x1: f64, y1: f64, z1: f64,
    x2: f64, y2: f64, z2: f64,
    v1: f64, v2: f64, iso: f64,
) -> [f64; 3] {
    if (v1 - v2).abs() < 1e-10 {
        return [x1, y1, z1];
    }
    let t = (iso - v1) / (v2 - v1);
    [
        x1 + t * (x2 - x1),
        y1 + t * (y2 - y1),
        z1 + t * (z2 - z1),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_vector_field() {
        let vx = vec![1.0; 8];
        let vy = vec![0.0; 8];
        let vz = vec![0.0; 8];
        let dims = [2, 2, 2];

        let vel = sample_vector_field(&vx, &vy, &vz, dims, [0.5, 0.5, 0.5]);
        assert!((vel[0] - 1.0).abs() < 1e-10);
        assert!((vel[1] - 0.0).abs() < 1e-10);
        assert!((vel[2] - 0.0).abs() < 1e-10);
    }
}
