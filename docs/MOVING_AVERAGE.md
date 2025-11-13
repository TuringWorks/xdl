# Moving Average Functions in XDL

This document describes the comprehensive suite of moving average functions implemented in XDL. These functions are essential for signal processing, time series analysis, financial data analysis, and statistical smoothing.

## Table of Contents

1. [Function Overview](#function-overview)
2. [SMOOTH - Simple Moving Average](#smooth---simple-moving-average)
3. [MOVING_AVERAGE - Configurable Edge Handling](#moving_average---configurable-edge-handling)
4. [WMA - Weighted Moving Average](#wma---weighted-moving-average)
5. [EMA - Exponential Moving Average](#ema---exponential-moving-average)
6. [CUMULATIVE_AVERAGE - Expanding Window](#cumulative_average---expanding-window)
7. [Usage Examples](#usage-examples)
8. [Performance Notes](#performance-notes)

---

## Function Overview

XDL provides five moving average implementations, each suited for different use cases:

| Function | Description | Output Length | Best For |
|----------|-------------|---------------|----------|
| **SMOOTH** | Simple moving average with edge reflection | Same as input | General-purpose smoothing, IDL compatibility |
| **MOVING_AVERAGE** | Configurable edge handling (4 modes) | Configurable | Advanced control over boundary behavior |
| **WMA** | Weighted moving average (linear weights) | `n - window + 1` | Emphasizing recent values |
| **EMA** | Exponential moving average | Same as input | Real-time data, financial analysis |
| **CUMULATIVE_AVERAGE** | Expanding window average | Same as input | Running statistics, progressive averaging |

---

## SMOOTH - Simple Moving Average

**Syntax:**

```xdl
result = SMOOTH(array, window_size)
```

**Description:**
Computes a simple moving average (boxcar smoothing) with edge reflection. This is the XDL equivalent to IDL's SMOOTH function.

**Parameters:**

- `array`: Input array (required)
- `window_size`: Size of the smoothing window (optional, default: 3)

**Edge Handling:**
Uses reflection at boundaries to maintain output size equal to input size.

**Examples:**

```xdl
; Basic usage
data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
smoothed = smooth(data, 5)
; Result: [2.2, 2.4, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 8.6, 8.8]

; Default window size (3)
smoothed_default = smooth(data)
```

**Use Cases:**

- General signal smoothing
- Noise reduction in measurements
- Preparation for visualization
- IDL code compatibility

---

## MOVING_AVERAGE - Configurable Edge Handling

**Syntax:**

```xdl
result = MOVING_AVERAGE(array, window_size, edge_mode)
```

**Description:**
Computes moving average with configurable edge handling modes for advanced control.

**Parameters:**

- `array`: Input array (required)
- `window_size`: Size of the moving window (required)
- `edge_mode`: Edge handling mode (optional, default: 2)
  - `0`: **TRUNCATE** - Only compute where full window fits (output smaller than input)
  - `1`: **WRAP** - Circular/periodic boundaries (wrap around)
  - `2`: **REFLECT** - Mirror at boundaries (default, same as SMOOTH)
  - `3`: **PAD_WITH_MEAN** - Use array mean for out-of-bounds values

**Examples:**

```xdl
data = [5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0]

; Truncate mode - returns 6 elements
ma_trunc = moving_average(data, 3, 0)
; Result: [10.0, 15.0, 20.0, 25.0, 30.0, 35.0]

; Wrap mode - circular boundaries
ma_wrap = moving_average(data, 3, 1)
; Result: [18.33, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 26.67]

; Reflect mode (default)
ma_reflect = moving_average(data, 3, 2)
; Result: [8.33, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 36.67]

; Pad with mean
ma_mean = moving_average(data, 3, 3)
; Result: [12.5, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 32.5]
```

**Use Cases:**

- Time series analysis with specific boundary requirements
- Periodic signals (use WRAP mode)
- Valid-only analysis (use TRUNCATE mode)
- Normalized edge handling (use PAD_WITH_MEAN mode)

---

## WMA - Weighted Moving Average

**Syntax:**

```xdl
result = WMA(array, window_size)
```

**Description:**
Computes weighted moving average with linearly increasing weights. Most recent values have the highest weight.

**Parameters:**

- `array`: Input array (required)
- `window_size`: Size of the moving window (required)

**Weighting Scheme:**
Linear weights from 1 to `window_size`:

- For window_size = 3: weights are [1, 2, 3]
- For window_size = 5: weights are [1, 2, 3, 4, 5]

**Formula:**

```text
WMA[i] = (1*x[i] + 2*x[i+1] + 3*x[i+2] + ... + n*x[i+n-1]) / (1+2+3+...+n)
```

**Output Length:**
`n - window_size + 1` (no edge padding)

**Examples:**

```xdl
prices = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0]

; WMA with window 3
wma3 = wma(prices, 3)
; Result: [23.33, 33.33, 43.33, 53.33, 63.33, 73.33]
; First value: (1*10 + 2*20 + 3*30) / 6 = 23.33

; WMA with window 5
wma5 = wma(prices, 5)
; Result: [40.0, 50.0, 60.0, 70.0]
```

**Use Cases:**

- Financial technical analysis
- Trend following
- When recent data is more important
- Forecasting with recency bias

---

## EMA - Exponential Moving Average

**Syntax:**

```xdl
result = EMA(array, alpha)
```

**Description:**
Computes exponential moving average using recursive formula with smoothing factor alpha.

**Parameters:**

- `array`: Input array (required)
- `alpha`: Smoothing factor, range (0, 1] (required)
  - Higher alpha: More weight on recent values (faster response)
  - Lower alpha: More smoothing (slower response)
  - Common conversion: `alpha = 2 / (N + 1)` for N-period EMA

**Formula:**

```text
EMA[0] = array[0]
EMA[i] = alpha * array[i] + (1 - alpha) * EMA[i-1]
```

**Common Alpha Values:**

- `alpha = 0.5`: ~3-period equivalent
- `alpha = 0.25`: ~7-period equivalent
- `alpha = 0.1818`: 10-period equivalent (2/(10+1))
- `alpha = 0.1`: ~19-period equivalent

**Examples:**

```xdl
prices = [100.0, 110.0, 105.0, 115.0, 120.0, 118.0, 125.0, 130.0]

; Fast response (similar to 3-period MA)
ema_fast = ema(prices, 0.5)
; Result: [100.0, 105.0, 105.0, 110.0, 115.0, 116.5, 120.75, 125.38]

; Moderate smoothing (similar to 6-period MA)
ema_moderate = ema(prices, 0.3)
; Result: [100.0, 103.0, 103.6, 107.02, 110.91, 113.04, 116.63, 120.64]

; Heavy smoothing (similar to 19-period MA)
ema_smooth = ema(prices, 0.1)
; Result: [100.0, 101.0, 101.4, 102.76, 104.48, 105.84, 107.75, 109.98]
```

**Use Cases:**

- Stock price analysis
- Trading indicators (MACD, RSI calculations)
- Real-time signal filtering
- When exponential decay is desired
- Adaptive systems

---

## CUMULATIVE_AVERAGE - Expanding Window

**Syntax:**

```xdl
result = CUMULATIVE_AVERAGE(array)
```

**Description:**
Computes cumulative (expanding window) moving average. Each element is the mean of all values from the start up to and including that position.

**Parameters:**

- `array`: Input array (required)

**Formula:**

```text
CUMAVG[i] = (sum of array[0] to array[i]) / (i + 1)
```

**Examples:**

```xdl
data = [5.0, 15.0, 10.0, 20.0, 25.0, 30.0]
cumavg = cumulative_average(data)
; Result: [5.0, 10.0, 10.0, 12.5, 15.0, 17.5]

; Interpretation:
; cumavg[0] = 5         ; mean of [5]
; cumavg[1] = 10        ; mean of [5, 15]
; cumavg[2] = 10        ; mean of [5, 15, 10]
; cumavg[3] = 12.5      ; mean of [5, 15, 10, 20]
; cumavg[4] = 15        ; mean of [5, 15, 10, 20, 25]
; cumavg[5] = 17.5      ; mean of all 6 elements
```

**Use Cases:**

- Running statistics
- Progressive data analysis
- Performance metrics over time
- Quality control (running average of measurements)
- Online learning algorithms

---

## Usage Examples

### Example 1: Noisy Signal Smoothing

```xdl
; Noisy sensor data
noisy_signal = [5.0, 8.2, 12.1, 14.8, 15.9, 14.5, 11.8, 8.3, 5.1, 3.2]

; Compare smoothing methods
smooth_sig = smooth(noisy_signal, 5)      ; Simple smoothing
ema_sig = ema(noisy_signal, 0.3)          ; Exponential smoothing
wma_sig = wma(noisy_signal, 5)            ; Weighted smoothing

print, "Original:", noisy_signal
print, "SMOOTH:", smooth_sig
print, "EMA:", ema_sig
print, "WMA:", wma_sig
```

### Example 2: Financial Technical Analysis

```xdl
; Daily stock prices
prices = [100.0, 102.0, 101.5, 103.0, 105.0, 104.0, 106.0, 108.0, 107.0, 109.0]

; Calculate common technical indicators
sma_5 = smooth(prices, 5)                 ; 5-day simple moving average
ema_10 = ema(prices, 0.1818)              ; 10-day exponential MA (alpha = 2/11)
ema_20 = ema(prices, 0.0952)              ; 20-day exponential MA (alpha = 2/21)

; Trading signals
; Buy signal: when 5-day SMA > 10-day EMA
; Sell signal: when 5-day SMA < 10-day EMA
```

### Example 3: Step Function Response Comparison

```xdl
; Step function (sudden change)
step = [10.0, 10.0, 10.0, 10.0, 50.0, 50.0, 50.0, 50.0]

; Compare response times
smooth_resp = smooth(step, 3)             ; Gradual transition
ema_fast = ema(step, 0.5)                 ; Fast response
ema_slow = ema(step, 0.2)                 ; Slow response
cumavg_resp = cumulative_average(step)    ; Progressive incorporation

print, "SMOOTH:", smooth_resp
print, "EMA (fast):", ema_fast
print, "EMA (slow):", ema_slow
print, "Cumulative:", cumavg_resp
```

### Example 4: Edge Handling Comparison

```xdl
data = [5.0, 10.0, 15.0, 20.0, 25.0, 30.0]

; Compare different edge modes
truncate = moving_average(data, 3, 0)     ; Only valid windows
wrap = moving_average(data, 3, 1)         ; Circular
reflect = moving_average(data, 3, 2)      ; Mirror
pad_mean = moving_average(data, 3, 3)     ; Pad with mean

print, "Truncate (6->4):", truncate
print, "Wrap:", wrap
print, "Reflect:", reflect
print, "Pad with mean:", pad_mean
```

---

## Performance Notes

### Computational Complexity

| Function | Time Complexity | Space Complexity | Notes |
|----------|----------------|------------------|-------|
| SMOOTH | O(n * w) | O(n) | w = window size |
| MOVING_AVERAGE | O(n * w) | O(n) | Depends on mode |
| WMA | O(n * w) | O(n) | Weighted computation |
| EMA | O(n) | O(n) | Recursive, most efficient |
| CUMULATIVE_AVERAGE | O(n) | O(n) | Single pass |

### Memory Usage

All functions create output arrays:

- **SMOOTH**, **EMA**, **CUMULATIVE_AVERAGE**: Output size = input size
- **MOVING_AVERAGE**: Size depends on mode (truncate mode reduces size)
- **WMA**: Output size = `n - window + 1`

### Accuracy Considerations

1. **Floating-point precision**: All calculations use 64-bit doubles
2. **Edge effects**: Different functions handle edges differently
3. **Numerical stability**: EMA is numerically stable for long sequences
4. **Weighting bias**: WMA gives more weight to recent values

### When to Use Which Function

**Use SMOOTH when:**

- You need IDL compatibility
- Simple general-purpose smoothing is sufficient
- Output size must match input size
- Default edge reflection is acceptable

**Use MOVING_AVERAGE when:**

- You need specific edge handling behavior
- Different modes are required for different data types
- You're working with periodic signals (wrap mode)
- You need valid-only results (truncate mode)

**Use WMA when:**

- Recent values are more important
- You're doing trend analysis
- You need weighted importance
- You accept reduced output size

**Use EMA when:**

- You need exponential decay
- Real-time processing is required
- Memory efficiency matters (single-pass algorithm)
- Financial analysis (standard in trading)
- You need fast response to changes

**Use CUMULATIVE_AVERAGE when:**

- You need running statistics
- Progressive averaging is required
- All historical data is equally weighted
- You're tracking performance over time

---

## Testing

Comprehensive tests are available in `examples/test_moving_average.xdl`:

```bash
xdl examples/test_moving_average.xdl
```

The test suite includes:

- Basic functionality tests for all functions
- Edge case handling (empty, single element, uniform arrays)
- Different window sizes
- Step function response
- Noisy signal smoothing
- Financial data examples
- Statistical property verification
- Performance with larger arrays (20+ elements)

All tests verify correctness with known inputs and expected outputs.

---

## Implementation Notes

### Edge Reflection (SMOOTH and MOVING_AVERAGE mode 2)

```text
Input:  [1, 2, 3, 4, 5]
         ↓  Reflect at edges
Virtual:[1, 2, 1, 2, 3, 4, 5, 4, 3]
```

### Wrap Mode (MOVING_AVERAGE mode 1)

```text
Input:  [1, 2, 3, 4, 5]
         ↓  Wrap around
Virtual:[4, 5, 1, 2, 3, 4, 5, 1, 2]
```

### EMA Convergence

EMA converges to recent values with decay factor `(1 - alpha)`:

- After 3 periods: ~95% weight on recent data (alpha=0.5)
- After 10 periods: ~89% weight on recent data (alpha=0.1)

---

## Related Functions

- **MEAN**: Simple arithmetic mean
- **MEDIAN**: Median value (robust to outliers)
- **TOTAL**: Sum of array elements
- **REVERSE**: Reverse array order
- **SORT**: Sort array elements

---

## References

1. Smith, S. W. (1997). *The Scientist and Engineer's Guide to Digital Signal Processing*. California Technical Publishing.
2. Murphy, J. J. (1999). *Technical Analysis of the Financial Markets*. New York Institute of Finance.
3. Press, W. H., et al. (2007). *Numerical Recipes: The Art of Scientific Computing* (3rd ed.). Cambridge University Press.

---

## Version History

- **v1.0** (2025-10): Initial implementation with all five moving average functions
  - SMOOTH with edge reflection
  - MOVING_AVERAGE with 4 edge modes
  - WMA with linear weights
  - EMA with configurable alpha
  - CUMULATIVE_AVERAGE for expanding windows

---

## See Also

- [XDL Array Functions](ARRAY_FUNCTIONS.md)
- [XDL Statistics Functions](STATISTICS.md)
- [XDL Math Functions](MATH_FUNCTIONS.md)
