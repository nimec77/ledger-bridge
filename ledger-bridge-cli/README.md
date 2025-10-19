# Ledger Bridge CLI

Command-line interface for converting financial data between bank statement formats.

## Overview

`ledger-bridge-cli` is a CLI tool that converts bank statements between CSV, MT940, and CAMT.053 formats. It demonstrates the power of Rust's standard library I/O traits by working seamlessly with files, stdin, and stdout.

## Installation

### From Source

```bash
cd ledger-bridge-cli
cargo install --path .
```

### Build Only

```bash
cargo build --release
# Binary will be at: target/release/ledger-bridge-cli
```

## Usage

### Basic Syntax

```bash
ledger-bridge-cli --in-format <FORMAT> --out-format <FORMAT> [OPTIONS]
```

### Options

- `--in-format <FORMAT>` - Input format: `csv`, `mt940`, or `camt053` (case-insensitive)
- `--out-format <FORMAT>` - Output format: `csv`, `mt940`, or `camt053` (case-insensitive)
- `-i, --input <FILE>` - Input file (default: stdin)
- `-o, --output <FILE>` - Output file (default: stdout)
- `--help` - Display help information
- `--version` - Display version information

## Examples

### File to File Conversion

```bash
# Convert CSV to MT940
ledger-bridge-cli --in-format csv --out-format mt940 \
  --input statement.csv --output statement.mt940

# Convert MT940 to CAMT.053 XML
ledger-bridge-cli --in-format mt940 --out-format camt053 \
  --input data.mt940 --output data.xml

# Convert CAMT.053 to CSV
ledger-bridge-cli --in-format camt053 --out-format csv \
  --input statement.xml --output statement.csv
```

### Using stdin and stdout

```bash
# Read from stdin, write to file
cat input.csv | ledger-bridge-cli --in-format csv --out-format mt940 \
  --output statement.mt940

# Read from file, write to stdout
ledger-bridge-cli --in-format mt940 --out-format camt053 \
  --input data.mt940 > output.xml

# Full pipeline (stdin to stdout)
cat input.csv | ledger-bridge-cli --in-format csv --out-format mt940 | \
  ledger-bridge --in-format mt940 --out-format camt053 > output.xml
```

### Format Names (Case-Insensitive)

All format names are case-insensitive:

```bash
# These are all equivalent
ledger-bridge-cli --in-format csv --out-format mt940 -i data.csv
ledger-bridge-cli --in-format CSV --out-format MT940 -i data.csv
ledger-bridge-cli --in-format Csv --out-format Mt940 -i data.csv
```

## Supported Formats

### CSV Format

**Input/Output**: Comma-separated values

Example features:
- Multi-line headers and footers
- Separate debit/credit columns
- Russian Sberbank format support

### MT940 Format

**Input/Output**: SWIFT MT940 message format

Example features:
- Block structure (`:1:`, `:2:`, `:4:`)
- Tag-based fields (`:20:`, `:25:`, `:60F:`, `:61:`, `:86:`, `:62F:`)
- Multi-line transaction descriptions

### CAMT.053 Format

**Input/Output**: ISO 20022 XML format

Example features:
- Full XML namespace support
- Balance types (OPBD/CLBD)
- Transaction entries with counterparty info

## Conversion Matrix

All format pairs support bidirectional conversion:

| From ↓ / To → | CSV | MT940 | CAMT.053 |
|---------------|-----|-------|----------|
| **CSV**       | -   | ✅     | ✅        |
| **MT940**     | ✅   | -     | ✅        |
| **CAMT.053**  | ✅   | ✅     | -        |

## Error Handling

The CLI returns appropriate exit codes:

- **Exit code 0**: Success
- **Exit code 1**: Error (parse error, I/O error, invalid format)

All errors are printed to stderr:

```bash
# Example error output
$ ledger-bridge-cli --in-format invalid --out-format csv -i data.txt
Error: Invalid format: Unknown input format: invalid. Supported: csv, mt940, camt053
$ echo $?
1
```

## Real-World Examples

### Convert Sberbank CSV to MT940

```bash
ledger-bridge-cli --in-format csv --out-format mt940 \
  --input sberbank_statement.csv \
  --output statement.mt940
```

### Convert Goldman Sachs MT940 to CAMT.053

```bash
ledger-bridge-cli --in-format mt940 --out-format camt053 \
  --input goldman_sachs.mt940 \
  --output statement.xml
```

### Extract Danske Bank CAMT.053 to CSV

```bash
ledger-bridge-cli --in-format camt053 --out-format csv \
  --input danske_bank.xml \
  --output statement.csv
```

## Integration Examples

### Shell Pipeline

```bash
# Multi-step conversion pipeline
cat sberbank.csv | \
  ledger-bridge-cli --in-format csv --out-format mt940 | \
  ledger-bridge-cli --in-format mt940 --out-format camt053 \
  > final.xml
```

### Batch Processing

```bash
# Convert all CSV files to MT940
for file in *.csv; do
  ledger-bridge-cli --in-format csv --out-format mt940 \
    --input "$file" \
    --output "${file%.csv}.mt940"
done
```

### Validation

```bash
# Round-trip test to verify data integrity
ledger-bridge-cli --in-format csv --out-format mt940 -i original.csv -o temp.mt940
ledger-bridge-cli --in-format mt940 --out-format csv -i temp.mt940 -o roundtrip.csv
diff original.csv roundtrip.csv
```

## Tips and Tricks

### 1. Using with `jq` for JSON Processing

Since the library uses Serde, you can extend it with JSON:

```bash
# Add to your own tool that outputs JSON
my-tool --format json | jq '.' | my-converter
```

### 2. Checking Format Validity

```bash
# Validate MT940 syntax
ledger-bridge-cli --in-format mt940 --out-format mt940 \
  --input suspicious.mt940 --output /dev/null
```

### 3. Quick Format Inspection

```bash
# Convert to a more readable format for inspection
ledger-bridge-cli --in-format mt940 --out-format csv \
  --input complex.mt940 | less -S
```

## Troubleshooting

### "Parse error" Messages

If you encounter parse errors:

1. **Check format name**: Must be `csv`, `mt940`, or `camt053`
2. **Verify file structure**: Ensure input matches expected format
3. **Check file encoding**: Should be UTF-8

### "File not found" Errors

```bash
# Ensure file path is correct
ls -la statement.csv

# Use absolute paths if needed
ledger-bridge-cli --in-format csv --out-format mt940 \
  --input /full/path/to/statement.csv \
  --output /full/path/to/output.mt940
```

### Empty Output

If output is empty:
- Check that input file is not empty
- Verify input format matches actual file format
- Check stderr for error messages

## Performance

The CLI is designed for typical bank statement files (up to thousands of transactions):

- **Memory usage**: Loads entire file into memory for parsing
- **Speed**: Sub-second processing for most statements
- **File size**: Tested with files up to several MB

## Development

### Building

```bash
cargo build
cargo build --release  # Optimized build
```

### Testing

```bash
# Run integration tests
cd ..
cargo test --all

# Test CLI manually
cargo run -- --in-format csv --out-format mt940 \
  --input ../example_files/example_of_account_statement.csv
```

## Library

This CLI uses the `ledger-parser` library. For programmatic use in your own Rust projects, see the [library documentation](../ledger-parser/).

## See Also

- [Library Documentation](../ledger-parser/) - Core parsing library
- [Project README](../README.md) - Main project documentation
- [Vision Document](../vision.md) - Technical specifications

## License

MIT License - See LICENSE file for details.
