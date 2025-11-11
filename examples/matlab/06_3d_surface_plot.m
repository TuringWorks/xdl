// File: 05_plotting_multiple_lines.m
% ============================================
% MATLAB Example 6: 3 D Surface Plot
% ============================================
% Create data

[X, Y] = meshgrid(-2:0.2:2);
Z = X .* exp(-X.^2 - Y.^2);

% Create surface plot
surf(X, Y, Z);

% Add labels and title
xlabel('X');
ylabel('Y');
zlabel('Z');
title('3D Surface Plot');
