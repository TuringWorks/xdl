# XDL Object-Oriented Programming System Design

## Overview

This document describes the design and implementation of object-oriented programming support in XDL, based on IDL/GDL OO syntax and semantics.

## IDL Object-Oriented Syntax Reference

### Class Definition

```idl
PRO ClassName__define
    ; Define structure with fields (properties)
    struct = {ClassName, $
              field1: 0, $
              field2: '', $
              field3: 0.0}
END
```

### Initialization Method

```idl
FUNCTION ClassName::Init, arg1, arg2, KEYWORD=value
    ; Initialize object fields
    self.field1 = arg1
    self.field2 = arg2
    ; Return 1 for success, 0 for failure
    RETURN, 1
END
```

### Cleanup Method

```idl
PRO ClassName::Cleanup
    ; Perform cleanup (free pointers, close files, etc.)
    ; SELF is automatically available
    PRINT, 'Cleaning up object'
END
```

### Regular Methods

```idl
FUNCTION ClassName::GetValue
    RETURN, self.field1
END

PRO ClassName::SetValue, new_value
    self.field1 = new_value
END
```

### Object Usage

```idl
; Create object
obj = OBJ_NEW('ClassName', arg1, arg2, KEYWORD=value)

; Call methods
value = obj->GetValue()
obj->SetValue, 42

; Destroy object
OBJ_DESTROY, obj
```

## XDL Implementation Architecture

### 1. AST Extensions (xdl-parser)

#### New Expression Types

```rust
pub enum Expression {
    // ... existing variants ...

    // Object creation
    ObjectNew {
        class_name: String,
        args: Vec<Expression>,
        keywords: Vec<KeywordArg>,
        location: Location,
    },
}
```

#### New Statement Types

```rust
pub enum Statement {
    // ... existing variants ...

    // Class definition: PRO ClassName__define
    ClassDefinition {
        name: String,
        body: Vec<Statement>,
        location: Location,
    },

    // Method definition: PRO/FUNCTION ClassName::MethodName
    MethodDefinition {
        class_name: String,
        method_name: String,
        is_function: bool,  // true for FUNCTION, false for PRO
        params: Vec<Parameter>,
        keywords: Vec<KeywordDecl>,
        body: Vec<Statement>,
        location: Location,
    },

    // Object destruction: OBJ_DESTROY, obj
    ObjectDestroy {
        objects: Vec<Expression>,
        location: Location,
    },
}
```

### 2. Core Type System (xdl-core)

#### Object Reference Type

```rust
pub enum XdlValue {
    // ... existing variants ...

    // Object reference (ID-based handle)
    Object(usize),
}
```

#### IDL Class Definition

```rust
pub struct ClassDef {
    pub name: String,
    pub fields: HashMap<String, FieldDef>,
    pub methods: HashMap<String, MethodDef>,
}

pub struct FieldDef {
    pub name: String,
    pub field_type: GdlType,
    pub default_value: XdlValue,
}

pub struct MethodDef {
    pub name: String,
    pub is_function: bool,
    pub params: Vec<Parameter>,
    pub keywords: Vec<KeywordDecl>,
    pub body: Vec<Statement>,
}
```

#### Object Instance

```rust
pub struct ObjectInstance {
    pub class_name: String,
    pub fields: HashMap<String, XdlValue>,
    pub id: usize,
}
```

### 3. Context Extensions (xdl-interpreter/src/context.rs)

```rust
pub struct Context {
    // ... existing fields ...

    /// Class definitions (class_name -> ClassDef)
    classes: HashMap<String, ClassDef>,

    /// Object instances (object_id -> ObjectInstance)
    objects: HashMap<usize, ObjectInstance>,

    /// Next object ID
    next_object_id: usize,
}

impl Context {
    /// Define a class
    pub fn define_class(&mut self, name: String, class_def: ClassDef);

    /// Get a class definition
    pub fn get_class(&self, name: &str) -> Option<&ClassDef>;

    /// Create an object instance
    pub fn create_object(&mut self, class_name: String, fields: HashMap<String, XdlValue>) -> usize;

    /// Get an object instance
    pub fn get_object(&self, id: usize) -> XdlResult<&ObjectInstance>;

    /// Get a mutable object instance
    pub fn get_object_mut(&mut self, id: usize) -> XdlResult<&mut ObjectInstance>;

    /// Destroy an object instance
    pub fn destroy_object(&mut self, id: usize) -> XdlResult<()>;
}
```

### 4. Evaluator Extensions (xdl-interpreter/src/evaluator.rs)

#### Object Creation (OBJ_NEW)

