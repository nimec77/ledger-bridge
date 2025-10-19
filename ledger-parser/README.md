# Ledger Parser

Core library for parsing and converting bank statement formats.

## Overview

The `ledger-parser` library provides parsing and formatting capabilities for three common bank statement formats:

- **CSV** - Comma-separated values format
- **MT940** - SWIFT MT940 message format
- **CAMT.053** - ISO 20022 XML format

All formats support bidirectional conversion using Rust's `From` trait.

## Features

- **Standard library I/O**: All parsers and formatters work with `std::io::Read` and `std::io::Write`
- **Format conversions**: Seamless conversion between formats using `From` trait
- **Unified data model**: Shared `Transaction` and balance types across all formats
- **No panics**: All errors returned explicitly through `Result` types
- **Serde integration**: All types support serialization/deserialization
- **Well tested**: Comprehensive unit and integration tests

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ledger-parser = { path = "path/to/ledger-parser" }
```

## Quick Start

### Parsing a Statement

```rust
use ledger_parser::Mt940Statement;
use std::fs::File;

// Parse MT940 from file
let mut file = File::open("statement.mt940")?;
let statement = Mt940Statement::from_read(&mut file)?;

println!("Account: {}", statement.account_number);
println!("Balance: {} {}", statement.closing_balance, statement.currency);
println!("Transactions: {}", statement.transactions.len());
```

### Converting Between Formats

```rust
use ledger_parser::{Mt940Statement, Camt053Statement};
use std::fs::File;

// Read MT940
let mut input = File::open("input.mt940")?;
let mt940 = Mt940Statement::from_read(&mut input)?;

// Convert to CAMT.053 using From trait
let camt053: Camt053Statement = mt940.into();

// Write as XML
let mut output = File::create("output.xml")?;
camt053.write_to(&mut output)?;
```

### Working with Transactions

```rust
use ledger_parser::{CsvStatement, TransactionType};
use std::fs::File;

let mut file = File::open("statement.csv")?;
let statement = CsvStatement::from_read(&mut file)?;

// Filter credit transactions
let credits: Vec<_> = statement.transactions.iter()
    .filter(|t| t.transaction_type == TransactionType::Credit)
    .collect();

println!("Found {} credit transactions", credits.len());

// Calculate total credits
let total: f64 = credits.iter()
    .map(|t| t.amount)
    .sum();

println!("Total credits: {:.2}", total);
```

### Using with In-Memory Data

The library works with any `Read`/`Write` source:

```rust
use ledger_parser::Mt940Statement;

// Parse from string
let data = r#"{4:
:20:STATEMENT
:25:12345678
:60F:C250101USD1000,00
:62F:C250131USD1500,00
-}"#;

let mut reader = data.as_bytes();
let statement = Mt940Statement::from_read(&mut reader)?;

// Write to Vec<u8> buffer
let mut output = Vec::new();
statement.write_to(&mut output)?;

