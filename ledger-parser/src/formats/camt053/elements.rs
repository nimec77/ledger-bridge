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
        match name.to_uppercase().as_str() {
            DOCUMENT_TAG => ElementName::Document,
            BK_TO_CSTM_STMT_TAG => ElementName::BkToCstmrStmt,
            STMT_TAG => ElementName::Stmt,
            ACCT_TAG => ElementName::Acct,
            ID_TAG => ElementName::Id,
            IBAN_TAG => ElementName::Iban,
            CCY_TAG => ElementName::Currency,
            BAL_TAG => ElementName::Balance,
            TP_TAG => ElementName::BalanceType,
            CD_OR_PRTY_TAG => ElementName::CodeOrProprietary,
            CD_TAG => ElementName::Code,
            AMT_TAG => ElementName::Amount,
            CDT_DBT_IND_TAG => ElementName::CreditDebit,
            DT_TAG => ElementName::Date,
            NTRY_TAG => ElementName::Entry,
            NTRY_REF_TAG => ElementName::EntryRef,
            BOOKG_DT_TAG => ElementName::BookingDate,
            VAL_DT_TAG => ElementName::ValueDate,
            NTRY_DTLS_TAG => ElementName::EntryDetails,
            TX_DTLS_TAG => ElementName::TransactionDetails,
            REFS_TAG => ElementName::References,
            TX_ID_TAG => ElementName::TransactionId,
            RMT_INF_TAG => ElementName::RemittanceInfo,
            USTRD_TAG => ElementName::UnstructuredRemittance,
            STRD_TAG => ElementName::StructuredRemittance,
            CDTR_REF_INF_TAG => ElementName::CreditorReferenceInfo,
            REF_TAG => ElementName::ReferenceValue,
            RLT_PTIES_TAG => ElementName::RelatedParties,
            DBTR_TAG => ElementName::Debtor,
            CDTR_TAG => ElementName::Creditor,
            DBTR_ACCT_TAG => ElementName::DebtorAccount,
            CDTR_ACCT_TAG => ElementName::CreditorAccount,
            NM_TAG => ElementName::Name,
            ADDTL_TX_INF_TAG => ElementName::AdditionalInfo,
            OTHR_TAG => ElementName::Other,
            _ => ElementName::Other,
        }
    }
}
