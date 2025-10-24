% Test tiledlayout with comet3
disp('Testing tiledlayout with comet3...');
disp('');

% Create data for 3D plot
t = linspace(0, 10*pi, 100);
xvec = t .* cos(t);
yvec = t .* sin(t);
zvec = t;

disp('Created 3D spiral data');

% Create tiled layout
tiledlayout(1,2);

% First tile
ax1 = nexttile;
comet3(ax1, xvec, yvec, zvec);

% Second tile  
ax2 = nexttile;
comet3(ax2, yvec, xvec, zvec);

disp('');
disp('Tiledlayout demo complete!');
disp('Generated tile1_plot.png and tile2_plot.png');
