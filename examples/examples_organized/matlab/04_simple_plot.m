% ============================================
% MATLAB Example 4: Simple Plotting
% ============================================
% Simplified plotting example that works with XDL

% Note: This is a simplified version
% Full MATLAB plotting features are not yet supported

disp('Creating simple sine wave data...');

% Create simple x data (using scalar approach)
% In full MATLAB, you would use: x = linspace(0, 2*pi, 20);
% For now, we'll demonstrate the concept

disp('');
disp('Simple plotting example:');
disp('To create plots in XDL, use:');
disp('  x = FINDGEN(100) * 0.0635');
disp('  y = SIN(x)');
disp('  PLOT, x, y, title=..., xtitle=..., ytitle=...');
disp('');

disp('Note: Full MATLAB plotting syntax like:');
disp('  - linspace()');
disp('  - figure, hold on/off');
disp('  - xlabel, ylabel, title, legend');
disp('  - line styles and markers');
disp('is not yet supported in the transpiler.');
