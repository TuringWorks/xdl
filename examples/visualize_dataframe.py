#!/usr/bin/env python3
"""
Visualize XDL DataFrame Analysis Results
Generates plots from the DataFrame demo output

This script runs XDL's simple_data_analysis.xdl demo to generate
employee data, then reads the CSV and creates visualizations using pandas.
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import subprocess
import sys
from pathlib import Path

# Set style
plt.style.use('seaborn-v0_8-darkgrid')
plt.rcParams['figure.figsize'] = (12, 8)

print("=== XDL DataFrame Visualization ===\n")

# Step 1: Run XDL demo to generate data
print("Step 1: Running XDL DataFrame demo to generate employee data...")
print("-" * 60)

# Find XDL executable
xdl_path = Path(__file__).parent.parent / "target" / "release" / "xdl"
if not xdl_path.exists():
    xdl_path = Path(__file__).parent.parent / "target" / "debug" / "xdl"
if not xdl_path.exists():
    print("ERROR: XDL executable not found. Please run 'cargo build --release' first.")
    sys.exit(1)

# Run simple_data_analysis demo
demo_script = Path(__file__).parent.parent / "xdl-dataframe" / "examples" / "simple_data_analysis.xdl"
if demo_script.exists():
    print(f"\nRunning: {demo_script.name}")
    try:
        result = subprocess.run(
            [str(xdl_path), str(demo_script)],
            capture_output=True,
            text=True,
            timeout=30,
            cwd=demo_script.parent.parent  # Run in xdl-dataframe directory
        )
        if result.returncode == 0:
            print("  ✓ Demo completed successfully")
        else:
            print(f"  ✗ Demo failed: {result.stderr[:200]}")
    except subprocess.TimeoutExpired:
        print("  ⚠ Demo timed out (may have opened visualization windows)")
    except Exception as e:
        print(f"  ✗ Error running demo: {e}")
else:
    print(f"WARNING: Demo script not found at {demo_script}")

print("\nStep 2: Loading generated data from CSV...")
print("-" * 60)

# Load data from generated CSV
csv_file = Path(__file__).parent.parent / "xdl-dataframe" / "sample_data.csv"
if csv_file.exists():
    print(f"✓ Loading data from: {csv_file}")
    df = pd.read_csv(csv_file)
    # Rename columns to match expected format
    df = df.rename(columns={'experience': 'years_experience'})
    print(f"  Loaded {len(df)} employee records")
else:
    print(f"WARNING: CSV file not found at {csv_file}")
    print("Falling back to hardcoded data...")
    # Fallback to hardcoded data
    employee_data = {
        'age': [28, 35, 42, 31, 26, 38, 29, 45, 33, 27, 40, 32, 30, 36, 25],
        'department': ['Engineering', 'Engineering', 'Management', 'Sales', 'Engineering',
                       'Sales', 'Engineering', 'Management', 'Sales', 'Engineering',
                       'Management', 'Sales', 'Engineering', 'Sales', 'Engineering'],
        'salary': [75000, 82000, 95000, 68000, 72000, 88000, 79000, 105000, 76000, 71000,
                   98000, 74000, 80000, 85000, 70000],
        'years_experience': [5, 10, 18, 7, 3, 12, 6, 20, 9, 4, 16, 8, 7, 11, 2]
    }
    df = pd.DataFrame(employee_data)

print("\nStep 3: Creating visualizations...")
print("-" * 60)
print()

# Create figure with subplots
fig = plt.figure(figsize=(16, 12))

# Plot 1: Age vs Salary scatter plot by department
ax1 = plt.subplot(2, 3, 1)
colors = {'Engineering': 'red', 'Sales': 'blue', 'Management': 'green'}
for dept in df['department'].unique():
    dept_data = df[df['department'] == dept]
    ax1.scatter(dept_data['age'], dept_data['salary'],
               label=dept, color=colors[dept], s=100, alpha=0.6, edgecolors='black')

ax1.set_xlabel('Age (years)', fontsize=12)
ax1.set_ylabel('Salary ($)', fontsize=12)
ax1.set_title('Age vs Salary Distribution', fontsize=14, fontweight='bold')
ax1.legend()
ax1.grid(True, alpha=0.3)

# Add correlation coefficient
correlation = df['age'].corr(df['salary'])
ax1.text(0.05, 0.95, f'Correlation: {correlation:.3f}',
        transform=ax1.transAxes, fontsize=10,
        bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))

# Plot 2: Average salary by department (bar chart)
ax2 = plt.subplot(2, 3, 2)
dept_avg = df.groupby('department')['salary'].mean().sort_values(ascending=False)
bars = ax2.bar(range(len(dept_avg)), dept_avg.values,
              color=[colors[d] for d in dept_avg.index], alpha=0.7, edgecolor='black')
ax2.set_xticks(range(len(dept_avg)))
ax2.set_xticklabels(dept_avg.index, rotation=0)
ax2.set_ylabel('Average Salary ($)', fontsize=12)
ax2.set_title('Average Salary by Department', fontsize=14, fontweight='bold')
ax2.grid(True, axis='y', alpha=0.3)

# Add value labels on bars
for i, (bar, value) in enumerate(zip(bars, dept_avg.values)):
    height = bar.get_height()
    ax2.text(bar.get_x() + bar.get_width()/2., height,
            f'${value:,.0f}',
            ha='center', va='bottom', fontsize=10)

# Plot 3: Employee count by department
ax3 = plt.subplot(2, 3, 3)
dept_count = df['department'].value_counts()
wedges, texts, autotexts = ax3.pie(dept_count.values, labels=dept_count.index,
                                     autopct='%1.1f%%', startangle=90,
                                     colors=[colors[d] for d in dept_count.index])
ax3.set_title('Employee Distribution by Department', fontsize=14, fontweight='bold')

# Plot 4: Experience vs Salary with trend line
ax4 = plt.subplot(2, 3, 4)
for dept in df['department'].unique():
    dept_data = df[df['department'] == dept]
    ax4.scatter(dept_data['years_experience'], dept_data['salary'],
               label=dept, color=colors[dept], s=100, alpha=0.6, edgecolors='black')

# Add trend line
z = np.polyfit(df['years_experience'], df['salary'], 1)
p = np.poly1d(z)
x_trend = np.linspace(df['years_experience'].min(), df['years_experience'].max(), 100)
ax4.plot(x_trend, p(x_trend), "r--", linewidth=2, label=f'Trend: y={z[0]:.0f}x+{z[1]:.0f}')

ax4.set_xlabel('Years of Experience', fontsize=12)
ax4.set_ylabel('Salary ($)', fontsize=12)
ax4.set_title('Experience vs Salary with Trend Line', fontsize=14, fontweight='bold')
ax4.legend()
ax4.grid(True, alpha=0.3)

# Plot 5: Age distribution histogram
ax5 = plt.subplot(2, 3, 5)
ax5.hist(df['age'], bins=8, color='skyblue', edgecolor='black', alpha=0.7)
ax5.set_xlabel('Age (years)', fontsize=12)
ax5.set_ylabel('Count', fontsize=12)
ax5.set_title('Age Distribution Histogram', fontsize=14, fontweight='bold')
ax5.grid(True, axis='y', alpha=0.3)

# Add statistics
mean_age = df['age'].mean()
ax5.axvline(mean_age, color='red', linestyle='--', linewidth=2, label=f'Mean: {mean_age:.1f}')
ax5.legend()

# Plot 6: Box plot of salaries by department
ax6 = plt.subplot(2, 3, 6)
dept_salaries = [df[df['department'] == dept]['salary'].values for dept in dept_avg.index]
bp = ax6.boxplot(dept_salaries, labels=dept_avg.index, patch_artist=True)

# Color the boxes
for patch, dept in zip(bp['boxes'], dept_avg.index):
    patch.set_facecolor(colors[dept])
    patch.set_alpha(0.7)

ax6.set_ylabel('Salary ($)', fontsize=12)
ax6.set_title('Salary Distribution by Department', fontsize=14, fontweight='bold')
ax6.grid(True, axis='y', alpha=0.3)

plt.tight_layout()

# Save the plot
output_file = 'dataframe_visualization.png'
plt.savefig(output_file, dpi=300, bbox_inches='tight')
print(f"✓ Saved visualization: {output_file}")

# Show the plot
plt.show()

# Print summary statistics
print("\n=== Statistical Summary ===")
print(f"\nDataFrame Shape: {df.shape[0]} rows × {df.shape[1]} columns")
print(f"\nOverall Statistics:")
print(f"  Average Age: {df['age'].mean():.1f} years")
print(f"  Average Salary: ${df['salary'].mean():,.2f}")
print(f"  Average Experience: {df['years_experience'].mean():.1f} years")
print(f"  Age-Salary Correlation: {df['age'].corr(df['salary']):.3f}")
print(f"  Experience-Salary Correlation: {df['years_experience'].corr(df['salary']):.3f}")

print(f"\nBy Department:")
for dept in df['department'].unique():
    dept_df = df[df['department'] == dept]
    print(f"\n  {dept}:")
    print(f"    Employees: {len(dept_df)}")
    print(f"    Avg Age: {dept_df['age'].mean():.1f}")
    print(f"    Avg Salary: ${dept_df['salary'].mean():,.2f}")
    print(f"    Avg Experience: {dept_df['years_experience'].mean():.1f}")

print("\n=== Visualization Complete ===")
