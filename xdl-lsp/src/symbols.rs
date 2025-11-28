//! Symbol table and document symbols provider

use std::collections::HashMap;
use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind};
use xdl_parser::ast::{Expression, Program, Statement};

use crate::document::DocumentState;

#[derive(Debug, Clone)]
pub struct BuiltinInfo {
    pub name: String,
    pub documentation: String,
    pub return_type: String,
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct SystemVarInfo {
    pub name: String,
    pub documentation: String,
    pub type_info: String,
}

pub struct SymbolTable {
    pub builtin_functions: HashMap<String, BuiltinInfo>,
    pub builtin_procedures: HashMap<String, BuiltinInfo>,
    pub system_variables: HashMap<String, SystemVarInfo>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            builtin_functions: HashMap::new(),
            builtin_procedures: HashMap::new(),
            system_variables: HashMap::new(),
        };
        table.initialize_builtins();
        table
    }

    fn initialize_builtins(&mut self) {
        // Math functions
        let math_funcs = vec![
            ("SIN", "sin(x) - Compute sine of angle x (radians)", "DOUBLE", "SIN(x)"),
            ("COS", "cos(x) - Compute cosine of angle x (radians)", "DOUBLE", "COS(x)"),
            ("TAN", "tan(x) - Compute tangent of angle x (radians)", "DOUBLE", "TAN(x)"),
            ("ASIN", "asin(x) - Compute arc sine", "DOUBLE", "ASIN(x)"),
            ("ACOS", "acos(x) - Compute arc cosine", "DOUBLE", "ACOS(x)"),
            ("ATAN", "atan(x) - Compute arc tangent", "DOUBLE", "ATAN(x)"),
            ("SINH", "sinh(x) - Compute hyperbolic sine", "DOUBLE", "SINH(x)"),
            ("COSH", "cosh(x) - Compute hyperbolic cosine", "DOUBLE", "COSH(x)"),
            ("TANH", "tanh(x) - Compute hyperbolic tangent", "DOUBLE", "TANH(x)"),
            ("SQRT", "sqrt(x) - Compute square root", "DOUBLE", "SQRT(x)"),
            ("EXP", "exp(x) - Compute e^x (exponential)", "DOUBLE", "EXP(x)"),
            ("ALOG", "alog(x) - Compute natural logarithm (ln)", "DOUBLE", "ALOG(x)"),
            ("ALOG10", "alog10(x) - Compute base-10 logarithm", "DOUBLE", "ALOG10(x)"),
            ("ABS", "abs(x) - Compute absolute value", "DOUBLE", "ABS(x)"),
            ("CEIL", "ceil(x) - Ceiling function", "LONG", "CEIL(x)"),
            ("FLOOR", "floor(x) - Floor function", "LONG", "FLOOR(x)"),
            ("ROUND", "round(x) - Round to nearest integer", "LONG", "ROUND(x)"),
            ("FIX", "fix(x) - Truncate to integer", "LONG", "FIX(x)"),
            ("FLOAT", "float(x) - Convert to float", "FLOAT", "FLOAT(x)"),
            ("DOUBLE", "double(x) - Convert to double", "DOUBLE", "DOUBLE(x)"),
            ("COMPLEX", "complex(real, imag) - Create complex number", "COMPLEX", "COMPLEX(real, imag)"),
        ];

        // Array functions
        let array_funcs = vec![
            ("FINDGEN", "findgen(n) - Generate float array [0, 1, ..., n-1]", "ARRAY", "FINDGEN(n)"),
            ("INDGEN", "indgen(n) - Generate integer array [0, 1, ..., n-1]", "ARRAY", "INDGEN(n)"),
            ("DINDGEN", "dindgen(n) - Generate double array [0, 1, ..., n-1]", "ARRAY", "DINDGEN(n)"),
            ("BINDGEN", "bindgen(n) - Generate byte array [0, 1, ..., n-1]", "ARRAY", "BINDGEN(n)"),
            ("FLTARR", "fltarr(dims...) - Create float array of zeros", "ARRAY", "FLTARR(d1, d2, ...)"),
            ("DBLARR", "dblarr(dims...) - Create double array of zeros", "ARRAY", "DBLARR(d1, d2, ...)"),
            ("INTARR", "intarr(dims...) - Create integer array of zeros", "ARRAY", "INTARR(d1, d2, ...)"),
            ("BYTARR", "bytarr(dims...) - Create byte array of zeros", "ARRAY", "BYTARR(d1, d2, ...)"),
            ("STRARR", "strarr(dims...) - Create string array", "ARRAY", "STRARR(d1, d2, ...)"),
            ("MAKE_ARRAY", "make_array(dims...) - Create array with specified type", "ARRAY", "MAKE_ARRAY(d1, d2, ..., TYPE=type)"),
            ("REPLICATE", "replicate(value, dims...) - Create array filled with value", "ARRAY", "REPLICATE(value, d1, d2, ...)"),
            ("WHERE", "where(condition) - Find indices where condition is true", "ARRAY", "WHERE(condition, [COUNT=count])"),
            ("N_ELEMENTS", "n_elements(arr) - Return number of elements", "LONG", "N_ELEMENTS(arr)"),
            ("SIZE", "size(arr) - Return array dimensions info", "ARRAY", "SIZE(arr, [/DIMENSIONS])"),
            ("REFORM", "reform(arr, dims...) - Reshape array", "ARRAY", "REFORM(arr, d1, d2, ...)"),
            ("TRANSPOSE", "transpose(arr) - Transpose 2D array", "ARRAY", "TRANSPOSE(arr)"),
            ("REVERSE", "reverse(arr) - Reverse array elements", "ARRAY", "REVERSE(arr, [dim])"),
            ("SHIFT", "shift(arr, s) - Circular shift array", "ARRAY", "SHIFT(arr, s1, s2, ...)"),
            ("ROTATE", "rotate(arr, dir) - Rotate 2D array", "ARRAY", "ROTATE(arr, direction)"),
            ("SORT", "sort(arr) - Return indices that sort array", "ARRAY", "SORT(arr)"),
            ("UNIQ", "uniq(arr) - Return indices of unique elements", "ARRAY", "UNIQ(arr, [/SORT])"),
        ];

        // Statistics functions
        let stat_funcs = vec![
            ("TOTAL", "total(arr) - Sum all elements", "DOUBLE", "TOTAL(arr, [dim])"),
            ("MEAN", "mean(arr) - Calculate arithmetic mean", "DOUBLE", "MEAN(arr)"),
            ("MEDIAN", "median(arr) - Calculate median value", "DOUBLE", "MEDIAN(arr)"),
            ("VARIANCE", "variance(arr) - Calculate variance", "DOUBLE", "VARIANCE(arr)"),
            ("STDDEV", "stddev(arr) - Calculate standard deviation", "DOUBLE", "STDDEV(arr)"),
            ("MIN", "min(arr) - Find minimum value", "DOUBLE", "MIN(arr, [MIN_SUBSCRIPT=idx])"),
            ("MAX", "max(arr) - Find maximum value", "DOUBLE", "MAX(arr, [MAX_SUBSCRIPT=idx])"),
            ("MOMENT", "moment(arr) - Calculate statistical moments", "ARRAY", "MOMENT(arr)"),
            ("CORRELATE", "correlate(x, y) - Compute correlation", "DOUBLE", "CORRELATE(x, y)"),
            ("HISTOGRAM", "histogram(arr) - Compute histogram", "ARRAY", "HISTOGRAM(arr, [BINSIZE=bs])"),
        ];

        // String functions
        let string_funcs = vec![
            ("STRLEN", "strlen(str) - Return string length", "LONG", "STRLEN(str)"),
            ("STRMID", "strmid(str, start, len) - Extract substring", "STRING", "STRMID(str, start, [len])"),
            ("STRPOS", "strpos(str, search) - Find substring position", "LONG", "STRPOS(str, search, [start])"),
            ("STRTRIM", "strtrim(str, flag) - Trim whitespace", "STRING", "STRTRIM(str, [flag])"),
            ("STRUPCASE", "strupcase(str) - Convert to uppercase", "STRING", "STRUPCASE(str)"),
            ("STRLOWCASE", "strlowcase(str) - Convert to lowercase", "STRING", "STRLOWCASE(str)"),
            ("STRING", "string(value) - Convert to string", "STRING", "STRING(value, [FORMAT=fmt])"),
            ("STRSPLIT", "strsplit(str, pattern) - Split string", "ARRAY", "STRSPLIT(str, pattern, [/EXTRACT])"),
            ("STRJOIN", "strjoin(arr, delim) - Join strings", "STRING", "STRJOIN(arr, [delimiter])"),
            ("STRCMP", "strcmp(s1, s2) - Compare strings", "LONG", "STRCMP(s1, s2, [/FOLD_CASE])"),
        ];

        // I/O functions
        let io_funcs = vec![
            ("READ_ASCII", "read_ascii(file) - Read ASCII data file", "STRUCT", "READ_ASCII(file)"),
            ("READ_CSV", "read_csv(file) - Read CSV file", "STRUCT", "READ_CSV(file)"),
            ("READ_BINARY", "read_binary(file) - Read binary file", "ARRAY", "READ_BINARY(file)"),
            ("READF", "readf(unit, vars...) - Formatted read", "VOID", "READF, unit, var1, var2, ..."),
            ("READU", "readu(unit, vars...) - Unformatted read", "VOID", "READU, unit, var1, var2, ..."),
            ("FILE_TEST", "file_test(path) - Test if file exists", "LONG", "FILE_TEST(path)"),
            ("FILE_INFO", "file_info(path) - Get file information", "STRUCT", "FILE_INFO(path)"),
            ("FILE_SEARCH", "file_search(pattern) - Search for files", "ARRAY", "FILE_SEARCH(pattern)"),
            ("FILE_LINES", "file_lines(file) - Count lines in file", "LONG", "FILE_LINES(file)"),
        ];

        // Add all functions
        for (name, doc, ret, sig) in math_funcs
            .into_iter()
            .chain(array_funcs)
            .chain(stat_funcs)
            .chain(string_funcs)
            .chain(io_funcs)
        {
            self.builtin_functions.insert(
                name.to_string(),
                BuiltinInfo {
                    name: name.to_string(),
                    documentation: doc.to_string(),
                    return_type: ret.to_string(),
                    signature: sig.to_string(),
                },
            );
        }

        // Procedures
        let procedures = vec![
            ("PRINT", "print, args... - Print values to standard output", "PRINT, expr1, expr2, ..."),
            ("PRINTF", "printf, unit, args... - Formatted print to file", "PRINTF, unit, format, expr1, ..."),
            ("WRITEF", "writef, unit, args... - Formatted write to file", "WRITEF, unit, format, expr1, ..."),
            ("WRITEU", "writeu, unit, args... - Unformatted write to file", "WRITEU, unit, expr1, ..."),
            ("OPENR", "openr, unit, file - Open file for reading", "OPENR, unit, filename"),
            ("OPENW", "openw, unit, file - Open file for writing", "OPENW, unit, filename"),
            ("OPENU", "openu, unit, file - Open file for update", "OPENU, unit, filename"),
            ("CLOSE", "close, unit - Close file", "CLOSE, unit"),
            ("FREE_LUN", "free_lun, unit - Free logical unit number", "FREE_LUN, unit"),
            ("PLOT", "plot, x, y - Create 2D line plot", "PLOT, [x,] y, [keywords]"),
            ("OPLOT", "oplot, x, y - Overplot on existing plot", "OPLOT, [x,] y, [keywords]"),
            ("CONTOUR", "contour, z, x, y - Create contour plot", "CONTOUR, z, [x, y], [keywords]"),
            ("SURFACE", "surface, z - Create 3D surface plot", "SURFACE, z, [x, y], [keywords]"),
            ("SHADE_SURF", "shade_surf, z - Create shaded surface", "SHADE_SURF, z, [x, y], [keywords]"),
            ("TV", "tv, image - Display image", "TV, image, [x, y]"),
            ("TVSCL", "tvscl, image - Display scaled image", "TVSCL, image, [x, y]"),
            ("WINDOW", "window, id - Create graphics window", "WINDOW, [window_id], [keywords]"),
            ("WSET", "wset, id - Set current window", "WSET, window_id"),
            ("WDELETE", "wdelete, id - Delete window", "WDELETE, [window_id]"),
            ("DEVICE", "device, keywords - Control graphics device", "DEVICE, [keywords]"),
            ("ERASE", "erase - Clear graphics window", "ERASE, [color]"),
            ("HELP", "help, var - Display variable information", "HELP, [variable]"),
            ("STOP", "stop - Stop program execution", "STOP"),
            ("MESSAGE", "message, text - Display message", "MESSAGE, text"),
            ("ON_ERROR", "on_error, action - Set error handling", "ON_ERROR, action"),
            ("CATCH", "catch, variable - Catch errors", "CATCH, error_status"),
            ("PTR_NEW", "ptr_new(value) - Create new pointer", "PTR_NEW([value])"),
            ("PTR_FREE", "ptr_free, ptr - Free pointer", "PTR_FREE, ptr"),
            ("OBJ_NEW", "obj_new(class) - Create new object", "OBJ_NEW(class, [args])"),
            ("OBJ_DESTROY", "obj_destroy, obj - Destroy object", "OBJ_DESTROY, obj"),
        ];

        for (name, doc, sig) in procedures {
            self.builtin_procedures.insert(
                name.to_string(),
                BuiltinInfo {
                    name: name.to_string(),
                    documentation: doc.to_string(),
                    return_type: String::new(),
                    signature: sig.to_string(),
                },
            );
        }

        // System variables
        let sys_vars = vec![
            ("PI", "3.14159265358979... - Pi constant", "DOUBLE"),
            ("DTOR", "Degrees to radians: PI/180", "DOUBLE"),
            ("RTOD", "Radians to degrees: 180/PI", "DOUBLE"),
            ("E", "2.71828182845905... - Euler's number", "DOUBLE"),
            ("NULL", "Null pointer/undefined value", "POINTER"),
            ("VALUES", "Special floating point values structure", "STRUCT"),
            ("D", "Current graphics device info", "STRUCT"),
            ("P", "Current plot parameters", "STRUCT"),
            ("X", "X-axis parameters", "STRUCT"),
            ("Y", "Y-axis parameters", "STRUCT"),
            ("Z", "Z-axis parameters", "STRUCT"),
            ("MAP", "Map projection parameters", "STRUCT"),
            ("ERROR_STATE", "Current error state", "STRUCT"),
            ("EXCEPT", "Floating-point exception status", "STRUCT"),
            ("CPU", "CPU information structure", "STRUCT"),
            ("PATH", "Current search path", "STRING"),
            ("DIR", "Current directory", "STRING"),
        ];

        for (name, doc, type_str) in sys_vars {
            self.system_variables.insert(
                name.to_string(),
                SystemVarInfo {
                    name: name.to_string(),
                    documentation: doc.to_string(),
                    type_info: type_str.to_string(),
                },
            );
        }
    }

    pub fn get_function(&self, name: &str) -> Option<&BuiltinInfo> {
        self.builtin_functions.get(&name.to_uppercase())
    }

    pub fn get_procedure(&self, name: &str) -> Option<&BuiltinInfo> {
        self.builtin_procedures.get(&name.to_uppercase())
    }

    pub fn get_system_variable(&self, name: &str) -> Option<&SystemVarInfo> {
        // Remove leading ! if present
        let name = name.trim_start_matches('!');
        self.system_variables.get(&name.to_uppercase())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_document_symbols(doc: &DocumentState) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    if let Some(ref ast) = doc.ast {
        for statement in &ast.statements {
            if let Some(symbol) = statement_to_symbol(statement) {
                symbols.push(symbol);
            }
        }
    }

    symbols
}

