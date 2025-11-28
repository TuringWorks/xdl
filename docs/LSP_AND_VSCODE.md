# XDL Language Server and VS Code Extension

**Version**: 1.0
**Date**: November 2025
**Status**: Complete ✅

---

## Overview

XDL includes a full-featured Language Server Protocol (LSP) implementation and VS Code extension, providing modern IDE features for XDL development.

---

## Features

### Language Server (xdl-lsp)

The XDL Language Server provides:

| Feature | Description |
|---------|-------------|
| **Document Sync** | Full text synchronization with incremental updates |
| **Diagnostics** | Real-time syntax error reporting |
| **Completion** | Auto-completion for keywords, functions, procedures, system variables |
| **Hover** | Documentation on hover for 100+ built-in functions |
| **Go to Definition** | Jump to function/procedure/variable definitions |
| **Find References** | Find all uses of a symbol |
| **Document Symbols** | Outline view of functions, procedures, and variables |
| **Semantic Tokens** | Enhanced syntax highlighting beyond TextMate grammar |

### VS Code Extension (vscode-xdl)

The extension provides:

- **Syntax Highlighting**: Full TextMate grammar for XDL syntax
- **Language Configuration**: Brackets, comments, folding, indentation
- **File Associations**: `.xdl` and `.pro` file extensions
- **Commands**: Restart server, run file
- **LSP Integration**: All language server features

---

## Installation

### Building the Language Server

```bash
# Build in release mode
cargo build -p xdl-lsp --release

# Binary location
./target/release/xdl-lsp.exe   # Windows
./target/release/xdl-lsp       # Linux/macOS
```

### Installing the VS Code Extension

```bash
# Navigate to extension directory
cd vscode-xdl

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Package extension
npx vsce package

# Install the .vsix file
code --install-extension xdl-language-0.1.0.vsix
```

### Configuration

The extension can be configured via VS Code settings:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `xdl.lsp.enabled` | boolean | `true` | Enable/disable the language server |
| `xdl.lsp.path` | string | `""` | Path to xdl-lsp executable (leave empty for bundled/PATH) |
| `xdl.trace.server` | string | `"off"` | Trace level: `off`, `messages`, `verbose` |

---

## LSP Server Architecture

### Components

```
xdl-lsp/
├── src/
│   ├── main.rs           # Entry point with tracing setup
│   ├── server.rs         # Tower-lsp LanguageServer implementation
│   ├── document.rs       # Document state with ropey text handling
│   ├── diagnostics.rs    # Parse error to LSP diagnostic conversion
│   ├── symbols.rs        # Symbol table with 100+ builtins
│   ├── completion.rs     # Completion provider
│   ├── hover.rs          # Hover information provider
│   ├── goto.rs           # Go-to-definition and references
│   ├── semantic_tokens.rs # Semantic token provider
│   └── utils.rs          # Position/offset utilities
```

### Dependencies

- **tower-lsp**: LSP protocol implementation
- **tokio**: Async runtime
- **ropey**: Efficient text rope for document handling
- **dashmap**: Concurrent hash map for document storage
- **xdl-parser**: XDL parsing for AST analysis

---

## VS Code Extension Architecture

### Components

```
vscode-xdl/
├── src/
│   └── extension.ts      # LSP client and command handlers
├── syntaxes/
│   └── xdl.tmLanguage.json  # TextMate grammar
├── package.json          # Extension manifest
├── language-configuration.json  # Language settings
└── tsconfig.json         # TypeScript configuration
```

### TextMate Grammar Scopes

| Scope | Elements |
|-------|----------|
| `comment.line.semicolon.xdl` | `;` line comments |
| `string.quoted.single.xdl` | Single-quoted strings |
| `string.quoted.double.xdl` | Double-quoted strings |
| `constant.numeric.*` | Numbers (integer, float, hex, octal, binary) |
| `keyword.control.xdl` | Control flow (IF, FOR, WHILE, etc.) |
| `keyword.other.xdl` | Declarations (FUNCTION, PRO, etc.) |
| `keyword.operator.*` | Operators (AND, OR, EQ, +, -, etc.) |
| `variable.language.system.xdl` | System variables (!PI, !DPI, etc.) |
| `entity.name.function.xdl` | Function/procedure names |
| `support.function.builtin.xdl` | Built-in functions (100+) |
| `support.function.procedure.xdl` | Built-in procedures (50+) |

---

## Supported Built-in Functions

The LSP provides hover documentation and completion for:

### Mathematical Functions
`SIN`, `COS`, `TAN`, `ASIN`, `ACOS`, `ATAN`, `SINH`, `COSH`, `TANH`, `SQRT`, `EXP`, `ALOG`, `ALOG10`, `ABS`, `CEIL`, `FLOOR`, `ROUND`, `FIX`, `FLOAT`, `DOUBLE`, `COMPLEX`, `IMAGINARY`, `CONJ`

### Array Functions
`FINDGEN`, `INDGEN`, `DINDGEN`, `BINDGEN`, `LINDGEN`, `FLTARR`, `DBLARR`, `INTARR`, `BYTARR`, `LONARR`, `STRARR`, `COMPLEXARR`, `MAKE_ARRAY`, `REPLICATE`, `WHERE`, `N_ELEMENTS`, `SIZE`, `REFORM`, `TRANSPOSE`, `REVERSE`, `SHIFT`, `ROTATE`, `SORT`, `UNIQ`

### Statistics
`TOTAL`, `MEAN`, `MEDIAN`, `VARIANCE`, `STDDEV`, `MIN`, `MAX`, `MOMENT`, `CORRELATE`, `HISTOGRAM`

### String Functions
`STRLEN`, `STRMID`, `STRPOS`, `STRTRIM`, `STRUPCASE`, `STRLOWCASE`, `STRING`, `STRSPLIT`, `STRJOIN`, `STRCMP`, `BYTE`

### I/O Functions
`PRINT`, `PRINTF`, `WRITEF`, `WRITEU`, `READF`, `READU`, `OPENR`, `OPENW`, `OPENU`, `CLOSE`, `FREE_LUN`, `GET_LUN`, `POINT_LUN`, `READ_ASCII`, `READ_CSV`, `READ_BINARY`, `DIALOG_PICKFILE`

### System Functions
`SYSTIME`, `FILEPATH`, `FILE_TEST`, `FILE_INFO`, `FILE_SEARCH`, `FILE_LINES`, `FILE_BASENAME`, `FILE_DIRNAME`, `GETENV`, `SETENV`, `SPAWN`

### Graphics Procedures
`PLOT`, `OPLOT`, `PLOTS`, `CONTOUR`, `SURFACE`, `SHADE_SURF`, `TV`, `TVSCL`, `WINDOW`, `WSET`, `WSHOW`, `WDELETE`, `DEVICE`, `ERASE`, `XYOUTS`, `AXIS`, `POLYFILL`, `CURSOR`

---

## System Variables

The LSP recognizes XDL system variables (prefixed with `!`):

| Variable | Description |
|----------|-------------|
| `!PI` | Pi (single precision) |
| `!DPI` | Pi (double precision) |
| `!E` | Euler's number |
| `!RADEG` | Radians to degrees conversion factor |
| `!DTOR` | Degrees to radians conversion factor |
| `!NULL` | Null value |
| `!TRUE` | Boolean true |
| `!FALSE` | Boolean false |
| `!VERSION` | XDL version information |
| `!DIR` | XDL installation directory |
| `!PATH` | Search path for procedures |
| `!PROMPT` | Command prompt string |
| `!QUIET` | Suppress informational messages |

---

## Commands

### VS Code Commands

| Command | Description |
|---------|-------------|
| `XDL: Restart Language Server` | Restart the LSP server |
| `XDL: Run Current File` | Execute the current XDL file |

---

## Troubleshooting

### Language Server Not Starting

1. Check that `xdl-lsp` is in your PATH or configure `xdl.lsp.path`
2. View the Output panel (View > Output > XDL Language Server)
3. Set `xdl.trace.server` to `verbose` for detailed logs

### Syntax Highlighting Not Working

1. Ensure file has `.xdl` or `.pro` extension
2. Check that language mode is set to "XDL" in status bar
3. Try reloading the VS Code window

### Diagnostics Not Appearing

1. Verify the LSP server is running (check Output panel)
2. Ensure the file is saved (diagnostics update on save)
3. Check for valid XDL syntax

---

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/TuringWorks/xdl
cd xdl

# Build LSP server
cargo build -p xdl-lsp --release

# Build VS Code extension
cd vscode-xdl
npm install
npm run compile
```

### Running Tests

```bash
# LSP server tests
cargo test -p xdl-lsp

# Extension compile check
cd vscode-xdl && npm run compile
```

---

## Future Enhancements

- [ ] Code formatting
- [ ] Rename symbol
- [ ] Code actions (quick fixes)
- [ ] Workspace symbol search
- [ ] Call hierarchy
- [ ] Type hierarchy
- [ ] Inlay hints
- [ ] Folding ranges from AST
