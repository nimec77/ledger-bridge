//! Ledger Bridge Parser Library
//!
//! A library for parsing and converting financial data between CSV, MT940, and CAMT.053 formats.

pub mod model;
pub mod error;

// Re-export shared types for convenience
pub use model::{Transaction, BalanceType, TransactionType};
pub use error::ParseError;
