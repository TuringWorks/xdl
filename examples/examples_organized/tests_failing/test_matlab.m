% Simple MATLAB test file
x = [1, 2, 3, 4, 5];
y = x .^ 2;

fprintf('x values: ');
disp(x);

fprintf('y = x^2 values: ');
disp(y);

% Plot if available
plot(x, y, 'title', 'Quadratic Function', 'xtitle', 'X Values', 'ytitle', 'Y = X^2');
