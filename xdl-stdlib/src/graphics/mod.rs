//! Graphics subsystem module organization

// Full implementation modules
pub mod plot2d;
pub mod plot3d;
pub mod state;

pub use plot2d::{Plot2DConfig, bar_plot, histogram_plot, plot_2d, plot_with_errors};
pub use plot3d::{ContourConfig, SurfaceConfig, contour_plot, plot_3d, surface_plot};
pub use state::{Color, ColorTable, GraphicsState, LineStyle, PlotStyle, GRAPHICS_STATE};
