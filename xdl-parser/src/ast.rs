//! XDL Abstract Syntax Tree definitions

use xdl_core::XdlValue;
/// Source location information
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub filename: Option<String>,
}

/// XDL Program (top-level AST node)
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub location: Location,
}

/// XDL Statements
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        target: Expression,
        value: Expression,
        location: Location,
    },
    Expression {
        expr: Expression,
        location: Location,
    },
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
        location: Location,
    },
    For {
        variable: String,
        start: Expression,
        end: Expression,
        step: Option<Expression>,
        body: Vec<Statement>,
        location: Location,
    },
    Foreach {
        variable: String,
        iterable: Expression,
        index_var: Option<String>,
        body: Vec<Statement>,
        location: Location,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
        location: Location,
    },
    Repeat {
        body: Vec<Statement>,
        condition: Expression,
        location: Location,
    },
    Break {
        location: Location,
    },
    Continue {
        location: Location,
    },
    Return {
        value: Option<Expression>,
        location: Location,
    },
    ProcedureCall {
        name: String,
        args: Vec<Expression>,
        keywords: Vec<Keyword>,
        location: Location,
    },
    Common {
        name: String,
        variables: Vec<String>,
        location: Location,
    },
    CompileOpt {
        options: Vec<String>,
        location: Location,
    },
    FunctionDef {
        name: String,
        params: Vec<Parameter>,
        keywords: Vec<KeywordDecl>,
        body: Vec<Statement>,
        location: Location,
    },
    ProcedureDef {
        name: String,
        params: Vec<Parameter>,
        keywords: Vec<KeywordDecl>,
        body: Vec<Statement>,
        location: Location,
    },
    Label {
        name: String,
        location: Location,
    },
    Goto {
        label: String,
        location: Location,
    },
}

/// XDL Expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal {
        value: XdlValue,
        location: Location,
    },
    Variable {
        name: String,
        location: Location,
    },
    SystemVariable {
        name: String,
        location: Location,
    },
    ArrayRef {
        array: Box<Expression>,
        indices: Vec<ArrayIndex>,
        location: Location,
    },
    StructRef {
        object: Box<Expression>,
        field: String,
        location: Location,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
        keywords: Vec<Keyword>,
        location: Location,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
        keywords: Vec<Keyword>,
        location: Location,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
        location: Location,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
        location: Location,
    },
    Ternary {
        condition: Box<Expression>,
        if_true: Box<Expression>,
        if_false: Box<Expression>,
        location: Location,
    },
    ArrayDef {
        elements: Vec<Expression>,
        location: Location,
    },
    StructDef {
        name: Option<String>,
        fields: Vec<StructField>,
        location: Location,
    },
    Pointer {
        expr: Box<Expression>,
        location: Location,
    },
    Deref {
        expr: Box<Expression>,
        location: Location,
    },
    PostIncrement {
        expr: Box<Expression>,
        location: Location,
    },
    PostDecrement {
        expr: Box<Expression>,
        location: Location,
    },
    PreIncrement {
        expr: Box<Expression>,
        location: Location,
    },
    PreDecrement {
        expr: Box<Expression>,
        location: Location,
    },
}

/// Array indexing expressions
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayIndex {
    Single(Box<Expression>),
    Range {
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
        step: Option<Box<Expression>>,
    },
    All, // *
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    MatrixMultiply,

    // Logical
    And,
    Or,
    Xor,

    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // String
    Concatenate,

    // Assignment
    Assign,
    PlusAssign,
    MinusAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    PowerAssign,
    AndAssign,
    OrAssign,
    XorAssign,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
    BitwiseNot,
}

/// Function/procedure parameters
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub by_reference: bool,
    pub optional: bool,
    pub location: Location,
}

/// Keyword declarations in function/procedure definitions
#[derive(Debug, Clone, PartialEq)]
pub struct KeywordDecl {
    pub name: String,
    pub by_reference: bool,
    pub location: Location,
}

/// Keyword arguments in function/procedure calls
#[derive(Debug, Clone, PartialEq)]
pub struct Keyword {
    pub name: String,
    pub value: Option<Expression>,
    pub location: Location,
}

/// Structure field definition
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub value: Expression,
    pub location: Location,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            filename: None,
        }
    }

    pub fn with_file(line: usize, column: usize, filename: String) -> Self {
        Self {
            line,
            column,
            filename: Some(filename),
        }
    }

    pub fn unknown() -> Self {
        Self {
            line: 0,
            column: 0,
            filename: None,
        }
    }
}

impl Statement {
    /// Get the location of this statement
    pub fn location(&self) -> &Location {
        match self {
            Statement::Assignment { location, .. } => location,
            Statement::Expression { location, .. } => location,
            Statement::If { location, .. } => location,
            Statement::For { location, .. } => location,
            Statement::Foreach { location, .. } => location,
            Statement::While { location, .. } => location,
            Statement::Repeat { location, .. } => location,
            Statement::Break { location } => location,
            Statement::Continue { location } => location,
            Statement::Return { location, .. } => location,
            Statement::ProcedureCall { location, .. } => location,
            Statement::Common { location, .. } => location,
            Statement::CompileOpt { location, .. } => location,
            Statement::FunctionDef { location, .. } => location,
            Statement::ProcedureDef { location, .. } => location,
            Statement::Label { location, .. } => location,
            Statement::Goto { location, .. } => location,
        }
    }
}

impl Expression {
    /// Get the location of this expression
    pub fn location(&self) -> &Location {
        match self {
            Expression::Literal { location, .. } => location,
            Expression::Variable { location, .. } => location,
            Expression::SystemVariable { location, .. } => location,
            Expression::ArrayRef { location, .. } => location,
            Expression::StructRef { location, .. } => location,
            Expression::MethodCall { location, .. } => location,
            Expression::FunctionCall { location, .. } => location,
            Expression::Binary { location, .. } => location,
            Expression::Unary { location, .. } => location,
            Expression::Ternary { location, .. } => location,
            Expression::ArrayDef { location, .. } => location,
            Expression::StructDef { location, .. } => location,
            Expression::Pointer { location, .. } => location,
            Expression::Deref { location, .. } => location,
            Expression::PostIncrement { location, .. } => location,
            Expression::PostDecrement { location, .. } => location,
            Expression::PreIncrement { location, .. } => location,
            Expression::PreDecrement { location, .. } => location,
        }
    }

    /// Check if this expression is a constant literal
    pub fn is_constant(&self) -> bool {
        matches!(self, Expression::Literal { .. })
    }

    /// Check if this expression is a simple variable reference
    pub fn is_variable(&self) -> bool {
        matches!(self, Expression::Variable { .. })
    }
}
