//! Execution context and variable management

use std::collections::HashMap;
use xdl_core::{XdlError, XdlResult, XdlValue};
use xdl_parser::{KeywordDecl, Parameter};

/// Function definition stored in context
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub params: Vec<Parameter>,
    pub keywords: Vec<KeywordDecl>,
    pub body: Vec<xdl_parser::Statement>,
}

/// Procedure definition stored in context
#[derive(Debug, Clone)]
pub struct ProcedureDef {
    pub params: Vec<Parameter>,
    pub keywords: Vec<KeywordDecl>,
    pub body: Vec<xdl_parser::Statement>,
}

/// Method definition (function or procedure belonging to a class)
#[derive(Debug, Clone)]
pub struct MethodDef {
    pub is_function: bool, // true = function, false = procedure
    pub params: Vec<Parameter>,
    pub keywords: Vec<KeywordDecl>,
    pub body: Vec<xdl_parser::Statement>,
}

/// Property definition for getter/setter support
#[derive(Debug, Clone)]
pub struct PropertyDef {
    pub name: String,
    pub getter: Option<String>,  // Method name for GET
    pub setter: Option<String>,  // Method name for SET
    pub init_value: Option<XdlValue>,
}

/// Class definition stored in context
#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub parent: Option<String>,  // Parent class name for inheritance
    pub fields: HashMap<String, XdlValue>, // Default field values from __define
    pub methods: HashMap<String, MethodDef>, // Method name -> definition (case-insensitive)
    pub class_methods: HashMap<String, MethodDef>, // Static/class methods
    pub properties: HashMap<String, PropertyDef>, // Property definitions with getters/setters
}

