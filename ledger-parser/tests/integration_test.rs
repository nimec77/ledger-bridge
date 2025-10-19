//! Integration tests for format conversions
//!
//! Tests all `From` trait implementations between format pairs and verifies
//! data integrity through conversions and round-trip operations.

use chrono::DateTime;
use ledger_parser::*;

/// Helper function to create a test MT940 statement
fn create_test_mt940() -> Mt940Statement {
    Mt940Statement {
        account_number: "DE89370400440532013000".to_string(),
        currency: "EUR".to_string(),
        opening_balance: 1000.50,
        opening_date: DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap(),
        opening_indicator: BalanceType::Credit,
        closing_balance: 1500.75,
        closing_date: DateTime::parse_from_rfc3339("2025-01-31T00:00:00Z").unwrap(),
        closing_indicator: BalanceType::Credit,
        transactions: vec![Transaction {
            booking_date: DateTime::parse_from_rfc3339("2025-01-15T00:00:00Z").unwrap(),
            value_date: Some("2025-01-15".to_string()),
            amount: 500.25,
            transaction_type: TransactionType::Credit,
            description: "Payment received".to_string(),
            reference: Some("REF001".to_string()),
            counterparty_name: Some("John Doe".to_string()),
            counterparty_account: Some("DE89370400440532013111".to_string()),
        }],
    }
}

/// Helper function to create a test CAMT.053 statement
fn create_test_camt053() -> Camt053Statement {
    Camt053Statement {
        account_number: "DK8030000001234567".to_string(),
        currency: "DKK".to_string(),
        opening_balance: 2000.00,
        opening_date: DateTime::parse_from_rfc3339("2025-02-01T00:00:00Z").unwrap(),
        opening_indicator: BalanceType::Debit,
        closing_balance: 2500.50,
        closing_date: DateTime::parse_from_rfc3339("2025-02-28T00:00:00Z").unwrap(),
        closing_indicator: BalanceType::Credit,
        transactions: vec![Transaction {
            booking_date: DateTime::parse_from_rfc3339("2025-02-10T00:00:00Z").unwrap(),
            value_date: Some("2025-02-10".to_string()),
            amount: 750.00,
            transaction_type: TransactionType::Debit,
            description: "Payment sent".to_string(),
            reference: Some("CAMTREF123".to_string()),
            counterparty_name: Some("Jane Smith".to_string()),
            counterparty_account: Some("DK9876543210987654".to_string()),
        }],
    }
}

/// Helper function to create a test CSV statement
fn create_test_csv() -> CsvStatement {
    CsvStatement {
        account_number: "40817810099910004312".to_string(),
        currency: "RUB".to_string(),
        opening_balance: 5000.00,
        opening_date: DateTime::parse_from_rfc3339("2025-03-01T00:00:00Z").unwrap(),
        opening_indicator: BalanceType::Credit,
        closing_balance: 4500.00,
        closing_date: DateTime::parse_from_rfc3339("2025-03-31T00:00:00Z").unwrap(),
        closing_indicator: BalanceType::Credit,
        transactions: vec![Transaction {
            booking_date: DateTime::parse_from_rfc3339("2025-03-15T00:00:00Z").unwrap(),
            value_date: Some("2025-03-15".to_string()),
            amount: 500.00,
            transaction_type: TransactionType::Debit,
            description: "Purchase".to_string(),
            reference: Some("CSV001".to_string()),
            counterparty_name: Some("Store ABC".to_string()),
            counterparty_account: Some("40817810099910004444".to_string()),
        }],
    }
}

// ============================================================================
// MT940 → CAMT053 Conversions
// ============================================================================

#[test]
fn test_mt940_to_camt053_conversion() {
    let mt940 = create_test_mt940();
    let camt053: Camt053Statement = mt940.clone().into();

    assert_eq!(camt053.account_number, mt940.account_number);
    assert_eq!(camt053.currency, mt940.currency);
    assert_eq!(camt053.opening_balance, mt940.opening_balance);
    assert_eq!(camt053.opening_date, mt940.opening_date);
    assert_eq!(camt053.opening_indicator, mt940.opening_indicator);
    assert_eq!(camt053.closing_balance, mt940.closing_balance);
    assert_eq!(camt053.closing_date, mt940.closing_date);
    assert_eq!(camt053.closing_indicator, mt940.closing_indicator);
    assert_eq!(camt053.transactions.len(), mt940.transactions.len());
}

