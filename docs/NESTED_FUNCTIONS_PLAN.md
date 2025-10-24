# Nested Function Definitions - Implementation Plan

## Status: 📋 PLANNED (Not Yet Implemented)

## Overview
Add support for defining functions within XDL scripts, either using MATLAB-style `function/end` syntax or GDL-style `PRO/FUNCTION/END` syntax.

## Current Limitation

```matlab
% MATLAB style - NOT YET SUPPORTED
function result = bernstein(i, n, t)
    result = nchoosek(n, i) * (t.^i) .* (1-t).^(n-i);
end
```

```xdl
; GDL style - PARTIALLY SUPPORTED (top-level only)
PRO my_procedure, arg1, arg2
    print, 'Processing:', arg1, arg2
END

FUNCTION my_function, x
    RETURN, x * 2
END
```

## Proposed Implementation

### Phase 1: Top-Level Functions (CURRENT STATUS)
✅ **Already Implemented**: Top-level PRO/FUNCTION definitions work
- Functions can be defined at the top level of scripts
- They are registered in the context
- Can be called from main script body

### Phase 2: Nested Function Scope
**To Implement**: Allow functions defined inside other functions/procedures

**Required Changes:**

1. **Parser** (`xdl-parser/src/parser.rs`)
   - Allow `Statement::FunctionDef` and `Statement::ProcedureDef` inside function bodies
   - Currently restricted to top-level only
   - Add validation for nested depth limits (prevent excessive nesting)

2. **Context** (`xdl-interpreter/src/context.rs`)
   - Implement function scope stack
   - Support lexical scoping for nested functions
   - Handle variable capture from parent scopes (closures)

3. **Interpreter** (`xdl-interpreter/src/lib.rs`)
   - Modify `execute_statement` to handle function definitions in nested contexts
   - Create new scope when entering function
   - Restore previous scope when exiting function
   - Handle `RETURN` statement properly in nested contexts

**Example Implementation Sketch:**

```rust
// In Context
pub struct Context {
    scopes: Vec<Scope>,  // Stack of scopes
    // ... existing fields
}

struct Scope {
    variables: HashMap<String, XdlValue>,
    functions: HashMap<String, FunctionDef>,
    parent_scope: Option<usize>,  // Link to parent scope
}

impl Context {
    pub fn push_scope(&mut self) {
        let parent_idx = if self.scopes.is_empty() {
            None
        } else {
            Some(self.scopes.len() - 1)
        };

        self.scopes.push(Scope {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent_scope,
        });
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_variable_with_scope(&self, name: &str) -> Option<&XdlValue> {
        // Search current scope first, then parent scopes
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.variables.get(name) {
                return Some(val);
            }
        }
        None
    }
}
```

### Phase 3: MATLAB-Style Function Syntax
**To Implement**: Parse MATLAB `function` keyword

**Required Changes:**

1. **Lexer** (`xdl-parser/src/lexer.rs`)
   - `Token::Function` already exists
   - Need to distinguish between:
     - GDL: `FUNCTION name, args` (procedure-style)
     - MATLAB: `function result = name(args)` (assignment-style)

2. **Parser** (`xdl-parser/src/parser.rs`)
   - Add `parse_matlab_function()`
   - Detect MATLAB syntax: `function` followed by `identifier =`
   - Parse return variable name
   - Parse parameter list in parentheses (MATLAB uses parens, GDL uses commas)

3. **AST** (`xdl-parser/src/ast.rs`)
   - Extend `FunctionDef` to include:
     - `return_var: Option<String>` for MATLAB-style named return
     - `style: FunctionStyle` enum (GDL vs MATLAB)

**Example:**

```rust
enum FunctionStyle {
    GDL,    // FUNCTION name, arg1, arg2
    MATLAB, // function result = name(arg1, arg2)
}

struct FunctionDef {
    name: String,
    params: Vec<Parameter>,
    return_var: Option<String>,  // For MATLAB: "result" in "function result = name(x)"
    style: FunctionStyle,
    body: Vec<Statement>,
}
```

### Phase 4: Inline/Anonymous Functions (Future)
Support MATLAB-style anonymous functions:

```matlab
f = @(x) x.^2 + 1
y = f(5)  % Returns 26
```

This requires:
- Lambda expression support in AST
- Closure implementation
- Function pointer/reference type

## Testing Strategy

### Test 1: Nested Helper Function
```xdl
FUNCTION main_calc, n
    ; Define helper function inside
    FUNCTION double_it, x
        RETURN, x * 2
    END

    result = double_it(n)
    RETURN, result
END

print, main_calc(21)  ; Should print 42
```

### Test 2: Scope Capture
```xdl
FUNCTION outer, multiplier
    FUNCTION inner, x
        ; Should access 'multiplier' from parent scope
        RETURN, x * multiplier
    END

    RETURN, inner(10)
END

print, outer(5)  ; Should print 50
```

### Test 3: MATLAB Style
```matlab
function y = quadratic(a, b, c, x)
    y = a*x^2 + b*x + c;
end

result = quadratic(1, -5, 6, 2);  % Should return 0
print, result
```

## Implementation Priority

1. **HIGH**: Phase 2 (Nested Function Scope)
   - Most impactful for code organization
   - Required for complex scientific applications
   - Estimated effort: 2-3 days

2. **MEDIUM**: Phase 3 (MATLAB Syntax)
   - Improves MATLAB compatibility
   - Many users expect this syntax
   - Estimated effort: 1-2 days

3. **LOW**: Phase 4 (Anonymous Functions)
   - Nice-to-have for functional programming
   - Can be deferred
   - Estimated effort: 3-4 days

## Current Workaround

Until nested functions are implemented, use separate top-level functions:

```xdl
; Instead of nested:
FUNCTION bernstein, i, n, t
    RETURN, nchoosek(n, i) * (t^i) * (1-t)^(n-i)
END

; Call from main code:
FOR i = 0, n DO BEGIN
    B = bernstein(i, n, t_values)
    ; Use B...
END
ENDFOR
```

## References

- GDL Documentation: https://gnudatalanguage.github.io/
- MATLAB Function Documentation: https://www.mathworks.com/help/matlab/ref/function.html
- XDL Current Implementation: `xdl-interpreter/src/lib.rs` lines 184-216

---

Last Updated: 2025-10-23
