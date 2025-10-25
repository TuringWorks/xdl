# GDL/IDL Syntax Guide for XDL

This document describes the GDL/IDL syntax rules that XDL supports, based on the parser implementation and working examples.

## Overview

XDL supports IDL (Interactive Data Language) and GDL (GNU Data Language) syntax for maximum compatibility with existing scientific code. This guide covers the key syntax patterns and common pitfalls.

## Control Flow Structures

### IF-THEN-ELSE Statements

#### Rule 1: Single Statement Form (requires ENDIF)

```idl
IF condition THEN statement

; OR with ELSE
IF condition THEN statement1 ELSE statement2

; Both require ENDIF when multi-line:
IF condition THEN
    statement
ENDIF
```

#### Rule 2: BEGIN...END Block Form (END closes both)

```idl
; Correct: END closes the BEGIN block
IF condition THEN BEGIN
    statement1
    statement2
END

; Correct: Both branches with BEGIN...END
IF condition THEN BEGIN
    statement1
END ELSE BEGIN
    statement2
END

; WRONG: Don't use ENDIF with BEGIN...END
IF condition THEN BEGIN
    statement1
ENDIF  ; ❌ Parse error!
```

**Important**: When using `BEGIN...END`, use `END` to close the block, **not** `ENDIF` or `ENDELSE`.

#### Rule 3: ENDIF/ENDELSE (optional with BEGIN...END)

```idl
; ENDIF is optional when using BEGIN...END
IF condition THEN BEGIN
    statements
END                  ; ✓ Correct

IF condition THEN BEGIN
    statements
ENDIF               ; ✓ Also acceptable (but END is preferred)
```

### FOR Loops

#### Rule 1: FOR with DO and Single Statement

```idl
FOR i = 0, 9 DO PRINT, i
; No ENDFOR needed for single statement
```

#### Rule 2: FOR with DO and BEGIN...END Block

```idl
; Correct: ENDFOR closes both BEGIN and FOR
FOR i = 0, 9 DO BEGIN
    statement1
    statement2
ENDFOR

; WRONG: Don't use END with FOR loops
FOR i = 0, 9 DO BEGIN
    statement1
END        ; ❌ Should be ENDFOR
```

#### Rule 3: Multi-line FOR without BEGIN (requires ENDFOR)

```idl
FOR i = 0, 9 DO
    PRINT, i
ENDFOR
```

**Important**: FOR loops with `BEGIN` blocks MUST be closed with `ENDFOR`, not `END`.

### WHILE Loops

```idl
; Single statement
WHILE condition DO statement

; Multiple statements
WHILE condition DO BEGIN
    statement1
    statement2
ENDWHILE
```

### REPEAT-UNTIL Loops

```idl
REPEAT BEGIN
    statements
ENDREP UNTIL condition
```

## Common Patterns

### Nested Loops

```idl
; Correct nesting with BEGIN...END
FOR i = 0, nx-1 DO BEGIN
    FOR j = 0, ny-1 DO BEGIN
        FOR k = 0, nz-1 DO BEGIN
            ; Inner code
            IF condition THEN BEGIN
                statement
            END ELSE BEGIN
                other_statement
            END
        ENDFOR
    ENDFOR
ENDFOR
```

### Line Continuation

```idl
; Use $ for line continuation
long_expression = value1 * $
                  value2 * $
                  value3

; Preferred: Join into single line when possible
long_expression = value1 * value2 * value3
```

### Keywords and Flags

```idl
; Keyword arguments
PROCEDURE_NAME, arg1, arg2, KEYWORD=value, /FLAG

; Examples
WINDOW, 0, XSIZE=800, YSIZE=600, TITLE='My Window'
VIZ3D_RENDER, /INTERACTIVE, TITLE='3D View'
```

### Arrays

```idl
; Array creation
arr = FLTARR(nx, ny, nz)

; Array indexing
value = arr[i, j, k]

; Slice extraction
slice = arr[*, *, k]           ; All x, all y, specific z
row = arr[i, *, k]             ; Specific x, all y, specific z

; Range extraction
subset = arr[10:20, 5:15, 0]  ; Ranges in x and y, first z
```

### Comments

```idl
; This is a comment

variable = value  ; Inline comment
```

## Functions and Procedures

### Function Definition

```idl
FUNCTION function_name, arg1, arg2, KEYWORD=default
    ; Function body
    result = computation
    RETURN, result
END
```

