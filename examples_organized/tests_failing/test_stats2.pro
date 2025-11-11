; Test script for new statistics functions

print, '=== Testing PERCENTILES ==='
data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
p25 = PERCENTILES(data, 25)
p50 = PERCENTILES(data, 50)
p75 = PERCENTILES(data, 75)
print, '25th percentile: ', p25, ' (expected: 3.25)'
print, '50th percentile (median): ', p50, ' (expected: 5.5)'
print, '75th percentile: ', p75, ' (expected: 7.75)'
print, ''

print, '=== Testing TRIMMED_MEAN ==='
data2 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0]  ; outlier at end
regular_mean = MEAN(data2)
trimmed = TRIMMED_MEAN(data2, 0.1)  ; Trim 10% from each end
print, 'Regular mean: ', regular_mean, ' (affected by outlier)'
print, 'Trimmed mean (10%): ', trimmed, ' (more robust)'
print, ''

print, '=== Testing ROBUST_MEAN ==='
data3 = [5.0, 5.1, 4.9, 5.2, 4.8, 5.0, 20.0]  ; one large outlier
regular = MEAN(data3)
robust = ROBUST_MEAN(data3)
print, 'Regular mean: ', regular
print, 'Robust mean: ', robust, ' (outlier removed)'
print, ''

print, '=== Testing RANDOM_POISSON ==='
seed = 42
lambda_val = 3.0
single = RANDOM_POISSON(seed, lambda_val)
print, 'Single Poisson(3.0): ', single
samples = RANDOM_POISSON(seed, lambda_val, 100)
poisson_mean = MEAN(samples)
print, 'Mean of 100 samples: ', poisson_mean, ' (expected: ~3.0)'
print, ''

print, 'All statistics tests completed!'
