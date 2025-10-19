//! Ledger Bridge CLI Application
//!
//! Command-line interface for converting financial data between formats.

use clap::Parser;
use ledger_parser::{Camt053, CsvStatement, Mt940, ParseError};
use std::fs::File;
use std::io::{self, Read, Write};

/// Convert financial data between CSV, MT940, and CAMT.053 formats
#[derive(Parser)]
#[command(name = "ledger-bridge")]
#[command(version)]
#[command(about = "Convert financial data between formats", long_about = None)]
struct Cli {
    /// Input format: csv, mt940, or camt053
    #[arg(long, value_name = "FORMAT")]
    in_format: String,

    /// Output format: csv, mt940, or camt053
    #[arg(long, value_name = "FORMAT")]
    out_format: String,

    /// Input file (default: stdin)
    #[arg(long, short = 'i', value_name = "FILE")]
    input: Option<String>,

    /// Output file (default: stdout)
    #[arg(long, short = 'o', value_name = "FILE")]
    output: Option<String>,
}

/// Enum to hold any of the three format types
enum Statement {
    Csv(CsvStatement),
    Mt940(Mt940),
    Camt053(Camt053),
}

fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Execute conversion and handle errors
    if let Err(e) = run_conversion(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Main conversion logic
fn run_conversion(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Handle input/output based on whether they are files or stdin/stdout
    match (&cli.input, &cli.output) {
        (Some(input_path), Some(output_path)) => {
            let mut input = File::open(input_path)?;
            let mut output = File::create(output_path)?;
            convert(&mut input, &mut output, &cli.in_format, &cli.out_format)?;
        }
        (Some(input_path), None) => {
            let mut input = File::open(input_path)?;
            let mut output = io::stdout();
            convert(&mut input, &mut output, &cli.in_format, &cli.out_format)?;
        }
        (None, Some(output_path)) => {
            let mut input = io::stdin();
            let mut output = File::create(output_path)?;
            convert(&mut input, &mut output, &cli.in_format, &cli.out_format)?;
        }
        (None, None) => {
            let mut input = io::stdin();
            let mut output = io::stdout();
            convert(&mut input, &mut output, &cli.in_format, &cli.out_format)?;
        }
    }

    Ok(())
}

/// Perform the actual conversion
fn convert<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    in_format: &str,
    out_format: &str,
) -> Result<(), ParseError> {
    // Parse based on input format
    let statement = parse_input(reader, in_format)?;

    // Convert and write based on output format
    write_output(statement, writer, out_format)?;

    Ok(())
}

/// Parse input based on format type
fn parse_input<R: Read>(reader: &mut R, format: &str) -> Result<Statement, ParseError> {
    match format.to_lowercase().as_str() {
        "csv" => Ok(Statement::Csv(CsvStatement::from_read(reader)?)),
        "mt940" => Ok(Statement::Mt940(Mt940::from_read(reader)?)),
        "camt053" => Ok(Statement::Camt053(Camt053::from_read(reader)?)),
        _ => Err(ParseError::InvalidFormat(format!(
            "Unknown input format: {}. Supported: csv, mt940, camt053",
            format
        ))),
    }
}

/// Convert and write output based on format type
fn write_output<W: Write>(
    statement: Statement,
    writer: &mut W,
    format: &str,
) -> Result<(), ParseError> {
    match format.to_lowercase().as_str() {
        "csv" => {
            let csv = match statement {
                Statement::Csv(s) => s,
                Statement::Mt940(s) => s.into(),
                Statement::Camt053(s) => s.into(),
            };
            csv.write_to(writer)
        }
        "mt940" => {
            let mt940 = match statement {
                Statement::Mt940(s) => s,
                Statement::Csv(s) => s.into(),
                Statement::Camt053(s) => s.into(),
            };
            mt940.write_to(writer)
        }
        "camt053" => {
            let camt053 = match statement {
                Statement::Camt053(s) => s,
                Statement::Mt940(s) => s.into(),
                Statement::Csv(s) => s.into(),
            };
            camt053.write_to(writer)
        }
        _ => Err(ParseError::InvalidFormat(format!(
            "Unknown output format: {}. Supported: csv, mt940, camt053",
            format
        ))),
    }
}
