//! Type conversions from Mt940 to other formats
//!
//! Implements the `From` trait to enable idiomatic conversions between MT940
//! and other format structures (CAMT.053, CSV).

use crate::{Camt053Statement, CsvStatement, Mt940};

/// Convert MT940 to CAMT.053 format
///
/// Performs a direct field-by-field conversion since both structures
/// share the same data model.
///
/// # Example
/// ```ignore
/// # use ledger_parser::{Mt940, Camt053};
/// let mt940 = Mt940 { /* ... */ };
/// let camt053: Camt053 = mt940.into();
/// ```
impl From<Mt940> for Camt053Statement {
    fn from(mt940: Mt940) -> Self {
        Camt053Statement {
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

/// Convert MT940 to CSV format
///
/// Performs a direct field-by-field conversion since both structures
/// share the same data model.
///
/// # Example
/// ```ignore
/// # use ledger_parser::{Mt940, CsvStatement};
/// let mt940 = Mt940 { /* ... */ };
/// let csv: CsvStatement = mt940.into();
/// ```
impl From<Mt940> for CsvStatement {
    fn from(mt940: Mt940) -> Self {
        CsvStatement {
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
