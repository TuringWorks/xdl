% Test all critical MATLAB fixes
disp('Testing MATLAB transpiler fixes...');
disp('');

% Test 1: pi constant
disp('Test 1: pi constant');
x = 2 * pi;
disp(x);

% Test 2: linspace
disp('');
disp('Test 2: linspace');
arr = linspace(0, 10, 5);
disp('linspace(0, 10, 5) created');

% Test 3: Array operations (sin of array)
disp('');
disp('Test 3: Array operations');
y = sin(arr);
disp('sin(array) computed');

% Test 4: figure/hold commands (should be ignored)
disp('');
disp('Test 4: figure/hold commands');
figure;
hold on;
disp('figure and hold commands processed');

% Test 5: xlabel/ylabel/title (should be ignored)
disp('');
disp('Test 5: xlabel/ylabel/title commands');
xlabel('X Axis');
ylabel('Y Axis');
title('Test Plot');
disp('Label commands processed');

% Test 6: Plot with line styles (styles should be ignored)
disp('');
disp('Test 6: Plot with line styles');
plot(arr, y, 'b-');
disp('Plot command with line style processed');

hold off;
disp('');
disp('All tests completed!');
