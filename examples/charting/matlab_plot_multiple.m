% MATLAB Multiple Plots Test
% Tests plotting multiple functions

disp('Creating multiple function plots...');

% Create X data
x = [0:0.1:6.28];

% First plot - Sine
y1 = sin(x);
plot(x, y1);
title('Sine Wave');

% Second plot - Cosine
y2 = cos(x);
plot(x, y2);
title('Cosine Wave');

% Third plot - Combined
y3 = sin(x) + cos(x);
plot(x, y3);
title('Sin + Cos');

disp('All plots created!');
