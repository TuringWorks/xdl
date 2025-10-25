; Test script for array functions: UNIQ, ARRAY_INDICES, ARRAY_EQUAL, PERMUTE

print, '=== Testing UNIQ ==='
data = [1, 1, 2, 2, 2, 3, 4, 4, 5]
print, 'Data: ', data
idx = UNIQ(data)
print, 'UNIQ indices: ', idx
print, 'Expected indices: [0, 2, 5, 6, 8] (positions of first occurrence of each unique value)'
print, ''

; Test with continuous data
print, '=== Testing UNIQ with more data ==='
data2 = [1.0, 1.0, 1.0, 2.0, 2.0, 3.0]
print, 'Data: ', data2
idx2 = UNIQ(data2)
print, 'UNIQ indices: ', idx2
print, 'Expected: [0, 3, 5]'
print, ''

; Test ARRAY_EQUAL
print, '=== Testing ARRAY_EQUAL ==='
a1 = [1.0, 2.0, 3.0, 4.0]
a2 = [1.0, 2.0, 3.0, 4.0]
a3 = [1.0, 2.0, 3.0, 5.0]
print, 'a1 = ', a1
print, 'a2 = ', a2
print, 'a3 = ', a3
result1 = ARRAY_EQUAL(a1, a2)
print, 'ARRAY_EQUAL(a1, a2) = ', result1, ' (expected: 1)'
result2 = ARRAY_EQUAL(a1, a3)
print, 'ARRAY_EQUAL(a1, a3) = ', result2, ' (expected: 0)'
print, ''

print, 'All array function tests completed!'
