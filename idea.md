# Ledger Bridge - Financial Data Parser

## Project Overview

A Rust-based library and CLI application for parsing, serializing, deserializing, and converting financial data across multiple formats.

## Learning Objectives

- Gain practical skills in trait-based static polymorphism
- Implement multiple parsers for different data types
- Create idiomatic multi-module Rust applications
- Master Rust's type system and abstraction capabilities

## Project Structure

### Library Crate (`ledger-parser`)
Core parsing/serialization library with format support

### CLI Crate (`ledger-bridge-cli`)
Command-line interface for format conversion

## Supported Formats

1. **CSV** - Standard bank/accounting exports
2. **MT940** - SWIFT-like bank statements
3. **CAMT.053** - ISO 20022 XML format for bank-to-customer statements

## Technical Requirements

### General Requirements

- ✅ Data conversion between formats using standard library traits
- ✅ Code hosted on GitHub
- ✅ Parser implementations use language abstractions (traits) for flexibility
- ✅ Parsers not directly tied to specific data types
- ✅ Separate parser crate from executable crates
- ✅ No `unwrap()` usage in production code
- ✅ Clear error types with proper error handling
- ✅ Public API fully documented with doc comments
- ✅ Comprehensive test coverage

### Functional Requirements

#### Parser Library Must:

- Read data in CSV, MT940, and CAMT.053 formats
- Write data in CSV, MT940, and CAMT.053 formats
- Convert MT940 to CAMT.053 (ISO 20022)
- Return descriptive errors during parsing failures

#### CLI Application Must:

- Import and use the parser library
- Read from file or stdin
- Write to file or stdout
- Support `--in-format` flag (csv, mt940, camt053)
- Support `--out-format` flag (csv, mt940, camt053)
- Implement all parser crate functionality

## Architecture Approach

### Trait-Based Design

Use Rust traits to define parsing contracts:
- Generic parser trait
- Format-specific implementations
- Standard library trait usage (From, TryFrom, etc.)

### Error Handling

- Custom error types using `thiserror` or similar
- Result types throughout
- Descriptive error messages

### Data Model

- Unified internal representation of financial transactions
- Format-agnostic transaction structure
- Conversion traits between formats

## Success Criteria

- All three formats can be read and written
- MT940 → CAMT.053 conversion works correctly
- CLI provides seamless format conversion
- Code is well-tested and documented
- Best practices followed throughout

