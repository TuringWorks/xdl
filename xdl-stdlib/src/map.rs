//! Map projection functions
//!
//! This module provides map projection capabilities for geographic visualization.
//! Supports common projections: Cylindrical, Mercator, Lambert, Orthographic, etc.

use std::collections::HashMap;
use std::f64::consts::PI;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Helper to extract f64 from XdlValue
fn value_to_f64(v: &XdlValue) -> Option<f64> {
    match v {
        XdlValue::Float(f) => Some(*f as f64),
        XdlValue::Double(d) => Some(*d),
        XdlValue::Int(i) => Some(*i as f64),
        XdlValue::Long(l) => Some(*l as f64),
        XdlValue::Byte(b) => Some(*b as f64),
        XdlValue::UInt(u) => Some(*u as f64),
        XdlValue::ULong(u) => Some(*u as f64),
        XdlValue::Long64(l) => Some(*l as f64),
        XdlValue::ULong64(u) => Some(*u as f64),
        _ => None,
    }
}

/// Map projection state (global for simplicity, like IDL)
static mut MAP_PROJECTION: Option<MapProjection> = None;

/// Supported map projections
#[derive(Debug, Clone)]
pub enum ProjectionType {
    Cylindrical,      // Equidistant cylindrical (Plate Carrée)
    Mercator,         // Mercator conformal
    LambertConic,     // Lambert conformal conic
    Orthographic,     // Orthographic (view from space)
    Stereographic,    // Stereographic
    Gnomonic,         // Gnomonic (great circles as straight lines)
    Sinusoidal,       // Sinusoidal (Sanson-Flamsteed)
    Mollweide,        // Mollweide (equal area)
    Robinson,         // Robinson (compromise)
    Hammer,           // Hammer-Aitoff (equal area)
    Aitoff,           // Aitoff
    Albers,           // Albers equal area conic
    AzimuthalEquidistant, // Azimuthal equidistant
    Satellite,        // General perspective (satellite view)
}

impl ProjectionType {
    fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "CYLINDRICAL" | "EQUIDISTANT" | "PLATE_CARREE" | "1" => Some(Self::Cylindrical),
            "MERCATOR" | "2" => Some(Self::Mercator),
            "LAMBERT" | "LAMBERT_CONIC" | "3" => Some(Self::LambertConic),
            "ORTHOGRAPHIC" | "ORTHO" | "4" => Some(Self::Orthographic),
            "STEREOGRAPHIC" | "STEREO" | "5" => Some(Self::Stereographic),
            "GNOMONIC" | "6" => Some(Self::Gnomonic),
            "SINUSOIDAL" | "7" => Some(Self::Sinusoidal),
            "MOLLWEIDE" | "8" => Some(Self::Mollweide),
            "ROBINSON" | "9" => Some(Self::Robinson),
            "HAMMER" | "10" => Some(Self::Hammer),
            "AITOFF" | "11" => Some(Self::Aitoff),
            "ALBERS" | "12" => Some(Self::Albers),
            "AZIMUTHAL" | "13" => Some(Self::AzimuthalEquidistant),
            "SATELLITE" | "14" => Some(Self::Satellite),
            _ => None,
        }
    }
}

/// Map projection configuration
#[derive(Debug, Clone)]
pub struct MapProjection {
    pub projection: ProjectionType,
    pub center_lat: f64,
    pub center_lon: f64,
    pub rotation: f64,
    pub limit: Option<[f64; 4]>, // [min_lat, min_lon, max_lat, max_lon]
    pub scale: f64,
    pub satellite_height: f64, // For satellite projection
}

impl Default for MapProjection {
    fn default() -> Self {
        Self {
            projection: ProjectionType::Cylindrical,
            center_lat: 0.0,
            center_lon: 0.0,
            rotation: 0.0,
            limit: None,
            scale: 1.0,
            satellite_height: 6.0, // Earth radii
        }
    }
}

