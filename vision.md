# Technical Vision - Ledger Bridge

> A KISS-focused technical blueprint for learning Rust standard library I/O traits and type conversions

---

## 1. Technologies & Dependencies

### Core Technologies
- **Rust**: 2021 edition (stable)
- **Build System**: Cargo workspace for multi-crate management

### Dependencies

#### Parser Library (`ledger-parser`)

**Core dependency:**
- `serde` (with `derive` feature) - Serialization/deserialization framework

**Why serde?**
- Enables `Serialize`/`Deserialize` traits on data structures
- Makes the data model interoperable with JSON, TOML, and other formats
- Provides robust derive macros reducing boilerplate
- Industry-standard choice for Rust data handling
- Allows future extensibility (export to JSON, etc.)
- Essential for XML parsing with quick-xml's serde integration

**Main requirement:** 
- **Parsers must use `std::io::Read` trait for input**
- **Formatters must use `std::io::Write` trait for output**
- This demonstrates how standard library traits enable code reuse across different data sources (files, stdin, buffers, network)
- No need for separate implementations for each I/O source

**Recommended parsing libraries:**
- **`csv`** - Fast, flexible CSV reader/writer with excellent Read/Write support (Trust Score: 9.1)
  - Supports Serde for automatic struct serialization/deserialization
  - Efficient buffering and memory usage
  - Handles quoted fields, custom delimiters, and edge cases
  
- **`quick-xml`** - High-performance XML parser for CAMT.053 (Trust Score: 9.2)
  - Native Read/Write trait support
  - Event-based parsing (similar to SAX)
  - Serde integration for automatic struct deserialization
  - Excellent for ISO 20022 XML formats
  
- **Manual parsing for MT940** - String processing recommended
  - MT940 format is intentionally complex and educational
  - Requires understanding of tag-based parsing
  - Good practice for manual text processing

**CSV Parsing (using `csv` crate):**
- Use `csv::Reader::from_reader()` to read from any `impl Read`
- Automatic field parsing and type conversion via Serde
- Handle quoted fields, escaped quotes, and custom delimiters
- Skip metadata rows (headers/footers) manually if needed
- Parse split debit/credit columns into unified transaction type
- Write using `csv::Writer::from_writer()` to any `impl Write`

**MT940 Parsing (manual implementation):**
- Read entire input using `Read::read_to_string()` or buffered reading
- Block extraction (`{1:...}{2:...}{4:...}`)
- Tag-based parsing (`:20:`, `:25:`, `:60F:`, `:61:`, `:86:`, `:62F:`)
- Multi-line field handling (`:86:` can span lines)
- Date parsing: YYMMDD → YYYY-MM-DD (century inference)
- Balance parsing: C/D indicator + date + currency + amount
- Transaction line parsing: complex format with embedded delimiters
- Amount parsing: handle comma as decimal separator
- **This format intentionally requires manual parsing to demonstrate text processing skills**

**CAMT.053 XML Parsing (using `quick-xml` crate):**
- Use `quick_xml::Reader::from_reader()` to read from any `impl Read`
- Event-based parsing with `read_event_into()` method
- Optional: Use Serde deserialization with `#[derive(Deserialize)]` for automatic struct mapping
- Handle namespaces in tag matching
- Parse nested structures (entries contain transaction details)
- Extract attributes (`Ccy="XXX"`) from events
- Handle multiple elements of same type (multiple `<Bal>` and `<Ntry>`)
- Filter balance types (use OPBD/CLBD, ignore OPAV/CLAV)
- Write using `quick_xml::Writer::new()` to any `impl Write`

**Common Utilities:**
- Date parsing: ISO 8601 (YYYY-MM-DD) and YYMMDD formats
- Amount parsing: handle both comma and dot as decimal separators
- String trimming and normalization
- Error implementation (`std::error::Error` trait)

#### CLI Application (`ledger-bridge-cli`)
- `clap` (with derive feature) - CLI argument parsing
- `ledger-parser` (workspace dependency)

### Standard Library Traits (Core Learning Focus)
The project demonstrates practical usage of Rust standard library traits:

**I/O Traits (Primary Focus):**
- `std::io::Read` - **Primary input trait for parsing** (works with files, stdin, buffers, network streams)
- `std::io::Write` - **Primary output trait for formatting** (works with files, stdout, buffers)
- These traits enable **static polymorphism** through generics
- No need for separate implementations for each data source

**Type Conversion Traits:**
- `From<T>` / `Into<T>` - **Used for format conversions** (infallible)
  - Example: `impl From<Mt940> for Camt053`
  - Enables idiomatic Rust type conversions
  - Automatic `Into` implementation from `From`
- `TryFrom<T>` / `TryInto<T>` - Fallible conversions (if needed)

**Error Handling:**
- `std::error::Error` - Standard error trait
- `std::fmt::Display` - User-friendly error messages
- `From<std::io::Error>` - Automatic I/O error conversion

