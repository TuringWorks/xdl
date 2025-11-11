% MATLAB Array Plot Test
% Tests plotting arrays without explicit X values

disp('Testing array plotting...');

% Create array data
data = [1, 4, 9, 16, 25, 36, 49, 64, 81, 100];

% Plot array (X should be auto-generated as indices)
plot(data);
title('Square Numbers');
xlabel('Index');
ylabel('Value');

disp('Array plot created successfully!');
