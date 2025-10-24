% Define the x-values
x = linspace(0, 2*pi, 100); % 100 points from 0 to 2*pi

% Define the y-values for two different functions
y1 = sin(x);
y2 = cos(x);

% Create a new figure window
figure;

% Plot the first function (sine) with a blue solid line
plot(x, y1, 'b-');

% Use 'hold on' to add subsequent plots to the same figure
hold on;

% Plot the second function (cosine) with a red dashed line and star markers
plot(x, y2, 'r--*');

% Add labels and a title to the plot
xlabel('x-axis');
ylabel('y-axis');
title('Sine and Cosine Functions');

% Add a legend to identify the lines
legend('Sine', 'Cosine');

% Turn off 'hold on'
hold off;