### Serde Benefits
With Serialize/Deserialize on all data structures:
- **Future extensibility**: Easy to add JSON/TOML/YAML export (with `serde_json`, etc.)
- **Testing**: Can serialize/deserialize test fixtures
- **Debugging**: Can dump Statement as JSON for inspection
- **API ready**: Data model ready for REST APIs or other integrations
- **Learning**: Demonstrates idiomatic Rust pattern (serde is ubiquitous)

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
├── example_files/          # Real-world format examples (for reference)
│   ├── sources.md         # Source attribution for examples
│   ├── *.camt             # CAMT.053 XML examples
│   ├── *.mt940            # MT940 SWIFT message examples
│   └── *.csv              # CSV statement examples
│
├── ledger-parser/          # Library crate
│   ├── Cargo.toml         # Dependencies: serde with derive feature
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
    ├── Cargo.toml         # Dependencies: clap, ledger-parser
    └── src/
        └── main.rs
```

### Cargo.toml Examples

**Workspace root (`Cargo.toml`):**
```toml
[workspace]
members = ["ledger-parser", "ledger-bridge-cli"]
resolver = "2"
```

**Library (`ledger-parser/Cargo.toml`):**
```toml
[package]
name = "ledger-parser"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }

# Format-specific parsing libraries
csv = "1.3"           # CSV parsing with Read/Write support
quick-xml = "0.31"    # XML parsing for CAMT.053 with Serde integration

