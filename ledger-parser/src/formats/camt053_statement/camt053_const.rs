/// XML tag name constants for CAMT.053 format.
///
/// These remain in the canonical ISO 20022 casing; the writer emits these tags
/// verbatim, and the parser compares them directly. Changing the literals (for
/// example uppercasing everything) breaks both read and write flows.
pub(super) const DOCUMENT_TAG: &str = "Document";
pub(super) const BK_TO_CSTM_STMT_TAG: &str = "BkToCstmrStmt";
pub(super) const STMT_TAG: &str = "Stmt";
pub(super) const ACCT_TAG: &str = "Acct";
pub(super) const ID_TAG: &str = "Id";
pub(super) const IBAN_TAG: &str = "IBAN";
pub(super) const CCY_TAG: &str = "Ccy";
pub(super) const BAL_TAG: &str = "Bal";
pub(super) const TP_TAG: &str = "Tp";
pub(super) const CD_OR_PRTY_TAG: &str = "CdOrPrtry";
pub(super) const CD_TAG: &str = "Cd";
pub(super) const AMT_TAG: &str = "Amt";
pub(super) const CDT_DBT_IND_TAG: &str = "CdtDbtInd";
pub(super) const DT_TAG: &str = "Dt";
pub(super) const NTRY_TAG: &str = "Ntry";
pub(super) const NTRY_REF_TAG: &str = "NtryRef";
pub(super) const BOOKG_DT_TAG: &str = "BookgDt";
pub(super) const VAL_DT_TAG: &str = "ValDt";
pub(super) const NTRY_DTLS_TAG: &str = "NtryDtls";
pub(super) const TX_DTLS_TAG: &str = "TxDtls";
pub(super) const REFS_TAG: &str = "Refs";
pub(super) const TX_ID_TAG: &str = "TxId";
pub(super) const RMT_INF_TAG: &str = "RmtInf";
pub(super) const USTRD_TAG: &str = "Ustrd";
pub(super) const STRD_TAG: &str = "Strd";
pub(super) const CDTR_REF_INF_TAG: &str = "CdtrRefInf";
pub(super) const REF_TAG: &str = "Ref";
pub(super) const RLT_PTIES_TAG: &str = "RltdPties";
pub(super) const DBTR_TAG: &str = "Dbtr";
pub(super) const CDTR_TAG: &str = "Cdtr";
pub(super) const DBTR_ACCT_TAG: &str = "DbtrAcct";
pub(super) const CDTR_ACCT_TAG: &str = "CdtrAcct";
pub(super) const NM_TAG: &str = "Nm";
pub(super) const ADDTL_TX_INF_TAG: &str = "AddtlTxInf";

// Balance type constants
pub(super) const OPBD_BALANCE_TYPE: &str = "OPBD";
pub(super) const CLBD_BALANCE_TYPE: &str = "CLBD";

// Credit/Debit indicator constants
pub(super) const CRDT_INDICATOR: &str = "CRDT";
pub(super) const DBIT_INDICATOR: &str = "DBIT";
