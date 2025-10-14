# Technical Vision - Ledger Bridge

> A KISS-focused technical blueprint for learning trait-based polymorphism in Rust

---

## 1. Technologies & Dependencies

### Core Technologies
- **Rust**: 2021 edition (stable)
- **Build System**: Cargo workspace for multi-crate management

### Dependencies

#### Parser Library (`ledger-parser`)
**Zero external dependencies** - Standard library only!

- Manual CSV parsing (commas and newlines)
- Manual MT940 parsing (text format with field tags)
- Manual CAMT.053 XML parsing (string pattern matching)
- Manual date/time parsing (ISO 8601 → custom types)
- Manual Error implementation (`std::error::Error` trait)

#### CLI Application (`ledger-bridge-cli`)
- `clap` (with derive feature) - CLI argument parsing
- `ledger-parser` (workspace dependency)

### Standard Library Traits (Core Learning Focus)
All conversions performed using:
- `From<T>` / `Into<T>` - Infallible conversions
- `TryFrom<T>` / `TryInto<T>` - Fallible conversions
- `std::fmt::Display` - Output formatting
- `std::error::Error` - Error handling
- `std::io::Read` / `std::io::Write` - I/O operations

---

## 2. Development Principles

### Core Principles

1. **Simplicity First**
   - Start with the simplest implementation that works
   - Refactor only when needed
   - Avoid premature optimization

2. **Explicit Over Implicit**
   - Clear, readable code over clever tricks
   - Explicit error handling (no `unwrap()` or `expect()`)
   - Type conversions using explicit trait implementations

3. **Learning-Oriented Code**
   - Implement traits manually to understand them deeply
   - Write verbose code where it aids understanding
   - Document *why* decisions were made, not just *what*

4. **Error Handling Rules**
   - All errors must be typed (custom error enums)
   - Use `Result<T, E>` everywhere fallible
   - Library returns errors; CLI decides how to display them
   - No panics in library code

5. **Documentation Standards**
   - All public API items have doc comments (`///`)
   - Include examples in docs where helpful
   - Document trait implementations and their contracts

6. **Testing Approach**
   - Unit tests for each parser/formatter
   - Integration tests for conversions
   - Test error cases, not just happy paths
   - Keep tests simple and readable

### Anti-Patterns to Avoid
- ❌ Using `.unwrap()` or `.expect()` in library code
- ❌ Overly generic code that obscures intent
- ❌ Magic numbers or undocumented format assumptions
- ❌ Silent error swallowing

---

## 3. Project Structure

### Cargo Workspace Layout

```
ledger-bridge/
├── Cargo.toml              # Workspace root
├── README.md
├── idea.md
├── vision.md
│
├── ledger-parser/          # Library crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs         # Public API exports
│   │   │
│   │   ├── model.rs       # Data model (Transaction, Statement)
│   │   ├── error.rs       # Error types
│   │   ├── traits.rs      # Parser & Formatter trait definitions
│   │   │
│   │   └── formats/       # Format implementations
│   │       ├── csv.rs
│   │       ├── mt940.rs
│   │       └── camt053.rs
│   │
│   └── tests/
│       ├── integration_test.rs
│       └── conversion_test.rs
│
└── ledger-bridge-cli/      # CLI binary crate
    ├── Cargo.toml
    └── src/
        └── main.rs
```

### Module Organization

**Modern Rust 2018+ structure** - No `mod.rs` files

#### `ledger-parser/src/lib.rs`
- Public API exports
- Re-exports core types and traits
- Module declarations

#### `ledger-parser/src/model.rs`
- Core data types: `Transaction`, `Statement`, etc.
- Pure data structures
- Implements: `Clone`, `Debug`, `PartialEq`
- No parsing/formatting logic

#### `ledger-parser/src/error.rs`
- Custom error enums
- Implements `std::error::Error` and `std::fmt::Display`
- Error types for parsing, formatting, conversion

#### `ledger-parser/src/traits.rs`
- `Parser<T>` trait - Generic parsing abstraction
- `Formatter<T>` trait - Generic formatting abstraction
- **Not tied to specific data types**
- Work with any type satisfying trait bounds

#### `ledger-parser/src/formats/*.rs`
- One file per format (CSV, MT940, CAMT.053)
- Both parser and formatter in same file
- Implements traits for specific types
- Format-specific logic isolated