```rust
fn evaluate_obj_new(
    &mut self,
    class_name: &str,
    args: &[Expression],
    keywords: &[KeywordArg],
    context: &mut Context,
) -> XdlResult<XdlValue> {
    // 1. Check if class exists
    let class_def = context.get_class(class_name)?;

    // 2. Create zeroed instance
    let mut fields = HashMap::new();
    for (field_name, field_def) in &class_def.fields {
        fields.insert(field_name.clone(), field_def.default_value.clone());
    }

    // 3. Create object and get ID
    let obj_id = context.create_object(class_name.to_string(), fields);

    // 4. Call Init method if it exists
    if let Some(init_method) = class_def.methods.get("init") {
        // Evaluate arguments
        let arg_values: Vec<XdlValue> = args.iter()
            .map(|arg| self.evaluate(arg, context))
            .collect::<XdlResult<Vec<_>>>()?;

        // Call Init with SELF = obj_id
        let result = self.call_method(obj_id, "init", &arg_values, keywords, context)?;

        // If Init returns 0, destroy object and return NULL
        if let XdlValue::Long(0) = result {
            context.destroy_object(obj_id)?;
            return Ok(XdlValue::Object(0)); // NULL object reference
        }
    }

    // 5. Return object reference
    Ok(XdlValue::Object(obj_id))
}
```

#### Object Destruction (OBJ_DESTROY)

```rust
fn evaluate_obj_destroy(
    &mut self,
    objects: &[Expression],
    context: &mut Context,
) -> XdlResult<()> {
    for obj_expr in objects {
        let obj_val = self.evaluate(obj_expr, context)?;

        if let XdlValue::Object(obj_id) = obj_val {
            // Skip NULL references
            if obj_id == 0 {
                continue;
            }

            // Get object to find its class
            let obj = context.get_object(obj_id)?;
            let class_name = obj.class_name.clone();

            // Call Cleanup method if it exists
            let class_def = context.get_class(&class_name)?;
            if class_def.methods.contains_key("cleanup") {
                self.call_method(obj_id, "cleanup", &[], &[], context)?;
            }

            // Destroy the object
            context.destroy_object(obj_id)?;
        }
    }

    Ok(())
}
```

#### Method Call Dispatch

```rust
fn call_method(
    &mut self,
    obj_id: usize,
    method_name: &str,
    args: &[XdlValue],
    keywords: &[KeywordArg],
    context: &mut Context,
) -> XdlResult<XdlValue> {
    // Get object and class
    let obj = context.get_object(obj_id)?;
    let class_name = obj.class_name.clone();
    let class_def = context.get_class(&class_name)?;

    // Find method
    let method = class_def.methods.get(method_name)
        .ok_or_else(|| XdlError::RuntimeError(
            format!("Method {} not found in class {}", method_name, class_name)
        ))?;

    // Create method execution scope with SELF
    context.push_scope();
    context.set_variable("SELF".to_string(), XdlValue::Object(obj_id));

    // Bind parameters
    for (i, param) in method.params.iter().enumerate() {
        if i < args.len() {
            context.set_variable(param.name.clone(), args[i].clone());
        }
    }

    // Bind keywords
    // TODO: Handle keyword arguments

    // Execute method body
    let mut result = XdlValue::Undefined;
    for stmt in &method.body {
        match self.execute(stmt, context) {
            Ok(_) => {},
            Err(XdlError::Return(val)) => {
                result = val;
                break;
            }
            Err(e) => return Err(e),
        }
    }

    // Pop scope
    context.pop_scope()?;

    Ok(result)
}
```

#### SELF Keyword Handling

```rust
// In evaluate() method, handle SELF as a special variable
Expression::Variable { name, .. } => {
    if name.to_uppercase() == "SELF" {
        context.get_variable("SELF")
            .map(|v| v.clone())
    } else {
        context.get_variable(name)
            .map(|v| v.clone())
    }
}
```

#### Field Access on SELF

```rust
// In StructRef evaluation
Expression::StructRef { object, field, .. } => {
    let obj_val = self.evaluate(object, context)?;

    if let XdlValue::Object(obj_id) = obj_val {
        let obj = context.get_object(obj_id)?;
        obj.fields.get(field)
            .cloned()
            .ok_or_else(|| XdlError::RuntimeError(
                format!("Field {} not found in object", field)
            ))
    } else {
        Err(XdlError::TypeMismatch {
            expected: "object".to_string(),
            actual: format!("{:?}", obj_val.gdl_type()),
        })
    }
}
```

### 5. Parser Implementation

#### Lexer Tokens

```rust
pub enum Token {
    // ... existing tokens ...

    // :: for method definitions
    DoubleColon,
}
```

#### Parsing Class Definitions

