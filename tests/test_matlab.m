% Simple MATLAB test script
% This demonstrates MATLAB compatibility

% Create array using zeros
x = zeros(10, 1);

% Fill array with values
for i = 1:10
    x(i) = sin(i * 0.1);
end

% Calculate statistics
mean_x = mean(x);
max_x = max(x);

% Display results
disp('Mean value:');
disp(mean_x);
disp('Max value:');
disp(max_x);

% Simple plot
y = 1:10;
plot(y, x);
title('MATLAB Test Plot');
xlabel('Index');
ylabel('Sin(x)');
