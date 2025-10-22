//! Graphics subsystem module organization

// Full implementation modules
pub mod plot2d;
pub mod plot3d;
pub mod state;

pub use plot2d::{bar_plot, histogram_plot, plot_2d, plot_with_errors, Plot2DConfig};
pub use plot3d::{contour_plot, plot_3d, surface_plot, ContourConfig, SurfaceConfig};
pub use state::{Color, ColorTable, GraphicsState, LineStyle, PlotStyle, GRAPHICS_STATE};
