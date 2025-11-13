//! DataFrame demonstration with data analysis and visualization output
//!
//! This example shows:
//! - Creating DataFrames from CSV data
//! - Statistical analysis
//! - Grouping and aggregation
//! - Filtering and sorting
//! - Generating data for plotting

use xdl_dataframe::{read_csv_string, CsvReaderOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== XDL DataFrame Demonstration ===\n");

    // Create sample employee data
    let csv_data = r#"name,age,department,salary,years_experience
Alice Johnson,28,Engineering,75000,5
Bob Smith,35,Engineering,82000,10
Carol White,42,Management,95000,18
David Brown,31,Sales,68000,7
Eve Davis,26,Engineering,72000,3
Frank Miller,38,Sales,88000,12
Grace Lee,29,Engineering,79000,6
Henry Wilson,45,Management,105000,20
Iris Chen,33,Sales,76000,9
Jack Taylor,27,Engineering,71000,4
Kate Anderson,40,Management,98000,16
Leo Martinez,32,Sales,74000,8
Maya Patel,30,Engineering,80000,7
Noah Kim,36,Sales,85000,11
Olivia Garcia,25,Engineering,70000,2"#;

    println!("1. Loading Data from CSV");
    println!("------------------------");
    let df = read_csv_string(csv_data, CsvReaderOptions::csv())?;
    println!(
        "✓ Loaded DataFrame: {} rows × {} columns",
        df.nrows(),
        df.ncols()
    );
    println!("  Columns: {:?}", df.column_names());
    println!();

    // Display data info
    println!("2. DataFrame Info");
    println!("-----------------");
    println!("{}", df.info());

    // Show first few rows
    println!("3. First 5 Rows");
    println!("---------------");
    let head = df.head(5)?;
    let head_json = head.to_json();
    for (i, row) in head_json.iter().enumerate() {
        println!("Row {}: {}", i, row);
    }
    println!();

    // Statistical summary
    println!("4. Statistical Summary");
    println!("----------------------");
    let stats = df.describe()?;
    for (col_name, col_stats) in &stats {
        println!("\n{}:", col_name);
        println!("  Count:  {:.0}", col_stats.get("count").unwrap_or(&0.0));
        println!("  Mean:   {:.2}", col_stats.get("mean").unwrap_or(&0.0));
        println!("  Std:    {:.2}", col_stats.get("std").unwrap_or(&0.0));
        println!("  Min:    {:.2}", col_stats.get("min").unwrap_or(&0.0));
        println!("  Median: {:.2}", col_stats.get("median").unwrap_or(&0.0));
        println!("  Max:    {:.2}", col_stats.get("max").unwrap_or(&0.0));
    }
    println!();

    // Group by department
    println!("5. Group By Department - Statistics");
    println!("------------------------------------");
    let grouped_mean = df.groupby(&["department"])?.mean()?;
    println!("\nAverage by Department:");
    let grouped_json = grouped_mean.to_json();
    for row in grouped_json.iter() {
        println!("{}", row);
    }
    println!();

    let grouped_count = df.groupby(&["department"])?.count()?;
    println!("Employee Count by Department:");
    let count_json = grouped_count.to_json();
    for row in count_json.iter() {
        println!("{}", row);
    }
    println!();

    // Filter data
    println!("6. Filtering Data");
    println!("-----------------");
    println!("Employees with salary > $80,000:");
    let high_earners = df.filter(|_idx, row| {
        if let Some(salary) = row.get("salary") {
            if let Ok(sal) = salary.to_double() {
                return sal > 80000.0;
            }
        }
        false
    })?;
    println!("  Found {} employees", high_earners.nrows());
    let selected = high_earners.select(&["name", "department", "salary"])?;
    for row_json in selected.to_json().iter() {
        println!("  {}", row_json);
    }
    println!();

    // Sort by salary
    println!("7. Top 5 Earners");
    println!("----------------");
    let sorted = df.sort_by(&["salary"], false)?;
    let top5 = sorted.head(5)?;
    let top5_selected = top5.select(&["name", "department", "salary"])?;
    for row_json in top5_selected.to_json().iter() {
        println!("{}", row_json);
    }
    println!();

    // Value counts
    println!("8. Value Counts");
    println!("---------------");
    let dept_series = df.column("department")?;
    let dept_counts = dept_series.value_counts();
    println!("Employees per Department:");
    for (dept, count) in dept_counts.iter() {
        println!("  {}: {}", dept, count);
    }
    println!();

    // Prepare data for visualization
    println!("9. Generating Visualization Data");
    println!("---------------------------------");

    // Extract age and salary for scatter plot
    let ages = df.column("age")?;
    let salaries = df.column("salary")?;

    println!("Data for Age vs Salary scatter plot:");
    println!("  Age data points: {}", ages.count());
    println!("  Salary data points: {}", salaries.count());

    // Calculate correlation
    if let (Ok(age_vals), Ok(sal_vals)) = (
        ages.data()
            .iter()
            .map(|v| v.to_double())
            .collect::<Result<Vec<_>, _>>(),
        salaries
            .data()
            .iter()
            .map(|v| v.to_double())
            .collect::<Result<Vec<_>, _>>(),
    ) {
        let mean_age: f64 = age_vals.iter().sum::<f64>() / age_vals.len() as f64;
        let mean_sal: f64 = sal_vals.iter().sum::<f64>() / sal_vals.len() as f64;

        let cov: f64 = age_vals
            .iter()
            .zip(sal_vals.iter())
            .map(|(a, s)| (a - mean_age) * (s - mean_sal))
            .sum::<f64>()
            / age_vals.len() as f64;

        let std_age: f64 = (age_vals.iter().map(|a| (a - mean_age).powi(2)).sum::<f64>()
            / age_vals.len() as f64)
            .sqrt();
        let std_sal: f64 = (sal_vals.iter().map(|s| (s - mean_sal).powi(2)).sum::<f64>()
            / sal_vals.len() as f64)
            .sqrt();

        let correlation = cov / (std_age * std_sal);
        println!("  Correlation (Age vs Salary): {:.3}", correlation);
    }
    println!();

    // Export processed data
    println!("10. Exporting Results");
    println!("---------------------");

    // Export summary to CSV
    use xdl_dataframe::write_csv;
    write_csv(&grouped_mean, "department_summary.csv", b',')?;
    println!("✓ Exported: department_summary.csv");

    // Export top earners
    write_csv(&top5_selected, "top_earners.csv", b',')?;
    println!("✓ Exported: top_earners.csv");

    println!();
    println!("=== DataFrame Demonstration Complete ===");
    println!();
    println!("Summary:");
    println!("  • Loaded and analyzed {} employee records", df.nrows());
    println!(
        "  • Identified {} high earners (>$80k)",
        high_earners.nrows()
    );
    println!("  • Calculated statistics by department");
    println!("  • Generated correlation analysis");
    println!("  • Exported 2 CSV files for further analysis");

    Ok(())
}