### Design Philosophy
✅ **Separation of concerns**: data ↔ traits ↔ implementations  
✅ **Trait-based abstraction**: Parsers work with any compatible type  
✅ **Extensibility**: Add formats without touching existing code  
✅ **KISS**: Flat structure, easy navigation  

---

## 4. Architecture & Traits

### Core Trait Design

**Generic traits - not tied to specific data types**

```rust
/// Generic parser trait - works with ANY type T
pub trait Parser<T> {
    type Error;
    
    /// Parse input string into type T
    fn parse(&self, input: &str) -> Result<T, Self::Error>;
}

/// Generic formatter trait - works with ANY type T  
pub trait Formatter<T> {
    type Error;
    
    /// Format data of type T into string
    fn format(&self, data: &T) -> Result<String, Self::Error>;
}
```

### Unified Data Model Approach

**Single `Statement` type** - All formats converge to this:

```rust
pub struct Statement {
    pub account: String,
    pub transactions: Vec<Transaction>,
    // Common fields across all formats
}

pub struct Transaction {
    pub date: String,          // ISO 8601
    pub amount: f64,
    pub description: String,
    // Other common fields
}
```

### Parser Implementations

**Zero-sized types** (empty structs):

```rust
pub struct CsvParser;
pub struct Mt940Parser;
pub struct Camt053Parser;

// All implement the same traits for Statement
impl Parser<Statement> for CsvParser { ... }
impl Parser<Statement> for Mt940Parser { ... }
impl Parser<Statement> for Camt053Parser { ... }

impl Formatter<Statement> for CsvParser { ... }
impl Formatter<Statement> for Mt940Parser { ... }
impl Formatter<Statement> for Camt053Parser { ... }
```

### Conversion Flow

```
CSV/MT940/CAMT.053 (String input)
         ↓
   Parser::parse()
         ↓
     Statement (unified type)
         ↓
   Formatter::format()
         ↓
CSV/MT940/CAMT.053 (String output)
```

**Format conversion**: Parse from format A → Unified Statement → Format to B

### Key Benefits
✅ **Generic traits**: Work with any type, not just Statement  
✅ **Simple conversions**: No complex type mapping  
✅ **Unified representation**: One data model for all formats  
✅ **Extensible**: Add new formats without changing traits  

---

## 5. Data Model

### Core Data Structures

```rust
/// Bank statement containing transactions and balances
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub account_number: String,
    pub currency: String,              // "USD", "EUR", etc.
    
    // Opening balance (starting position)
    pub opening_balance: f64,
    pub opening_date: String,          // ISO 8601: "YYYY-MM-DD"
    
    // Closing balance (ending position)
    pub closing_balance: f64,
    pub closing_date: String,          // ISO 8601: "YYYY-MM-DD"
    
    pub transactions: Vec<Transaction>,
}

/// Individual transaction entry
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub date: String,                  // ISO 8601: "YYYY-MM-DD"
    pub amount: f64,                   // Positive = credit, Negative = debit
    pub description: String,           // Transaction description/narrative
    pub reference: Option<String>,     // Optional reference/transaction ID
}
```

### Field Mapping Across Formats

| Field | CSV | MT940 | CAMT.053 |
|-------|-----|-------|----------|
| account_number | "Account" | :25: tag | `<Acct><Id>` |
| currency | "Currency" | :60F: (3 chars) | `<Amt Ccy="XXX">` |
| opening_balance | "Opening" | :60F: amount | `<Bal><Tp><Cd>OPBD` |
| opening_date | "From Date" | :60F: date | `<Bal><Dt>` |
| closing_balance | "Closing" | :62F: amount | `<Bal><Tp><Cd>CLBD` |
| closing_date | "To Date" | :62F: date | `<Bal><Dt>` |
| date | "Date" | :61: date | `<BookgDt>` |
| amount | "Amount" | :61: amount | `<Amt>` |
| description | "Description" | :86: | `<RmtInf><Ustrd>` |
| reference | "Reference" | :61: ref | `<NtryRef>` |

### Design Decisions
✅ **Dates as strings** - ISO 8601 format, no external library  
✅ **Signed amounts** - Negative = debit, Positive = credit  
✅ **Statement period** - Opening/closing dates define range  
✅ **Minimal fields** - Only common fields across all formats  
✅ **Derive traits** - Debug, Clone, PartialEq for testing  