#[test]
fn test_mt940_to_camt053_preserves_transactions() {
    let mt940 = create_test_mt940();
    let camt053: Camt053Statement = mt940.clone().into();

    assert_eq!(camt053.transactions.len(), 1);
    let tx_orig = &mt940.transactions[0];
    let tx_conv = &camt053.transactions[0];

    assert_eq!(tx_conv.booking_date, tx_orig.booking_date);
    assert_eq!(tx_conv.amount, tx_orig.amount);
    assert_eq!(tx_conv.transaction_type, tx_orig.transaction_type);
    assert_eq!(tx_conv.description, tx_orig.description);
    assert_eq!(tx_conv.reference, tx_orig.reference);
}

// ============================================================================
// CAMT053 → MT940 Conversions
// ============================================================================

#[test]
fn test_camt053_to_mt940_conversion() {
    let camt053 = create_test_camt053();
    let mt940: Mt940Statement = camt053.clone().into();

    assert_eq!(mt940.account_number, camt053.account_number);
    assert_eq!(mt940.currency, camt053.currency);
    assert_eq!(mt940.opening_balance, camt053.opening_balance);
    assert_eq!(mt940.opening_date, camt053.opening_date);
    assert_eq!(mt940.opening_indicator, camt053.opening_indicator);
    assert_eq!(mt940.closing_balance, camt053.closing_balance);
    assert_eq!(mt940.closing_date, camt053.closing_date);
    assert_eq!(mt940.closing_indicator, camt053.closing_indicator);
    assert_eq!(mt940.transactions.len(), camt053.transactions.len());
}

// ============================================================================
// MT940 → CSV Conversions
// ============================================================================

#[test]
fn test_mt940_to_csv_conversion() {
    let mt940 = create_test_mt940();
    let csv: CsvStatement = mt940.clone().into();

    assert_eq!(csv.account_number, mt940.account_number);
    assert_eq!(csv.currency, mt940.currency);
    assert_eq!(csv.opening_balance, mt940.opening_balance);
    assert_eq!(csv.opening_date, mt940.opening_date);
    assert_eq!(csv.opening_indicator, mt940.opening_indicator);
    assert_eq!(csv.closing_balance, mt940.closing_balance);
    assert_eq!(csv.closing_date, mt940.closing_date);
    assert_eq!(csv.closing_indicator, mt940.closing_indicator);
    assert_eq!(csv.transactions.len(), mt940.transactions.len());
}

// ============================================================================
// CSV → MT940 Conversions
// ============================================================================

#[test]
fn test_csv_to_mt940_conversion() {
    let csv = create_test_csv();
    let mt940: Mt940Statement = csv.clone().into();

    assert_eq!(mt940.account_number, csv.account_number);
    assert_eq!(mt940.currency, csv.currency);
    assert_eq!(mt940.opening_balance, csv.opening_balance);
    assert_eq!(mt940.opening_date, csv.opening_date);
    assert_eq!(mt940.opening_indicator, csv.opening_indicator);
    assert_eq!(mt940.closing_balance, csv.closing_balance);
    assert_eq!(mt940.closing_date, csv.closing_date);
    assert_eq!(mt940.closing_indicator, csv.closing_indicator);
    assert_eq!(mt940.transactions.len(), csv.transactions.len());
}

// ============================================================================
// CAMT053 → CSV Conversions
// ============================================================================

#[test]
fn test_camt053_to_csv_conversion() {
    let camt053 = create_test_camt053();
    let csv: CsvStatement = camt053.clone().into();

    assert_eq!(csv.account_number, camt053.account_number);
    assert_eq!(csv.currency, camt053.currency);
    assert_eq!(csv.opening_balance, camt053.opening_balance);
    assert_eq!(csv.opening_date, camt053.opening_date);
    assert_eq!(csv.opening_indicator, camt053.opening_indicator);
    assert_eq!(csv.closing_balance, camt053.closing_balance);
    assert_eq!(csv.closing_date, camt053.closing_date);
    assert_eq!(csv.closing_indicator, camt053.closing_indicator);
    assert_eq!(csv.transactions.len(), camt053.transactions.len());
}

