% Complete tiledlayout demo with comet3
% Create 3D spiral data
t = linspace(0, 10*pi, 100);
xvec = t .* cos(t);
yvec = t .* sin(t);
zvec = t;

% Create tiled layout and plot
tiledlayout(1,2);
ax1 = nexttile;
ax2 = nexttile;
comet3(ax1,xvec,yvec,zvec)
comet3(ax2,yvec,xvec,zvec)