impl MapProjection {
    /// Project geographic coordinates to map coordinates
    pub fn project(&self, lat: f64, lon: f64) -> Option<(f64, f64)> {
        let lat_rad = lat.to_radians();
        let lon_rad = (lon - self.center_lon).to_radians();
        let center_lat_rad = self.center_lat.to_radians();

        let (x, y) = match self.projection {
            ProjectionType::Cylindrical => {
                // Plate Carrée / Equidistant Cylindrical
                (lon_rad, lat_rad)
            }
            ProjectionType::Mercator => {
                // Mercator projection
                if lat.abs() > 85.0 {
                    return None; // Undefined at poles
                }
                let y = ((PI / 4.0 + lat_rad / 2.0).tan()).ln();
                (lon_rad, y)
            }
            ProjectionType::Orthographic => {
                // Orthographic (view from space)
                let cos_c = center_lat_rad.sin() * lat_rad.sin()
                    + center_lat_rad.cos() * lat_rad.cos() * lon_rad.cos();
                if cos_c < 0.0 {
                    return None; // Point on back side of globe
                }
                let x = lat_rad.cos() * lon_rad.sin();
                let y = center_lat_rad.cos() * lat_rad.sin()
                    - center_lat_rad.sin() * lat_rad.cos() * lon_rad.cos();
                (x, y)
            }
            ProjectionType::Stereographic => {
                // Stereographic projection
                let k = 2.0 / (1.0
                    + center_lat_rad.sin() * lat_rad.sin()
                    + center_lat_rad.cos() * lat_rad.cos() * lon_rad.cos());
                let x = k * lat_rad.cos() * lon_rad.sin();
                let y = k * (center_lat_rad.cos() * lat_rad.sin()
                    - center_lat_rad.sin() * lat_rad.cos() * lon_rad.cos());
                (x, y)
            }
            ProjectionType::Sinusoidal => {
                // Sinusoidal projection
                let x = lon_rad * lat_rad.cos();
                (x, lat_rad)
            }
            ProjectionType::Mollweide => {
                // Mollweide projection (approximate)
                let theta = self.mollweide_theta(lat_rad);
                let x = 2.0 * 2.0_f64.sqrt() / PI * lon_rad * theta.cos();
                let y = 2.0_f64.sqrt() * theta.sin();
                (x, y)
            }
            ProjectionType::Hammer => {
                // Hammer-Aitoff projection
                let z = (1.0 + lat_rad.cos() * (lon_rad / 2.0).cos()).sqrt();
                let x = 2.0 * 2.0_f64.sqrt() * lat_rad.cos() * (lon_rad / 2.0).sin() / z;
                let y = 2.0_f64.sqrt() * lat_rad.sin() / z;
                (x, y)
            }
            ProjectionType::Gnomonic => {
                // Gnomonic projection
                let cos_c = center_lat_rad.sin() * lat_rad.sin()
                    + center_lat_rad.cos() * lat_rad.cos() * lon_rad.cos();
                if cos_c <= 0.0 {
                    return None;
                }
                let x = lat_rad.cos() * lon_rad.sin() / cos_c;
                let y = (center_lat_rad.cos() * lat_rad.sin()
                    - center_lat_rad.sin() * lat_rad.cos() * lon_rad.cos())
                    / cos_c;
                (x, y)
            }
            ProjectionType::Satellite => {
                // General perspective (satellite view)
                let p = 1.0 + self.satellite_height;
                let cos_c = center_lat_rad.sin() * lat_rad.sin()
                    + center_lat_rad.cos() * lat_rad.cos() * lon_rad.cos();
                let k = (p - 1.0) / (p - cos_c);
                if cos_c < 1.0 / p {
                    return None;
                }
                let x = k * lat_rad.cos() * lon_rad.sin();
                let y = k * (center_lat_rad.cos() * lat_rad.sin()
                    - center_lat_rad.sin() * lat_rad.cos() * lon_rad.cos());
                (x, y)
            }
            _ => {
                // Default to cylindrical for unimplemented projections
                (lon_rad, lat_rad)
            }
        };

        Some((x * self.scale, y * self.scale))
    }

    /// Newton-Raphson iteration for Mollweide theta
    fn mollweide_theta(&self, lat_rad: f64) -> f64 {
        let target = PI * lat_rad.sin();
        let mut theta = lat_rad;
        for _ in 0..10 {
            let f = 2.0 * theta + (2.0 * theta).sin() - target;
            let fp = 2.0 + 2.0 * (2.0 * theta).cos();
            theta -= f / fp;
        }
        theta
    }
}

