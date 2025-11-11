; Test script for RANDOMN (normal/Gaussian random numbers)

print, '=== Testing RANDOMN ==='
print, ''

; Generate single normal random number
seed = 42
r = RANDOMN(seed)
print, 'Single normal random number (seed=42): ', r
print, ''

; Generate array of normal random numbers
print, '=== Generating 1000 normal random numbers ==='
seed = 12345
data = RANDOMN(seed, 1000)
print, 'Generated ', N_ELEMENTS(data), ' values'
print, ''

; Compute statistics
mean_val = MEAN(data)
std_val = STDDEV(data)
min_val = MIN(data)
max_val = MAX(data)

print, 'Statistics of generated data:'
print, 'Mean: ', mean_val, ' (expected: ~0.0)'
print, 'Std Dev: ', std_val, ' (expected: ~1.0)'
print, 'Min: ', min_val
print, 'Max: ', max_val
print, ''

; Check distribution by computing moments
moments = MOMENT(data)
print, 'Moments:'
print, 'Mean: ', moments[0]
print, 'Variance: ', moments[1], ' (expected: ~1.0)'
print, ''

; Generate smaller sample to display
print, '=== Sample of 10 values ==='
seed = 999
sample = RANDOMN(seed, 10)
print, sample
print, ''

print, 'All tests completed!'
print, 'RANDOMN generates normally distributed random numbers with mean=0, std=1'
