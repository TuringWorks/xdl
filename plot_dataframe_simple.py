#!/usr/bin/env python3
"""
Simple DataFrame Visualization (no pandas required)
"""

import matplotlib.pyplot as plt
import numpy as np

print("=== XDL DataFrame Visualization ===\n")

# Employee data from the DataFrame demo
names = ['Alice Johnson', 'Bob Smith', 'Carol White', 'David Brown', 'Eve Davis',
         'Frank Miller', 'Grace Lee', 'Henry Wilson', 'Iris Chen', 'Jack Taylor',
         'Kate Anderson', 'Leo Martinez', 'Maya Patel', 'Noah Kim', 'Olivia Garcia']

ages = [28, 35, 42, 31, 26, 38, 29, 45, 33, 27, 40, 32, 30, 36, 25]
departments = ['Engineering', 'Engineering', 'Management', 'Sales', 'Engineering',
               'Sales', 'Engineering', 'Management', 'Sales', 'Engineering',
               'Management', 'Sales', 'Engineering', 'Sales', 'Engineering']
salaries = [75000, 82000, 95000, 68000, 72000, 88000, 79000, 105000, 76000, 71000,
            98000, 74000, 80000, 85000, 70000]
experience = [5, 10, 18, 7, 3, 12, 6, 20, 9, 4, 16, 8, 7, 11, 2]

# Create figure
fig, axes = plt.subplots(2, 3, figsize=(16, 10))
fig.suptitle('XDL DataFrame Analysis - Employee Data', fontsize=16, fontweight='bold')

# Plot 1: Age vs Salary scatter
ax = axes[0, 0]
colors_map = {'Engineering': 'red', 'Sales': 'blue', 'Management': 'green'}
for dept in set(departments):
    dept_indices = [i for i, d in enumerate(departments) if d == dept]
    dept_ages = [ages[i] for i in dept_indices]
    dept_salaries = [salaries[i] for i in dept_indices]
    ax.scatter(dept_ages, dept_salaries, label=dept, s=100, alpha=0.6,
               color=colors_map[dept], edgecolors='black', linewidths=1.5)

ax.set_xlabel('Age (years)', fontsize=11)
ax.set_ylabel('Salary ($)', fontsize=11)
ax.set_title('Age vs Salary by Department', fontsize=12, fontweight='bold')
ax.legend()
ax.grid(True, alpha=0.3)

# Calculate and display correlation
correlation = np.corrcoef(ages, salaries)[0, 1]
ax.text(0.05, 0.95, f'Correlation: {correlation:.3f}',
        transform=ax.transAxes, fontsize=10,
        bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.7))

# Plot 2: Average Salary by Department
ax = axes[0, 1]
dept_stats = {}
for dept in set(departments):
    dept_salaries = [salaries[i] for i, d in enumerate(departments) if d == dept]
    dept_stats[dept] = sum(dept_salaries) / len(dept_salaries)

sorted_depts = sorted(dept_stats.items(), key=lambda x: x[1], reverse=True)
dept_names = [d[0] for d in sorted_depts]
dept_values = [d[1] for d in sorted_depts]

bars = ax.bar(range(len(dept_names)), dept_values,
             color=[colors_map[d] for d in dept_names], alpha=0.7, edgecolor='black', linewidth=1.5)
ax.set_xticks(range(len(dept_names)))
ax.set_xticklabels(dept_names, rotation=0)
ax.set_ylabel('Average Salary ($)', fontsize=11)
ax.set_title('Average Salary by Department', fontsize=12, fontweight='bold')
ax.grid(True, axis='y', alpha=0.3)

# Add value labels
for bar, value in zip(bars, dept_values):
    height = bar.get_height()
    ax.text(bar.get_x() + bar.get_width()/2., height,
            f'${value:,.0f}', ha='center', va='bottom', fontsize=9)

# Plot 3: Employee Distribution
ax = axes[0, 2]
dept_counts = {}
for dept in set(departments):
    dept_counts[dept] = departments.count(dept)

