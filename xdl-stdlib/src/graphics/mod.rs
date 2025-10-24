//! Graphics subsystem module organization

// Core plotting modules
pub mod plot2d;
pub mod plot3d;
pub mod state;

// Scientific visualization modules
pub mod colormap;
pub mod export;
#[cfg(feature = "gis")]
pub mod gis;
pub mod sciviz;
pub mod terrain;

// Re-exports for convenience
pub use plot2d::{bar_plot, histogram_plot, plot_2d, plot_with_errors, Plot2DConfig};
pub use plot3d::{contour_plot, plot_3d, surface_plot, ContourConfig, SurfaceConfig};
pub use state::{Color, ColorTable, GraphicsState, LineStyle, PlotStyle, GRAPHICS_STATE};

// Scientific visualization exports
pub use colormap::{
    inferno, ocean, plasma, terrain as terrain_colormap, turbo, viridis, ColorMap, ColorMapType,
};

#[cfg(feature = "gis")]
pub use gis::{
    draw_graticule, draw_map, map_scatter, CoastlineData, MapProjection, ProjectionType,
};

pub use export::{export_to_html, ExportConfig, ExportFormat};
pub use sciviz::{
    integrate_streamline, render_isosurface_slice, render_quiver, render_streamlines,
    render_volume_mip, ScalarField3D, VectorField2D, VectorField3D,
};
pub use terrain::{
    generate_contours, render_elevation_map, render_hillshade, render_shaded_relief,
    render_terrain_3d, DigitalElevationModel,
};
