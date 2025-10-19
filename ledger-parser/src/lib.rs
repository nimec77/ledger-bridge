//! Ledger Bridge Parser Library
//!
//! A library for parsing and converting financial data between CSV, MT940, and CAMT.053 formats.

pub mod error;
pub mod model;
mod formats {
    pub(crate) mod camt053_statement;
    pub(crate) mod csv_statement;
    pub(crate) mod mt940;
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
pub use formats::mt940::Mt940;
pub use model::{BalanceType, Transaction, TransactionType};
