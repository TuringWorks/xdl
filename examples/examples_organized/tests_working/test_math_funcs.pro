; Test script for new math functions

print, '=== Testing Inverse Hyperbolic Functions ==='
print, 'ASINH(1.0) = ', ASINH(1.0), ' (expected: ~0.881)'
print, 'ACOSH(2.0) = ', ACOSH(2.0), ' (expected: ~1.317)'
print, 'ATANH(0.5) = ', ATANH(0.5), ' (expected: ~0.549)'
print, ''

print, '=== Testing BETA Function ==='
print, 'BETA(2, 3) = ', BETA(2, 3), ' (expected: 0.0833...)'
print, 'BETA(1, 1) = ', BETA(1, 1), ' (expected: 1.0)'
print, ''

print, '=== Testing GCD and LCM ==='
print, 'GCD(48, 18) = ', GCD(48, 18), ' (expected: 6)'
print, 'GCD(100, 35) = ', GCD(100, 35), ' (expected: 5)'
print, 'LCM(12, 18) = ', LCM(12, 18), ' (expected: 36)'
print, 'LCM(7, 5) = ', LCM(7, 5), ' (expected: 35)'
print, ''

print, '=== Testing POLY ==='
coeffs = [1.0, 2.0, 3.0]  ; Polynomial: 1 + 2x + 3x^2
x_val = 2.0
result = POLY(x_val, coeffs)
print, 'POLY(2.0, [1,2,3]) = ', result, ' (expected: 17 = 1 + 2*2 + 3*4)'
print, ''

print, '=== Testing BINOMIAL ==='
print, 'BINOMIAL(5, 2) = ', BINOMIAL(5, 2), ' (expected: 10)'
print, 'BINOMIAL(10, 3) = ', BINOMIAL(10, 3), ' (expected: 120)'
print, 'BINOMIAL(4, 4) = ', BINOMIAL(4, 4), ' (expected: 1)'
print, ''

print, 'All math function tests completed!'