# Optional: serde_xml_rs as alternative for XML deserialization
# serde-xml-rs = "0.6"
```

**CLI (`ledger-bridge-cli/Cargo.toml`):**
```toml
[package]
name = "ledger-bridge-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
ledger-parser = { path = "../ledger-parser" }
clap = { version = "4.0", features = ["derive"] }
```

### Module Organization

**Modern Rust 2018+ structure** - No `mod.rs` files

#### `ledger-parser/src/lib.rs`
- Public API exports
- Re-exports core types and traits
- Module declarations

#### `ledger-parser/src/model.rs`
- Shared data types: `Transaction`, `TransactionType`, `BalanceType`
- Pure data structures used across all formats
- Implements: `Clone`, `Debug`, `PartialEq`, `Serialize`, `Deserialize`

#### `ledger-parser/src/error.rs`
- Custom error enums (`ParseError`)
- Implements `std::error::Error` and `std::fmt::Display`
- Implements `From<std::io::Error>` for automatic conversion

#### `ledger-parser/src/formats/*.rs`
- One file per format (csv.rs, mt940.rs, camt053.rs)
- Each defines format-specific struct (CsvStatement, Mt940, Camt053)
- Each implements:
  - `from_read<R: Read>(&mut R) -> Result<Self, ParseError>` - Parsing
  - `write_to<W: Write>(&mut W) -> Result<(), ParseError>` - Formatting
  - `From<OtherFormat>` - Type conversions
- Format-specific logic isolated
- Uses external crates where appropriate (csv, quick-xml)

### Design Philosophy
✅ **Separation of concerns**: shared types ↔ format-specific structs ↔ conversions  
✅ **Standard library traits**: Read/Write for I/O, From for conversions  
✅ **Format-specific types**: Each format has its own struct for type safety  
✅ **Extensibility**: Add formats by implementing from_read/write_to methods  
✅ **KISS**: Flat structure, easy navigation, straightforward implementation  

---

## 4. Architecture & Design

### Format-Specific Structures

**Each format has its own data structure** - No unified model:

```rust
use std::io::{Read, Write};

/// MT940 SWIFT message structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mt940 {
    pub account_number: String,
    pub currency: String,
    pub opening_balance: f64,
    pub opening_date: String,
    pub opening_indicator: BalanceType,
    pub closing_balance: f64,
    pub closing_date: String,
    pub closing_indicator: BalanceType,
    pub transactions: Vec<Transaction>,
}

impl Mt940 {
    /// Parse MT940 from any source implementing Read
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Read data from reader
        // Parse MT940 format
        // Return Mt940 struct
    }
    
    /// Write MT940 to any destination implementing Write
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Format self as MT940
        // Write to writer
    }
}
```

### CAMT.053 Structure

```rust
/// ISO 20022 CAMT.053 XML structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camt053 {
    pub account_number: String,
    pub currency: String,
    pub opening_balance: f64,
    pub opening_date: String,
    pub opening_indicator: BalanceType,
    pub closing_balance: f64,
    pub closing_date: String,
    pub closing_indicator: BalanceType,
    pub transactions: Vec<Transaction>,
}

impl Camt053 {
    /// Parse CAMT.053 from any source implementing Read
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Use quick-xml to read and parse XML
        // Return Camt053 struct
    }
    
    /// Write CAMT.053 to any destination implementing Write
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Use quick-xml to generate XML
        // Write to writer
    }
}
```

### CSV Statement Structure

```rust
/// CSV bank statement structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsvStatement {
    pub account_number: String,
    pub currency: String,
    pub opening_balance: f64,
    pub opening_date: String,
    pub opening_indicator: BalanceType,
    pub closing_balance: f64,
    pub closing_date: String,
    pub closing_indicator: BalanceType,
    pub transactions: Vec<Transaction>,
}

impl CsvStatement {
    /// Parse CSV from any source implementing Read
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Use csv crate with Reader::from_reader(reader)
        // Parse and return CsvStatement
    }
    
    /// Write CSV to any destination implementing Write
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Use csv crate with Writer::from_writer(writer)
        // Write CSV data
    }
}
```

### Format Conversion using `From` Trait

**Bidirectional conversions between formats:**

```rust
/// Convert MT940 to CAMT.053
impl From<Mt940> for Camt053 {
    fn from(mt940: Mt940) -> Self {
        Camt053 {
            account_number: mt940.account_number,
            currency: mt940.currency,
            opening_balance: mt940.opening_balance,
            opening_date: mt940.opening_date,
            opening_indicator: mt940.opening_indicator,
            closing_balance: mt940.closing_balance,
            closing_date: mt940.closing_date,
            closing_indicator: mt940.closing_indicator,
            transactions: mt940.transactions,
        }
    }
}

/// Convert CAMT.053 to MT940
impl From<Camt053> for Mt940 {
    fn from(camt: Camt053) -> Self {
        Mt940 {
            account_number: camt.account_number,
            currency: camt.currency,
            opening_balance: camt.opening_balance,
            opening_date: camt.opening_date,
            opening_indicator: camt.opening_indicator,
            closing_balance: camt.closing_balance,
            closing_date: camt.closing_date,
            closing_indicator: camt.closing_indicator,
            transactions: camt.transactions,
        }
    }
}

/// Similar implementations for CSV conversions
impl From<Mt940> for CsvStatement { ... }
impl From<CsvStatement> for Mt940 { ... }
impl From<Camt053> for CsvStatement { ... }
impl From<CsvStatement> for Camt053 { ... }
```

### Conversion Flow

```
File/Stdin (Read source)
         ↓
   Format::from_read(&mut reader)
         ↓
     Mt940 / Camt053 / CsvStatement
         ↓
   Into::<TargetFormat>::into()  [using From trait]
         ↓
     Target format struct
         ↓
   .write_to(&mut writer)
         ↓
File/Stdout (Write destination)
```

**Format conversion example:**
```rust
use std::fs::File;
use std::io::{BufReader, BufWriter};

// Read MT940 from file
let mut file = File::open("input.mt940")?;
let mt940 = Mt940::from_read(&mut file)?;

// Convert to CAMT.053 using From trait
let camt053: Camt053 = mt940.into();

// Write to file
let mut output = File::create("output.xml")?;
camt053.write_to(&mut output)?;
```

### Key Benefits
✅ **Read/Write traits**: Standard library abstractions for I/O  
✅ **Format-specific structs**: Each format has its own representation  
✅ **From trait conversions**: Idiomatic Rust type conversions  
✅ **Static polymorphism**: Monomorphization ensures only used code is compiled  
✅ **No runtime overhead**: Generic functions compiled to specific implementations  
✅ **Extensible**: Add new formats by implementing from_read/write_to methods  
✅ **Library agnostic**: Works with files, stdin, stdout, buffers, network streams  

---

## 5. Data Model

### Core Data Structures

**Format-specific structures** (Mt940, Camt053, CsvStatement) share the same field structure:

```rust
use serde::{Deserialize, Serialize};

/// MT940 SWIFT message (example - similar structure for Camt053 and CsvStatement)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mt940 {
    pub account_number: String,
    pub currency: String,              // "USD", "EUR", "DKK", "RUB", etc.
    
    // Opening balance (starting position)
    pub opening_balance: f64,
    pub opening_date: DateTime<FixedOffset>,          // ISO 8601: "YYYY-MM-DD"
    pub opening_indicator: BalanceType, // Credit or Debit
    
    // Closing balance (ending position)
    pub closing_balance: f64,
    pub closing_date: DateTime<FixedOffset>,          // ISO 8601: "YYYY-MM-DD"
    pub closing_indicator: BalanceType, // Credit or Debit
    
    pub transactions: Vec<Transaction>,
}

impl Mt940 {
    /// Parse from any Read source (file, stdin, buffer)
    pub fn from_read<R: std::io::Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Implementation
    }
    
    /// Write to any Write destination (file, stdout, buffer)
    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Implementation
    }
}

// Similar structures for Camt053 and CsvStatement with identical fields
// This enables seamless From trait conversions

/// Balance type indicator (credit or debit position)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BalanceType {
    Credit,  // Positive balance (CRDT in CAMT, C in MT940)
    Debit,   // Negative balance (DBIT in CAMT, D in MT940)
}

/// Individual transaction entry (shared across all formats)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub booking_date: DateTime<FixedOffset>,           // ISO 8601: "YYYY-MM-DD" (when booked)
    pub value_date: Option<String>,     // ISO 8601: "YYYY-MM-DD" (value date, optional)
    pub amount: f64,                    // Always positive number
    pub transaction_type: TransactionType, // Credit or Debit
    pub description: String,            // Transaction description/narrative
    pub reference: Option<String>,      // Optional reference/transaction ID
    pub counterparty_name: Option<String>, // Debtor/Creditor name
    pub counterparty_account: Option<String>, // Counterparty account/IBAN
}

/// Transaction type (credit/debit indicator)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionType {
    Credit,  // Money received (CRDT in CAMT, C in MT940)
    Debit,   // Money paid (DBIT in CAMT, D in MT940)
}
```

### Design Rationale

**Why separate structures with identical fields?**
- Each format is a distinct type in the Rust type system
- Enables type-safe conversions using `From` trait
- Makes format origin explicit in function signatures
- Allows format-specific methods and behavior
- Demonstrates Rust's newtype pattern for domain modeling

**Conversion between formats:**
Since all three structures (Mt940, Camt053, CsvStatement) have identical fields, implementing `From` trait is straightforward field-by-field copying.

### Field Mapping Across Formats

**Statement-level fields:**

| Field | CSV | MT940 | CAMT.053 |
|-------|-----|-------|----------|
| account_number | Account column | `:25:` tag | `<Acct><Id><IBAN>` or `<Othr><Id>` |
| currency | Derived from amounts | `:60F:` (3 chars, e.g. "USD") | `<Amt Ccy="XXX">` attribute |
| opening_balance | "Входящий остаток" row | `:60F:` amount (after C/D) | `<Bal><Tp><Cd>OPBD</Cd>` |
| opening_date | Statement period start | `:60F:` date (YYMMDD) | `<Bal><Tp><Cd>OPBD</Cd><Dt>` |
| opening_indicator | Derived from amount sign | `:60F:` first char (C/D) | `<Bal><CdtDbtInd>CRDT/DBIT` |
| closing_balance | "Исходящий остаток" row | `:62F:` amount | `<Bal><Tp><Cd>CLBD</Cd>` |
| closing_date | Statement period end | `:62F:` date | `<Bal><Tp><Cd>CLBD</Cd><Dt>` |
| closing_indicator | Derived from amount sign | `:62F:` first char (C/D) | `<Bal><CdtDbtInd>CRDT/DBIT` |

**Transaction-level fields:**

| Field | CSV | MT940 | CAMT.053 |
|-------|-----|-------|----------|
| booking_date | "Дата проводки" | `:61:` date (YYMMDD) | `<BookgDt><Dt>` |
| value_date | Same as booking | `:61:` date (often same) | `<ValDt><Dt>` |
| amount | "Сумма по дебету/кредиту" | `:61:` amount | `<Amt Ccy="XXX">` |
| transaction_type | Column (debit/credit split) | `:61:` C/D indicator | `<CdtDbtInd>CRDT/DBIT` |
| description | "Назначение платежа" | `:86:` information | `<RmtInf><Ustrd>` or `<AddtlTxInf>` |
| reference | "№ документа" | `:61:` reference field | `<NtryRef>` or `<TxId>` |
| counterparty_name | Extracted from description | `:86:` (not standardized) | `<RltdPties><Dbtr><Nm>` or `<Cdtr><Nm>` |
| counterparty_account | Debit/Credit account | `:86:` (not standardized) | `<DbtrAcct><Id>` or `<CdtrAcct><Id>` |

### Design Decisions
✅ **Dates as strings** - ISO 8601 format ("YYYY-MM-DD"), converted from YYMMDD (MT940) and XML dates (CAMT.053)  
✅ **Unsigned amounts with type indicators** - Amount always positive, separate Credit/Debit enum  
✅ **Balance indicators** - Explicit Credit/Debit enum for opening/closing balances  
✅ **Statement period** - Opening/closing dates define range  
✅ **Optional fields** - value_date, reference, counterparty info (not always present)  
✅ **Serde integration** - All data structures derive Serialize/Deserialize for interoperability  
✅ **Derive traits** - Debug, Clone, PartialEq for testing + Serialize/Deserialize for data exchange  

### Format-Specific Parsing Challenges

**CSV:**
- Multi-line headers and footers (metadata rows)
- Localized column names (Russian in example)
- Split debit/credit columns (need to merge into single transaction type)
- Summary rows to skip ("Итого оборотов", "Количество операций")

**MT940:**
- SWIFT message envelope (`{1:...}{2:...}{4:...}`)
- Tag-based structure (`:20:`, `:25:`, `:61:`, etc.)
- Date format: YYMMDD (need century inference: 20YY for dates)
- Balance format: C/D + YYMMDD + CCY + amount (e.g., "C250218USD2732398848,02")
- Transaction line `:61:`: date + C/D + amount + type code + reference
- Multi-line `:86:` fields (continuation lines)

**CAMT.053 (XML):**
- ISO 20022 schema (complex nested structure)
- Multiple balance types: OPBD (opening booked), CLBD (closing booked), OPAV (opening available), CLAV (closing available)
- Entries (`<Ntry>`) can contain batched transactions (`<TxDtls>`)
- Remittance info can be structured (`<Strd>`) or unstructured (`<Ustrd>`)
- Counterparty info: Debtor for credits, Creditor for debits
- Multiple reference fields: `<NtryRef>`, `<EndToEndId>`, `<TxId>`, `<AcctSvcRef>`

### Simplifications
- **No timezone handling** - Dates as strings, no time-of-day
- **No decimal precision types** - f64 sufficient for tutorial (production would use `rust_decimal`)
- **Single balance per type** - Ignore available balance (OPAV/CLAV), use only booked (OPBD/CLBD)
- **Flatten batches** - CAMT.053 batched transactions treated as separate entries
- **Basic XML parsing** - Manual string parsing, no full XML library
- **No currency conversion** - Amounts in original currency
- **Century assumption** - MT940 dates: 00-49 → 2000-2049, 50-99 → 1950-1999

---

## 6. Format Specifications (from Real Examples)

### CSV Format Structure

Based on the Russian Sberbank example:

```
Header rows (metadata):
- Line 1: Empty or separators
- Line 2: Report title and version
- Line 3: Bank name
- Line 4-8: Statement metadata (dates, account info)

Data section:
- Line 11: Column headers
  Columns: Date | Debit Account | Credit Account | Debit Amount | Credit Amount | Doc# | VO | BIC/Bank | Description
- Lines 13+: Transaction rows

Footer rows (summary):
- Blank line
- "б/с" (summary header)
- Transaction count row
- Opening balance row ("Входящий остаток")
- Totals row ("Итого оборотов")  
- Closing balance row ("Исходящий остаток")
```

**Key observations:**
- Debit and credit are in separate columns (not merged)
- Account numbers include INN (tax ID) on separate line
- Amounts use Russian decimal separator (comma)
- Multi-line cells (account info spans 2 lines)

### MT940 Format Structure

Based on Goldman Sachs and other bank examples:

```
{1:F01BANKCODEXXX...}     Block 1: Basic header (sender BIC)
{2:I940...}               Block 2: Application header (message type)
{4:                       Block 4: Text block (actual data)
:20:Reference             Transaction reference
:25:AccountNumber         Account identification
:28C:SequenceNo/Page      Statement number/sequence
:60F:C/D+Date+CCY+Amt     Opening balance (F=final, M=intermediate)
:61:Date+C/D+Amt+Type+Ref Statement line (transaction)
:86:Description           Transaction details (multi-line)
:62F:C/D+Date+CCY+Amt     Closing balance
:64:C/D+Date+CCY+Amt      Available balance (optional)
-}                        End of block 4
```

**Key observations:**
- Tags can repeat (multiple `:61:` and `:86:` pairs)
- `:86:` can span multiple lines
- Balance format: indicator + date (YYMMDD) + currency (3 chars) + amount
- Transaction line `:61:`: ValDate + BookDate (optional) + C/D + amount + type + ref
- Some MT940 variants omit blocks {1} and {2}

**Balance line example:** `:60M:C250218USD2732398848,02`
- `M` = intermediate (F = final)
- `C` = credit
- `250218` = Feb 18, 2025
- `USD` = currency
- `2732398848,02` = amount

**Transaction line example:** `:61:2502180218D12,01NTRFGSLNVSHSUTKWDR//GI2504900007841`
- `250218` = value date
- `0218` = booking date
- `D` = debit
- `12,01` = amount
- `NTRF` = transaction type
- `GSLNVSHSUTKWDR//GI2504900007841` = reference

### CAMT.053 XML Format Structure

Based on Danske Bank example (ISO 20022 standard):

```xml
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
  <BkToCstmrStmt>                    <!-- Bank to Customer Statement -->
    <GrpHdr>                          <!-- Group Header -->
      <MsgId>...</MsgId>
      <CreDtTm>2023-04-20T23:24:31</CreDtTm>
    </GrpHdr>
    <Stmt>                            <!-- Statement -->
      <Id>...</Id>
      <Acct>                          <!-- Account -->
        <Id><IBAN>DK8030000001234567</IBAN></Id>
        <Ccy>DKK</Ccy>
        <Nm>Account name</Nm>
        <Ownr>...</Ownr>
      </Acct>
      <Bal>                           <!-- Balance (multiple) -->
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>  <!-- Type: OPBD, CLBD, OPAV, CLAV -->
        <Amt Ccy="DKK">12345.67</Amt>
        <CdtDbtInd>DBIT</CdtDbtInd>   <!-- CRDT or DBIT -->
        <Dt><Dt>2023-04-20</Dt></Dt>
      </Bal>
      <Ntry>                          <!-- Entry (transaction) -->
        <NtryRef>1</NtryRef>
        <Amt Ccy="DKK">591.15</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt><Dt>2023-04-20</Dt></BookgDt>
        <ValDt><Dt>2023-04-20</Dt></ValDt>
        <NtryDtls>
          <TxDtls>                    <!-- Transaction Details -->
            <Refs>
              <EndToEndId>...</EndToEndId>
              <TxId>...</TxId>
            </Refs>
            <RltdPties>               <!-- Related Parties -->
              <Dbtr><Nm>Debtor name</Nm></Dbtr>
              <DbtrAcct><Id><IBAN>...</IBAN></Id></DbtrAcct>
              <Cdtr><Nm>Creditor name</Nm></Cdtr>
              <CdtrAcct><Id><IBAN>...</IBAN></Id></CdtrAcct>
            </RltdPties>
            <RmtInf>                  <!-- Remittance Information -->
              <Ustrd>Payment description</Ustrd>
            </RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>
```

**Key observations:**
- Balance types: OPBD (opening booked), CLBD (closing booked), OPAV (opening available), CLAV (closing available), PRCD (previous closing)
- One `<Ntry>` can contain multiple `<TxDtls>` (batched transactions)
- Counterparty naming: Debtor for incoming, Creditor for outgoing
- Dates in ISO format: `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SS`
- Currency as attribute: `<Amt Ccy="DKK">`
- Remittance info can be structured (`<Strd>`) or unstructured (`<Ustrd>`)

**Balance type code meanings:**
- `OPBD` - Opening Balance (Booked)
- `CLBD` - Closing Balance (Booked)
- `OPAV` - Opening Available Balance
- `CLAV` - Closing Available Balance
- `PRCD` - Previous Closing Date

### Implementation Tips from Real Examples

**CSV parsing tips:**
1. **Identify transaction rows**: Look for lines with both date and amount fields populated
2. **Skip summary rows**: Check for keywords like "Итого", "Количество", "Входящий", "Исходящий"
3. **Handle multi-line cells**: Account info often spans 2+ lines (INN on second line)
4. **Decimal separator**: Russian banks use comma, Western banks use dot
5. **Extract account from debit/credit**: Statement account appears in either debit OR credit column consistently

**MT940 parsing tips:**
1. **Block 4 focus**: All useful data is in `{4:...}` block, can ignore blocks 1-3
2. **Tag association**: Each `:61:` (transaction) is followed by `:86:` (description)
3. **Multi-line handling**: Lines not starting with `:` are continuations of previous field
4. **Century inference**: If YY >= 50, use 19YY; otherwise use 20YY
5. **Amount format**: Both comma and dot can be decimal separators (bank-dependent)
6. **Transaction line parsing**: Format varies by bank; split on known markers (C/D, amount pattern)

**CAMT.053 parsing tips:**
1. **Balance extraction**: Find all `<Bal>` elements, filter by `<Tp><Cd>` (use OPBD/CLBD)
2. **Entry iteration**: Each `<Ntry>` is a statement line, may contain multiple `<TxDtls>`
3. **Counterparty logic**: For CRDT transactions, use `<Dbtr>`; for DBIT, use `<Cdtr>`
4. **Remittance info**: Try `<Ustrd>` first, fall back to `<Strd><CdtrRefInf><Ref>`
5. **Namespace handling**: Match tags with or without namespace prefixes
6. **IBAN extraction**: Look in `<Id><IBAN>` or `<Id><Othr><Id>` for non-IBAN accounts
7. **Attribute parsing**: Currency in `Ccy="XXX"` attribute of `<Amt>` tag

**Date handling (all formats):**
- MT940: `YYMMDD` → apply century rule, format as `YYYY-MM-DD`
- CAMT: May include time (`2023-04-20T23:24:31`) → extract date part only
- CSV: Already in readable format, may need normalization

**Amount handling (all formats):**
- Parse both comma and dot as decimal separator
- Remove thousands separators (spaces, commas)
- Store as f64 (sufficient for tutorial; production should use decimal types)

---

## 7. Error Handling

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
use std::io::Read;

// Format structs implement from_read and use ParseError
impl Mt940 {
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let mut content = String::new();
        
        // Read all content
        reader.read_to_string(&mut content)
            .map_err(|e| ParseError::IoError(e.to_string()))?;
        
        // Check if empty
        if content.is_empty() {
            return Err(ParseError::Mt940Error("Empty input".to_string()));
        }
        
        // ... parsing logic using content string
        Ok(Mt940 { /* ... */ })
    }
}
```

### CLI Error Handling

```rust
use std::io::Read;
use std::fs::File;

