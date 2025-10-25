//! Rayleigh-Taylor Instability Visualization Driver
//!
//! Creates multiple visualizations of the RT instability using all
//! available color maps and visualization techniques.

mod rayleigh_taylor_demo;

use rayleigh_taylor_demo::RTSimulation;
use plotters::prelude::*;
use std::error::Error;

/// Render density field with a given colormap
fn render_density_field(
    sim: &RTSimulation,
    colormap_fn: impl Fn(f64) -> (u8, u8, u8),
    filename: &str,
    title: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..sim.width as f64, 0.0..sim.height as f64)?;

    chart.configure_mesh()
        .x_desc("X")
        .y_desc("Y")
        .draw()?;

    // Get density range
    let (min_d, max_d, _) = sim.density_stats();

    // Draw density as colored pixels
    for y in 0..sim.height {
        for x in 0..sim.width {
            let density = sim.density[y][x];
            let normalized = if max_d > min_d {
                (density - min_d) / (max_d - min_d)
            } else {
                0.5
            };

            let (r, g, b) = colormap_fn(normalized);
            let color = RGBColor(r, g, b);

            chart.draw_series(std::iter::once(Rectangle::new(
                [(x as f64, y as f64), (x as f64 + 1.0, y as f64 + 1.0)],
                ShapeStyle::from(&color).filled(),
            )))?;
        }
    }

    root.present()?;
    Ok(())
}

/// Render velocity field as arrows
fn render_velocity_field(
    sim: &RTSimulation,
    filename: &str,
    subsample: usize,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Velocity Field (Quiver Plot)", ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..sim.width as f64, 0.0..sim.height as f64)?;

    chart.configure_mesh()
        .x_desc("X")
        .y_desc("Y")
        .draw()?;

    // Find max velocity for scaling
    let mut max_vel = 0.0;
    for y in 0..sim.height {
        for x in 0..sim.width {
            let vel = (sim.velocity_u[y][x].powi(2) + sim.velocity_v[y][x].powi(2)).sqrt();
            max_vel = max_vel.max(vel);
        }
    }

    let scale = 5.0;

    // Draw arrows
    for y in (0..sim.height).step_by(subsample) {
        for x in (0..sim.width).step_by(subsample) {
            let u = sim.velocity_u[y][x];
            let v = sim.velocity_v[y][x];
            let mag = (u * u + v * v).sqrt();

            if mag > 1e-6 {
                let x0 = x as f64 + 0.5;
                let y0 = y as f64 + 0.5;
                let x1 = x0 + u * scale;
                let y1 = y0 + v * scale;

                // Color by magnitude
                let t = if max_vel > 0.0 { mag / max_vel } else { 0.5 };
                let color_val = (t * 200.0) as u8 + 55;
                let color = RGBColor(color_val, 100, 255 - color_val);

                // Draw arrow line
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(x0, y0), (x1, y1)],
                    &color,
                )))?;

                // Simple arrowhead
                let arrow_len = 0.5;
                let angle = v.atan2(u);
                let ah1_x = x1 - arrow_len * (angle + 0.5).cos();
                let ah1_y = y1 - arrow_len * (angle + 0.5).sin();
                let ah2_x = x1 - arrow_len * (angle - 0.5).cos();
                let ah2_y = y1 - arrow_len * (angle - 0.5).sin();

                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(x1, y1), (ah1_x, ah1_y)],
                    &color,
                )))?;
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(x1, y1), (ah2_x, ah2_y)],
                    &color,
                )))?;
            }
        }
    }

    root.present()?;
    Ok(())
}

/// Viridis colormap
fn viridis(t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    // Approximation of viridis colormap
    let r = ((-4.37e-1 + t * (1.43e0 + t * (-1.32e0))) * 255.0) as u8;
    let g = ((9.32e-2 + t * (2.56e0 + t * (-1.97e0))) * 255.0) as u8;
    let b = ((3.28e-1 + t * (1.54e0 + t * (-9.98e-1))) * 255.0) as u8;
    (r, g, b)
}

/// Plasma colormap
fn plasma(t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let r = ((5.06e-1 + t * (1.06e0 + t * (-5.08e-1))) * 255.0) as u8;
    let g = ((2.15e-2 + t * (1.98e0 + t * (-2.49e0 + t * 1.41e0))) * 255.0) as u8;
    let b = ((6.33e-1 + t * (-7.88e-1 + t * 1.34e0)) * 255.0) as u8;
    (r, g, b)
}

/// Turbo colormap
fn turbo(t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let r = ((3.40e-1 + t * (2.83e0 + t * (-1.96e0))) * 255.0) as u8;
    let g = ((2.39e-2 + t * (4.18e0 + t * (-5.28e0 + t * 2.11e0))) * 255.0) as u8;
    let b = ((6.67e-1 + t * (-1.30e0 + t * 6.33e-1)) * 255.0) as u8;
    (r, g, b)
}