impl ClassDef {
    pub fn new(name: String) -> Self {
        Self {
            name,
            parent: None,
            fields: HashMap::new(),
            methods: HashMap::new(),
            class_methods: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    /// Create a class with a parent (for inheritance)
    pub fn with_parent(name: String, parent: String) -> Self {
        Self {
            name,
            parent: Some(parent),
            fields: HashMap::new(),
            methods: HashMap::new(),
            class_methods: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    /// Add a method to this class
    pub fn add_method(&mut self, name: String, method: MethodDef) {
        self.methods.insert(name.to_uppercase(), method);
    }

    /// Add a class (static) method
    pub fn add_class_method(&mut self, name: String, method: MethodDef) {
        self.class_methods.insert(name.to_uppercase(), method);
    }

    /// Get a method by name (case-insensitive) - only this class, no inheritance
    pub fn get_method(&self, name: &str) -> Option<&MethodDef> {
        self.methods.get(&name.to_uppercase())
    }

    /// Get a class (static) method by name
    pub fn get_class_method(&self, name: &str) -> Option<&MethodDef> {
        self.class_methods.get(&name.to_uppercase())
    }

    /// Set default field values from structure definition
    pub fn set_fields(&mut self, fields: HashMap<String, XdlValue>) {
        self.fields = fields;
    }

    /// Set parent class (for inheritance)
    pub fn set_parent(&mut self, parent: String) {
        self.parent = Some(parent);
    }

    /// Add a property definition
    pub fn add_property(&mut self, prop: PropertyDef) {
        self.properties.insert(prop.name.to_uppercase(), prop);
    }

    /// Get a property definition
    pub fn get_property(&self, name: &str) -> Option<&PropertyDef> {
        self.properties.get(&name.to_uppercase())
    }

    /// Check if this class has a parent
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    /// Get the parent class name
    pub fn parent_name(&self) -> Option<&str> {
        self.parent.as_deref()
    }
}

/// Object instance (runtime instance of a class)
#[derive(Debug, Clone)]
pub struct ObjectInstance {
    pub class_name: String,
    pub id: usize,
    pub fields: HashMap<String, XdlValue>, // Instance field values (case-insensitive)
}

impl ObjectInstance {
    pub fn new(class_name: String, id: usize, default_fields: &HashMap<String, XdlValue>) -> Self {
        Self {
            class_name,
            id,
            fields: default_fields.clone(),
        }
    }

    /// Get a field value (case-insensitive)
    pub fn get_field(&self, name: &str) -> Option<&XdlValue> {
        self.fields.get(&name.to_uppercase())
    }

    /// Set a field value (case-insensitive)
    pub fn set_field(&mut self, name: String, value: XdlValue) {
        self.fields.insert(name.to_uppercase(), value);
    }
}

/// Variable scope for nested scope management
#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, XdlValue>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: String, value: XdlValue) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&XdlValue> {
        self.variables.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
}

/// Execution context managing variables, functions, and scope
pub struct Context {
    /// Stack of variable scopes (innermost scope is last)
    scopes: Vec<Scope>,
    /// Global functions
    functions: HashMap<String, FunctionDef>,
    /// Global procedures
    procedures: HashMap<String, ProcedureDef>,
    /// System variables (!PI, !E, etc.)
    system_variables: HashMap<String, XdlValue>,
    /// DataFrame storage (ID -> DataFrame)
    dataframes: HashMap<usize, xdl_dataframe::DataFrame>,
    /// Next DataFrame ID
    next_dataframe_id: usize,
    /// Class definitions (case-insensitive class name -> ClassDef)
    classes: HashMap<String, ClassDef>,
    /// Object instances (ID -> ObjectInstance)
    objects: HashMap<usize, ObjectInstance>,
    /// Next object ID (0 is reserved for NULL)
    next_object_id: usize,
    /// Current SELF object ID (for method execution)
    current_self: Option<usize>,
}

impl Context {
    pub fn new() -> Self {
        let mut context = Self {
            scopes: vec![Scope::new()], // Start with global scope
            functions: HashMap::new(),
            procedures: HashMap::new(),
            system_variables: HashMap::new(),
            dataframes: HashMap::new(),
            next_dataframe_id: 0,
            classes: HashMap::new(),
            objects: HashMap::new(),
            next_object_id: 1,  // 0 is reserved for NULL
            current_self: None, // No SELF by default
        };

        // Initialize system variables
        context.init_system_variables();
        context
    }

    /// Initialize common system variables
    fn init_system_variables(&mut self) {
        use std::f64::consts;
        self.system_variables
            .insert("PI".to_string(), XdlValue::Double(consts::PI));
        self.system_variables
            .insert("E".to_string(), XdlValue::Double(consts::E));
        self.system_variables
            .insert("DTOR".to_string(), XdlValue::Double(consts::PI / 180.0)); // Degrees to radians
        self.system_variables
            .insert("RTOD".to_string(), XdlValue::Double(180.0 / consts::PI)); // Radians to degrees
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pop the current scope from the stack
    pub fn pop_scope(&mut self) -> XdlResult<()> {
        if self.scopes.len() <= 1 {
            return Err(XdlError::RuntimeError(
                "Cannot pop global scope".to_string(),
            ));
        }
        self.scopes.pop();
        Ok(())
    }

    /// Set a variable in the current scope
    pub fn set_variable(&mut self, name: String, value: XdlValue) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.set(name, value);
        }
    }

    /// Get a variable, searching from innermost to outermost scope
    pub fn get_variable(&self, name: &str) -> XdlResult<&XdlValue> {
        // Search scopes from innermost to outermost
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value);
            }
        }

