// File: 05_plotting_multiple_lines.m
% ============================================
% MATLAB Example 5: Plotting Multiple Lines
% ============================================
% Plotting Multiple Lines
% You can plot multiple lines on the same axes.

% Create data
x = linspace(-2*pi, 2*pi, 100);
y1 = sin(x);
y2 = cos(x);

% Plot both lines
figure; % Create a new figure window
plot(x, y1, x, y2);

% Add a legend
legend('sin(x)', 'cos(x)');
xlabel('x');
ylabel('y');
title('Sine and Cosine Waves');



// Click Execute to run this code
