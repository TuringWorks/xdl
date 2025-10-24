//! Graphics subsystem module organization

// Core plotting modules
pub mod plot2d;
pub mod plot3d;
pub mod state;

// Scientific visualization modules
pub mod colormap;
#[cfg(feature = "gis")]
pub mod gis;
pub mod terrain;
pub mod sciviz;
pub mod export;

// Re-exports for convenience
pub use plot2d::{bar_plot, histogram_plot, plot_2d, plot_with_errors, Plot2DConfig};
pub use plot3d::{contour_plot, plot_3d, surface_plot, ContourConfig, SurfaceConfig};
pub use state::{Color, ColorTable, GraphicsState, LineStyle, PlotStyle, GRAPHICS_STATE};

// Scientific visualization exports
pub use colormap::{ColorMap, ColorMapType, viridis, plasma, inferno, turbo, terrain as terrain_colormap, ocean};

#[cfg(feature = "gis")]
pub use gis::{MapProjection, ProjectionType, CoastlineData, draw_map, draw_graticule, map_scatter};

pub use terrain::{
    DigitalElevationModel, render_elevation_map, render_hillshade, render_shaded_relief,
    generate_contours, render_terrain_3d,
};
pub use sciviz::{
    VectorField2D, VectorField3D, ScalarField3D, render_quiver, render_streamlines,
    render_volume_mip, render_isosurface_slice, integrate_streamline,
};
pub use export::{ExportConfig, ExportFormat, export_to_html};
