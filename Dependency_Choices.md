# Dependency Choices

This document outlines chosen crates and rationale for the toy payments engine.

---


### 1. CSV I/O

#### **Best Option: `csv`**
  * De facto Rust standard (30M+ downloads)
  * Streaming, zero-copy deserialization via Serde
  * Robust handling of headers, quoting, and flexible delimiters
  * Actively maintained and common usage infers it's a decent choice

#### **Alternatives**
  * `simd-csv`
    * *Pros:* A high-performance parser using SIMD for 2–5× faster throughput on large files.
    * *Cons:* Lacks Serde integration, smaller community, and less documentation.
  * `csv-core`
    * *Pros:* The low-level parsing engine for embedded or no-std needs. It's used under the hood by `csv`.
    * *Cons:* Requires custom Serde glue and reimplementation of high-level conveniences.
  * No crate
    * *Pros:* Zero dependencies and full control over parsing.
    * *Cons:* Reimplementing CSV parsing is error-prone, fragile, and unnecessarily time consuming.

#### **Decision**
  Use `csv` for robust, streaming Serde support without reinventing CSV parsing.

#### **Details**
  I chose the `csv` crate because it’s the de-facto Rust library for parsing and writing CSV files. It integrates seamlessly with Serde for zero-boilerplate deserialization, supports streaming large inputs without buffering the entire file, and correctly handles all CSV edge cases (quoting, embedded delimiters, whitespace). Alternatives like `simd-csv` offer raw speed but lack Serde support and community maturity; `csv-core` is too low-level; using no dependency would take more time and risk subtle parsing bugs.

---

### 2. Decimal Arithmetic

#### **Best Option: `rust_decimal`**
  * Base-10 fixed point (96-bit integer + scale)
  * Exact decimal precision with configurable scale (4 places for this task)
  * Built-in Serde support and arithmetic traits
  * Idiomatic operator overloads (`a + b` instead of `a.add(b)`)

#### **Alternatives**
  * `bigdecimal`
    * *Pros:* Arbitrary-precision decimal via BigInt backing; no risk of overflow or precision limits.
    * *Cons:* BigInt operations introduce allocation and overhead; slower performance and higher memory churn.
  * `fixed`
    * *Pros:* Very fast binary fixed-point using bit-shifts and integer operations; zero allocations.
    * *Cons:* Cannot exactly represent common decimal fractions (e.g. 0.1); base-10 currency scenarios can yield surprises.
  * No crate
    * *Pros:* Zero dependencies; direct use of built-in types.
    * *Cons:* Using `f64` risks silent rounding errors; manual `i64` scaling adds boilerplate and is error-prone.

#### **Decision**
  Use `rust_decimal` for precise, efficient decimal arithmetic, pre-built `+`, `-` operator overloads, and serialization simplicity.

#### **Details**
  I'm using `rust_decimal` because it provides exact, base-10 fixed-point arithmetic and the configurable scale supports values requiring four decimal places. It integrates out-of-the-box with Serde for CSV (de)serialization, supports idiomatic operator overloading (`+`, `-`, `*`, `/`), and has predictable, native-integer performance. Alternatives like `bigdecimal` introduce big-integer overhead, `fixed` can’t guarantee decimal correctness, and building from scratch using `f64` risks silent rounding errors; `i64` is error-prone.

---

### 3. CLI Parsing

#### **Best Option:  `clap` v4**
  * Widely adopted (14M+ downloads)
  * Derive-based API for minimal boilerplate
  * Zero boilerplate parsing into a struct.
  * Validation (e.g., ensure exactly one argument, readable file).
  * Self-documenting help text generated for free.
  * Minimal footprint, since this is a small CLI.

#### **Alternatives**
  * `structopt`
    * *Pros:* Ergonomic derive-based API inherited from early `clap` versions.
    * *Cons:* Deprecated in favor of `clap` v3+ derive; no active maintenance or new features.
  * `gumdrop`
    * *Pros:* Lightweight, minimal macro usage, simple attribute syntax.
    * *Cons:* Lacks built-in validation and subcommand support; smaller community.
  * `pico-args`
    * *Pros:* Zero-allocation parsing with minimal runtime overhead.
    * *Cons:* No derive macros; manual parsing and error reporting boilerplate; no auto-help/version.
  * No crate
    * *Pros:* Zero dependencies; full control over argument parsing logic.
    * *Cons:* Reinventing argument parsing, fragile error handling, no built-in help or validation.

#### **Decision**
  Use `clap` v4 derive for a future-proof, ergonomic CLI.

