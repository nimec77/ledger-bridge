# Ledger Bridge

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust library and CLI tool for parsing and converting financial data between CSV, MT940, and CAMT.053 bank statement formats.

## 🎯 Overview

**Ledger Bridge** provides seamless conversion between three common bank statement formats:

- **CSV** - Comma-separated values format (e.g., Sberbank export)
- **MT940** - SWIFT MT940 message format (international banking standard)
- **CAMT.053** - ISO 20022 XML format (modern banking standard)

The project demonstrates idiomatic Rust patterns including:
- Standard library I/O traits (`Read`/`Write`)
- Type conversions with the `From` trait
- Static polymorphism through generics
- Comprehensive error handling without panics

## ✨ Features

- **📖 Parse multiple formats**: Read CSV, MT940, and CAMT.053 statements
- **✍️ Write to any format**: Convert and export to any supported format
- **🔄 Bidirectional conversions**: Use Rust's `From` trait for type-safe conversions
- **💾 Flexible I/O**: Works with files, stdin/stdout, in-memory buffers, or any `Read`/`Write` source
- **🔒 No panics**: All errors returned explicitly through `Result` types
- **🧪 Well tested**: 64 tests covering parsing, writing, and conversions
- **📚 Comprehensive docs**: Full API documentation with examples

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ledger-parser = { path = "path/to/ledger-parser" }
```

Or install the CLI tool:

```bash
cargo install --path ledger-bridge-cli
```

### Library Usage

```rust
use ledger_parser::{Mt940Statement, Camt053Statement};
use std::fs::File;

// Parse MT940 from file
let mut input = File::open("statement.mt940")?;
let mt940 = Mt940Statement::from_read(&mut input)?;

// Convert to CAMT.053
let camt053: Camt053Statement = mt940.into();

// Write as XML
let mut output = File::create("output.xml")?;
camt053.write_to(&mut output)?;
```

### CLI Usage

```bash
# Convert CSV to MT940
ledger-bridge --in-format csv --out-format mt940 --input statement.csv --output statement.mt940

# Convert MT940 to CAMT.053 XML
ledger-bridge --in-format mt940 --out-format camt053 --input data.mt940 --output data.xml

# Use stdin/stdout
cat input.csv | ledger-bridge --in-format csv --out-format mt940 > output.mt940

# Display help
ledger-bridge --help
```

## 📦 Project Structure

This project is organized as a Cargo workspace with two crates:

```
ledger-bridge/
├── ledger-parser/          # Core library (parsing and conversion)
│   ├── src/
│   │   ├── lib.rs          # Public API
│   │   ├── model.rs        # Shared data types
│   │   ├── error.rs        # Error handling
│   │   └── formats/        # Format implementations
│   └── tests/              # Integration tests
│
└── ledger-bridge-cli/      # CLI application
    └── src/
        └── main.rs         # Command-line interface
```

## 🔧 Supported Formats

### CSV Format
- Russian Sberbank CSV export format
- Multi-line headers and footers
- Separate debit/credit columns
- Russian text and comma decimal separators

### MT940 Format
- SWIFT MT940 message structure
- Block-based format with tags (`:20:`, `:25:`, `:60F:`, `:61:`, `:86:`, `:62F:`)
- YYMMDD date format with century inference
- Support for multi-line transaction descriptions

### CAMT.053 Format
- ISO 20022 XML standard
- Full namespace support
- Balance types: OPBD (opening booked), CLBD (closing booked)
- Transaction entries with counterparty information

## 🧪 Testing

Run the full test suite:

```bash
# Run all tests
cargo test --all

# Run with output
cargo test --all -- --nocapture

# Run specific format tests
cargo test csv
cargo test mt940
cargo test camt053

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check
```

**Test Coverage**: 64 tests (47 unit + 17 integration)

## 📖 Documentation

Generate and view the API documentation:

```bash
cargo doc --no-deps --open
```

Documentation includes:
- Comprehensive API reference
- Usage examples for all public types
- Format specifications and field mappings
- Error handling patterns

## 🎓 Learning Objectives

This project is designed to demonstrate:

1. **Standard library I/O traits** - How `Read` and `Write` enable code reuse
2. **Type conversions** - Implementing `From` trait for domain type conversions
3. **Static polymorphism** - Generic functions without runtime overhead
4. **Practical parsing** - Real-world financial data formats
5. **Error handling** - Custom error types with `std::error::Error` trait
6. **Third-party integration** - Using `csv` and `quick-xml` crates effectively

## 🛠️ Development

### Prerequisites

- Rust 2021 edition or later
- Cargo

### Build

```bash
# Build library and CLI
cargo build

# Build with optimizations
cargo build --release

# Check without building
cargo check
```

### Code Quality

```bash
# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

## 🤝 Contributing

This is an educational project focused on learning Rust standard library patterns. While primarily for learning, improvements and suggestions are welcome!

## 📄 License

This project is available under the MIT License.

## 🙏 Acknowledgments

Real-world example files used for testing:
- **CAMT.053**: Danske Bank and Treasurease examples
- **MT940**: Goldman Sachs, ASN Bank samples
- **CSV**: Sberbank export format

See `example_files/sources.md` for detailed attribution.

## 📚 Additional Resources

- [SWIFT MT940 Specification](https://www.swift.com/)
- [ISO 20022 CAMT.053 Standard](https://www.iso20022.org/)
- [Rust I/O Traits Documentation](https://doc.rust-lang.org/std/io/)
- [Project Vision Document](vision.md)
- [Development Tasklist](doc/tasklist.md)