### Procedure Definition

```idl
PRO procedure_name, arg1, arg2, KEYWORD=default
    ; Procedure body
    ; Procedures don't return values
END

; Alternative keyword
PROCEDURE procedure_name, args
    ; Body
END
```

## Data Types

```idl
; Numeric types
int_val = 42
float_val = 3.14
double_val = 3.14d0

; Arrays
int_array = INTARR(10)
flt_array = FLTARR(10, 20)
dbl_array = DBLARR(5, 5, 5)

; Strings
str = 'Hello, World'
str2 = "Double quotes work too"

; Structures
struct = {name: 'value', count: 42, data: [1, 2, 3]}
```

## System Variables

```idl
!PI          ; Mathematical constant pi
!D           ; Graphics device settings
!P           ; Plotting system variables
!X, !Y, !Z   ; Axis system variables
```

## Best Practices

### 1. **Consistent Block Closures**

```idl
; ✓ Good: Consistent use of END for IF with BEGIN
IF condition THEN BEGIN
    code
END ELSE BEGIN
    code
END

; ✗ Bad: Mixing END and ENDIF
IF condition THEN BEGIN
    code
ENDIF  ; Wrong!
```

### 2. **FOR Loop Closures**

```idl
; ✓ Good: ENDFOR for FOR loops
FOR i = 0, 10 DO BEGIN
    code
ENDFOR

; ✗ Bad: END instead of ENDFOR
FOR i = 0, 10 DO BEGIN
    code
END  ; Wrong!
```

### 3. **Clear Nesting**

```idl
; ✓ Good: Clear indentation and structure
FOR i = 0, n-1 DO BEGIN
    IF condition THEN BEGIN
        statement
    END
ENDFOR

; ✗ Bad: Confusing nesting
FOR i = 0, n-1 DO BEGIN
IF condition THEN BEGIN
statement
END
ENDFOR
```

## Troubleshooting

### Parse Error: "Expected ENDFOR"

**Problem**: Using `END` instead of `ENDFOR` for FOR loops.

```idl
; ✗ Wrong
FOR i = 0, 10 DO BEGIN
    code
END

; ✓ Correct
FOR i = 0, 10 DO BEGIN
    code
ENDFOR
```

### Parse Error: "Unexpected token: Endif"

**Problem**: Using `ENDIF` with `BEGIN...END` blocks.

```idl
; ✗ Wrong
IF condition THEN BEGIN
    code
ENDIF

; ✓ Correct
IF condition THEN BEGIN
    code
END
```

### Parse Error: "Expected ENDIF"

**Problem**: Missing ENDIF for single-statement IF without BEGIN...END.

```idl
; ✗ Wrong (multi-line without END/ENDIF)
IF condition THEN
    statement1
    statement2

; ✓ Correct
IF condition THEN BEGIN
    statement1
    statement2
END
```

## Quick Reference Table

| Structure | With BEGIN...END | Without BEGIN...END |
|-----------|-----------------|-------------------|
| IF-THEN | `IF cond THEN BEGIN ... END` | `IF cond THEN stmt` |
| IF-THEN-ELSE | `IF cond THEN BEGIN ... END ELSE BEGIN ... END` | `IF cond THEN stmt1 ELSE stmt2` |
| FOR loop | `FOR i=0,n DO BEGIN ... ENDFOR` | `FOR i=0,n DO stmt` |
| WHILE loop | `WHILE cond DO BEGIN ... ENDWHILE` | `WHILE cond DO stmt` |
| REPEAT loop | `REPEAT BEGIN ... ENDREP UNTIL cond` | N/A |

## Summary

**Key Rules:**
1. **FOR loops**: Always use `ENDFOR` (never `END`)
2. **IF with BEGIN**: Use `END` (not `ENDIF`)
3. **IF without BEGIN**: Requires `ENDIF` for multi-line
4. **Consistent indentation**: Helps avoid nesting errors
5. **Line continuation**: Use `$` or join lines

**When in doubt:**
- Use `BEGIN...END` blocks for clarity
- Use proper closure keywords (ENDFOR, ENDWHILE, etc.)
- Check working examples in `examples/` directory

## References

- Working examples: `examples/scientific/*.xdl`
- Parser implementation: `xdl-parser/src/parser.rs`
- IDL Documentation: https://www.nv5geospatialsoftware.com/docs/routines.html
- GDL Documentation: https://github.com/gnudatalanguage/gdl
