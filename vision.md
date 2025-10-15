# Technical Vision - Ledger Bridge

> A KISS-focused technical blueprint for learning trait-based polymorphism in Rust

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

**Manual parsing approach maintained:**
- MT940 and CAMT.053 parsers still implemented manually (string processing)
- CSV parsing done manually (learning exercise)
- Serde used only for data structure traits, not for parsing logic

**CSV Parsing:**
- Line-by-line processing using `str::lines()`
- Field splitting on commas (handle quoted fields if needed)
- Skip metadata rows (headers/footers)
- Detect transaction rows vs. summary rows
- Handle localized column names (pattern matching)
- Parse split debit/credit columns into unified transaction type

**MT940 Parsing:**
- Block extraction (`{1:...}{2:...}{4:...}`)
- Tag-based parsing (`:20:`, `:25:`, `:60F:`, `:61:`, `:86:`, `:62F:`)
- Multi-line field handling (`:86:` can span lines)
- Date parsing: YYMMDD → YYYY-MM-DD (century inference)
- Balance parsing: C/D indicator + date + currency + amount
- Transaction line parsing: complex format with embedded delimiters
- Amount parsing: handle comma as decimal separator

**CAMT.053 XML Parsing:**
- Simple pattern matching (no full XML parser)
- Extract text between XML tags using `str::find()` and string slicing
- Handle namespaces in tag matching
- Parse nested structures (entries contain transaction details)
- Extract attributes (`Ccy="XXX"`)
- Handle multiple elements of same type (multiple `<Bal>` and `<Ntry>`)
- Filter balance types (use OPBD/CLBD, ignore OPAV/CLAV)

**Common Utilities:**
- Date parsing: ISO 8601 (YYYY-MM-DD) and YYMMDD formats
- Amount parsing: handle both comma and dot as decimal separators
- String trimming and normalization
- Error implementation (`std::error::Error` trait)

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
use serde::{Deserialize, Serialize};

/// Bank statement containing transactions and balances
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statement {
    pub account_number: String,
    pub currency: String,              // "USD", "EUR", "DKK", "RUB", etc.
    
    // Opening balance (starting position)
    pub opening_balance: f64,
    pub opening_date: String,          // ISO 8601: "YYYY-MM-DD"
    pub opening_indicator: BalanceType, // Credit or Debit
    
    // Closing balance (ending position)
    pub closing_balance: f64,
    pub closing_date: String,          // ISO 8601: "YYYY-MM-DD"
    pub closing_indicator: BalanceType, // Credit or Debit
    
    pub transactions: Vec<Transaction>,
}

/// Balance type indicator (credit or debit position)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BalanceType {
    Credit,  // Positive balance (CRDT in CAMT, C in MT940)
    Debit,   // Negative balance (DBIT in CAMT, D in MT940)
}

/// Individual transaction entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub booking_date: String,           // ISO 8601: "YYYY-MM-DD" (when booked)
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

**Note:** This vision document has been refined based on analysis of real-world bank statement files:
- **CAMT.053**: Danske Bank (Denmark) and Treasurease examples
- **MT940**: Goldman Sachs, ASN Bank (Netherlands), and other international banks
- **CSV**: Sberbank (Russia) with localized format

All field mappings, format structures, and parsing strategies are based on actual production data.

#### Key Design Decisions

✅ **Minimal dependencies** - Only serde for data structures (manual parsing maintained)  
✅ **Unified data model** (single Statement type with Serialize/Deserialize)  
✅ **Generic traits** (not tied to specific types)  
✅ **Simple error handling** (one ParseError type)  
✅ **Manual parsing** - All format parsers hand-written for learning  
✅ **Serde integration** - Data structures serializable for future extensibility  
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