let result_string = String::from_utf8(output)?;
println!("Generated MT940:\n{}", result_string);
```

## Format Support

Each format is represented by its own struct type:

### CsvStatement

```rust
pub struct CsvStatement {
    pub account_number: String,
    pub currency: String,
    pub opening_balance: f64,
    pub opening_date: DateTime<FixedOffset>,
    pub opening_indicator: BalanceType,
    pub closing_balance: f64,
    pub closing_date: DateTime<FixedOffset>,
    pub closing_indicator: BalanceType,
    pub transactions: Vec<Transaction>,
}
```

**Supported CSV format:**
- Russian Sberbank CSV export format
- Multi-line headers and footers
- Separate debit/credit columns

### Mt940Statement

```rust
pub struct Mt940Statement {
    pub account_number: String,
    pub currency: String,
    pub opening_balance: f64,
    pub opening_date: DateTime<FixedOffset>,
    pub opening_indicator: BalanceType,
    pub closing_balance: f64,
    pub closing_date: DateTime<FixedOffset>,
    pub closing_indicator: BalanceType,
    pub transactions: Vec<Transaction>,
}
```

**Supported MT940 features:**
- Block structure (`:1:`, `:2:`, `:4:`)
- Tag-based parsing (`:20:`, `:25:`, `:60F:`, `:61:`, `:86:`, `:62F:`)
- Multi-line `:86:` fields
- YYMMDD date format with century inference

### Camt053Statement

```rust
pub struct Camt053Statement {
    pub account_number: String,
    pub currency: String,
    pub opening_balance: f64,
    pub opening_date: DateTime<FixedOffset>,
    pub opening_indicator: BalanceType,
    pub closing_balance: f64,
    pub closing_date: DateTime<FixedOffset>,
    pub closing_indicator: BalanceType,
    pub transactions: Vec<Transaction>,
}
```

**Supported CAMT.053 features:**
- ISO 20022 XML parsing
- Balance types: OPBD (opening booked), CLBD (closing booked)
- Transaction entries with counterparty information
- Namespace support

## Shared Types

### Transaction

```rust
pub struct Transaction {
    pub booking_date: DateTime<FixedOffset>,
    pub value_date: Option<String>,
    pub amount: f64,
    pub transaction_type: TransactionType,
    pub description: String,
    pub reference: Option<String>,
    pub counterparty_name: Option<String>,
    pub counterparty_account: Option<String>,
}
```

### BalanceType

```rust
pub enum BalanceType {
    Credit,  // Positive balance
    Debit,   // Negative balance
}
```

### TransactionType

```rust
pub enum TransactionType {
    Credit,  // Money received
    Debit,   // Money paid out
}
```

## Error Handling

All operations return `Result<T, ParseError>`:

```rust
use ledger_parser::{Mt940Statement, ParseError};

let data = "invalid mt940 data";
let mut reader = data.as_bytes();

match Mt940Statement::from_read(&mut reader) {
    Ok(statement) => println!("Parsed successfully"),
    Err(ParseError::Mt940Error(msg)) => eprintln!("Parse error: {}", msg),
    Err(ParseError::IoError(msg)) => eprintln!("I/O error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

### ParseError Variants

- `InvalidFormat(String)` - Invalid or unsupported format
- `MissingField(String)` - Required field missing
- `InvalidFieldValue { field, value }` - Field value cannot be parsed
- `CsvError(String)` - CSV parsing error
- `Mt940Error(String)` - MT940 parsing error
- `Camt053Error(String)` - CAMT.053 XML parsing error
- `IoError(String)` - I/O operation error

## Format Conversions

All conversions are implemented using the `From` trait:

```rust
// MT940 ↔ CAMT.053
let camt: Camt053Statement = mt940.into();
let mt940: Mt940Statement = camt.into();

// CSV ↔ MT940
let mt940: Mt940Statement = csv.into();
let csv: CsvStatement = mt940.into();

// CSV ↔ CAMT.053
let camt: Camt053Statement = csv.into();
let csv: CsvStatement = camt.into();
```

Conversions are **lossless** - all fields are preserved during format conversion.

## Testing

Run the test suite:

```bash
# All tests
cargo test

# Specific format
cargo test csv
cargo test mt940
cargo test camt053

# Integration tests
cargo test --test integration_test
```

## Documentation

Generate API documentation:

```bash
cargo doc --no-deps --open
```

## Dependencies

- `serde` (1.0) - Serialization framework
- `csv` (1.3) - CSV parsing with Read/Write support
- `quick-xml` (0.31) - XML parsing for CAMT.053
- `chrono` (0.4) - Date and time handling

## License

MIT License - See LICENSE file for details.

## See Also

- [CLI Application](../ledger-bridge-cli/) - Command-line interface
- [Project README](../README.md) - Main project documentation
- [Vision Document](../vision.md) - Technical specifications