fn statement_to_symbol(stmt: &Statement) -> Option<DocumentSymbol> {
    match stmt {
        Statement::FunctionDef {
            name,
            params,
            location,
            ..
        } => {
            let range = location_to_range(location);
            let detail = format!(
                "({})",
                params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            #[allow(deprecated)]
            Some(DocumentSymbol {
                name: name.clone(),
                detail: Some(detail),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            })
        }
        Statement::ProcedureDef {
            name,
            params,
            location,
            ..
        } => {
            let range = location_to_range(location);
            let detail = format!(
                "({})",
                params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            #[allow(deprecated)]
            Some(DocumentSymbol {
                name: name.clone(),
                detail: Some(detail),
                kind: SymbolKind::METHOD,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            })
        }
        Statement::Assignment { target, location, .. } => {
            if let Expression::Variable { name, .. } = target {
                let range = location_to_range(location);
                #[allow(deprecated)]
                Some(DocumentSymbol {
                    name: name.clone(),
                    detail: None,
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn location_to_range(location: &xdl_parser::ast::Location) -> Range {
    Range {
        start: Position {
            line: location.line.saturating_sub(1) as u32,
            character: location.column as u32,
        },
        end: Position {
            line: location.line.saturating_sub(1) as u32,
            character: (location.column + 10) as u32,
        },
    }
}