#### **Details**
  I selected `clap` v4 because it offers a derive-based API that generates a robust, user-friendly command line interface with zero boilerplate. It auto-provides `--help` and `--version` flags drawn from `Cargo.toml` metadata and comments on the Parser data structure, supports validation and subcommands, and is a very widely adopted parsing library. Alternatives considered: `structopt` is deprecated, `gumdrop` lacks key features like validation, `pico-args` requires manual boilerplate, and building from scratch would risk bugs and require manual validation.

---

### 4. Logging

#### **Best Option: `log` + `env_logger`**
  * Error logs sent to STDERR by default, so no pollution in STDOUT since the output is STDOUT to file redirect
  * Handy `log` crate defined macros (`info!`, `warn!`, `error!`, `debug!`) and a global logger interface keep code readable and clean
  * Runtime-configurable verbosity (via env var) so debug logging can be toggled easily during dev iteration (comes from `env_logger`)
  * Ability to silence or capture logs in tests

#### **Alternatives**
  * `tracing`
    * *Pros:* Structured, span-based instrumentation; rich features for async and performance tracing.
    * *Cons:* Higher setup and API complexity; overkill for a small CLI; additional dependency weight.
  * `fern`
    * *Pros:* Fluent builder for flexible output configurations (console, file, formats, colorized output, multiple sinks).
    * *Cons:* More boilerplate and configuration; unnecessary complexity for our simple use-case. Still depends on `log` macros but need to wire up `fern::Dispatch`.
  * No crate
    * *Pros:* Zero dependencies; simplest implementation; trivial to call `eprintln!("[WARN] skipping row")`.
    * *Cons:* Lacks levels and filtering; no runtime control over verbosity; harder to disable logs in tests.

#### **Decision**
  Adopt `log` + `env_logger` for straightforward, leveled logging.

#### **Details**
  I chose `log` paired with `env_logger` because it's widely-adopted with minimal setup and simple usage for leveled logging, and as a bonus it's configurable at runtime via `RUST_LOG`. It directs logs to STDERR by default (preserving STDOUT for CSV output) and integrates with standard `log` macros without extra boilerplate. While `tracing` offers structured spans and advanced features, it adds complexity and dependency weight that's not needed for a CLI; `fern` provides more customization but with extra setup; and building from scratch with `eprintln!` would sacrifice filtering and level control.

---

### 5. Error Handling

#### **Best Option:`thiserror`**
  * Derive error enums with minimal boilerplate
  * Explicit variants ensure type safety
  * Auto-implements `From` conversions and `Display` implementations for clear messages.
  * The `#[from]` attribute auto-implements `From<std::io::Error> for ApplicationError`
  * The `#[source]` attribute marks the inner error so context is retained for callers of `.source()` (not explicitly relevant here but worth noting)
  * Top-level functions can now return `Result<T, ApplicationError>`, and `?` will convert lower-level errors automatically

#### **Alternatives**
  * **`anyhow`**
    * *Pros:* Single, dynamic error type (`anyhow::Error`) that can wrap any error; extremely ergonomic for prototyping.
    * *Cons:* Loses compile-time enumeration of error kinds; harder to match on specific cases; less type-safe.
  * **`snafu`**
    * *Pros:* Rich context attachment and backtrace support; declarative derive-based API for error definitions.
    * *Cons:* Steeper learning curve; more macro complexity and boilerplate than `thiserror`.
  * **`error-chain`**
    * *Pros:* Macro-based error definitions with automatic chaining (historically popular).
    * *Cons:* Deprecated and unmaintained; bloated API; not recommended for new projects.
  * **No crate**
    * *Pros:* Zero dependencies; full control over error type and conversions.
    * *Cons:* Verbose boilerplate (`enum` definitions, `impl From`, `impl Display`); easy to omit conversions and lose consistency.

#### **Decision**
  Use `thiserror` for concise, compile-time error types, seamless `?`-based propagation, and maintainable error messages.

#### **Details**
  I selected `thiserror` because it allows defining a strongly-typed `enum ApplicationError` with minimal boilerplate. The `#[error("...")]` macro generates `Display` implementations, the `#[from]` attribute auto-implements conversions from underlying errors (`std::io::Error`, `csv::Error`, etc.), and the `#[source]` attribute enables standard error chaining so you retain the context of inner causes. Alternatives like `anyhow` trade type safety for convenience, `snafu` adds complexity, `error-chain` is deprecated, and building error types from scratch would be verbose and error-prone.

---

### 6. Testing & Test Helpers

#### **Best Option: Built-in `#[test]` + `rstest` for simple Unit Tests**
  * No extra dependencies for unit testing
  * `rstest` macros enable parameterized tests, reducing boilerplate and improving readability.
  * Still requires some setup/teardown code
  * Does not cover integration or CLI-level testing on its own

