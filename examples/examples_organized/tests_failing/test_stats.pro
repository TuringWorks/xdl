; Test script for CORRELATE, REGRESS, and LINFIT functions

; Test CORRELATE
print, '=== Testing CORRELATE ==='
x = [1.0, 2.0, 3.0, 4.0, 5.0]
y = [2.0, 4.0, 6.0, 8.0, 10.0]
print, 'x = ', x
print, 'y = ', y
r = CORRELATE(x, y)
print, 'Correlation coefficient: ', r
print, 'Expected: ~1.0 (perfect positive correlation)'
print, ''

; Test with negative correlation
print, '=== Testing CORRELATE (negative) ==='
y2 = [10.0, 8.0, 6.0, 4.0, 2.0]
print, 'y2 = ', y2
r2 = CORRELATE(x, y2)
print, 'Correlation coefficient: ', r2
print, 'Expected: ~-1.0 (perfect negative correlation)'
print, ''

; Test LINFIT
print, '=== Testing LINFIT ==='
print, 'x = ', x
print, 'y = ', y
coeffs = LINFIT(x, y)
print, 'Linear fit coefficients [intercept, slope]: ', coeffs
print, 'Expected: [0.0, 2.0] since y = 2*x'
print, ''

; Test REGRESS
print, '=== Testing REGRESS (simple) ==='
result = REGRESS(x, y)
print, 'Regression coefficients [intercept, slope]: ', result
print, 'Expected: [0.0, 2.0] since y = 2*x'
print, ''

; Test with real data
print, '=== Testing with noisy data ==='
x_noise = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
y_noise = [2.1, 3.9, 6.2, 7.8, 10.1, 11.9, 14.2, 15.8]
print, 'x = ', x_noise
print, 'y = ', y_noise
r_noise = CORRELATE(x_noise, y_noise)
print, 'Correlation: ', r_noise
coeffs_noise = LINFIT(x_noise, y_noise)
print, 'LINFIT coefficients: ', coeffs_noise
regress_noise = REGRESS(x_noise, y_noise)
print, 'REGRESS coefficients: ', regress_noise
print, 'All three should show strong linear relationship close to y = 2*x'
print, ''

print, 'All tests completed!'
