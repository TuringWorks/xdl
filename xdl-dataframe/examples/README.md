# XDL DataFrame Examples

This directory contains comprehensive examples demonstrating the XDL DataFrame module's capabilities for data manipulation, analysis, and integration with XDL's machine learning and visualization features.

## Examples Overview

### 1. **csv_analysis.xdl** - CSV Data Analysis
Demonstrates basic DataFrame operations with CSV data:
- Reading CSV files
- Data exploration (head, tail, describe)
- Column selection and filtering
- Grouping and aggregation
- Sorting and value counts
- Integration with XDL plotting
- Exporting to TSV format

**Key Features:**
- Employee data analysis
- Statistical summaries
- Data filtering and sorting
- Scatter plot visualization

### 2. **parquet_example.xdl** - Parquet Format Support
Shows how to work with Parquet files for big data:
- Reading Parquet files (columnar format)
- Efficient data processing
- Advanced filtering and grouping
- Feature extraction for ML
- Data normalization

**Key Features:**
- Big data handling
- ML feature preparation
- Multi-column grouping
- Complex filtering operations

### 3. **database_integration.xdl** - Database Integration
Demonstrates seamless integration between SQL databases and DataFrames:
- Connecting to SQLite databases
- Converting Recordsets to DataFrames
- DataFrame analytics on query results
- Joining DataFrames
- Writing results back to database
- Bar chart visualization

**Key Features:**
- SQL to DataFrame conversion
- Data joining operations
- Calculated columns
- Database round-trip (query → DataFrame → database)

### 4. **ml_and_viz_integration.xdl** - ML and Visualization
Comprehensive example showing DataFrame integration with XDL's ML and visualization:
- Loading scientific datasets
- Feature engineering
- Data normalization
- Training classification models
- Model evaluation (accuracy, confusion matrix)
- 2D scatter plots and correlation heatmaps
- 3D surface plots
- 3D scatter plots with classification results
- Time series visualization

**Key Features:**
- Sensor data analysis
- Random Forest classification
- Feature importance analysis
- Correlation matrix visualization
- 3D surface and scatter plots
- Time series trends

## Running the Examples

### Prerequisites

1. Build XDL with DataFrame support:
```bash
cargo build --features "all"
```

2. For Parquet support:
```bash
cargo build --features "parquet-support"
```

3. For Avro support:
```bash
cargo build --features "avro-support"
```

### Execution

Run any example using the XDL interpreter:
```bash
xdl csv_analysis.xdl
xdl database_integration.xdl
xdl ml_and_viz_integration.xdl
```

## Data Formats Supported

- **CSV** - Comma-separated values (default)
- **TSV** - Tab-separated values
- **Parquet** - Columnar storage format (requires feature flag)
- **Avro** - Binary serialization format (requires feature flag)
- **Database** - Integration with SQL databases via Recordset

## DataFrame Operations

### Basic Operations
- `Head(n)` - First n rows
- `Tail(n)` - Last n rows
- `Shape()` - Get dimensions
- `ColumnNames()` - List column names
- `Describe()` - Statistical summary

### Selection and Filtering
- `Select(columns)` - Select specific columns
- `Filter(predicate)` - Filter rows based on condition
- `Column(name)` - Get specific column

### Aggregation
- `GroupBy(columns)` - Group data
- `Sum()` - Sum of groups
- `Mean()` - Mean of groups
- `Count()` - Count of groups

### Sorting
- `SortBy(columns, ascending)` - Sort by columns

### Data Export
- `WriteCSV(path)` - Export to CSV
- `WriteTSV(path)` - Export to TSV
- `ToJSON()` - Convert to JSON
- `ToXDLValue()` - Convert to XDL arrays

## Integration Points

### With XDL ML Functions
```xdl
; Extract features
X = df->Select(['feature1', 'feature2', 'feature3'])->ToXDLValue()

; Normalize
X_norm = XDLML_NORMALIZE(X, METHOD='minmax')

; Train model
model = XDLML_TRAIN_CLASSIFIER(X_train, y_train, ALGORITHM='random_forest')

; Predict
y_pred = XDLML_PREDICT(model, X_test)
```

### With XDL Charts
```xdl
; Extract data for plotting
x_data = df->Column('x')->Data()
y_data = df->Column('y')->Data()

; Create plot
PLOT, x_data, y_data, PSYM=4, XTITLE='X', YTITLE='Y'

; Create bar chart
XDLCHART_BAR, labels, values, TITLE='My Chart', OUTPUT='chart.html'
```

### With XDL 3D Visualization
```xdl
; 3D scatter plot
x = df->Column('x')->Data()
y = df->Column('y')->Data()
z = df->Column('z')->Data()

XDLVIZ3D_SCATTER, x, y, z, TITLE='3D Data', OUTPUT='scatter3d.html'

; 3D surface plot
XDLVIZ3D_SURFACE, x_grid, y_grid, z_surface, COLORMAP='jet'
```

### With Databases
```xdl
; Query to DataFrame
recordset = objdb->ExecuteSQL('SELECT * FROM table')
df = XDLDATAFRAME_FROM_RECORDSET(recordset)

; DataFrame to CSV for database import
df->WriteCSV, 'for_import.csv'
```

## Performance Tips

1. **Use Parquet for large datasets** - Columnar format is much faster
2. **Filter early** - Reduce data size before complex operations
3. **Type inference** - Let CSV reader infer types automatically
4. **Column selection** - Select only needed columns
5. **Batch operations** - Use GroupBy instead of row-by-row processing

## Common Patterns

### ETL Pipeline
```xdl
; Extract
df = XDLDATAFRAME_READ_CSV('raw_data.csv')

; Transform
df_clean = df->Filter(COLUMN='value', CONDITION='>0')
df_grouped = df_clean->GroupBy(['category'])->Mean()

; Load
df_grouped->WriteCSV, 'summary.csv'
```

### Data Quality Check
```xdl
; Check for missing values
stats = df->Describe()

; Check value distributions
counts = df->Column('status')->ValueCounts()

; Detect outliers
threshold = stats.value.mean + 3 * stats.value.std
outliers = df->Filter(COLUMN='value', CONDITION='>' + STRING(threshold))
```

### Feature Engineering
```xdl
; Normalize
X_norm = XDLML_NORMALIZE(X_raw)

; Create interaction features
df->AddColumn('feature_product', col1 * col2)

; Binning
df->AddColumn('age_group', BIN(ages, [0, 18, 35, 50, 100]))
```

## Additional Resources

- [XDL Documentation](https://xdl-lang.org/docs)
- [DataFrame API Reference](https://xdl-lang.org/docs/dataframe)
- [ML Functions Reference](https://xdl-lang.org/docs/ml)
- [Visualization Guide](https://xdl-lang.org/docs/visualization)

## Support

For questions and issues:
- GitHub Issues: https://github.com/xdl-lang/xdl/issues
- Documentation: https://xdl-lang.org/docs