// ============================================================================
// CSV → CAMT053 Conversions
// ============================================================================

#[test]
fn test_csv_to_camt053_conversion() {
    let csv = create_test_csv();
    let camt053: Camt053Statement = csv.clone().into();

    assert_eq!(camt053.account_number, csv.account_number);
    assert_eq!(camt053.currency, csv.currency);
    assert_eq!(camt053.opening_balance, csv.opening_balance);
    assert_eq!(camt053.opening_date, csv.opening_date);
    assert_eq!(camt053.opening_indicator, csv.opening_indicator);
    assert_eq!(camt053.closing_balance, csv.closing_balance);
    assert_eq!(camt053.closing_date, csv.closing_date);
    assert_eq!(camt053.closing_indicator, csv.closing_indicator);
    assert_eq!(camt053.transactions.len(), csv.transactions.len());
}

// ============================================================================
// Round-Trip Conversions (Data Integrity Tests)
// ============================================================================

#[test]
fn test_round_trip_mt940_camt053_mt940() {
    let original = create_test_mt940();
    let camt053: Camt053Statement = original.clone().into();
    let back: Mt940Statement = camt053.into();

    assert_eq!(back.account_number, original.account_number);
    assert_eq!(back.currency, original.currency);
    assert_eq!(back.opening_balance, original.opening_balance);
    assert_eq!(back.opening_date, original.opening_date);
    assert_eq!(back.opening_indicator, original.opening_indicator);
    assert_eq!(back.closing_balance, original.closing_balance);
    assert_eq!(back.closing_date, original.closing_date);
    assert_eq!(back.closing_indicator, original.closing_indicator);
    assert_eq!(back.transactions.len(), original.transactions.len());
}

#[test]
fn test_round_trip_camt053_mt940_camt053() {
    let original = create_test_camt053();
    let mt940: Mt940Statement = original.clone().into();
    let back: Camt053Statement = mt940.into();

    assert_eq!(back.account_number, original.account_number);
    assert_eq!(back.currency, original.currency);
    assert_eq!(back.opening_balance, original.opening_balance);
    assert_eq!(back.opening_date, original.opening_date);
    assert_eq!(back.opening_indicator, original.opening_indicator);
    assert_eq!(back.closing_balance, original.closing_balance);
    assert_eq!(back.closing_date, original.closing_date);
    assert_eq!(back.closing_indicator, original.closing_indicator);
    assert_eq!(back.transactions.len(), original.transactions.len());
}

#[test]
fn test_round_trip_csv_mt940_csv() {
    let original = create_test_csv();
    let mt940: Mt940Statement = original.clone().into();
    let back: CsvStatement = mt940.into();

    assert_eq!(back.account_number, original.account_number);
    assert_eq!(back.currency, original.currency);
    assert_eq!(back.opening_balance, original.opening_balance);
    assert_eq!(back.opening_date, original.opening_date);
    assert_eq!(back.opening_indicator, original.opening_indicator);
    assert_eq!(back.closing_balance, original.closing_balance);
    assert_eq!(back.closing_date, original.closing_date);
    assert_eq!(back.closing_indicator, original.closing_indicator);
    assert_eq!(back.transactions.len(), original.transactions.len());
}

#[test]
fn test_round_trip_csv_camt053_csv() {
    let original = create_test_csv();
    let camt053: Camt053Statement = original.clone().into();
    let back: CsvStatement = camt053.into();

    assert_eq!(back.account_number, original.account_number);
    assert_eq!(back.currency, original.currency);
    assert_eq!(back.opening_balance, original.opening_balance);
    assert_eq!(back.opening_date, original.opening_date);
    assert_eq!(back.opening_indicator, original.opening_indicator);
    assert_eq!(back.closing_balance, original.closing_balance);
    assert_eq!(back.closing_date, original.closing_date);
    assert_eq!(back.closing_indicator, original.closing_indicator);
    assert_eq!(back.transactions.len(), original.transactions.len());
}

