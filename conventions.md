# Development Conventions - Ledger Bridge

> Essential code development rules for this project. See [@vision.md](vision.md) for complete technical blueprint.

---

## Code Organization

### Module Structure
- Use **Rust 2018+ style** - No `mod.rs` files
- One module per format in `formats/` directory
- Keep parsers and formatters in the same file (per format)
- Separate concerns: data model ↔ traits ↔ implementations

### File Layout
```
lib.rs        → Public API exports, module declarations
model.rs      → Pure data structures (no logic)
error.rs      → Error types with Display + Error traits
traits.rs     → Parser and Formatter trait definitions
formats/*.rs  → Format implementations (parser + formatter together)
```

---

## Error Handling (Critical)

### Mandatory Rules
- ❌ **NEVER** use `.unwrap()` or `.expect()` in library code
- ❌ **NO PANICS** in library code
- ✅ Always return `Result<T, ParseError>`
- ✅ Use explicit error propagation with `?` operator
- ✅ Implement `std::error::Error` and `std::fmt::Display` for all error types
- ✅ CLI handles display and exit codes; library only returns errors

### Error Type
- Use single unified `ParseError` enum across entire library
- Provide descriptive error messages (context-specific)
- Implement `From<std::io::Error>` for automatic conversion

---

## Trait Implementation

### Parser Trait
- Must use `std::io::BufRead` or `std::io::Read` (wrapped in `BufReader`)
- Generic over type `T`, not tied to specific data structures
- Associated error type: `type Error = ParseError`

### Formatter Trait
- Returns `Result<String, ParseError>`
- Generic over type `T`
- No I/O operations (formatters produce strings)

### Implementation Pattern
```rust
impl Parser<Statement> for CsvParser {
    type Error = ParseError;
    fn parse<R: BufRead>(&self, reader: R) -> Result<Statement, Self::Error> {
        // Implementation using BufRead
    }
}
```

---

## Code Style

### Explicitness
- **Explicit over implicit** - Clear code over clever tricks
- Explicit type conversions using trait implementations
- No magic numbers - use named constants or document inline

### Simplicity
- Start with simplest implementation that works
- Avoid premature optimization
- Refactor only when needed
- Keep parsing logic straightforward (manual string processing is fine)

### Data Structures
- All public types must derive:
  - `Debug`, `Clone`, `PartialEq` (testing)
  - `Serialize`, `Deserialize` (serde integration)
- Use zero-sized types (empty structs) for parsers/formatters

---

## Documentation

### Requirements
- ✅ All **public** items need doc comments (`///`)
- ✅ Document **why** decisions were made, not just what
- ✅ Include examples in docs where helpful
- ✅ Document trait implementation contracts

### Style
```rust
/// Parses CSV bank statements into unified Statement format.
///
/// # Errors
/// Returns `ParseError::CsvError` if the CSV structure is invalid.
///
/// # Example
/// ```
/// let reader = BufReader::new(csv_data.as_bytes());
/// let statement = CsvParser.parse(reader)?;
/// ```
pub struct CsvParser;
```

---

## Testing

### Coverage Requirements
- Unit tests for each parser (happy path + error cases)
- Unit tests for each formatter (happy path + error cases)
- Integration tests for format conversions
- Test error cases, not just success paths

### Test Organization
- Unit tests in `#[cfg(test)]` modules within implementation files
- Integration tests in `tests/` directory
- Use inline test data (string literals), no external files
- Keep tests simple and readable

### Test Data
```rust
#[test]
fn test_parse_csv() {
    let input = "Account,Currency,...\n...";
    let reader = BufReader::new(input.as_bytes());
    let result = CsvParser.parse(reader);
    assert!(result.is_ok());
}
```

---

## Dependencies

### Allowed Libraries
- **Required**: `serde` (with derive feature) - Data structure serialization
- **CLI only**: `clap` (with derive feature) - Argument parsing
- **Optional**: Simple helpers like `csv`, `quick-xml` (only if needed, keep it simple)

### Dependency Rules
- Avoid heavy dependencies
- No parser combinators unless needed for learning
- Prefer manual parsing for simple formats
- Standard library traits preferred over external abstractions

---

## Anti-Patterns (Must Avoid)

### Forbidden Practices
- ❌ `.unwrap()` or `.expect()` in library code
- ❌ Overly generic code that obscures intent
- ❌ Silent error swallowing (always propagate or handle explicitly)
- ❌ Undocumented format assumptions
- ❌ Complex abstractions when simple code suffices

### Code Smells
- Magic numbers without explanation
- Deeply nested match statements (flatten with early returns)
- Duplicated parsing logic (extract to helper functions)
- Missing error context (wrap errors with descriptive messages)

---

## I/O Patterns

### Input Handling
- All parsers accept `impl BufRead` or `impl Read`
- Wrap `Read` with `BufReader::new()` for buffering
- Support stdin, files, in-memory buffers (via BufRead abstraction)
- Line-by-line reading for CSV and MT940
- Read-all acceptable for small XML files

### Output Handling
- Formatters return `Result<String, ParseError>`
- CLI decides where to write (stdout or file)
- No direct file I/O in library formatters

---

## Naming Conventions

### Types
- Parsers: `CsvParser`, `Mt940Parser`, `Camt053Parser` (zero-sized structs)
- Data: `Statement`, `Transaction`, `BalanceType`, `TransactionType`
- Errors: `ParseError` (single unified enum)

### Methods
- Parsing: `fn parse<R: BufRead>(&self, reader: R) -> Result<T, Self::Error>`
- Formatting: `fn format(&self, data: &T) -> Result<String, Self::Error>`

### Variables
- Use descriptive names (no single-letter except loop indices)
- `reader` for BufRead/Read sources
- `statement` for parsed Statement instances
- `output` for formatted strings

---

## Version Control

### Commits
- Commit working increments (parsable code)
- Write clear commit messages
- Don't commit broken code (must compile)

### Git Workflow
- Keep commits focused (one logical change)
- Test before committing

---

## CLI Guidelines

### Argument Parsing
- Use `clap` with derive macros
- Support both file and stdin/stdout
- Case-insensitive format names (`csv`, `CSV`, `Csv` all valid)

### Error Display
- Print parse/format errors to stderr
- Exit with code 1 on error, 0 on success
- Use `unwrap_or_else()` with custom error messages for CLI (library stays panic-free)

### Usage Pattern
```bash
ledger-bridge --in-format csv --out-format mt940 --input data.csv --output result.mt940
cat input.csv | ledger-bridge --in-format csv --out-format camt053 > output.xml
```

---

## Learning Focus

### Priorities
1. **Understand trait-based polymorphism** - Implement traits manually
2. **Standard library I/O patterns** - Use `BufRead`, `Read`, `Write` traits
3. **Error handling** - Explicit `Result` types, custom error implementations
4. **Type conversions** - Use `From`/`Into`, `TryFrom`/`TryInto` traits

### Write Verbose Code When
- It aids understanding of trait mechanics
- Makes error handling explicit
- Clarifies type conversions
- Documents design decisions

---

## Summary

**Key Principle**: Keep It Simple and Stupid (KISS)

When in doubt:
1. Choose simpler implementation
2. Be explicit about errors
3. Document the "why"
4. Test both success and failure
5. Reference [@vision.md](vision.md) for detailed specifications

---

*This document contains only the essential rules that impact code quality. For complete technical specifications, architecture details, and format parsing strategies, see [vision.md](vision.md).*

