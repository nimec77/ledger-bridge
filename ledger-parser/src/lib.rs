//! Ledger Bridge Parser Library
//!
//! A library for parsing and converting financial data between CSV, MT940, and CAMT.053 formats.
//!
//! # Overview
//!
//! This library provides parsing and formatting capabilities for three common bank statement formats:
//! - **CSV**: Comma-separated values format (e.g., Sberbank export format)
//! - **MT940**: SWIFT MT940 message format (international banking standard)
//! - **CAMT.053**: ISO 20022 XML format (modern banking standard)
//!
//! All formats can be converted bidirectionally using the Rust `From` trait.
//!
//! # Features
//!
//! - **Read/Write trait support**: All parsers and formatters work with `std::io::Read` and `std::io::Write`
//! - **Format conversions**: Seamless conversion between formats using `From` trait
//! - **Unified data model**: Shared `Transaction` and balance types across all formats
//! - **Error handling**: Comprehensive `ParseError` type with descriptive messages
//! - **Serde integration**: All types support serialization/deserialization
//!
//! # Quick Start
//!
//! ## Parsing a Statement
//!
//! ```no_run
//! use ledger_parser::Mt940Statement;
//! use std::fs::File;
//!
//! // Parse MT940 from file
//! let mut file = File::open("statement.mt940").unwrap();
//! let statement = Mt940Statement::from_read(&mut file).unwrap();
//!
//! println!("Account: {}", statement.account_number);
//! println!("Balance: {} {}", statement.closing_balance, statement.currency);
//! println!("Transactions: {}", statement.transactions.len());
//! ```
//!
//! ## Converting Between Formats
//!
//! ```no_run
//! use ledger_parser::{Mt940Statement, Camt053Statement};
//! use std::fs::File;
//!
//! // Read MT940
//! let mut input = File::open("input.mt940").unwrap();
//! let mt940 = Mt940Statement::from_read(&mut input).unwrap();
//!
//! // Convert to CAMT.053 using From trait
//! let camt053: Camt053Statement = mt940.into();
//!
//! // Write as XML
//! let mut output = File::create("output.xml").unwrap();
//! camt053.write_to(&mut output).unwrap();
//! ```
//!
//! ## Working with Transactions
//!
//! ```no_run
//! use ledger_parser::{CsvStatement, TransactionType};
//! use std::fs::File;
//!
//! let mut file = File::open("statement.csv").unwrap();
//! let statement = CsvStatement::from_read(&mut file).unwrap();
//!
//! // Filter credit transactions
//! let credits: Vec<_> = statement.transactions.iter()
//!     .filter(|t| t.transaction_type == TransactionType::Credit)
//!     .collect();
//!
//! println!("Found {} credit transactions", credits.len());
//! ```
//!
//! # Format Support
//!
//! Each format is represented by its own struct type:
//! - [`CsvStatement`] - CSV bank statement format
//! - [`Mt940Statement`] - SWIFT MT940 message format
//! - [`Camt053Statement`] - ISO 20022 CAMT.053 XML format
//!
//! All format structs implement:
//! - `from_read<R: Read>(&mut R) -> Result<Self, ParseError>` - Parse from any reader
//! - `write_to<W: Write>(&mut W) -> Result<(), ParseError>` - Write to any writer
//! - `From<OtherFormat>` - Convert between formats
//!
//! # Error Handling
//!
//! All operations return `Result<T, ParseError>`. The library never panics; all errors
//! are returned explicitly through the [`ParseError`] type.
//!
//! ```no_run
//! use ledger_parser::{Mt940Statement, ParseError};
//!
//! let data = "invalid mt940 data";
//! let mut reader = data.as_bytes();
//!
//! match Mt940Statement::from_read(&mut reader) {
//!     Ok(statement) => println!("Parsed successfully"),
//!     Err(ParseError::Mt940Error(msg)) => eprintln!("Parse error: {}", msg),
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```

pub mod error;
pub mod model;
mod formats {
    pub(crate) mod camt053_statement;
    pub(crate) mod csv_statement;
    pub(crate) mod mt940_statement;
    pub(crate) mod utils;

    // Format conversion modules
    mod camt053_conversions;
    mod csv_conversions;
    mod mt940_conversions;
}

// Re-export shared types for convenience
pub use error::ParseError;
pub use formats::camt053_statement::Camt053Statement;
pub use formats::csv_statement::CsvStatement;
pub use formats::mt940_statement::Mt940Statement;
pub use model::{BalanceType, Transaction, TransactionType};
