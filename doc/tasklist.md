# Ledger Bridge - Development Task List

## 📊 Progress Report

| Phase | Status | Progress | Description |
|-------|--------|----------|-------------|
| **Phase 0** | ⏳ Pending | 0/3 | Project Setup & Foundation |
| **Phase 1** | ⏳ Pending | 0/4 | CSV Format Implementation |
| **Phase 2** | ⏳ Pending | 0/4 | MT940 Format Implementation |
| **Phase 3** | ⏳ Pending | 0/4 | CAMT.053 Format Implementation |
| **Phase 4** | ⏳ Pending | 0/3 | CLI Application |
| **Phase 5** | ⏳ Pending | 0/3 | Testing & Validation |

**Legend:** ⏳ Pending | 🔄 In Progress | ✅ Complete | ❌ Blocked

---

## Phase 0: Project Setup & Foundation

**Goal:** Create workspace structure and core data model

### 0.1 Workspace Setup
- [ ] Create Cargo workspace with two crates
  - [ ] Root `Cargo.toml` with workspace members
  - [ ] `ledger-parser/` library crate
  - [ ] `ledger-bridge-cli/` binary crate
- [ ] Add dependencies to `ledger-parser/Cargo.toml`
  - [ ] `serde = { version = "1.0", features = ["derive"] }`
- [ ] Add dependencies to `ledger-bridge-cli/Cargo.toml`
  - [ ] `clap = { version = "4.0", features = ["derive"] }`
  - [ ] `ledger-parser = { path = "../ledger-parser" }`
- [ ] **Test:** `cargo build` succeeds for workspace

### 0.2 Data Model Implementation
- [ ] Create `ledger-parser/src/model.rs`
  - [ ] `Statement` struct with all fields
  - [ ] `Transaction` struct with all fields
  - [ ] `BalanceType` enum (Credit, Debit)
  - [ ] `TransactionType` enum (Credit, Debit)
  - [ ] Derive: `Debug, Clone, PartialEq, Serialize, Deserialize`
- [ ] **Test:** Create sample `Statement` in unit test, verify serialization

### 0.3 Error & Trait Definitions
- [ ] Create `ledger-parser/src/error.rs`
  - [ ] `ParseError` enum with variants
  - [ ] Implement `std::fmt::Display`
  - [ ] Implement `std::error::Error`
  - [ ] Implement `From<std::io::Error>`
- [ ] Create `ledger-parser/src/traits.rs`
  - [ ] `Parser<T>` trait with `parse<R: BufRead>` method
  - [ ] `Formatter<T>` trait with `format` method
- [ ] Update `ledger-parser/src/lib.rs`
  - [ ] Module declarations and public API exports
- [ ] **Test:** `cargo build` succeeds, docs compile with `cargo doc`

---

## Phase 1: CSV Format Implementation

**Goal:** Parse and format CSV bank statements (simplest format first)

### 1.1 CSV Parser Structure
- [ ] Create `ledger-parser/src/formats/csv.rs`
  - [ ] `CsvParser` zero-sized struct
  - [ ] Helper function: `parse_amount(s: &str) -> Result<f64>`
  - [ ] Helper function: `parse_date(s: &str) -> Result<String>`
- [ ] **Test:** Helpers work with sample inputs

### 1.2 CSV Parser Implementation
- [ ] Implement `Parser<Statement> for CsvParser`
  - [ ] Read lines using `BufRead::lines()`
  - [ ] Skip header/footer rows (identify transaction rows)
  - [ ] Parse transaction rows into `Transaction` structs
  - [ ] Extract opening/closing balances from summary rows
  - [ ] Handle split debit/credit columns
- [ ] **Test:** Parse minimal CSV (3 transactions) from in-memory string

### 1.3 CSV Formatter Implementation
- [ ] Implement `Formatter<Statement> for CsvParser`
  - [ ] Generate CSV header row
  - [ ] Format each transaction as CSV row
  - [ ] Add summary rows (opening/closing balance)
  - [ ] Handle proper quoting for description fields
- [ ] **Test:** Format sample `Statement`, verify CSV structure

### 1.4 CSV Round-Trip Test
- [ ] Create integration test in `ledger-parser/src/formats/csv.rs`
  - [ ] Parse CSV → Statement
  - [ ] Format Statement → CSV
  - [ ] Verify key fields preserved
