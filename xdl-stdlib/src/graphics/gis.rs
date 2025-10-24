//! Geographic Information System (GIS) and cartographic projections
//!
//! Provides map projections, coordinate transformations, and geographic
//! visualization capabilities for scientific data.
//!
//! **Note**: This module requires the `gis` feature flag and the PROJ library
//! to be installed on your system. Enable with: `cargo build --features gis`

#[cfg(feature = "gis")]
use super::colormap::ColorMap;
#[cfg(feature = "gis")]
use super::state::{Color, GRAPHICS_STATE};
#[cfg(feature = "gis")]
use geo::{
    BoundingRect, Coord, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
#[cfg(feature = "gis")]
use geo_types::Geometry;
#[cfg(feature = "gis")]
use geojson::{Feature, FeatureCollection, GeoJson};
#[cfg(feature = "gis")]
use plotters::prelude::*;
#[cfg(feature = "gis")]
use proj::Proj;
#[cfg(feature = "gis")]
use std::f64::consts::PI;
#[cfg(feature = "gis")]
use xdl_core::{XdlError, XdlResult};

/// Map projection types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType {
    /// Cylindrical projections
    Mercator,
    MillerCylindrical,
    PlateCarree, // Equirectangular
    
    /// Conic projections
    LambertConformal,
    AlbersEqualArea,
    
    /// Azimuthal projections
    Stereographic,
    Orthographic,
    Gnomonic,
    AzimuthalEquidistant,
    
    /// Pseudocylindrical
    Mollweide,
    Sinusoidal,
    Robinson,
    
    /// Custom PROJ string
    Custom(String),
}

/// Map projection configuration
pub struct MapProjection {
    proj_type: ProjectionType,
    center_lon: f64,
    center_lat: f64,
    scale: f64,
    width: f64,
    height: f64,
    limits: Option<(f64, f64, f64, f64)>, // (lon_min, lat_min, lon_max, lat_max)
    proj: Option<Proj>,
}

impl MapProjection {
    /// Create a new map projection
    pub fn new(proj_type: ProjectionType, center: (f64, f64)) -> XdlResult<Self> {
        let (center_lon, center_lat) = center;
        
        let proj_string = match proj_type {
            ProjectionType::Mercator => {
                format!("+proj=merc +lon_0={} +lat_0=0 +x_0=0 +y_0=0 +ellps=WGS84", center_lon)
            }
            ProjectionType::MillerCylindrical => {
                format!("+proj=mill +lon_0={} +x_0=0 +y_0=0 +ellps=WGS84", center_lon)
            }
            ProjectionType::PlateCarree => {
                format!("+proj=eqc +lon_0={} +lat_0=0 +x_0=0 +y_0=0 +ellps=WGS84", center_lon)
            }
            ProjectionType::LambertConformal => {
                format!("+proj=lcc +lon_0={} +lat_0={} +lat_1={} +lat_2={} +x_0=0 +y_0=0 +ellps=WGS84",
                    center_lon, center_lat, center_lat - 10.0, center_lat + 10.0)
            }
            ProjectionType::AlbersEqualArea => {
                format!("+proj=aea +lon_0={} +lat_0={} +lat_1={} +lat_2={} +x_0=0 +y_0=0 +ellps=WGS84",
                    center_lon, center_lat, center_lat - 10.0, center_lat + 10.0)
            }
            ProjectionType::Stereographic => {
                format!("+proj=stere +lon_0={} +lat_0={} +x_0=0 +y_0=0 +ellps=WGS84",
                    center_lon, center_lat)
            }
            ProjectionType::Orthographic => {
                format!("+proj=ortho +lon_0={} +lat_0={} +x_0=0 +y_0=0 +ellps=WGS84",
                    center_lon, center_lat)
            }
            ProjectionType::Gnomonic => {
                format!("+proj=gnom +lon_0={} +lat_0={} +x_0=0 +y_0=0 +ellps=WGS84",
                    center_lon, center_lat)
            }
            ProjectionType::AzimuthalEquidistant => {
                format!("+proj=aeqd +lon_0={} +lat_0={} +x_0=0 +y_0=0 +ellps=WGS84",
                    center_lon, center_lat)
            }
            ProjectionType::Mollweide => {
                format!("+proj=moll +lon_0={} +x_0=0 +y_0=0 +ellps=WGS84", center_lon)
            }
            ProjectionType::Sinusoidal => {
                format!("+proj=sinu +lon_0={} +x_0=0 +y_0=0 +ellps=WGS84", center_lon)
            }
            ProjectionType::Robinson => {
                format!("+proj=robin +lon_0={} +x_0=0 +y_0=0 +ellps=WGS84", center_lon)
            }
            ProjectionType::Custom(ref s) => s.clone(),
        };
        
        let proj = Proj::new_known_crs(&format!("{} +to +proj=longlat +ellps=WGS84", proj_string), None, None)
            .map_err(|e| XdlError::RuntimeError(format!("Failed to create projection: {}", e)))?;
        
        Ok(Self {
            proj_type,
            center_lon,
            center_lat,
            scale: 1.0,
            width: 800.0,
            height: 600.0,
            limits: None,
            proj: Some(proj),
        })
    }
    
