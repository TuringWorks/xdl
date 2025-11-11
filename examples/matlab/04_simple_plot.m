% ============================================
% MATLAB Example 4: Basic 2 D Plotting
% ============================================
% Simplified plotting example that works with XDL

% Create data
x = 0:pi/100:2*pi;
y = sin(x);

% Plot the data
plot(x, y);

% Add labels and title
xlabel('x');
ylabel('sin(x)');
title('Sine Wave');