#[test]
fn test_round_trip_mt940_csv_mt940() {
    let original = create_test_mt940();
    let csv: CsvStatement = original.clone().into();
    let back: Mt940Statement = csv.into();

    assert_eq!(back.account_number, original.account_number);
    assert_eq!(back.currency, original.currency);
    assert_eq!(back.opening_balance, original.opening_balance);
    assert_eq!(back.opening_date, original.opening_date);
    assert_eq!(back.opening_indicator, original.opening_indicator);
    assert_eq!(back.closing_balance, original.closing_balance);
    assert_eq!(back.closing_date, original.closing_date);
    assert_eq!(back.closing_indicator, original.closing_indicator);
    assert_eq!(back.transactions.len(), original.transactions.len());
}

#[test]
fn test_round_trip_camt053_csv_camt053() {
    let original = create_test_camt053();
    let csv: CsvStatement = original.clone().into();
    let back: Camt053Statement = csv.into();

    assert_eq!(back.account_number, original.account_number);
    assert_eq!(back.currency, original.currency);
    assert_eq!(back.opening_balance, original.opening_balance);
    assert_eq!(back.opening_date, original.opening_date);
    assert_eq!(back.opening_indicator, original.opening_indicator);
    assert_eq!(back.closing_balance, original.closing_balance);
    assert_eq!(back.closing_date, original.closing_date);
    assert_eq!(back.closing_indicator, original.closing_indicator);
    assert_eq!(back.transactions.len(), original.transactions.len());
}

// ============================================================================
// Chain Conversions (Test Three-Way Conversions)
// ============================================================================

#[test]
fn test_chain_conversion_mt940_to_camt053_to_csv() {
    let mt940 = create_test_mt940();
    let camt053: Camt053Statement = mt940.clone().into();
    let csv: CsvStatement = camt053.into();

    // Verify data preserved through chain
    assert_eq!(csv.account_number, mt940.account_number);
    assert_eq!(csv.currency, mt940.currency);
    assert_eq!(csv.opening_balance, mt940.opening_balance);
    assert_eq!(csv.closing_balance, mt940.closing_balance);
    assert_eq!(csv.transactions.len(), mt940.transactions.len());
}

#[test]
fn test_chain_conversion_csv_to_mt940_to_camt053() {
    let csv = create_test_csv();
    let mt940: Mt940Statement = csv.clone().into();
    let camt053: Camt053Statement = mt940.into();

    // Verify data preserved through chain
    assert_eq!(camt053.account_number, csv.account_number);
    assert_eq!(camt053.currency, csv.currency);
    assert_eq!(camt053.opening_balance, csv.opening_balance);
    assert_eq!(camt053.closing_balance, csv.closing_balance);
    assert_eq!(camt053.transactions.len(), csv.transactions.len());
}

#[test]
fn test_chain_conversion_camt053_to_csv_to_mt940() {
    let camt053 = create_test_camt053();
    let csv: CsvStatement = camt053.clone().into();
    let mt940: Mt940Statement = csv.into();

    // Verify data preserved through chain
    assert_eq!(mt940.account_number, camt053.account_number);
    assert_eq!(mt940.currency, camt053.currency);
    assert_eq!(mt940.opening_balance, camt053.opening_balance);
    assert_eq!(mt940.closing_balance, camt053.closing_balance);
    assert_eq!(mt940.transactions.len(), camt053.transactions.len());
}

// ============================================================================
// Empty Transaction Lists
// ============================================================================

#[test]
fn test_conversion_with_empty_transactions() {
    let mt940 = Mt940Statement {
        account_number: "TEST123".to_string(),
        currency: "USD".to_string(),
        opening_balance: 1000.0,
        opening_date: DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap(),
        opening_indicator: BalanceType::Credit,
        closing_balance: 1000.0,
        closing_date: DateTime::parse_from_rfc3339("2025-01-31T00:00:00Z").unwrap(),
        closing_indicator: BalanceType::Credit,
        transactions: vec![],
    };

    let camt053: Camt053Statement = mt940.clone().into();
    let csv: CsvStatement = mt940.clone().into();

    assert_eq!(camt053.transactions.len(), 0);
    assert_eq!(csv.transactions.len(), 0);
}