        Err(XdlError::VariableNotFound(name.to_string()))
    }

    /// Get a system variable
    pub fn get_system_variable(&self, name: &str) -> XdlResult<&XdlValue> {
        self.system_variables
            .get(name)
            .ok_or_else(|| XdlError::VariableNotFound(format!("!{}", name)))
    }

    /// Set a system variable (IDL/GDL style variables starting with !)
    pub fn set_system_variable(&mut self, name: String, value: XdlValue) {
        self.system_variables.insert(name, value);
    }

    /// Define a function
    pub fn define_function(&mut self, name: String, func: FunctionDef) {
        self.functions.insert(name, func);
    }

    /// Get a function definition
    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.get(name)
    }

    /// Define a procedure
    pub fn define_procedure(&mut self, name: String, proc: ProcedureDef) {
        self.procedures.insert(name, proc);
    }

    /// Get a procedure definition
    pub fn get_procedure(&self, name: &str) -> Option<&ProcedureDef> {
        self.procedures.get(name)
    }

    /// Check if a variable exists in any scope
    pub fn has_variable(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }

    /// Get current scope depth
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }

    /// Get all variables from all scopes (for inspection/debugging)
    pub fn get_all_variables(&self) -> HashMap<String, &XdlValue> {
        let mut all_vars = HashMap::new();
        // Iterate from global to local scope, so local variables override globals
        for scope in &self.scopes {
            for (name, value) in &scope.variables {
                all_vars.insert(name.clone(), value);
            }
        }
        all_vars
    }

    /// Store a DataFrame and return its ID
    pub fn store_dataframe(&mut self, df: xdl_dataframe::DataFrame) -> usize {
        let id = self.next_dataframe_id;
        self.next_dataframe_id += 1;
        self.dataframes.insert(id, df);
        id
    }

    /// Get a reference to a DataFrame by ID
    pub fn get_dataframe(&self, id: usize) -> XdlResult<&xdl_dataframe::DataFrame> {
        self.dataframes
            .get(&id)
            .ok_or_else(|| XdlError::RuntimeError(format!("DataFrame {} not found", id)))
    }

    /// Get a mutable reference to a DataFrame by ID
    pub fn get_dataframe_mut(&mut self, id: usize) -> XdlResult<&mut xdl_dataframe::DataFrame> {
        self.dataframes
            .get_mut(&id)
            .ok_or_else(|| XdlError::RuntimeError(format!("DataFrame {} not found", id)))
    }

    /// Remove a DataFrame from storage
    pub fn remove_dataframe(&mut self, id: usize) -> XdlResult<xdl_dataframe::DataFrame> {
        self.dataframes
            .remove(&id)
            .ok_or_else(|| XdlError::RuntimeError(format!("DataFrame {} not found", id)))
    }

    /// Define a class (case-insensitive)
    pub fn define_class(&mut self, name: String, class: ClassDef) {
        self.classes.insert(name.to_uppercase(), class);
    }

    /// Get a class definition (case-insensitive)
    pub fn get_class(&self, name: &str) -> XdlResult<&ClassDef> {
        self.classes
            .get(&name.to_uppercase())
            .ok_or_else(|| XdlError::RuntimeError(format!("Class '{}' not defined", name)))
    }

    /// Get a mutable reference to a class definition (case-insensitive)
    pub fn get_class_mut(&mut self, name: &str) -> XdlResult<&mut ClassDef> {
        self.classes
            .get_mut(&name.to_uppercase())
            .ok_or_else(|| XdlError::RuntimeError(format!("Class '{}' not defined", name)))
    }

    /// Store an object instance and return its ID
    pub fn store_object(&mut self, obj: ObjectInstance) -> usize {
        let id = obj.id;
        self.objects.insert(id, obj);
        id
    }

    /// Create a new object instance (allocates ID and stores it)
    pub fn create_object(
        &mut self,
        class_name: String,
        default_fields: &HashMap<String, XdlValue>,
    ) -> usize {
        let id = self.next_object_id;
        self.next_object_id += 1;
        let obj = ObjectInstance::new(class_name, id, default_fields);
        self.objects.insert(id, obj);
        id
    }

    /// Get a reference to an object instance by ID
    pub fn get_object(&self, id: usize) -> XdlResult<&ObjectInstance> {
        if id == 0 {
            return Err(XdlError::RuntimeError(
                "Cannot access NULL object".to_string(),
            ));
        }
        self.objects
            .get(&id)
            .ok_or_else(|| XdlError::RuntimeError(format!("Object {} not found", id)))
    }

    /// Get a mutable reference to an object instance by ID
    pub fn get_object_mut(&mut self, id: usize) -> XdlResult<&mut ObjectInstance> {
        if id == 0 {
            return Err(XdlError::RuntimeError(
                "Cannot access NULL object".to_string(),
            ));
        }
        self.objects
            .get_mut(&id)
            .ok_or_else(|| XdlError::RuntimeError(format!("Object {} not found", id)))
    }

    /// Remove an object instance from storage (for OBJ_DESTROY)
    pub fn remove_object(&mut self, id: usize) -> XdlResult<ObjectInstance> {
        if id == 0 {
            return Err(XdlError::RuntimeError(
                "Cannot destroy NULL object".to_string(),
            ));
        }
        self.objects
            .remove(&id)
            .ok_or_else(|| XdlError::RuntimeError(format!("Object {} not found", id)))
    }

    /// Set the current SELF object (for method execution)
    pub fn set_self(&mut self, object_id: usize) {
        self.current_self = Some(object_id);
    }

    /// Clear the current SELF object (after method execution)
    pub fn clear_self(&mut self) {
        self.current_self = None;
    }

    /// Get the current SELF object value
    pub fn get_self(&self) -> XdlResult<XdlValue> {
        match self.current_self {
            Some(id) => Ok(XdlValue::Object(id)),
            None => Err(XdlError::RuntimeError(
                "SELF is not defined (not in method context)".to_string(),
            )),
        }
    }

    /// Get the current SELF object ID (for internal use)
    pub fn get_self_id(&self) -> Option<usize> {
        self.current_self
    }

    /// Resolve a method on a class, following the inheritance chain
    /// Returns (class_name, method_def) for the class that defines the method
    pub fn resolve_method(&self, class_name: &str, method_name: &str) -> XdlResult<(String, MethodDef)> {
        let mut current_class = class_name.to_uppercase();
        let mut visited = std::collections::HashSet::new();

        loop {
            // Prevent infinite loops from circular inheritance
            if visited.contains(&current_class) {
                return Err(XdlError::RuntimeError(format!(
                    "Circular inheritance detected while looking for method '{}' in class '{}'",
                    method_name, class_name
                )));
            }
            visited.insert(current_class.clone());

            // Get the class definition
            let class = self.classes.get(&current_class)
                .ok_or_else(|| XdlError::RuntimeError(format!("Class '{}' not defined", current_class)))?;

            // Check if this class has the method
            if let Some(method) = class.get_method(method_name) {
                return Ok((current_class, method.clone()));
            }

            // Check parent class if exists
            match &class.parent {
                Some(parent) => {
                    current_class = parent.to_uppercase();
                }
                None => {
                    // No parent and method not found
                    return Err(XdlError::RuntimeError(format!(
                        "Method '{}' not found in class '{}' or its parent classes",
                        method_name, class_name
                    )));
                }
            }
        }
    }

    /// Resolve a class method (static method), following the inheritance chain
    pub fn resolve_class_method(&self, class_name: &str, method_name: &str) -> XdlResult<(String, MethodDef)> {
        let mut current_class = class_name.to_uppercase();
        let mut visited = std::collections::HashSet::new();

        loop {
            if visited.contains(&current_class) {
                return Err(XdlError::RuntimeError(format!(
                    "Circular inheritance detected while looking for class method '{}' in class '{}'",
                    method_name, class_name
                )));
            }
            visited.insert(current_class.clone());

            let class = self.classes.get(&current_class)
                .ok_or_else(|| XdlError::RuntimeError(format!("Class '{}' not defined", current_class)))?;

            if let Some(method) = class.get_class_method(method_name) {
                return Ok((current_class, method.clone()));
            }

            match &class.parent {
                Some(parent) => {
                    current_class = parent.to_uppercase();
                }
                None => {
                    return Err(XdlError::RuntimeError(format!(
                        "Class method '{}' not found in class '{}' or its parent classes",
                        method_name, class_name
                    )));
                }
            }
        }
    }

    /// Check if a class inherits from another class (directly or indirectly)
    pub fn class_isa(&self, class_name: &str, target_class: &str) -> XdlResult<bool> {
        let mut current_class = class_name.to_uppercase();
        let target = target_class.to_uppercase();
        let mut visited = std::collections::HashSet::new();

        loop {
            if current_class == target {
                return Ok(true);
            }

            if visited.contains(&current_class) {
                return Ok(false); // Circular, target not in chain
            }
            visited.insert(current_class.clone());

            let class = match self.classes.get(&current_class) {
                Some(c) => c,
                None => return Ok(false),
            };

            match &class.parent {
                Some(parent) => {
                    current_class = parent.to_uppercase();
                }
                None => {
                    return Ok(false);
                }
            }
        }
    }

    /// Get the inheritance chain for a class (from most derived to base)
    pub fn get_class_hierarchy(&self, class_name: &str) -> XdlResult<Vec<String>> {
        let mut chain = Vec::new();
        let mut current_class = class_name.to_uppercase();
        let mut visited = std::collections::HashSet::new();

        loop {
            if visited.contains(&current_class) {
                break; // Prevent infinite loop
            }
            visited.insert(current_class.clone());
            chain.push(current_class.clone());

            let class = match self.classes.get(&current_class) {
                Some(c) => c,
                None => break,
            };

            match &class.parent {
                Some(parent) => {
                    current_class = parent.to_uppercase();
                }
                None => break,
            }
        }

        Ok(chain)
    }

    /// Resolve a property definition, following the inheritance chain
    pub fn resolve_property(&self, class_name: &str, prop_name: &str) -> XdlResult<(String, PropertyDef)> {
        let mut current_class = class_name.to_uppercase();
        let mut visited = std::collections::HashSet::new();

        loop {
            if visited.contains(&current_class) {
                return Err(XdlError::RuntimeError(format!(
                    "Circular inheritance detected while looking for property '{}' in class '{}'",
                    prop_name, class_name
                )));
            }
            visited.insert(current_class.clone());

            let class = self.classes.get(&current_class)
                .ok_or_else(|| XdlError::RuntimeError(format!("Class '{}' not defined", current_class)))?;

            if let Some(prop) = class.get_property(prop_name) {
                return Ok((current_class, prop.clone()));
            }

            match &class.parent {
                Some(parent) => {
                    current_class = parent.to_uppercase();
                }
                None => {
                    return Err(XdlError::RuntimeError(format!(
                        "Property '{}' not found in class '{}' or its parent classes",
                        prop_name, class_name
                    )));
                }
            }
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_scoping() {
        let mut ctx = Context::new();

        // Set variable in global scope
        ctx.set_variable("x".to_string(), XdlValue::Long(42));
        assert_eq!(ctx.get_variable("x").unwrap(), &XdlValue::Long(42));

        // Push new scope and set same variable
        ctx.push_scope();
        ctx.set_variable("x".to_string(), XdlValue::Long(100));
        assert_eq!(ctx.get_variable("x").unwrap(), &XdlValue::Long(100));

        // Pop scope, should see original value
        ctx.pop_scope().unwrap();
        assert_eq!(ctx.get_variable("x").unwrap(), &XdlValue::Long(42));
    }

    #[test]
    fn test_system_variables() {
        let ctx = Context::new();
        let pi = ctx.get_system_variable("PI").unwrap();
        match pi {
            XdlValue::Double(value) => assert!((value - std::f64::consts::PI).abs() < 1e-10),
            _ => panic!("PI should be a Double"),
        }
    }

    #[test]
    fn test_variable_not_found() {
        let ctx = Context::new();
        assert!(matches!(
            ctx.get_variable("nonexistent"),
            Err(XdlError::VariableNotFound(_))
        ));
    }
}
