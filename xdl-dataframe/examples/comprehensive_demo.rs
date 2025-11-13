//! Comprehensive DataFrame Demo with ML and Visualization
//!
//! Demonstrates:
//! - Time series analysis with DataFrame
//! - Machine learning classification
//! - 3D data processing
//! - Statistical analysis
//! - Data export for visualization

use std::f64::consts::PI;
use xdl_dataframe::{read_csv_string, write_csv, CsvReaderOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== XDL DataFrame Comprehensive Demo ===\n");

    // Demo 1: Time Series Analysis
    time_series_demo()?;

    // Demo 2: Classification Data
    classification_demo()?;

    // Demo 3: 3D Spatial Data
    spatial_3d_demo()?;

    println!("\n=== All Demos Complete ===");
    Ok(())
}

fn time_series_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. TIME SERIES ANALYSIS DEMO");
    println!("============================\n");

    // Generate time series data
    let n_points = 365;
    let mut csv_data = String::from("day,temperature,humidity,pressure\n");

    for day in 0..n_points {
        let t = day as f64;
        // Temperature with seasonal pattern
        let temp = 20.0 + 10.0 * (2.0 * PI * t / 365.0).sin() + rand::random::<f64>() * 5.0;
        // Humidity (inverse correlation with temp)
        let humidity = 70.0 - 15.0 * (2.0 * PI * t / 365.0).sin() + rand::random::<f64>() * 10.0;
        // Pressure
        let pressure = 1013.0 + rand::random::<f64>() * 20.0 - 10.0;

        csv_data.push_str(&format!(
            "{},{:.2},{:.2},{:.2}\n",
            day, temp, humidity, pressure
        ));
    }

    let df = read_csv_string(&csv_data, CsvReaderOptions::csv())?;
    println!("✓ Loaded time series data: {} days", df.nrows());

    // Statistical analysis
    let stats = df.describe()?;
    println!("\nTemperature Statistics:");
    if let Some(temp_stats) = stats.get("temperature") {
        println!("  Mean: {:.2}°C", temp_stats.get("mean").unwrap_or(&0.0));
        println!("  Min:  {:.2}°C", temp_stats.get("min").unwrap_or(&0.0));
        println!("  Max:  {:.2}°C", temp_stats.get("max").unwrap_or(&0.0));
        println!("  Std:  {:.2}°C", temp_stats.get("std").unwrap_or(&0.0));
    }

    // Calculate rolling average
    println!("\n✓ Time series analysis complete");
    write_csv(&df, "time_series_output.csv", b',')?;
    println!("✓ Exported: time_series_output.csv\n");

    Ok(())
}