- [ ] **Test:** Round-trip conversion works

---

## Phase 2: MT940 Format Implementation

**Goal:** Parse and format SWIFT MT940 messages

### 2.1 MT940 Parser Structure
- [ ] Create `ledger-parser/src/formats/mt940.rs`
  - [ ] `Mt940Parser` zero-sized struct
  - [ ] Helper: `extract_block4(input: &str) -> Result<&str>`
  - [ ] Helper: `parse_date_yymmdd(date: &str) -> Result<String>`
  - [ ] Helper: `parse_balance_line(tag: &str) -> Result<(...)>`
  - [ ] Helper: `parse_transaction_line(tag61: &str, tag86: &str) -> Result<Transaction>`
- [ ] **Test:** Each helper with sample MT940 snippets

### 2.2 MT940 Parser Implementation
- [ ] Implement `Parser<Statement> for Mt940Parser`
  - [ ] Read all using `read_to_string()` from BufRead
  - [ ] Extract Block 4 content
  - [ ] Parse `:25:` (account number)
  - [ ] Parse `:60F:` (opening balance with C/D, date, currency, amount)
  - [ ] Parse `:61:` + `:86:` pairs (transactions)
  - [ ] Parse `:62F:` (closing balance)
  - [ ] Handle multi-line `:86:` fields
- [ ] **Test:** Parse minimal MT940 message (2 transactions)

### 2.3 MT940 Formatter Implementation
- [ ] Implement `Formatter<Statement> for Mt940Parser`
  - [ ] Generate Block 4 structure
  - [ ] Format `:20:` (reference)
  - [ ] Format `:25:` (account)
  - [ ] Format `:60F:` (opening balance)
  - [ ] Format `:61:` + `:86:` pairs for each transaction
  - [ ] Format `:62F:` (closing balance)
- [ ] **Test:** Format sample `Statement`, verify MT940 structure

### 2.4 MT940 Round-Trip & Conversion Test
- [ ] Create integration test
  - [ ] Parse MT940 → Statement → MT940 (round-trip)
  - [ ] Parse MT940 → Statement → CSV (conversion)
- [ ] **Test:** Both conversions work

---

## Phase 3: CAMT.053 Format Implementation

**Goal:** Parse and format ISO 20022 CAMT.053 XML

### 3.1 CAMT.053 Parser Structure
- [ ] Create `ledger-parser/src/formats/camt053.rs`
  - [ ] `Camt053Parser` zero-sized struct
  - [ ] Helper: `extract_tag_text(xml: &str, tag: &str) -> Option<String>`
  - [ ] Helper: `extract_attribute(tag_str: &str, attr: &str) -> Option<String>`
  - [ ] Helper: `parse_balance(xml: &str, bal_type: &str) -> Result<(...)>`
  - [ ] Helper: `parse_entry(entry_xml: &str) -> Result<Transaction>`
- [ ] **Test:** Each helper with XML snippets

### 3.2 CAMT.053 Parser Implementation
- [ ] Implement `Parser<Statement> for Camt053Parser`
  - [ ] Read all XML using `read_to_string()`
  - [ ] Extract account from `<Acct><Id><IBAN>` or `<Othr><Id>`
  - [ ] Extract currency from `<Amt Ccy="XXX">` attribute
  - [ ] Parse opening balance (OPBD type)
  - [ ] Parse closing balance (CLBD type)
  - [ ] Parse each `<Ntry>` into `Transaction`
  - [ ] Extract counterparty (Dbtr for CRDT, Cdtr for DBIT)
  - [ ] Extract description from `<RmtInf><Ustrd>` or `<AddtlTxInf>`
- [ ] **Test:** Parse minimal CAMT.053 XML (2 transactions)

### 3.3 CAMT.053 Formatter Implementation
- [ ] Implement `Formatter<Statement> for Camt053Parser`
  - [ ] Generate XML header with namespace
  - [ ] Format `<BkToCstmrStmt>` structure
  - [ ] Format `<Acct>` with IBAN and currency
  - [ ] Format opening balance (`<Bal>` with OPBD type)
  - [ ] Format closing balance (`<Bal>` with CLBD type)
  - [ ] Format each transaction as `<Ntry>` with `<TxDtls>`
  - [ ] Include counterparty and remittance info
