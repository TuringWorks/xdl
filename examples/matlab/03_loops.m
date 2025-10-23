% ============================================
% MATLAB Example 3: Loops
% ============================================
% Demonstrates FOR loops in MATLAB

disp('Counting from 1 to 10:');
for i = 1:10
    disp(i);
end
disp('');

% Calculate factorial
n = 5;
factorial = 1;
for i = 1:n
    factorial = factorial * i;
end
disp('Factorial of 5 is');
disp(factorial);

disp('');

% Sum of squares
sum_squares = 0;
for i = 1:10
    sum_squares = sum_squares + i * i;
end
disp('Sum of squares 1^2 + 2^2 + ... + 10^2 =');
disp(sum_squares);