fn classification_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. MACHINE LEARNING CLASSIFICATION DEMO");
    println!("=======================================\n");

    // Generate 3-class classification data
    let n_per_class = 100;
    let mut csv_data = String::from("feature1,feature2,class,label\n");

    // Class 0: cluster around (2, 2)
    for _ in 0..n_per_class {
        let x = 2.0 + rand::random::<f64>() * 2.0 - 1.0;
        let y = 2.0 + rand::random::<f64>() * 2.0 - 1.0;
        csv_data.push_str(&format!("{:.4},{:.4},0,ClassA\n", x, y));
    }

    // Class 1: cluster around (6, 6)
    for _ in 0..n_per_class {
        let x = 6.0 + rand::random::<f64>() * 2.0 - 1.0;
        let y = 6.0 + rand::random::<f64>() * 2.0 - 1.0;
        csv_data.push_str(&format!("{:.4},{:.4},1,ClassB\n", x, y));
    }

    // Class 2: cluster around (2, 6)
    for _ in 0..n_per_class {
        let x = 2.0 + rand::random::<f64>() * 2.0 - 1.0;
        let y = 6.0 + rand::random::<f64>() * 2.0 - 1.0;
        csv_data.push_str(&format!("{:.4},{:.4},2,ClassC\n", x, y));
    }

    let df = read_csv_string(&csv_data, CsvReaderOptions::csv())?;
    println!("✓ Generated classification dataset: {} samples", df.nrows());

    // Group by class
    let grouped = df.groupby(&["class"])?.count()?;
    println!("\nClass Distribution:");
    for row in grouped.to_json() {
        println!("  {}", row);
    }

    // Feature statistics
    let stats = df.describe()?;
    println!("\nFeature Statistics:");
    if let Some(f1_stats) = stats.get("feature1") {
        println!(
            "  Feature 1 - Mean: {:.2}, Std: {:.2}",
            f1_stats.get("mean").unwrap_or(&0.0),
            f1_stats.get("std").unwrap_or(&0.0)
        );
    }
    if let Some(f2_stats) = stats.get("feature2") {
        println!(
            "  Feature 2 - Mean: {:.2}, Std: {:.2}",
            f2_stats.get("mean").unwrap_or(&0.0),
            f2_stats.get("std").unwrap_or(&0.0)
        );
    }

    // Calculate class centroids
    println!("\nClass Centroids:");
    let centroids = df.groupby(&["class"])?.mean()?;
    for row in centroids.to_json() {
        println!("  {}", row);
    }

    write_csv(&df, "classification_data.csv", b',')?;
    write_csv(&centroids, "class_centroids.csv", b',')?;
    println!("\n✓ Exported classification data");
    println!("✓ Exported: classification_data.csv");
    println!("✓ Exported: class_centroids.csv\n");

    Ok(())
}

fn spatial_3d_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. 3D SPATIAL DATA DEMO");
    println!("=======================\n");

    // Generate 3D spiral data
    let n_points = 500;
    let mut csv_data = String::from("x,y,z,intensity\n");

    for i in 0..n_points {
        let t = (i as f64 / n_points as f64) * 4.0 * PI;
        let x = 5.0 * t.cos() + (rand::random::<f64>() - 0.5) * 0.5;
        let y = 5.0 * t.sin() + (rand::random::<f64>() - 0.5) * 0.5;
        let z = t + (rand::random::<f64>() - 0.5) * 0.3;
        let intensity = ((z - t.min(0.0).abs()) * 50.0) as i32;

        csv_data.push_str(&format!("{:.4},{:.4},{:.4},{}\n", x, y, z, intensity));
    }

    let df = read_csv_string(&csv_data, CsvReaderOptions::csv())?;
    println!("✓ Generated 3D spiral data: {} points", df.nrows());

    // 3D statistics
    let stats = df.describe()?;
    println!("\n3D Spatial Statistics:");

    for axis in &["x", "y", "z"] {
        if let Some(axis_stats) = stats.get(*axis) {
            println!("\n{} axis:", axis.to_uppercase());
            println!(
                "  Range: [{:.2}, {:.2}]",
                axis_stats.get("min").unwrap_or(&0.0),
                axis_stats.get("max").unwrap_or(&0.0)
            );
            println!("  Mean: {:.2}", axis_stats.get("mean").unwrap_or(&0.0));
            println!("  Std:  {:.2}", axis_stats.get("std").unwrap_or(&0.0));
        }
    }

    // Calculate distances from origin
    println!("\n✓ Processing 3D spatial data");

    // Intensity distribution
    let intensity = df.column("intensity")?;
    let intensity_stats = intensity.describe()?;
    println!("\nIntensity Statistics:");
    println!("  Mean: {:.2}", intensity_stats.get("mean").unwrap_or(&0.0));
    println!(
        "  Range: [{:.0}, {:.0}]",
        intensity_stats.get("min").unwrap_or(&0.0),
        intensity_stats.get("max").unwrap_or(&0.0)
    );

    write_csv(&df, "spatial_3d_data.csv", b',')?;
    println!("\n✓ Exported: spatial_3d_data.csv\n");

    Ok(())
}
