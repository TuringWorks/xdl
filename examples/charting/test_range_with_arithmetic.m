% Test range expressions with arithmetic operations
disp('Testing range with arithmetic...');

% Simple range
x1 = 0:0.1:1;
disp('x1 created (0:0.1:1)');

% Range with pi constant
x2 = 0:pi/100:2*pi;
disp('x2 created (0:pi/100:2*pi)');

% Compute sine
y = sin(x2);
disp('y = sin(x2) computed');

% Plot
plot(x2, y);
title('Sine Wave with pi range');
xlabel('x (0 to 2*pi)');
ylabel('sin(x)');

disp('Plot created successfully!');
