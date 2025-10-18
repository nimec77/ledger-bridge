use crate::error::ParseError;

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
        match name.to_lowercase().as_str() {
            "document" => ElementName::Document,
            "bktocstmrstmt" => ElementName::BkToCstmrStmt,
            "stmt" => ElementName::Stmt,
            "acct" => ElementName::Acct,
            "id" => ElementName::Id,
            "iban" => ElementName::Iban,
            "ccy" => ElementName::Currency,
            "bal" => ElementName::Balance,
            "tp" => ElementName::BalanceType,
            "cdorprtry" => ElementName::CodeOrProprietary,
            "cd" => ElementName::Code,
            "amt" => ElementName::Amount,
            "cdtdbtind" => ElementName::CreditDebit,
            "dt" => ElementName::Date,
            "ntry" => ElementName::Entry,
            "ntryref" => ElementName::EntryRef,
            "bookgdt" => ElementName::BookingDate,
            "valdt" => ElementName::ValueDate,
            "ntrydtls" => ElementName::EntryDetails,
            "txdtls" => ElementName::TransactionDetails,
            "refs" => ElementName::References,
            "txid" => ElementName::TransactionId,
            "rmtinf" => ElementName::RemittanceInfo,
            "ustrd" => ElementName::UnstructuredRemittance,
            "strd" => ElementName::StructuredRemittance,
            "cdtrrefinf" => ElementName::CreditorReferenceInfo,
            "ref" => ElementName::ReferenceValue,
            "rltdpties" => ElementName::RelatedParties,
            "dbtr" => ElementName::Debtor,
            "cdtr" => ElementName::Creditor,
            "dbtracct" => ElementName::DebtorAccount,
            "cdtracct" => ElementName::CreditorAccount,
            "nm" => ElementName::Name,
            "addtltxinf" => ElementName::AdditionalInfo,
            "othr" => ElementName::Other,
            _ => ElementName::Other,
        }
    }
}