### Simplifications
- No timezone handling (date strings only)
- No decimal precision types (f64 sufficient for tutorial)
- No nested transaction details
- Optional reference field (not all formats require it)

---

## 6. Error Handling

### Unified Error Type

```rust
/// Error type for all parsing and formatting operations
#[derive(Debug)]
pub enum ParseError {
    // General errors
    InvalidFormat(String),
    MissingField(String),
    InvalidFieldValue { field: String, value: String },
    
    // Format-specific errors
    CsvError(String),
    Mt940Error(String),
    Camt053Error(String),
    
    // I/O errors
    IoError(String),
}
```

### Trait Implementations

```rust
// User-friendly error messages
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ParseError::InvalidFieldValue { field, value } => 
                write!(f, "Invalid value '{}' for field '{}'", value, field),
            ParseError::CsvError(msg) => write!(f, "CSV error: {}", msg),
            ParseError::Mt940Error(msg) => write!(f, "MT940 error: {}", msg),
            ParseError::Camt053Error(msg) => write!(f, "CAMT.053 error: {}", msg),
            ParseError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

// Standard error trait
impl std::error::Error for ParseError {}

// Convert std::io::Error to ParseError
impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
    }
}
```

### Usage Pattern

```rust
// Both Parser and Formatter use ParseError
impl Parser<Statement> for CsvParser {
    type Error = ParseError;
    
    fn parse(&self, input: &str) -> Result<Statement, Self::Error> {
        if input.is_empty() {
            return Err(ParseError::CsvError("Empty input".to_string()));
        }
        // ... parsing logic
    }
}
```

### CLI Error Handling

```rust
// CLI displays errors and exits gracefully
match parser.parse(&content) {
    Ok(statement) => { /* process */ },
    Err(e) => {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

### Error Handling Rules
✅ **One unified error type** - Simple, used across entire library  
✅ **Descriptive messages** - Clear indication of what went wrong  
✅ **No panics** - Always return `Result<T, ParseError>`  
✅ **No `.unwrap()` or `.expect()`** - Explicit error propagation  
✅ **No line/column tracking** - Keep implementation simple  

---

## 7. Testing Strategy

### Test Organization

```
ledger-parser/
├── src/
│   ├── formats/csv.rs         # Unit tests in #[cfg(test)] module
│   ├── formats/mt940.rs       # Unit tests in #[cfg(test)] module
│   └── formats/camt053.rs     # Unit tests in #[cfg(test)] module
└── tests/
    └── integration_test.rs    # Format conversions
