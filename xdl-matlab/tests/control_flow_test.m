% Test file for MATLAB control flow support

% ===== SWITCH/CASE =====
function result = test_switch(x)
    switch x
        case 1
            result = 'one';
        case 2
            result = 'two';
        case {3, 4}
            result = 'three or four';
        otherwise
            result = 'other';
    end
end

% ===== TRY/CATCH =====
function safe_divide(a, b)
    try
        result = a / b;
        disp(result);
    catch err
        disp('Error: division failed');
        result = NaN;
    end
end

% ===== BREAK/CONTINUE =====
function find_first_positive(arr)
    for i = 1:length(arr)
        if arr(i) < 0
            continue;  % Skip negative numbers
        end
        if arr(i) > 10
            break;     % Stop if number is too large
        end
        disp(arr(i));
    end
end

% ===== RETURN STATEMENT =====
function result = early_return(x)
    if x < 0
        result = 0;
        return;
    end
    result = sqrt(x);
end

% ===== COMPLEX FOR LOOPS WITH STEPS =====
for i = 1:2:10
    % Loop with step of 2
    disp(i);
end

for j = 10:-1:1
    % Descending loop
    disp(j);
end

for k = 0:0.5:5
    % Fractional step
    disp(k);
end

% ===== NESTED CONTROL FLOW =====
function classify(x)
    if x > 0
        for i = 1:x
            if mod(i, 2) == 0
                continue;
            end
            disp(i);
        end
    else
        switch x
            case 0
                disp('zero');
            case -1
                disp('negative one');
            otherwise
                disp('negative');
        end
    end
end

% ===== WHILE WITH BREAK =====
function countdown(n)
    while n > 0
        if n == 5
            disp('Skipping 5!');
            n = n - 1;
            continue;
        end
        disp(n);
        n = n - 1;
        if n == 2
            break;
        end
    end
end
