use crate::error::ParseError;
use crate::formats::camt053::camt053_const::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ElementName {
    Document,
    BkToCstmrStmt,
    Stmt,
    Acct,
    Id,
    Iban,
    Currency,
    Balance,
    BalanceType,
    CodeOrProprietary,
    Code,
    Amount,
    CreditDebit,
    Date,
    Entry,
    EntryRef,
    BookingDate,
    ValueDate,
    EntryDetails,
    TransactionDetails,
    References,
    TransactionId,
    RemittanceInfo,
    UnstructuredRemittance,
    StructuredRemittance,
    CreditorReferenceInfo,
    ReferenceValue,
    RelatedParties,
    Debtor,
    Creditor,
    DebtorAccount,
    CreditorAccount,
    Name,
    AdditionalInfo,
    Other,
}

impl ElementName {
    pub fn from_name_bytes(raw: &[u8]) -> Result<Self, ParseError> {
        let name = std::str::from_utf8(raw).map_err(|err| {
            ParseError::Camt053Error(format!("Invalid XML tag name encoding: {}", err))
        })?;
        let normalized = name.rsplit(':').next().unwrap_or(name);

        Ok(ElementName::from_name(normalized))
    }

    fn from_name(name: &str) -> Self {
        let name_upper = name.to_uppercase();
        match name_upper.as_str() {
            s if s == DOCUMENT_TAG.to_uppercase() => ElementName::Document,
            s if s == BK_TO_CSTM_STMT_TAG.to_uppercase() => ElementName::BkToCstmrStmt,
            s if s == STMT_TAG.to_uppercase() => ElementName::Stmt,
            s if s == ACCT_TAG.to_uppercase() => ElementName::Acct,
            s if s == ID_TAG.to_uppercase() => ElementName::Id,
            s if s == IBAN_TAG.to_uppercase() => ElementName::Iban,
            s if s == CCY_TAG.to_uppercase() => ElementName::Currency,
            s if s == BAL_TAG.to_uppercase() => ElementName::Balance,
            s if s == TP_TAG.to_uppercase() => ElementName::BalanceType,
            s if s == CD_OR_PRTY_TAG.to_uppercase() => ElementName::CodeOrProprietary,
            s if s == CD_TAG.to_uppercase() => ElementName::Code,
            s if s == AMT_TAG.to_uppercase() => ElementName::Amount,
            s if s == CDT_DBT_IND_TAG.to_uppercase() => ElementName::CreditDebit,
            s if s == DT_TAG.to_uppercase() => ElementName::Date,
            s if s == NTRY_TAG.to_uppercase() => ElementName::Entry,
            s if s == NTRY_REF_TAG.to_uppercase() => ElementName::EntryRef,
            s if s == BOOKG_DT_TAG.to_uppercase() => ElementName::BookingDate,
            s if s == VAL_DT_TAG.to_uppercase() => ElementName::ValueDate,
            s if s == NTRY_DTLS_TAG.to_uppercase() => ElementName::EntryDetails,
            s if s == TX_DTLS_TAG.to_uppercase() => ElementName::TransactionDetails,
            s if s == REFS_TAG.to_uppercase() => ElementName::References,
            s if s == TX_ID_TAG.to_uppercase() => ElementName::TransactionId,
            s if s == RMT_INF_TAG.to_uppercase() => ElementName::RemittanceInfo,
            s if s == USTRD_TAG.to_uppercase() => ElementName::UnstructuredRemittance,
            s if s == STRD_TAG.to_uppercase() => ElementName::StructuredRemittance,
            s if s == CDTR_REF_INF_TAG.to_uppercase() => ElementName::CreditorReferenceInfo,
            s if s == REF_TAG.to_uppercase() => ElementName::ReferenceValue,
            s if s == RLT_PTIES_TAG.to_uppercase() => ElementName::RelatedParties,
            s if s == DBTR_TAG.to_uppercase() => ElementName::Debtor,
            s if s == CDTR_TAG.to_uppercase() => ElementName::Creditor,
            s if s == DBTR_ACCT_TAG.to_uppercase() => ElementName::DebtorAccount,
            s if s == CDTR_ACCT_TAG.to_uppercase() => ElementName::CreditorAccount,
            s if s == NM_TAG.to_uppercase() => ElementName::Name,
            s if s == ADDTL_TX_INF_TAG.to_uppercase() => ElementName::AdditionalInfo,
            _ => ElementName::Other,
        }
    }
}