    /// Set the map limits (lon_min, lat_min, lon_max, lat_max)
    pub fn set_limits(&mut self, limits: (f64, f64, f64, f64)) {
        self.limits = Some(limits);
    }
    
    /// Set the output dimensions
    pub fn set_dimensions(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
    }
    
    /// Project a geographic coordinate (lon, lat) to map coordinates (x, y)
    pub fn project(&self, lon: f64, lat: f64) -> Option<(f64, f64)> {
        if let Some(ref proj) = self.proj {
            // Convert degrees to radians for proj
            let lon_rad = lon.to_radians();
            let lat_rad = lat.to_radians();
            
            match proj.convert((lon_rad, lat_rad)) {
                Ok((x, y)) => Some((x * self.scale, y * self.scale)),
                Err(_) => None,
            }
        } else {
            // Fallback to simple plate carrée
            Some((lon * self.scale, lat * self.scale))
        }
    }
    
    /// Project multiple points
    pub fn project_points(&self, coords: &[(f64, f64)]) -> Vec<(f64, f64)> {
        coords
            .iter()
            .filter_map(|&(lon, lat)| self.project(lon, lat))
            .collect()
    }
    
    /// Check if a point is within the map limits
    pub fn in_bounds(&self, lon: f64, lat: f64) -> bool {
        if let Some((lon_min, lat_min, lon_max, lat_max)) = self.limits {
            lon >= lon_min && lon <= lon_max && lat >= lat_min && lat <= lat_max
        } else {
            true
        }
    }
}

/// Simplified coastline data (for demonstration - in production use Natural Earth data)
pub struct CoastlineData {
    lines: Vec<Vec<(f64, f64)>>,
}

impl CoastlineData {
    /// Load simplified world coastlines
    pub fn load_world() -> Self {
        // Simplified coastline segments (lon, lat)
        // In production, load from Natural Earth shapefiles or GeoJSON
        let lines = vec![
            // Sample: simplified North America west coast
            vec![
                (-125.0, 50.0),
                (-124.0, 48.0),
                (-123.0, 47.0),
                (-122.0, 47.0),
                (-120.0, 45.0),
            ],
            // Add more coastline segments here
        ];
        
        Self { lines }
    }
    
