use env_logger::Env;
use log::info;
use clap::Parser;
use csv::{ReaderBuilder, Trim, Writer};
use crate::model::TransactionRecord;
use crate::engine::Engine;

mod model;
mod engine;
mod error;

/// Simple Payments Engine
#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Input CSV file of transactions
    #[clap(value_parser)]
    input: std::path::PathBuf,
}

fn main() -> Result<(), error::ApplicationError> {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting payments engine");

    let args = Args::parse();

    let mut engine = Engine::new();
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_path(&args.input)?;

    // Prepare a buffer to collect (row_index, raw_line, error_message)
    let mut errors: Vec<(usize, String, String)> = Vec::new();
    for (index, result) in reader.deserialize::<TransactionRecord>().enumerate() {
        match result {
            Ok(record) => {
                // Try to process; on Err, collect and continue
                if let Err(e) = engine.process_transaction(record.clone()) {
                    errors.push((
                        index,
                        format!("{:?}", record),
                        e.to_string(),
                    ));
                }
            }
            Err(e) => {
                // CSV parse error: collect and continue
                errors.push((
                    index,
                    String::new(), // no record available
                    format!("CSV parse error: {}", e),
                ));
            }
        }
    }

    // Output results to CSV on stdout
    let mut writer = Writer::from_writer(std::io::stdout());
    // Write header
    writer.write_record(["client", "available", "held", "total", "locked"])?;
    // Sort client IDs for deterministic output
    let mut client_ids: Vec<u16> = engine.accounts.keys().cloned().collect();
    client_ids.sort_unstable();
    for client_id in client_ids {
        if let Some(account) = engine.accounts.get(&client_id) {
            writer.write_record(&[
                client_id.to_string(),
                format!("{:.4}", account.available),
                format!("{:.4}", account.held),
                format!("{:.4}", account.total),
                account.locked.to_string(),
            ])?;
        }
    }
    writer.flush()?;

    // Emit collected errors to stderr
    for (row, raw, msg) in errors {
        if raw.is_empty() {
            eprintln!("Error at row {}: {}.", row, msg);
        } else {
            eprintln!("Error at row {} (record={}): {}", row, raw, msg);
        }
    }
    Ok(())
}
