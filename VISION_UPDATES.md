# Vision.md Updates Summary

## Overview
The `vision.md` file has been comprehensively updated to align with the new requirements from the Russian course materials while keeping CSV format support intact.

---

## Major Architectural Changes

### 1. **I/O Traits: BufRead → Read/Write**

**Before:**
- Used `std::io::BufRead` for parsing
- Formatters returned `Result<String, ParseError>`

**After:**
- Use `std::io::Read` for parsing
- Use `std::io::Write` for formatting
- Direct I/O operations, no intermediate string construction

**Rationale:** Demonstrates standard library trait usage and enables working with any Read/Write source without code duplication.

---

### 2. **Data Model: Unified → Format-Specific**

**Before:**
```rust
pub struct Statement { ... }

impl Parser<Statement> for CsvParser { ... }
impl Parser<Statement> for Mt940Parser { ... }
impl Parser<Statement> for Camt053Parser { ... }
```

**After:**
```rust
pub struct Mt940 { ... }
pub struct Camt053 { ... }
pub struct CsvStatement { ... }

impl Mt940 {
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> { ... }
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> { ... }
}
```

**Rationale:** 
- Each format is a distinct type
- Type-safe conversions
- No generic traits needed
- Explicit format origin in function signatures

---

### 3. **Conversion Mechanism: Unified Model → From Trait**

**Before:**
```rust
// Parse from format A to unified Statement
let statement = CsvParser.parse(reader)?;

// Format from Statement to format B
let output = Mt940Parser.format(&statement)?;
```

**After:**
```rust
// Parse to format-specific struct
let csv = CsvStatement::from_read(&mut reader)?;

// Convert using From trait
let mt940: Mt940 = csv.into();

// Write to output
mt940.write_to(&mut writer)?;
```

**Rationale:**
- Idiomatic Rust type conversions
- Demonstrates `From` trait implementation
- Type-safe, compile-time checked conversions

---

### 4. **Method Signatures**

**Before:**
```rust
pub trait Parser<T> {
    type Error;
    fn parse<R: BufRead>(&self, reader: R) -> Result<T, Self::Error>;
}

pub trait Formatter<T> {
    type Error;
    fn format(&self, data: &T) -> Result<String, Self::Error>;
}
```

**After:**
```rust
impl Mt940 {
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Implementation
    }
    
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Implementation
    }
}
```

**Rationale:**
- Simpler API (no traits to understand)
- Standard Rust pattern for parsing (like `from_str`)
- Direct Read/Write usage

---

## Third-Party Library Recommendations

### Added to Dependencies Section:

1. **`csv` crate (v1.3)** - Trust Score: 9.1
   - Fast, flexible CSV reader/writer
   - Native Read/Write support
   - Serde integration for automatic struct mapping
   - Excellent for CSV parsing with minimal code

2. **`quick-xml` crate (v0.31)** - Trust Score: 9.2
   - High-performance XML parser
   - Event-based parsing (SAX-like)
   - Native Read/Write support
   - Serde integration for CAMT.053 XML
   - Perfect for ISO 20022 formats

3. **Manual parsing for MT940**
   - Educational value
   - Demonstrates text processing skills
   - No library can fully handle MT940 variants
   - Intentionally complex to teach parsing techniques

---

## Updated Cargo.toml Example

```toml
[package]
name = "ledger-parser"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
csv = "1.3"           # CSV parsing with Read/Write support
quick-xml = "0.31"    # XML parsing for CAMT.053 with Serde integration
```

---

## Learning Objectives (Updated)

The project now emphasizes:

1. **Standard Library I/O Traits**
   - How `Read` enables reading from files, stdin, buffers, network
   - How `Write` enables writing to files, stdout, buffers
   - No code duplication across different I/O sources

2. **Type Conversions with From Trait**
   - Implementing `From` for domain-specific conversions
   - Automatic `Into` implementation
   - Type-safe, compile-time checked conversions

3. **Static Polymorphism**
   - Generic functions monomorphized at compile time
   - No runtime overhead
   - Demonstrates Rust's zero-cost abstractions

