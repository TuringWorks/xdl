% Simple MATLAB Plot Test
% Tests basic plotting functionality

% Create data
x = [0:0.1:10];
y = sin(x);

% Plot
plot(x, y);
title('Simple Sine Wave');
xlabel('X axis');
ylabel('Y = sin(X)');

disp('Plot created successfully!');