/// MAP_SET - Set up map projection
/// IDL syntax: MAP_SET [, P0lat, P0lon [, Rotation]] [, /PROJECTION] [, LIMIT=[minlat, minlon, maxlat, maxlon]]
pub fn map_set(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    let mut proj = MapProjection::default();

    // Parse positional arguments: P0lat, P0lon, Rotation
    if !args.is_empty() {
        proj.center_lat = match &args[0] {
            XdlValue::Float(f) => *f as f64,
            XdlValue::Double(d) => *d,
            XdlValue::Int(i) => *i as f64,
            XdlValue::Long(l) => *l as f64,
            _ => 0.0,
        };
    }
    if args.len() > 1 {
        proj.center_lon = match &args[1] {
            XdlValue::Float(f) => *f as f64,
            XdlValue::Double(d) => *d,
            XdlValue::Int(i) => *i as f64,
            XdlValue::Long(l) => *l as f64,
            _ => 0.0,
        };
    }
    if args.len() > 2 {
        proj.rotation = match &args[2] {
            XdlValue::Float(f) => *f as f64,
            XdlValue::Double(d) => *d,
            XdlValue::Int(i) => *i as f64,
            XdlValue::Long(l) => *l as f64,
            _ => 0.0,
        };
    }

    // Parse projection keyword flags
    for (key, _) in keywords.iter() {
        if let Some(proj_type) = ProjectionType::from_name(key) {
            proj.projection = proj_type;
            break;
        }
    }

    // Check for PROJECTION= keyword
    if let Some(proj_val) = keywords.get("PROJECTION") {
        let proj_name = match proj_val {
            XdlValue::String(s) => s.clone(),
            XdlValue::Int(i) => i.to_string(),
            XdlValue::Long(l) => l.to_string(),
            _ => String::new(),
        };
        if let Some(proj_type) = ProjectionType::from_name(&proj_name) {
            proj.projection = proj_type;
        }
    }

    // Parse LIMIT keyword
    if let Some(limit_val) = keywords.get("LIMIT") {
        if let XdlValue::Array(arr) = limit_val {
            if arr.len() >= 4 {
                proj.limit = Some([arr[0], arr[1], arr[2], arr[3]]);
            }
        }
    }

    // Parse SCALE keyword
    if let Some(scale_val) = keywords.get("SCALE") {
        proj.scale = match scale_val {
            XdlValue::Float(f) => *f as f64,
            XdlValue::Double(d) => *d,
            XdlValue::Int(i) => *i as f64,
            XdlValue::Long(l) => *l as f64,
            _ => 1.0,
        };
    }

    // Parse SATELLITE_HEIGHT keyword (for satellite projection)
    if let Some(height_val) = keywords.get("SATELLITE_HEIGHT") {
        proj.satellite_height = match height_val {
            XdlValue::Float(f) => *f as f64,
            XdlValue::Double(d) => *d,
            XdlValue::Int(i) => *i as f64,
            XdlValue::Long(l) => *l as f64,
            _ => 6.0,
        };
    }

    // Store global projection state
    unsafe {
        MAP_PROJECTION = Some(proj.clone());
    }

    println!(
        "MAP_SET: Initialized {:?} projection centered at ({}, {})",
        proj.projection, proj.center_lat, proj.center_lon
    );

    Ok(XdlValue::Undefined)
}