4. **Practical Parsing**
   - Real-world financial data formats
   - Using external libraries effectively
   - Manual parsing for educational purposes

---

## CSV Format Support

**Maintained as requested:**
- CsvStatement struct with same fields as Mt940/Camt053
- Full bidirectional conversions:
  - `impl From<Mt940> for CsvStatement`
  - `impl From<CsvStatement> for Mt940`
  - `impl From<Camt053> for CsvStatement`
  - `impl From<CsvStatement> for Camt053`
- Uses `csv` crate for robust parsing
- Handles Russian Sberbank format from examples

---

## CLI Changes

**Before:**
```rust
let reader: Box<dyn BufRead> = ...;
let statement = CsvParser.parse(reader)?;
let output = Mt940Parser.format(&statement)?;
std::fs::write(path, output)?;
```

**After:**
```rust
let mut input: Box<dyn Read> = ...;
let csv = CsvStatement::from_read(&mut input)?;
let mt940: Mt940 = csv.into();
let mut output: Box<dyn Write> = ...;
mt940.write_to(&mut output)?;
```

**Key Benefits:**
- Demonstrates Read/Write trait flexibility
- Type-safe conversions with From trait
- No intermediate string allocations
- Works seamlessly with files, stdin/stdout, buffers

---

## Testing Strategy Updates

**Unit Tests:**
```rust
#[test]
fn test_parse_csv() {
    let input = "...";
    let mut reader = input.as_bytes();
    let result = CsvStatement::from_read(&mut reader);
    assert!(result.is_ok());
}

#[test]
fn test_write_csv() {
    let statement = CsvStatement { ... };
    let mut output = Vec::new();
    let result = statement.write_to(&mut output);
    assert!(result.is_ok());
}
```

**Integration Tests:**
```rust
#[test]
fn test_mt940_to_camt053_conversion() {
    let mut reader = mt940_input.as_bytes();
    let mt940 = Mt940::from_read(&mut reader).unwrap();
    let camt053: Camt053 = mt940.into();
    
    let mut output = Vec::new();
    camt053.write_to(&mut output).unwrap();
    assert!(!output.is_empty());
}
```

---

## Documentation Emphasis

Updated subtitle from:
> "A KISS-focused technical blueprint for learning trait-based polymorphism in Rust"

To:
> "A KISS-focused technical blueprint for learning Rust standard library I/O traits and type conversions"

This better reflects the new learning objectives.

---

## Format-Specific Implementation Notes

### MT940 (Manual Parsing)
- Read entire input using `Read::read_to_string()`
- Tag-based parsing (`:20:`, `:25:`, `:60F:`, `:61:`, `:86:`, `:62F:`)
- Century inference for YYMMDD dates
- Intentionally manual for educational value

### CAMT.053 (quick-xml)
- Use `quick_xml::Reader::from_reader()` with Read trait
- Event-based parsing or Serde deserialization
- Handle namespaces and nested structures
- Filter balance types (OPBD/CLBD)

### CSV (csv crate)
- Use `csv::Reader::from_reader()` with Read trait
- Automatic Serde deserialization
- Handle Russian localized format
- Split debit/credit columns

---

## Key Takeaways

1. ✅ **Architecture completely redesigned** to use Read/Write traits
2. ✅ **From trait conversions** replace unified data model approach
3. ✅ **Format-specific structs** provide type safety
4. ✅ **Third-party libraries researched** and recommended with trust scores
5. ✅ **CSV support maintained** as requested
6. ✅ **All examples updated** to reflect new architecture
7. ✅ **Learning objectives clarified** to emphasize standard library patterns
8. ✅ **No linter errors** in updated document

---

## Alignment with Course Requirements

The updated `vision.md` now fully aligns with the Russian course requirements:

✅ Parsers use `std::io::Read` trait  
✅ Formatters use `std::io::Write` trait  
✅ Format-specific structures (not unified model)  
✅ `From` trait for type conversions  
✅ Static polymorphism through generics  
✅ No code duplication for different I/O sources  
✅ Demonstrates practical value of standard library traits  

---

**Document Updated:** October 18, 2025  
**Status:** Complete, ready for implementation