```

### Unit Tests (Inline with Implementation)

```rust
// In src/formats/csv.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Statement, Transaction};
    
    #[test]
    fn test_parse_csv() {
        let input = "Account,Currency,...\nACC001,USD,...";
        let parser = CsvParser;
        let result = parser.parse(input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_format_csv() {
        let statement = Statement { /* ... */ };
        let formatter = CsvParser;
        let result = formatter.format(&statement);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_parse_error() {
        let parser = CsvParser;
        let result = parser.parse("");
        assert!(result.is_err());
    }
}
```

### Integration Tests (Format Conversions)

```rust
// tests/integration_test.rs
use ledger_parser::*;

#[test]
fn test_csv_to_mt940_conversion() {
    let csv_input = "...";
    let statement = formats::CsvParser.parse(csv_input).unwrap();
    let mt940_output = formats::Mt940Parser.format(&statement).unwrap();
    assert!(!mt940_output.is_empty());
}

#[test]
fn test_mt940_to_camt053_conversion() {
    let mt940_input = "...";
    let statement = formats::Mt940Parser.parse(mt940_input).unwrap();
    let camt053_output = formats::Camt053Parser.format(&statement).unwrap();
    assert!(!camt053_output.is_empty());
}
```

### Minimal Coverage Requirements
✅ **Unit tests** - Parse + format + error case for each format  
✅ **Integration tests** - Key conversion paths between formats  
✅ **Inline test data** - String literals in test code  
✅ **Simple assertions** - Focus on correctness, not exhaustive edge cases  

### Test Data Approach
- Minimal valid examples as inline strings
- No external test fixture files
- Cover happy path + basic error handling
- Keep tests simple and readable

---

## 8. CLI Workflow

### Command-Line Interface

```bash
# Read from stdin, write to stdout
cat input.csv | ledger-bridge --in-format csv --out-format mt940 > output.mt940

# Read from file, write to file
ledger-bridge --in-format mt940 --out-format camt053 --input data.mt940 --output result.xml

# Read from file, write to stdout
ledger-bridge --in-format csv --out-format camt053 --input data.csv

# Read from stdin, write to file
cat data.mt940 | ledger-bridge --in-format mt940 --out-format csv --output result.csv
```

### CLI Arguments (using clap)

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "ledger-bridge")]
#[command(version)]
#[command(about = "Convert financial data between formats")]
struct Cli {
    /// Input format: csv, mt940, camt053
    #[arg(long, value_name = "FORMAT")]
    in_format: String,
    
    /// Output format: csv, mt940, camt053
    #[arg(long, value_name = "FORMAT")]
    out_format: String,
    
    /// Input file (default: stdin)
    #[arg(long, short = 'i')]
    input: Option<String>,
    
    /// Output file (default: stdout)
    #[arg(long, short = 'o')]
    output: Option<String>,
}
```

### Main Logic Flow

```rust
fn main() {
    let cli = Cli::parse();
    
    // 1. Read input (file or stdin)
    let content = match &cli.input {
        Some(path) => std::fs::read_to_string(path).expect("Failed to read input"),
        None => read_stdin().expect("Failed to read stdin"),
    };
    
    // 2. Select parser based on --in-format (case-insensitive)
    let statement = match cli.in_format.to_lowercase().as_str() {
        "csv" => CsvParser.parse(&content),
        "mt940" => Mt940Parser.parse(&content),
        "camt053" => Camt053Parser.parse(&content),
        _ => { eprintln!("Unknown format"); std::process::exit(1); }
    }.unwrap_or_else(|e| { eprintln!("Parse error: {}", e); std::process::exit(1); });
    
    // 3. Select formatter based on --out-format (case-insensitive)
    let output = match cli.out_format.to_lowercase().as_str() {
        "csv" => CsvParser.format(&statement),
        "mt940" => Mt940Parser.format(&statement),
        "camt053" => Camt053Parser.format(&statement),
        _ => { eprintln!("Unknown format"); std::process::exit(1); }
    }.unwrap_or_else(|e| { eprintln!("Format error: {}", e); std::process::exit(1); });
    
    // 4. Write output (file or stdout)
    match &cli.output {
        Some(path) => std::fs::write(path, output).expect("Failed to write output"),
        None => print!("{}", output),
    }
}
```

### Error Handling

- **Parse errors** → stderr + exit code 1
- **I/O errors** → stderr + exit code 1  
- **Invalid format names** → stderr + exit code 1
- **Success** → exit code 0

### Supported Formats (case-insensitive)
- `csv`, `CSV`, `Csv` → CSV format
- `mt940`, `MT940`, `Mt940` → MT940 format
- `camt053`, `CAMT053`, `Camt053` → CAMT.053 format

### Built-in Flags (provided by clap)
- `--help` → Display usage information
- `--version` → Display version information

---

## Summary

### Technical Vision Overview

**Ledger Bridge** is designed as a learning-focused Rust project that demonstrates trait-based polymorphism through practical financial data parsing.

#### Key Design Decisions

✅ **Zero dependencies** in parser library (standard library only)  
✅ **Unified data model** (single Statement type)  
✅ **Generic traits** (not tied to specific types)  
✅ **Simple error handling** (one ParseError type)  
✅ **Minimal testing** (unit + integration)  
✅ **Clean CLI** (clap for argument parsing)  
✅ **KISS principle** throughout

#### Project Deliverables

1. **`ledger-parser` library** - Reusable parsing/formatting engine
2. **`ledger-bridge-cli` binary** - Command-line conversion tool
3. **Documentation** - All public APIs documented
4. **Tests** - Unit and integration test coverage
5. **Format support** - CSV, MT940, CAMT.053

#### Next Steps

With this technical vision established, we're ready to:
1. Set up the Cargo workspace
2. Implement the data model
3. Define traits
4. Implement parsers/formatters
5. Build the CLI
6. Write tests
7. Document the code

This vision serves as our blueprint for development. All decisions prioritize simplicity and learning over production-grade features.

