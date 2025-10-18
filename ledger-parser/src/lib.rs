//! Ledger Bridge Parser Library
//!
//! A library for parsing and converting financial data between CSV, MT940, and CAMT.053 formats.

pub mod error;
pub mod model;
mod formats {
    pub(crate) mod camt053;
    pub(crate) mod csv;
    pub(crate) mod mt940;
    pub(crate) mod utils;
}

// Re-export shared types for convenience
pub use error::ParseError;
pub use formats::camt053::Camt053;
pub use formats::csv::CsvStatement;
pub use formats::mt940::Mt940;
pub use model::{BalanceType, Transaction, TransactionType};