    /// Load from GeoJSON
    pub fn from_geojson(json_str: &str) -> XdlResult<Self> {
        let geojson = json_str.parse::<GeoJson>()
            .map_err(|e| XdlError::RuntimeError(format!("Failed to parse GeoJSON: {}", e)))?;
        
        let mut lines = Vec::new();
        
        match geojson {
            GeoJson::FeatureCollection(fc) => {
                for feature in fc.features {
                    if let Some(geom) = feature.geometry {
                        Self::extract_lines(&geom.value, &mut lines);
                    }
                }
            }
            GeoJson::Geometry(geom) => {
                Self::extract_lines(&geom.value, &mut lines);
            }
            _ => {}
        }
        
        Ok(Self { lines })
    }
    
    fn extract_lines(geom: &geojson::Value, lines: &mut Vec<Vec<(f64, f64)>>) {
        match geom {
            geojson::Value::LineString(coords) => {
                let line: Vec<(f64, f64)> = coords
                    .iter()
                    .map(|c| (c[0], c[1]))
                    .collect();
                lines.push(line);
            }
            geojson::Value::MultiLineString(multi) => {
                for line_coords in multi {
                    let line: Vec<(f64, f64)> = line_coords
                        .iter()
                        .map(|c| (c[0], c[1]))
                        .collect();
                    lines.push(line);
                }
            }
            geojson::Value::Polygon(poly) => {
                for ring in poly {
                    let line: Vec<(f64, f64)> = ring
                        .iter()
                        .map(|c| (c[0], c[1]))
                        .collect();
                    lines.push(line);
                }
            }
            geojson::Value::MultiPolygon(multi) => {
                for poly in multi {
                    for ring in poly {
                        let line: Vec<(f64, f64)> = ring
                            .iter()
                            .map(|c| (c[0], c[1]))
                            .collect();
                        lines.push(line);
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Get all coastline segments
    pub fn segments(&self) -> &[Vec<(f64, f64)>] {
        &self.lines
    }
}

/// Draw a map with coastlines
pub fn draw_map(
    projection: &MapProjection,
    coastlines: &CoastlineData,
    filename: &str,
) -> XdlResult<()> {
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };
    
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;
    
    // Determine map bounds from projection
    let (x_min, x_max, y_min, y_max) = if let Some((lon_min, lat_min, lon_max, lat_max)) = projection.limits {
        let p1 = projection.project(lon_min, lat_min).unwrap_or((-180.0, -90.0));
        let p2 = projection.project(lon_max, lat_max).unwrap_or((180.0, 90.0));
        (p1.0.min(p2.0), p1.0.max(p2.0), p1.1.min(p2.1), p1.1.max(p2.1))
    } else {
        (-180.0, 180.0, -90.0, 90.0)
    };
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Geographic Map", ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    
    chart.configure_mesh()
        .x_desc("Longitude")
        .y_desc("Latitude")
        .draw()?;
    
    // Draw coastlines
    for segment in coastlines.segments() {
        let projected: Vec<(f64, f64)> = segment
            .iter()
            .filter(|(lon, lat)| projection.in_bounds(*lon, *lat))
            .filter_map(|&(lon, lat)| projection.project(lon, lat))
            .collect();
        
        if projected.len() > 1 {
            chart.draw_series(LineSeries::new(projected, &BLACK))?;
        }
    }
    
    root.present()?;
    Ok(())
}

/// Draw a graticule (grid of parallels and meridians)
pub fn draw_graticule(
    projection: &MapProjection,
    lon_step: f64,
    lat_step: f64,
    filename: &str,
) -> XdlResult<()> {
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };
    
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let (lon_min, lat_min, lon_max, lat_max) = projection.limits.unwrap_or((-180.0, -90.0, 180.0, 90.0));
    
    let (x_min, x_max, y_min, y_max) = {
        let p1 = projection.project(lon_min, lat_min).unwrap_or((-180.0, -90.0));
        let p2 = projection.project(lon_max, lat_max).unwrap_or((180.0, 90.0));
        (p1.0.min(p2.0), p1.0.max(p2.0), p1.1.min(p2.1), p1.1.max(p2.1))
    };
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Map Graticule", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    
    // Draw meridians (lines of constant longitude)
    let mut lon = (lon_min / lon_step).ceil() * lon_step;
    while lon <= lon_max {
        let mut points = Vec::new();
        let mut lat = lat_min;
        while lat <= lat_max {
            if let Some(p) = projection.project(lon, lat) {
                points.push(p);
            }
            lat += 1.0;
        }
        if points.len() > 1 {
            chart.draw_series(LineSeries::new(points, &BLUE.mix(0.3)))?;
        }
        lon += lon_step;
    }
    
    // Draw parallels (lines of constant latitude)
    let mut lat = (lat_min / lat_step).ceil() * lat_step;
    while lat <= lat_max {
        let mut points = Vec::new();
        let mut lon = lon_min;
        while lon <= lon_max {
            if let Some(p) = projection.project(lon, lat) {
                points.push(p);
            }
            lon += 1.0;
        }
        if points.len() > 1 {
            chart.draw_series(LineSeries::new(points, &BLUE.mix(0.3)))?;
        }
        lat += lat_step;
    }
    
    root.present()?;
    Ok(())
}

/// Plot data points on a map
pub fn map_scatter(
    projection: &MapProjection,
    lons: &[f64],
    lats: &[f64],
    values: Option<&[f64]>,
    colormap: Option<&ColorMap>,
    filename: &str,
) -> XdlResult<()> {
    if lons.len() != lats.len() {
        return Err(XdlError::InvalidArgument(
            "Longitude and latitude arrays must have same length".to_string(),
        ));
    }
    
    let (width, height) = {
        let state = GRAPHICS_STATE.lock().unwrap();
        let win = state.get_current_window().unwrap();
        (win.width, win.height)
    };
    
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let (lon_min, lat_min, lon_max, lat_max) = projection.limits.unwrap_or((-180.0, -90.0, 180.0, 90.0));
    let p1 = projection.project(lon_min, lat_min).unwrap_or((-180.0, -90.0));
    let p2 = projection.project(lon_max, lat_max).unwrap_or((180.0, 90.0));
    let (x_min, x_max, y_min, y_max) = (p1.0.min(p2.0), p1.0.max(p2.0), p1.1.min(p2.1), p1.1.max(p2.1));
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Map Scatter Plot", ("sans-serif", 30))
        .margin(20)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;
    
    chart.configure_mesh().draw()?;
    
    // Normalize values if provided
    let (v_min, v_max) = if let Some(vals) = values {
        let min = vals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = vals.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        (min, max)
    } else {
        (0.0, 1.0)
    };
    
    // Plot points
    for i in 0..lons.len() {
        if let Some((x, y)) = projection.project(lons[i], lats[i]) {
            let color = if let (Some(vals), Some(cmap)) = (values, colormap) {
                let t = if v_max > v_min {
                    (vals[i] - v_min) / (v_max - v_min)
                } else {
                    0.5
                };
                let c = cmap.map(t);
                RGBColor(c.r, c.g, c.b)
            } else {
                RED
            };
            
            chart.draw_series(std::iter::once(Circle::new(
                (x, y),
                5,
                ShapeStyle::from(&color).filled(),
            )))?;
        }
    }
    
    root.present()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mercator_projection() {
        let proj = MapProjection::new(ProjectionType::Mercator, (0.0, 0.0)).unwrap();
        
        // Test equator projection
        let (x, y) = proj.project(0.0, 0.0).unwrap();
        assert!(x.abs() < 1e-6);
        assert!(y.abs() < 1e-6);
    }
    
    #[test]
    fn test_projection_bounds() {
        let mut proj = MapProjection::new(ProjectionType::PlateCarree, (0.0, 0.0)).unwrap();
        proj.set_limits((-180.0, -90.0, 180.0, 90.0));
        
        assert!(proj.in_bounds(0.0, 0.0));
        assert!(!proj.in_bounds(200.0, 0.0));
    }
}
