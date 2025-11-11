% Comprehensive MATLAB Test
% Tests various MATLAB features with plotting

disp('===========================================');
disp('MATLAB Comprehensive Feature Test');
disp('===========================================');
disp('');

% 1. Basic arithmetic
disp('1. Testing basic arithmetic...');
a = 10;
b = 20;
c = a + b;
disp('a = ');
disp(a);
disp('b = ');
disp(b);
disp('c = a + b = ');
disp(c);
disp('');

% 2. Arrays and ranges
disp('2. Creating arrays...');
arr1 = [1, 2, 3, 4, 5];
arr2 = [0:0.5:5];
disp('Arrays created successfully');
disp('');

% 3. Mathematical functions
disp('3. Testing math functions...');
x = [0:0.2:6.28];
y_sin = sin(x);
y_cos = cos(x);
y_exp = exp(x / 10);
disp('Math functions computed');
disp('');

% 4. Plotting
disp('4. Creating plots...');
plot(x, y_sin);
title('Sine Function');
xlabel('X');
ylabel('sin(X)');
disp('First plot created');

plot(x, y_cos);
title('Cosine Function');
disp('Second plot created');
disp('');

% 5. Conditional logic
disp('5. Testing conditionals...');
if a < b
    disp('a is less than b - correct!');
end

if c == 30
    disp('c equals 30 - correct!');
end
disp('');

% 6. Loops
disp('6. Testing loops...');
sum_val = 0;
for i = [1:5]
    sum_val = sum_val + i;
end
disp('Sum of 1 to 5 = ');
disp(sum_val);
disp('');

disp('===========================================');
disp('All tests completed successfully!');
disp('===========================================');