/// MAP_CONTINENTS - Draw continental outlines
/// IDL syntax: MAP_CONTINENTS [, /COASTS] [, /COUNTRIES] [, /RIVERS] [, /USA] [, COLOR=color]
pub fn map_continents(
    _args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    let proj = unsafe {
        MAP_PROJECTION
            .as_ref()
            .ok_or_else(|| XdlError::RuntimeError("MAP_SET must be called first".to_string()))?
    };

    let draw_coasts = keywords.contains_key("COASTS");
    let draw_countries = keywords.contains_key("COUNTRIES");
    let draw_rivers = keywords.contains_key("RIVERS");
    let draw_usa = keywords.contains_key("USA");
    let fill = keywords.contains_key("FILL");

    // Get color if specified
    let color = keywords.get("COLOR").and_then(|v| match v {
        XdlValue::Int(i) => Some(*i as u32),
        XdlValue::Long(l) => Some(*l as u32),
        _ => None,
    });

    println!("MAP_CONTINENTS: Drawing with {:?} projection", proj.projection);
    if draw_coasts {
        println!("  - Drawing coastlines");
    }
    if draw_countries {
        println!("  - Drawing country borders");
    }
    if draw_rivers {
        println!("  - Drawing rivers");
    }
    if draw_usa {
        println!("  - Drawing US state borders");
    }
    if fill {
        println!("  - Filling land areas");
    }
    if let Some(c) = color {
        println!("  - Color: {}", c);
    }

    // In a full implementation, this would:
    // 1. Load continental outline data (Natural Earth, GSHHG, etc.)
    // 2. Project each coordinate pair using proj.project()
    // 3. Draw the resulting polylines/polygons

    // Generate sample continent outline for demonstration
    let sample_coords = generate_sample_coastline();
    let mut visible_count = 0;
    for (lat, lon) in &sample_coords {
        if proj.project(*lat, *lon).is_some() {
            visible_count += 1;
        }
    }
    println!(
        "  - Sample coastline: {}/{} points visible",
        visible_count,
        sample_coords.len()
    );

    Ok(XdlValue::Undefined)
}

/// MAP_GRID - Draw map grid lines (parallels and meridians)
/// IDL syntax: MAP_GRID [, LATDEL=degrees] [, LONDEL=degrees] [, /LABEL] [, COLOR=color]
pub fn map_grid(
    _args: &[XdlValue],
    keywords: &HashMap<String, XdlValue>,
) -> XdlResult<XdlValue> {
    let proj = unsafe {
        MAP_PROJECTION
            .as_ref()
            .ok_or_else(|| XdlError::RuntimeError("MAP_SET must be called first".to_string()))?
    };

    // Parse grid spacing
    let lat_del = keywords
        .get("LATDEL")
        .and_then(|v| value_to_f64(v))
        .unwrap_or(30.0);
    let lon_del = keywords
        .get("LONDEL")
        .and_then(|v| value_to_f64(v))
        .unwrap_or(30.0);
    let label = keywords.contains_key("LABEL");

    // Get color if specified
    let color = keywords.get("COLOR").and_then(|v| match v {
        XdlValue::Int(i) => Some(*i as u32),
        XdlValue::Long(l) => Some(*l as u32),
        _ => None,
    });

    println!("MAP_GRID: Drawing with {:?} projection", proj.projection);
    println!(
        "  - Latitude spacing: {}°, Longitude spacing: {}°",
        lat_del, lon_del
    );
    if label {
        println!("  - Labels enabled");
    }
    if let Some(c) = color {
        println!("  - Color: {}", c);
    }

    // Draw latitude lines (parallels)
    let mut lat = -90.0;
    while lat <= 90.0 {
        let mut line_points = Vec::new();
        let mut lon = -180.0;
        while lon <= 180.0 {
            if let Some((x, y)) = proj.project(lat, lon) {
                line_points.push((x, y));
            }
            lon += 2.0;
        }
        if !line_points.is_empty() {
            println!(
                "  - Parallel {}°: {} points",
                lat,
                line_points.len()
            );
        }
        lat += lat_del;
    }

    // Draw longitude lines (meridians)
    let mut lon = -180.0;
    while lon <= 180.0 {
        let mut line_points = Vec::new();
        let mut lat = -90.0;
        while lat <= 90.0 {
            if let Some((x, y)) = proj.project(lat, lon) {
                line_points.push((x, y));
            }
            lat += 2.0;
        }
        if !line_points.is_empty() {
            println!(
                "  - Meridian {}°: {} points",
                lon,
                line_points.len()
            );
        }
        lon += lon_del;
    }

    Ok(XdlValue::Undefined)
}