/// Inferno colormap
fn inferno(t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let r = ((7.70e-3 + t * (2.81e0 + t * (-1.49e0))) * 255.0) as u8;
    let g = ((1.75e-2 + t * (1.91e0 + t * (-1.63e0))) * 255.0) as u8;
    let b = ((1.48e-1 + t * (1.45e0 + t * (-1.02e0))) * 255.0) as u8;
    (r, g, b)
}

/// Generate comparison image with multiple colormaps
fn render_comparison(
    sim: &RTSimulation,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(filename, (1600, 1200)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((2, 2));

    let colormaps = [
        (viridis as fn(f64) -> (u8, u8, u8), "Viridis"),
        (plasma, "Plasma"),
        (turbo, "Turbo"),
        (inferno, "Inferno"),
    ];

    let (min_d, max_d, _) = sim.density_stats();

    for (idx, (colormap, name)) in colormaps.iter().enumerate() {
        let mut chart = ChartBuilder::on(&areas[idx])
            .caption(*name, ("sans-serif", 30))
            .margin(5)
            .build_cartesian_2d(0.0..sim.width as f64, 0.0..sim.height as f64)?;

        for y in 0..sim.height {
            for x in 0..sim.width {
                let density = sim.density[y][x];
                let normalized = if max_d > min_d {
                    (density - min_d) / (max_d - min_d)
                } else {
                    0.5
                };

                let (r, g, b) = colormap(normalized);
                let color = RGBColor(r, g, b);

                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x as f64, y as f64), (x as f64 + 1.0, y as f64 + 1.0)],
                    ShapeStyle::from(&color).filled(),
                )))?;
            }
        }
    }

    root.present()?;
    Ok(())
}

/// Main visualization driver
pub fn main() -> Result<(), Box<dyn Error>> {
    println!("Rayleigh-Taylor Instability Visualization Demo");
    println!("================================================\n");

    // Create simulation
    println!("Initializing simulation (200x200 grid)...");
    let mut sim = RTSimulation::new(200, 200);

    println!("Initial density stats: {:?}\n", sim.density_stats());

    // Render initial state
    println!("Rendering initial state with multiple colormaps...");
    render_comparison(&sim, "rt_initial_comparison.png")?;
    println!("  ✓ Saved: rt_initial_comparison.png");

    // Simulate and visualize multiple frames
    let timesteps = [0, 50, 100, 200, 400, 800];

    for (idx, &steps) in timesteps.iter().enumerate() {
        if steps > 0 {
            let step_size = if idx > 0 {
                steps - timesteps[idx - 1]
            } else {
                steps
            };

            println!("\nSimulating {} steps...", step_size);
            sim.simulate(step_size);
            println!("Density stats: {:?}", sim.density_stats());
        }

        // Render with different colormaps
        let frame_num = format!("{:04}", steps);

        println!("Rendering frame {} visualizations:", steps);

        render_density_field(
            &sim,
            viridis,
            &format!("rt_viridis_{}.png", frame_num),
            &format!("Rayleigh-Taylor (Viridis) - Step {}", steps),
        )?;
        println!("  ✓ Saved: rt_viridis_{}.png", frame_num);

        render_density_field(
            &sim,
            plasma,
            &format!("rt_plasma_{}.png", frame_num),
            &format!("Rayleigh-Taylor (Plasma) - Step {}", steps),
        )?;
        println!("  ✓ Saved: rt_plasma_{}.png", frame_num);

        render_density_field(
            &sim,
            turbo,
            &format!("rt_turbo_{}.png", frame_num),
            &format!("Rayleigh-Taylor (Turbo) - Step {}", steps),
        )?;
        println!("  ✓ Saved: rt_turbo_{}.png", frame_num);

        // Render velocity field
        render_velocity_field(&sim, &format!("rt_velocity_{}.png", frame_num), 8)?;
        println!("  ✓ Saved: rt_velocity_{}.png", frame_num);

        // Render comparison
        render_comparison(&sim, &format!("rt_comparison_{}.png", frame_num))?;
        println!("  ✓ Saved: rt_comparison_{}.png", frame_num);
    }

    println!("\n✅ Visualization complete!");
    println!("\nGenerated {} images showcasing:", timesteps.len() * 5);
    println!("  • Density field evolution with Viridis, Plasma, Turbo, and Inferno colormaps");
    println!("  • Velocity field quiver plots");
    println!("  • Side-by-side colormap comparisons");
    println!("\nThe Rayleigh-Taylor instability demonstrates:");
    println!("  • Heavy fluid (top) falling through light fluid (bottom)");
    println!("  • Mushroom-shaped plumes forming");
    println!("  • Complex fluid mixing dynamics");
    println!("  • Perceptually uniform colormap advantages");

    Ok(())
}
