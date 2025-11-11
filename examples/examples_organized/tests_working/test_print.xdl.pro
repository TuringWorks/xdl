; Test script for verifying XDL interpreter output capture
; This script tests various print statements and control flow

print, "=== XDL Output Capture Test ==="

; Test basic print
print, "Hello from XDL!"

; Test variable assignment and print
x = 42
print, "The answer is:", x

; Test arithmetic
y = x * 2
print, "Double the answer:", y

; Test loop with print
for i = 1, 5 do begin
    print, "  Iteration", i, "value =", i * 10
end
endfor

; Test conditional with print
if x gt 40 then begin
    print, "X is greater than 40"
end else begin
    print, "X is not greater than 40"
end
endif

print, "=== Test completed ==="
