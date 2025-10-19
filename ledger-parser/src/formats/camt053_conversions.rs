//! Type conversions from Camt053 to other formats
//!
//! Implements the `From` trait to enable idiomatic conversions between CAMT.053
//! and other format structures (MT940, CSV).

use crate::{Camt053Statement, CsvStatement, Mt940Statement};

/// Convert CAMT.053 to MT940 format
///
/// Performs a direct field-by-field conversion since both structures
/// share the same data model.
///
/// # Example
/// ```ignore
/// # use ledger_parser::{Camt053, Mt940};
/// let camt053 = Camt053 { /* ... */ };
/// let mt940: Mt940 = camt053.into();
/// ```
impl From<Camt053Statement> for Mt940Statement {
    fn from(camt: Camt053Statement) -> Self {
        Mt940Statement {
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

/// Convert CAMT.053 to CSV format
///
/// Performs a direct field-by-field conversion since both structures
/// share the same data model.
///
/// # Example
/// ```ignore
/// # use ledger_parser::{Camt053, CsvStatement};
/// let camt053 = Camt053 { /* ... */ };
/// let csv: CsvStatement = camt053.into();
/// ```
impl From<Camt053Statement> for CsvStatement {
    fn from(camt: Camt053Statement) -> Self {
        CsvStatement {
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