- [ ] **Test:** Format sample `Statement`, verify XML structure

### 3.4 CAMT.053 Round-Trip & Conversion Tests
- [ ] Create integration tests
  - [ ] Parse CAMT.053 → Statement → CAMT.053 (round-trip)
  - [ ] Parse CAMT.053 → Statement → CSV
  - [ ] Parse CAMT.053 → Statement → MT940
  - [ ] Cross-format: CSV → MT940 → CAMT.053
- [ ] **Test:** All format conversions work

---

## Phase 4: CLI Application

**Goal:** Build command-line interface for format conversion

### 4.1 CLI Argument Parsing
- [ ] Create `ledger-bridge-cli/src/main.rs`
  - [ ] Define `Cli` struct with clap derive
  - [ ] Arguments: `--in-format`, `--out-format`
  - [ ] Optional: `--input` (default stdin), `--output` (default stdout)
  - [ ] Add version and help metadata
- [ ] **Test:** `cargo run -- --help` displays usage

### 4.2 CLI Main Logic
- [ ] Implement `main()` function
  - [ ] Parse arguments
  - [ ] Create `Box<dyn BufRead>` from input (file or stdin)
  - [ ] Select parser based on `--in-format` (case-insensitive)
  - [ ] Parse input to `Statement`
  - [ ] Select formatter based on `--out-format`
  - [ ] Format `Statement` to output string
  - [ ] Write to file or stdout
  - [ ] Handle all errors gracefully (stderr + exit code 1)
- [ ] **Test:** Basic conversion: `echo "<data>" | cargo run -- --in-format csv --out-format mt940`

### 4.3 CLI Integration Testing
- [ ] Test all format combinations
  - [ ] CSV → MT940 (via stdin/stdout)
  - [ ] MT940 → CAMT.053 (via files)
  - [ ] CAMT.053 → CSV (mixed stdin/file)
  - [ ] Test error handling (invalid format, missing file, parse errors)
- [ ] **Test:** End-to-end CLI usage with real example files

---

## Phase 5: Testing & Validation

**Goal:** Comprehensive testing and validation

### 5.1 Unit Test Coverage
- [ ] Add error case tests for CSV parser
- [ ] Add error case tests for MT940 parser
- [ ] Add error case tests for CAMT.053 parser
- [ ] Verify all parsers handle malformed input gracefully
- [ ] **Test:** `cargo test` passes with >80% coverage

### 5.2 Real-World Data Testing
- [ ] Test with `example_files/example_of_account_statement.csv`
- [ ] Test with `example_files/mt 940 gs.mt940`
- [ ] Test with `example_files/camt 053 danske bank.camt`
- [ ] Document any format variations encountered
- [ ] **Test:** All example files parse successfully

### 5.3 Documentation & Cleanup
- [ ] Add doc comments to all public API items
- [ ] Generate documentation: `cargo doc --no-deps --open`
- [ ] Update main `README.md` with usage examples
- [ ] Verify all clippy lints pass: `cargo clippy -- -D warnings`
- [ ] Format all code: `cargo fmt`
- [ ] **Test:** Documentation builds, clippy/fmt clean

---

## Development Notes

### KISS Principles Applied
- Start with simplest format (CSV) before complex formats (MT940, CAMT.053)
- Manual parsing using standard library (no heavy dependencies)
- Each phase is independently testable
- Incremental functionality (parse → format → convert)

### Testing Strategy
- Test after each subtask (granular validation)
- Use in-memory test data for unit tests
- Real files for integration testing
- Both happy path and error cases

### Dependencies Strategy
- Core: Only `serde` for data model
- Optional: Add `csv` or `quick-xml` only if manual parsing becomes too complex
- Keep it minimal and standard-library-first

---

## Completion Criteria

✅ **Project Complete When:**
1. All 6 phases marked complete
2. All tests pass (`cargo test`)
3. All clippy warnings resolved
4. Documentation generated successfully
5. CLI converts between all format pairs
6. Example files parse without errors

**Estimated Iterations:** 6 phases × 3-4 tasks each = 18-24 testable increments