colors = [colors_map[d] for d in dept_counts.keys()]
ax.pie(dept_counts.values(), labels=dept_counts.keys(), autopct='%1.1f%%',
       startangle=90, colors=colors, textprops={'fontsize': 10})
ax.set_title('Employee Distribution', fontsize=12, fontweight='bold')

# Plot 4: Experience vs Salary
ax = axes[1, 0]
for dept in set(departments):
    dept_indices = [i for i, d in enumerate(departments) if d == dept]
    dept_exp = [experience[i] for i in dept_indices]
    dept_sal = [salaries[i] for i in dept_indices]
    ax.scatter(dept_exp, dept_sal, label=dept, s=100, alpha=0.6,
               color=colors_map[dept], edgecolors='black', linewidths=1.5)

# Trend line
z = np.polyfit(experience, salaries, 1)
p = np.poly1d(z)
x_trend = np.linspace(min(experience), max(experience), 100)
ax.plot(x_trend, p(x_trend), "r--", linewidth=2,
        label=f'Trend: y={z[0]:.0f}x+{z[1]:.0f}')

ax.set_xlabel('Years of Experience', fontsize=11)
ax.set_ylabel('Salary ($)', fontsize=11)
ax.set_title('Experience vs Salary', fontsize=12, fontweight='bold')
ax.legend(fontsize=9)
ax.grid(True, alpha=0.3)

# Plot 5: Age Distribution
ax = axes[1, 1]
ax.hist(ages, bins=8, color='skyblue', edgecolor='black', alpha=0.7, linewidth=1.5)
ax.axvline(np.mean(ages), color='red', linestyle='--', linewidth=2,
           label=f'Mean: {np.mean(ages):.1f}')
ax.set_xlabel('Age (years)', fontsize=11)
ax.set_ylabel('Count', fontsize=11)
ax.set_title('Age Distribution', fontsize=12, fontweight='bold')
ax.legend()
ax.grid(True, axis='y', alpha=0.3)

# Plot 6: Salary Box Plot by Department
ax = axes[1, 2]
dept_salary_data = []
dept_labels = []
for dept in ['Management', 'Sales', 'Engineering']:
    dept_sals = [salaries[i] for i, d in enumerate(departments) if d == dept]
    if dept_sals:
        dept_salary_data.append(dept_sals)
        dept_labels.append(dept)

bp = ax.boxplot(dept_salary_data, labels=dept_labels, patch_artist=True)
for patch, dept in zip(bp['boxes'], dept_labels):
    patch.set_facecolor(colors_map[dept])
    patch.set_alpha(0.7)

ax.set_ylabel('Salary ($)', fontsize=11)
ax.set_title('Salary Distribution by Department', fontsize=12, fontweight='bold')
ax.grid(True, axis='y', alpha=0.3)

plt.tight_layout()

# Save
output_file = 'dataframe_visualization.png'
plt.savefig(output_file, dpi=300, bbox_inches='tight')
print(f"✓ Saved: {output_file}")

# Print statistics
print("\n=== Statistical Summary ===")
print(f"\nTotal Employees: {len(names)}")
print(f"Average Age: {np.mean(ages):.1f} years")
print(f"Average Salary: ${np.mean(salaries):,.2f}")
print(f"Average Experience: {np.mean(experience):.1f} years")
print(f"\nCorrelations:")
print(f"  Age-Salary: {np.corrcoef(ages, salaries)[0,1]:.3f}")
print(f"  Experience-Salary: {np.corrcoef(experience, salaries)[0,1]:.3f}")

print(f"\nBy Department:")
for dept in set(departments):
    dept_data = [(ages[i], salaries[i], experience[i])
                 for i, d in enumerate(departments) if d == dept]
    dept_ages, dept_sals, dept_exp = zip(*dept_data)
    print(f"\n  {dept}:")
    print(f"    Count: {len(dept_data)}")
    print(f"    Avg Age: {np.mean(dept_ages):.1f}")
    print(f"    Avg Salary: ${np.mean(dept_sals):,.2f}")
    print(f"    Avg Experience: {np.mean(dept_exp):.1f}")

print("\n✓ Visualization complete!")
print(f"  Open '{output_file}' to view the plots")