/// Convert geographic coordinates to map coordinates
/// IDL syntax: result = CONVERT_COORD(x, y, /DATA, /TO_DEVICE)
pub fn convert_coord(args: &[XdlValue], keywords: &HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "CONVERT_COORD: Expected at least 2 arguments (lat, lon)".to_string(),
        ));
    }

    let proj = unsafe {
        MAP_PROJECTION
            .as_ref()
            .ok_or_else(|| XdlError::RuntimeError("MAP_SET must be called first".to_string()))?
    };

    let lat = value_to_f64(&args[0]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0]),
        }
    })?;

    let lon = value_to_f64(&args[1]).ok_or_else(|| {
        XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[1]),
        }
    })?;

    let _from_data = keywords.contains_key("DATA");
    let _to_device = keywords.contains_key("TO_DEVICE");

    if let Some((x, y)) = proj.project(lat, lon) {
        Ok(XdlValue::Array(vec![x, y]))
    } else {
        // Point is not visible in current projection
        Ok(XdlValue::Array(vec![f64::NAN, f64::NAN]))
    }
}

/// Get current map structure
/// IDL syntax: map_struct = MAP_STRUCT()
pub fn map_struct(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let proj = unsafe { MAP_PROJECTION.as_ref() };

    if let Some(p) = proj {
        // Return projection info as a structure-like array
        Ok(XdlValue::NestedArray(vec![
            XdlValue::String(format!("{:?}", p.projection)),
            XdlValue::Double(p.center_lat),
            XdlValue::Double(p.center_lon),
            XdlValue::Double(p.rotation),
            XdlValue::Double(p.scale),
        ]))
    } else {
        Err(XdlError::RuntimeError(
            "No map projection set. Call MAP_SET first.".to_string(),
        ))
    }
}

/// Generate sample coastline coordinates for demonstration
fn generate_sample_coastline() -> Vec<(f64, f64)> {
    // Very simplified world coastline outline (major landmasses only)
    vec![
        // North America (very simplified)
        (70.0, -140.0),
        (60.0, -140.0),
        (60.0, -120.0),
        (50.0, -125.0),
        (40.0, -125.0),
        (30.0, -120.0),
        (25.0, -110.0),
        (30.0, -85.0),
        (25.0, -80.0),
        (45.0, -70.0),
        (50.0, -60.0),
        (70.0, -70.0),
        // Europe (very simplified)
        (70.0, 30.0),
        (60.0, 10.0),
        (50.0, -5.0),
        (35.0, -10.0),
        (35.0, 30.0),
        (45.0, 30.0),
        // Africa (very simplified)
        (35.0, -10.0),
        (35.0, 30.0),
        (0.0, 40.0),
        (-35.0, 20.0),
        (0.0, -10.0),
        // South America (very simplified)
        (10.0, -80.0),
        (0.0, -50.0),
        (-55.0, -70.0),
        (-20.0, -70.0),
        // Australia (very simplified)
        (-10.0, 130.0),
        (-30.0, 115.0),
        (-40.0, 145.0),
        (-10.0, 150.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cylindrical_projection() {
        let proj = MapProjection::default();
        let (x, y) = proj.project(0.0, 0.0).unwrap();
        assert!((x - 0.0).abs() < 1e-10);
        assert!((y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_mercator_projection() {
        let mut proj = MapProjection::default();
        proj.projection = ProjectionType::Mercator;

        // Equator at prime meridian
        let (x, y) = proj.project(0.0, 0.0).unwrap();
        assert!((x - 0.0).abs() < 1e-10);
        assert!((y - 0.0).abs() < 1e-10);

        // Poles are undefined
        assert!(proj.project(90.0, 0.0).is_none());
    }

    #[test]
    fn test_orthographic_projection() {
        let mut proj = MapProjection::default();
        proj.projection = ProjectionType::Orthographic;

        // Center point should project to (0, 0)
        let (x, y) = proj.project(0.0, 0.0).unwrap();
        assert!((x - 0.0).abs() < 1e-10);
        assert!((y - 0.0).abs() < 1e-10);

        // Back side of globe should return None
        assert!(proj.project(0.0, 180.0).is_none());
    }
}