#### **Best Option: `assert_cmd` + `predicates` for more comprehensive Integration Testing**
  * Facilitates end-to-end CLI tests by spawning the binary, asserting on stdout/stderr and exit status
  * Integrates seamlessly with Cargo test harness
  * It does involve spawning external processes, leading to slower test execution compared to in-process tests, but for this task it's still fast enough

#### **Alternatives**
  * `test-case`
    * *Pros:* Lightweight parameterized tests with table-driven syntax; fewer macros than `rstest`.
    * *Cons:* Smaller community and ecosystem; less active maintenance.
  * `proptest` / `quickcheck`
    * *Pros:* Property-based testing that automatically generates diverse inputs and uncovers edge cases.
    * *Cons:* Steeper learning curve; non-deterministic test failures can be harder to debug.
  * `insta`
    * *Pros:* Snapshot testing for large outputs (e.g., CSV), making regression tests simple to write.
    * *Cons:* Snapshots can be brittle—minor formatting changes break tests and require snapshot updates.
  * No crate
    * *Pros:* Zero dependencies; rely solely on Rust’s built-in test framework.
    * *Cons:* Manual setup for CLI tests; lacks conveniences for parameterization and output assertions.

#### **Decision**
  Combine the built-in `#[test]` harness with `rstest` for DRY (don't repeat yourself -- avoid copy/paste code), readable unit tests, and leverage `assert_cmd` with `predicates` for concise, robust integration tests.

#### **Details**
  It makes sense to use Rust’s native test framework augmented by `rstest` to define parameterized unit tests. With little effort repetitive code for similar Unit Test cases can be reduced. For full CLI Integration Tests, `assert_cmd` alongside `predicates` can execute the binary in a controlled environment and verify both CSV output and log behavior. This approach strikes a balance between simplicity, test coverage, readability and maintainability. Property-based frameworks like `proptest` offer deeper fuzzing but add complexity, while snapshot tools like `insta` can break on innocuous formatting changes; opting for `rstest` for Unit Tests and `assert_cmd` + `predicates` for Integration Tests ensures predictable, easy-to-understand tests without heavy dependencies.

  I also considered adding something like `logtest` to validate expected errors and warnings in Unit Tests. It might make sense in a production-grade project, but I'm considering it out of scope for now, so I didn't list alternatives or a full description.

---

### 7. Continuous Integration (Not Implemented Yet)

  Continuous Integration is a nice-to-have if time permits. In production quality projects, I'd expect to ensure code quality and consistency across commits. It's a bit overkill for this task, but in terms of next steps I would take if this task were a real-life project, CI would probably be first among them.

  **GitHub Actions** is the obvious choice since this task is submitted as a GitHub repository. A simple workflow would:
  * Checkout the code
  * Install the Rust toolchain
  * Cache dependencies and build artifacts
  * Run `cargo fmt -- --check` to enforce code formatting
  * Run `cargo clippy -- -D warnings` to enforce lint rules
  * Run `cargo test` to validate functionality

#### **Alternatives**
  Other CI services like Travis CI, CircleCI, and GitLab CI exist, but since this code lives on GitHub, integrating with GitHub Actions is the most seamless option and avoids external tooling.

#### **Decision**
  Add basic CI with GitHub Actions if time allows; otherwise, omit as non-critical.

---

### 8. Benchmarking (Not Implemented Yet)

#### **Best Option: `criterion`**
  * Statistically rigorous measurements (warm-up, multiple samples, outlier rejection)
  * Built-in plotting and comparison reports
  * Easy to write benchmarks alongside your code
  * `#[bench]`-style API via `criterion::Criterion` and `criterion_group!` macros
  * Tuning parameters (sample size, measurement time) to suit your needs
  * Ability to compare multiple implementations side by side

#### **Alternatives**
  * Built-in `#[bench]`
    * *Pros:* Uses Rust’s standard library benchmark API; no extra crates required.
    * *Cons:* Unstable feature; lacks advanced configuration and statistical analysis.
  * **Manual timing**
    * *Pros:* Zero dependencies; quick to implement using `std::time::Instant`.
    * *Cons:* No statistical rigor; results can be noisy and hard to interpret reliably.
  * **No crate**
    * *Pros:* Zero dependencies; preserves focus on core functionality and tests.
    * *Cons:* No automated performance validation or regression detection.

#### **Decision**
  Skip detailed benchmarking for now to prioritize correctness, maintainability, and comprehensive testing.

#### **Details**
  Although benchmarks can validate performance claims and detect regressions, for this task it’s more important to implement core features and tests. In a production setting, I would use `criterion` to establish performance baselines, compare implementations, and generate HTML reports. Criterion’s statistical rigor and reporting tools make it the standard for Rust benchmarking, but here it’s an optional enhancement rather than a requirement.

---