```rust
fn parse_class_definition(&mut self) -> XdlResult<Statement> {
    // Expect PRO ClassName__define
    let name_with_define = self.parse_identifier()?;

    // Check for __define suffix
    if !name_with_define.ends_with("__define") {
        return Err(XdlError::ParseError {
            message: format!("Expected class definition with __define suffix, got {}", name_with_define),
            line: self.line,
            column: self.column,
        });
    }

    // Extract class name
    let class_name = name_with_define.trim_end_matches("__define").to_string();

    // Parse body until ENDPRO
    let body = self.parse_procedure_body()?;

    Ok(Statement::ClassDefinition {
        name: class_name,
        body,
        location: Location::unknown(),
    })
}
```

#### Parsing Method Definitions

```rust
fn parse_method_definition(&mut self, is_function: bool) -> XdlResult<Statement> {
    // Expect ClassName::MethodName
    let full_name = self.parse_identifier()?;

    // Split on ::
    let parts: Vec<&str> = full_name.split("::").collect();
    if parts.len() != 2 {
        return Err(XdlError::ParseError {
            message: format!("Expected ClassName::MethodName, got {}", full_name),
            line: self.line,
            column: self.column,
        });
    }

    let class_name = parts[0].to_string();
    let method_name = parts[1].to_string();

    // Parse parameters and keywords
    let (params, keywords) = self.parse_parameter_list()?;

    // Parse body
    let body = if is_function {
        self.parse_function_body()?
    } else {
        self.parse_procedure_body()?
    };

    Ok(Statement::MethodDefinition {
        class_name,
        method_name,
        is_function,
        params,
        keywords,
        body,
        location: Location::unknown(),
    })
}
```

#### Parsing OBJ_NEW

```rust
// In parse_function_call, check for OBJ_NEW
if name.to_uppercase() == "OBJ_NEW" {
    return Ok(Expression::ObjectNew {
        class_name: // first argument
        args: // remaining arguments
        keywords: // keywords
        location: Location::unknown(),
    });
}
```

## Implementation Phases

### Phase 1: Core Infrastructure ✓ (Partially Complete)

- [x] Method call syntax (`->`) - Already implemented
- [ ] Class definition AST nodes
- [ ] Method definition AST nodes
- [ ] Object type in XdlValue

### Phase 2: Parser Extensions

- [ ] Lexer: Add `::` token
- [ ] Parse `PRO ClassName__define`
- [ ] Parse `PRO/FUNCTION ClassName::MethodName`
- [ ] Parse `OBJ_NEW` expression
- [ ] Parse `OBJ_DESTROY` statement

### Phase 3: Context & Storage

- [ ] Class definition storage
- [ ] Object instance storage
- [ ] Object lifecycle management

### Phase 4: Evaluator Implementation

- [ ] Execute class definitions
- [ ] Execute method definitions
- [ ] Implement OBJ_NEW
- [ ] Implement OBJ_DESTROY
- [ ] Implement SELF keyword
- [ ] Implement field access on objects

### Phase 5: Testing & Documentation

- [ ] Create comprehensive test suite
- [ ] Test inheritance (if implemented)
- [ ] Update user documentation
- [ ] Create examples

## Example: Complete XDL Object Class

```xdl
; Define a simple Point class
PRO Point__define
    struct = {Point, $
              x: 0.0, $
              y: 0.0}
END

; Initialize method
FUNCTION Point::Init, x, y
    self.x = x
    self.y = y
    RETURN, 1  ; Success
END

; Cleanup method
PRO Point::Cleanup
    PRINT, 'Destroying point at (', self.x, ',', self.y, ')'
END

; Get distance from origin
FUNCTION Point::Distance
    RETURN, SQRT(self.x^2 + self.y^2)
END

; Move point
PRO Point::Move, dx, dy
    self.x = self.x + dx
    self.y = self.y + dy
END

; Usage example
p = OBJ_NEW('Point', 3.0, 4.0)
PRINT, 'Distance:', p->Distance()  ; Prints 5.0
p->Move, 1.0, 1.0
PRINT, 'New distance:', p->Distance()  ; Prints ~5.66
OBJ_DESTROY, p
```

## Notes

- NULL object references are represented as `XdlValue::Object(0)`
- SELF is always lowercase "self" in IDL, but we'll support both cases
- Init methods must return 1 (success) or 0 (failure)
- If Init returns 0, the object is destroyed and NULL is returned
- Cleanup methods are optional but recommended for resource management
- Method names are case-insensitive like all IDL identifiers

## Future Enhancements

- Inheritance with INHERITS keyword
- Property methods (GetProperty, SetProperty)
- Static methods
- Operator overloading
- Object arrays
- Pointer fields in objects
