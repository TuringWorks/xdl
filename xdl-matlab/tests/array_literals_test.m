% Test file for MATLAB array literal support

% Simple row vector
a = [1, 2, 3, 4, 5];

% Row vector with spaces
b = [1 2 3 4 5];

% Column vector
c = [1; 2; 3; 4; 5];

% 2D matrix (rows separated by semicolons)
M = [1, 2, 3; 4, 5, 6; 7, 8, 9];

% Colon operator for ranges
x = 1:10;           % 1 to 10 with step 1
y = 0:0.1:1;        % 0 to 1 with step 0.1
z = 10:-1:1;        % 10 to 1 with step -1

% Array generation functions
zeros_arr = zeros(5);
ones_arr = ones(3, 4);
eye_mat = eye(4);
rand_arr = rand(3, 3);
linspace_arr = linspace(0, 10, 100);

% Complex expressions with arrays
d = [1:5, 10:15];   % Concatenating ranges
e = [a, b];         % Concatenating arrays

% Nested expressions
f = [sin(0:pi/4:pi)];

% Array operations
result = a + b;
product = a .* b;
