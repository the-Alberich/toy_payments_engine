# Payments Engine Toy

This repository contains a simple payments engine implemented in Rust. It reads a stream of transactions from an input CSV, processes deposits, withdrawals, disputes, resolves, and chargebacks, and writes the resulting client account states to an output CSV.

---

## Overview

* **Input**: CSV file of transactions (`type, client, tx, amount`)
* **Output**: CSV of client accounts (`client, available, held, total, locked`) to stdout
* **CLI**:

  ```bash
  cargo run -- transactions.csv > accounts.csv
  ```

---

## Getting Started

### Prerequisites

* Rust (stable) ≥ 1.70.0
* `cargo`

### Build & Run

1. Clone this repo:

   ```bash
   git clone https://github.com/the-Alberich/toy_payments_engine
   cd payments-engine
   ```
2. Build:

   ```bash
   cargo build --release
   ```
3. Run:

   ```bash
   cargo run -- transactions.csv > accounts.csv
   ```

---

## Project Structure

```
toy_payments_engine/
├── Cargo.toml
├── src/
│   ├── main.rs                    # CLI entrypoint & CSV I/O
│   ├── model.rs                   # Account and TransactionRecord structs
│   ├── engine.rs                  # Core processing logic
│   └── error.rs                   # Error definitions
├── tests/
│   ├── fixtures/                  # Sample CSV files (basic.csv and disputes.csv)
│   ├── engine_unit_tests.rs       # Unit tests for engine logic
│   └── cli_integration_tests.rs   # End-to-end CLI tests
└── README.md                      # This file
```

---

## Dependencies

- **csv** for CSV parsing/writing
- **serde** + **serde\_derive** for (de)serialization
- **rust\_decimal** for fixed-point decimals (4 fractional places)
- **clap** for CLI argument parsing
- **log** + **env\_logger** for structured logging (to stderr)
- **thiserror** for ergonomic error types

---

## Dev Dependencies

- **rstest** for Unit Tests
- **assert_cmd** + **predicates** for CLI Integration Tests

---

## Assumptions & Particulars

1. **Error Handling**: On parse errors or invalid operations (e.g. non-existent transaction in dispute), a warning is logged to STDERR and the record which resulted in a parse error or invalid operation is *skipped*, continuing processing. The assumption is this is an error in the input CSV and should be ignored. There are some edge cases that would represent a functional error in the payments error that may be logged as an error to STDERR. Once processing of all transactions and output of resulting client account states is complete, the compiled set of all such application errors (excluding bad input errors) will be output to STDERR for debugging purposes.

2. **Output Formatting**: CSV header emits exactly once as `client,available,held,total,locked`. Row order is arbitrary; for determinism *sorting* is done by client ID when emitting. This could easily be disabled either by CLI arg or environment variable, but for now it's left in with no toggle.

3. **Data Structures**: Use `HashMap<u16, Account>` and `HashMap<u32, TransactionRecord>` for O(1) lookups. HashMap iteration is unordered; client_id keys of accounts are sorted before output to guarantee stable ordering.

4. **Decimal Precision & Formatting**: Internally `Decimal` is used with four decimal places. For output this implementation *always* formats to exactly four fractional digits (e.g. `1.5000`) for consistency and human readability.

5. **Account Locking**: Once a chargeback locks a client account, all subsequent transactions for that client are ignored. There is no way to unlock a locked account currently. Locked status emits in output.

6. **Logging**: All informational and warning logs are sent to STDERR via `log` + `env_logger` to avoid polluting STDOUT CSV output.

7. **Memory & Input Streaming**: Input CSV length is not known. Stream reading of the input CSV is used to reduce memory footprint, however all processed transactions are stored so disputes can reference by transaction ID. Disputes are stored so resolves / chargebacks can reference them (also by transaction ID). While steps are taken to consider memory footprint, transactions, disputes, and accounts are stored and updated as each transaction is processed *in memory* currently. For very large datasets, a persistent store (e.g. SQLite via rusqlite) could replace the in‐memory maps if needed, however that is currently not implemented here.

---

## Testing

- **Unit tests**: Cover each transaction type, most edge cases (although it's not guaranteed that all edge cases are covered), and invalid operations. No unit tests currently cover the application errors that represent a broken state of the core payments engine.
- **Integration test**: Runs the CLI against sample fixtures in `tests/fixtures` to verify end-to-end behavior. These tests cover some simple and some more complex use case scenarios, but are not guaranteed to be exhaustive. These tests also do not currently cover the application errors that represent a broken state of the core payments engine.

To run tests:

```bash
cargo test
````

---