// CLI displays errors and exits gracefully
// Example: reading from file
let mut file = File::open("input.mt940").unwrap_or_else(|e| {
    eprintln!("Failed to open file: {}", e);
    std::process::exit(1);
});

match Mt940::from_read(&mut file) {
    Ok(mt940) => { /* process */ },
    Err(e) => {
        eprintln!("Parse error: {}", e);
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

## 8. Testing Strategy

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
    use crate::model::{CsvStatement, Transaction};
    
    #[test]
    fn test_parse_csv() {
        let input = "Account,Currency,...\nACC001,USD,...";
        let mut reader = input.as_bytes();
        let result = CsvStatement::from_read(&mut reader);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_write_csv() {
        let statement = CsvStatement { /* ... */ };
        let mut output = Vec::new();
        let result = statement.write_to(&mut output);
        assert!(result.is_ok());
        assert!(!output.is_empty());
    }
    
    #[test]
    fn test_parse_error() {
        let input = "";
        let mut reader = input.as_bytes();
        let result = CsvStatement::from_read(&mut reader);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_from_bytes() {
        // Read works with any byte source
        let data: &[u8] = b"Account,Currency\nACC001,USD";
        let mut reader = data;
        let result = CsvStatement::from_read(&mut reader);
        assert!(result.is_ok());
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
    let mut reader = csv_input.as_bytes();
    
    // Parse CSV
    let csv_statement = CsvStatement::from_read(&mut reader).unwrap();
    
    // Convert to MT940 using From trait
    let mt940: Mt940 = csv_statement.into();
    
    // Write MT940
    let mut output = Vec::new();
    mt940.write_to(&mut output).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_mt940_to_camt053_conversion() {
    let mt940_input = "...";
    let mut reader = mt940_input.as_bytes();
    
    // Parse MT940
    let mt940 = Mt940::from_read(&mut reader).unwrap();
    
    // Convert to CAMT.053 using From trait
    let camt053: Camt053 = mt940.into();
    
    // Write CAMT.053
    let mut output = Vec::new();
    camt053.write_to(&mut output).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_round_trip_conversion() {
    // Test MT940 -> CAMT.053 -> MT940
    let original_mt940 = "...";
    let mut reader = original_mt940.as_bytes();
    
    let mt940_original = Mt940::from_read(&mut reader).unwrap();
    let camt053: Camt053 = mt940_original.clone().into();
    let mt940_converted: Mt940 = camt053.into();
    
    // Verify data integrity
    assert_eq!(mt940_original.account_number, mt940_converted.account_number);
    assert_eq!(mt940_original.currency, mt940_converted.currency);
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

## 9. CLI Workflow

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
use std::io::{self, Read, Write};
use std::fs::File;

fn main() {
    let cli = Cli::parse();
    
    // 1. Create Read source (file or stdin)
    let mut input: Box<dyn Read> = match &cli.input {
        Some(path) => {
            let file = File::open(path).unwrap_or_else(|e| {
                eprintln!("Failed to open input file: {}", e);
                std::process::exit(1);
            });
            Box::new(file)
        },
        None => {
            // Read from stdin
            Box::new(io::stdin())
        }
    };
    
    // 2. Parse based on --in-format (case-insensitive)
    // Note: This is a simplified example. In practice, you'd need an enum or trait object
    // to handle different return types. See implementation notes below.
    let (mt940_opt, camt053_opt, csv_opt) = match cli.in_format.to_lowercase().as_str() {
        "mt940" => {
            let mt940 = Mt940::from_read(&mut input).unwrap_or_else(|e| {
                eprintln!("Parse error: {}", e);
                std::process::exit(1);
            });
            (Some(mt940), None, None)
        },
        "camt053" => {
            let camt = Camt053::from_read(&mut input).unwrap_or_else(|e| {
                eprintln!("Parse error: {}", e);
                std::process::exit(1);
            });
            (None, Some(camt), None)
        },
        "csv" => {
            let csv = CsvStatement::from_read(&mut input).unwrap_or_else(|e| {
                eprintln!("Parse error: {}", e);
                std::process::exit(1);
            });
            (None, None, Some(csv))
        },
        _ => {
            eprintln!("Unknown input format: {}", cli.in_format);
            std::process::exit(1);
        }
    };
    
    // 3. Create Write destination (file or stdout)
    let mut output: Box<dyn Write> = match &cli.output {
        Some(path) => {
            let file = File::create(path).unwrap_or_else(|e| {
                eprintln!("Failed to create output file: {}", e);
                std::process::exit(1);
            });
            Box::new(file)
        },
        None => Box::new(io::stdout()),
    };
    
    // 4. Convert and write based on --out-format
    match cli.out_format.to_lowercase().as_str() {
        "mt940" => {
            let mt940 = if let Some(m) = mt940_opt {
                m
            } else if let Some(c) = camt053_opt {
                c.into()
            } else if let Some(csv) = csv_opt {
                csv.into()
            } else {
                unreachable!()
            };
            mt940.write_to(&mut output).unwrap_or_else(|e| {
                eprintln!("Write error: {}", e);
                std::process::exit(1);
            });
        },
        "camt053" => {
            let camt053 = if let Some(c) = camt053_opt {
                c
            } else if let Some(m) = mt940_opt {
                m.into()
            } else if let Some(csv) = csv_opt {
                csv.into()
            } else {
                unreachable!()
            };
            camt053.write_to(&mut output).unwrap_or_else(|e| {
                eprintln!("Write error: {}", e);
                std::process::exit(1);
            });
        },
        "csv" => {
            let csv = if let Some(csv) = csv_opt {
                csv
            } else if let Some(m) = mt940_opt {
                m.into()
            } else if let Some(c) = camt053_opt {
                c.into()
            } else {
                unreachable!()
            };
            csv.write_to(&mut output).unwrap_or_else(|e| {
                eprintln!("Write error: {}", e);
                std::process::exit(1);
            });
        },
        _ => {
            eprintln!("Unknown output format: {}", cli.out_format);
            std::process::exit(1);
        }
    }
}
```

**Implementation Note:** The above code can be simplified using an enum to hold any of the three format types, or by introducing a common trait. The key learning points are:
1. **Read trait** enables reading from files or stdin without code duplication
2. **Write trait** enables writing to files or stdout without code duplication
3. **From trait** enables type-safe conversions between formats
4. **Static dispatch** means no runtime overhead for generic code

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

**Ledger Bridge** is designed as a learning-focused Rust project that demonstrates **standard library I/O traits** (`Read`/`Write`) and **type conversions** (`From` trait) through practical financial data parsing.

**Note:** This vision document has been refined based on:
1. Analysis of real-world bank statement files:
   - **CAMT.053**: Danske Bank (Denmark) and Treasurease examples
   - **MT940**: Goldman Sachs, ASN Bank (Netherlands), and other international banks
   - **CSV**: Sberbank (Russia) with localized format
2. Educational requirements emphasizing standard library trait usage
3. Third-party library research for optimal parsing solutions

All field mappings, format structures, and parsing strategies are based on actual production data.

#### Key Design Decisions

✅ **Read/Write traits** - All parsers use `std::io::Read`, all formatters use `std::io::Write`  
✅ **Format-specific structures** - Mt940, Camt053, CsvStatement (not unified model)  
✅ **From trait conversions** - Idiomatic Rust type conversions between formats  
✅ **Method-based API** - `from_read()` and `write_to()` methods on structs  
✅ **Recommended libraries**:
  - **csv** crate (Trust Score: 9.1) - CSV parsing with Read/Write support
  - **quick-xml** crate (Trust Score: 9.2) - XML parsing with event-based API
  - **Manual parsing** for MT940 - Educational value, demonstrates text processing
✅ **Serde integration** - All data structures derive Serialize/Deserialize  
✅ **Simple error handling** - One ParseError type  
✅ **Static polymorphism** - Generic methods monomorphized at compile time  
✅ **Clean CLI** - Demonstrates Read/Write abstraction benefits  
✅ **KISS principle** throughout

#### Learning Objectives

This project teaches:
1. **Standard library I/O traits** - How `Read` and `Write` enable code reuse
2. **Type conversions** - Implementing `From` trait for domain type conversions
3. **Static polymorphism** - Generic functions without runtime overhead
4. **Practical parsing** - Real-world financial data formats
5. **Error handling** - Custom error types with `std::error::Error` trait
6. **Third-party integration** - Using established crates effectively

#### Project Deliverables

1. **`ledger-parser` library** - Reusable parsing/formatting engine
   - Mt940 struct with `from_read()` and `write_to()` methods
   - Camt053 struct with `from_read()` and `write_to()` methods
   - CsvStatement struct with `from_read()` and `write_to()` methods
   - `From` trait implementations for all conversion pairs
   - Shared Transaction and BalanceType types

2. **`ledger-bridge-cli` binary** - Command-line conversion tool
   - Reads from file or stdin (via `Read` trait)
   - Writes to file or stdout (via `Write` trait)
   - Converts between any supported formats
   - Demonstrates standard library trait benefits

3. **Documentation** - All public APIs documented
4. **Tests** - Unit and integration test coverage
5. **Format support** - CSV, MT940, CAMT.053 (bidirectional)

#### Next Steps

With this technical vision established, we're ready to:
1. Set up the Cargo workspace
2. Define shared types (Transaction, BalanceType, TransactionType)
3. Implement format-specific structures (Mt940, Camt053, CsvStatement)
4. Implement `from_read()` and `write_to()` methods
5. Implement `From` trait conversions
6. Build the CLI application
7. Write tests
8. Document the code

This vision serves as our blueprint for development. All decisions prioritize **learning standard library patterns** and **idiomatic Rust** over production-grade features.

