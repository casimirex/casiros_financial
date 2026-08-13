# CASIROS Financial ERP — Claude Code Implementation Prompt

> **Mission:** Build CASIROS, a NASA/JPL-grade Financial ERP Operating System in Rust (Edition 2024). This is not a toy. Every formula is pure. Every transaction is causal. Every audit trail is mathematically provable. Treat this as flight software.

---

## 1. Agent Identity & Development Discipline

You are a principal Rust engineer with NASA JPL flight software discipline. You write code that would survive a Mars landing. Your constraints are absolute:

1. **TDD FIRST:** Write the doc-test BEFORE the function body. Watch it fail. Then implement.
2. **NO PANICS IN DOMAIN CODE:** Every fallible operation returns `Result<T, CalculationError>`. `.unwrap()` and `.expect()` are FORBIDDEN outside `#[cfg(test)]` blocks.
3. **DECIMAL ONLY:** `rust_decimal::Decimal` for ALL money, rates, and ratios. `f64` is allowed ONLY for stochastic noise generation in the Monte Carlo engine and must be immediately converted to `Decimal`.
4. **NO UNSAFE:** `#![forbid(unsafe_code)]` in EVERY crate. No exceptions.
5. **NO RECURSION IN CORE:** All algorithms use iteration. If recursion is unavoidable in DAG/tree crates, it must have an explicit depth limit.
6. **FUNCTION LENGTH:** Maximum 60 lines per function body (excluding doc comments and blank lines).
7. **ASSERTION DENSITY:** Every public function must have ≥2 assertions across doc-tests and unit tests.
8. **PURE DOMAIN:** Core crate functions are pure — no I/O, no global state, no side effects. Same input → same output.
9. **LAYER DISCIPLINE:** Domain (`core`) → Application (`dag`, `simulator`, `erp`) → Infrastructure (`api`). Dependencies point INWARD. Inner layers never import outer layers.
10. **TRAIT BOUNDARIES:** Every layer boundary is a trait defined by the inner layer, implemented by the outer layer.

---

## 2. Workspace Genesis — Phase 0

Create the following workspace structure EXACTLY. Do not deviate.

```
casiros/
├── Cargo.toml                      # Workspace root
├── .cargo/
│   └── config.toml                 # Global compiler flags
├── rust-toolchain.toml             # Pin stable Rust 2024
├── clippy.toml                     # Workspace Clippy config
├── deny.toml                       # cargo-deny config
├── config/
│   ├── default.toml
│   ├── development.toml
│   └── production.toml
├── crates/
│   ├── core/                       # Domain Layer — The Physics Engine
│   ├── dag/                        # Application Layer — Causality Graph
│   ├── simulator/                  # Application Layer — Multiverse Engine
│   ├── erp/                        # Application Layer — ERP Controllers
│   ├── api/                        # Infrastructure Layer — HTTP + WebSocket
│   └── macros/                     # Procedural Macros — Narrative Engine
├── benches/
├── fuzz/
├── docker/
└── .github/workflows/
```

### 2.1 Root `Cargo.toml`

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["CASIROS Engineering <engineering@casiros.dev>"]
license = "MIT OR Apache-2.0"
rust-version = "1.82"

[workspace.dependencies]
rust_decimal = { version = "1.36", features = ["maths"] }
rust_decimal_macros = "1.36"
thiserror = "2.0"
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.40", features = ["full"] }
```

### 2.2 `.cargo/config.toml`

```toml
[build]
rustflags = [
    "-D", "warnings",
    "-D", "missing_docs",
    "-D", "clippy::pedantic",
    "-D", "clippy::cognitive_complexity",
    "-D", "clippy::recursion",
    "-D", "unreachable_pub",
    "-D", "rust_2018_idioms",
    "-D", "rust_2024_compatibility",
]
```

### 2.3 `rust-toolchain.toml`

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

### 2.4 `clippy.toml`

```toml
cognitive-complexity-threshold = 10
too-many-arguments-threshold = 7
```

---

## 3. The Core Crate — Domain Layer

This is the mathematical kernel. ZERO dependencies on other CASIROS crates. Only `rust_decimal`, `thiserror`, and `tracing` (for `#[instrument]` in non-hot paths only).

### 3.1 `crates/core/Cargo.toml`

```toml
[package]
name = "casiros-core"
version.workspace = true
edition.workspace = true

[dependencies]
rust_decimal.workspace = true
rust_decimal_macros.workspace = true
thiserror.workspace = true

[dev-dependencies]
proptest = "1.5"
```

### 3.2 `crates/core/src/lib.rs`

This file establishes the compile-time guarantees. Copy this EXACTLY.

```rust
//! # CASIROS Core Mathematics
//!
//! The fundamental, immutable financial formulas that form the computational backbone.
//!
//! ## Design Principles
//!
//! - **Purity:** Every public function is a pure computation. Same input → same output. No I/O.
//! - **Decimal Precision:** All monetary values use [`rust_decimal::Decimal`]. `f32`/`f64` is BANNED.
//! - **Defensive:** Every function validates preconditions and returns [`Result<T, CalculationError>`].
//! - **Documented:** Every public item has a doc-comment with at least one comprehensive doc-test.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(clippy::implicit_return)]
#![deny(clippy::cognitive_complexity)]
#![deny(clippy::recursion)]
#![deny(unreachable_pub)]
#![deny(rust_2018_idioms)]
#![deny(rust_2024_compatibility)]
#![allow(clippy::needless_return)]

pub mod prelude {
    //! Re-exports for ergonomic use across the workspace.
    pub use crate::error::CalculationError;
    pub use crate::types::{Amounts, Dollar, Periods, Rate, Ratio};
    pub use rust_decimal::Decimal;
    pub use rust_decimal_macros::dec;
}

pub mod error;
pub mod types;
pub mod general;
pub mod financial;
pub mod banking;
pub mod markets;
pub mod stocks_bonds;
pub mod corporate;
```

### 3.3 `crates/core/src/error.rs`

This is the universal error type. ALL fallible operations return this.

```rust
use rust_decimal::Decimal;
use thiserror::Error;

/// The universal error type for all CASIROS computations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CalculationError {
    #[error("Division by zero in {formula}")]
    DivisionByZero { formula: &'static str },

    #[error("Invalid value {value} in {context}: must be strictly positive")]
    NegativeValueInvalid { context: &'static str, value: Decimal },

    #[error("Value {value} in {context} is outside the valid range [0, 1]")]
    RangeViolation { context: &'static str, value: Decimal },

    #[error("Cannot compute logarithm of {value}: must be strictly positive")]
    LogarithmDomainError { value: Decimal },

    #[error("Invalid rate {rate}: must be greater than -1.0")]
    InvalidRate { rate: Decimal },

    #[error("Numeric overflow in {formula}")]
    Overflow { formula: &'static str },

    #[error("{formula} failed to converge after {iterations} iterations")]
    ConvergenceFailure { formula: &'static str, iterations: u32 },

    #[error("Missing required input '{parameter}' for formula '{formula}'")]
    MissingInput { formula: &'static str, parameter: &'static str },
}
```

### 3.4 `crates/core/src/types.rs`

Newtype pattern for type safety. No naked `Decimal`s in function signatures.

```rust
use rust_decimal::Decimal;

/// Monetary value in the base currency (e.g., USD).
pub type Dollar = Decimal;

/// An interest rate, discount rate, or growth rate expressed as a decimal.
/// Example: 5% = `dec!(0.05)`.
pub type Rate = Decimal;

/// A dimensionless ratio (e.g., 0.6 for 60%).
pub type Ratio = Decimal;

/// A number of compounding periods (years, months, quarters).
pub type Periods = u32;

/// The three fundamental time-value-of-money quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amounts {
    /// Present value (PV)
    pub principal: Dollar,
    /// Future value (FV)
    pub future_value: Dollar,
    /// Periodic payment (PMT)
    pub payment: Dollar,
}

impl Amounts {
    /// Creates a new `Amounts` with all fields validated as non-negative.
    pub fn new(
        principal: Dollar,
        future_value: Dollar,
        payment: Dollar,
    ) -> Result<Self, crate::error::CalculationError> {
        use crate::error::CalculationError;
        if principal < Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "Amounts::principal",
                value: principal,
            });
        }
        if future_value < Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "Amounts::future_value",
                value: future_value,
            });
        }
        if payment < Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "Amounts::payment",
                value: payment,
            });
        }
        Ok(Self { principal, future_value, payment })
    }
}
```

### 3.5 Formula Implementation Pattern

EVERY formula in the core crate MUST follow this EXACT pattern:

```rust
/// [Short description]
///
/// # Mathematical Definition
///
/// \[ Formula = \frac{Numerator}{Denominator} \]
///
/// # Constraints
///
/// - `param1` MUST be >= 0.
/// - `param2` MUST be > 0.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `param2` is zero.
///
/// # Examples
///
/// ```
/// use casiros_core::[module]::[function];
/// use rust_decimal_macros::dec;
///
/// let result = [function](dec!(100.0), dec!(50.0)).unwrap();
/// assert_eq!(result, dec!(2.0));
/// assert!(result > dec!(0.0));
/// ```
pub fn [function](param1: Decimal, param2: Decimal) -> Result<Decimal, CalculationError> {
    // Precondition checks FIRST
    if param1 < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "[function] - param1",
            value: param1,
        });
    }
    if param2 == Decimal::ZERO {
        return Err(CalculationError::DivisionByZero {
            formula: "[Function Name]",
        });
    }

    // Computation
    Ok(param1 / param2)
}
```

### 3.6 Required Formula Modules

Implement ALL of the following modules with full doc-tests and unit tests:

**`general.rs`** — Time Value of Money
- `future_value`, `present_value`, `annuity_future_value`, `annuity_present_value`
- `perpetuity_present_value`, `growing_perpetuity`, `effective_annual_rate`, `continuous_compounding`

**`financial.rs`** — Financial Ratios
- `return_on_equity`, `return_on_assets`, `return_on_investment`
- `profit_margin`, `asset_turnover`, `equity_multiplier`, `dupont_roe`
- `current_ratio`, `quick_ratio`, `debt_to_equity`, `interest_coverage`
- `inventory_turnover`, `cash_conversion_cycle`

**`banking.rs`** — Banking Metrics
- `net_interest_margin`, `loan_to_deposit_ratio`, `capital_adequacy_ratio`, `provision_coverage`

**`markets.rs`** — Market Metrics
- `beta`, `sharpe_ratio`, `treynor_ratio`, `jensens_alpha`
- `value_at_risk`, `expected_shortfall`

**`stocks_bonds.rs`** — Equity & Fixed Income
- `dividend_discount_model`, `discounted_cash_flow`, `bond_price`
- `yield_to_maturity` (Newton-Raphson, max 100 iterations, return `ConvergenceFailure` if exceeded)
- `duration`, `modified_duration`, `convexity`

**`corporate.rs`** — Corporate Finance
- `wacc`, `free_cash_flow_to_firm`, `free_cash_flow_to_equity`
- `economic_value_added`, `sustainable_growth_rate`, `internal_growth_rate`

For `yield_to_maturity`, use Newton-Raphson with `const MAX_NEWTON_ITERATIONS: u32 = 100`. Clamp iterations. No recursion.

---

## 4. The DAG Crate — Causality Engine

### 4.1 `crates/dag/Cargo.toml`

```toml
[package]
name = "casiros-dag"
version.workspace = true
edition.workspace = true

[dependencies]
casiros-core = { path = "../core" }
petgraph = "0.6"
thiserror.workspace = true
```

### 4.2 `crates/dag/src/lib.rs`

```rust
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(warnings)]

pub mod graph;
pub mod evaluator;
```

### 4.3 `crates/dag/src/graph.rs`

Build a directed acyclic graph where:
- Nodes are `FormulaNode` enum variants (one per core formula)
- Edges represent data dependency (A → B means "B depends on A")
- Use `petgraph` for the underlying graph structure
- `CausalityEngine::execution_order()` returns topological sort
- Cycle detection is a HARD ERROR — return `Result<Vec<FormulaNode>, String>`

The `FormulaNode` enum must have a variant for EVERY public function in `casiros_core`. Example:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormulaNode {
    FutureValue,
    PresentValue,
    // ... all 44+ formulas
    Wacc,
    FreeCashFlowToFirm,
    SustainableGrowthRate,
}
```

### 4.4 `crates/dag/src/evaluator.rs`

`EvaluationContext` stores:
- `results: HashMap<FormulaNode, Decimal>` — computed outputs
- `inputs: HashMap<String, Decimal>` — raw inputs from the caller

`evaluate_dag(engine, ctx)` walks the topological order and calls the appropriate core function for each node. If a formula's inputs are not in `inputs` AND not in `results` (from prior nodes), return `CalculationError::MissingInput`.

---

## 5. The Simulator Crate — Multiverse Engine

### 5.1 `crates/simulator/Cargo.toml`

```toml
[package]
name = "casiros-simulator"
version.workspace = true
edition.workspace = true

[dependencies]
casiros-core = { path = "../core" }
casiros-dag = { path = "../dag" }
rayon = "1.10"
rand = "0.8"
rand_distr = "0.4"
rust_decimal.workspace = true
rust_decimal_macros.workspace = true
```

### 5.2 `crates/simulator/src/universe.rs`

Define `Universe` — a single economic scenario with ALL possible input variables (macroeconomic, company-specific, balance sheet, market). Define `UniverseMetrics` — the computed outputs for that universe.

### 5.3 `crates/simulator/src/monte_carlo.rs`

- `MonteCarloConfig` with `iterations`, `seed`, `track_convergence`, `convergence_threshold`, `convergence_batch_size`
- `generate_scenarios(baseline, config)` — perturbs baseline using `Normal` and `LogNormal` distributions. Immediately convert `f64` samples to `Decimal`.
- `run_simulation(scenarios)` — uses `rayon` parallel iteration to compute metrics for each universe
- `compute_universe_metrics(universe)` — calls core financial functions to populate `UniverseMetrics`

### 5.4 `crates/simulator/src/aggregation.rs`

`SimulationResults` aggregates across all universes:
- mean, median, stddev, min, max
- percentile_5, percentile_25, percentile_75, percentile_95

Use Welford's algorithm for numerically stable mean/variance computation. Convert to `f64` ONLY for `sqrt`, then back to `Decimal`.

---

## 6. The ERP Crate — Enterprise Logic

This is NEW. It sits in the Application Layer between Core and API.

### 6.1 `crates/erp/Cargo.toml`

```toml
[package]
name = "casiros-erp"
version.workspace = true
edition.workspace = true

[dependencies]
casiros-core = { path = "../core" }
casiros-dag = { path = "../dag" }
casiros-simulator = { path = "../simulator" }
serde.workspace = true
rust_decimal.workspace = true
thiserror.workspace = true
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.11", features = ["v4", "serde"] }
```

### 6.2 ERP Module Structure

```
crates/erp/src/
├── lib.rs
├── ledger/           # Causal General Ledger
│   ├── mod.rs
│   ├── account.rs    # Chart of accounts
│   ├── journal.rs    # Journal entries with causal links
│   ├── period.rs     # Fiscal periods
│   └── consolidation.rs
├── ap/               # Accounts Payable
│   ├── mod.rs
│   ├── invoice.rs
│   ├── payment.rs
│   └── supplier.rs
├── ar/               # Accounts Receivable
│   ├── mod.rs
│   ├── invoice.rs
│   ├── receipt.rs
│   └── customer.rs
├── treasury/         # Treasury & Cash
│   ├── mod.rs
│   ├── cashflow.rs
│   ├── fx.rs
│   └── hedge.rs
├── tax/              # Tax Engine
│   ├── mod.rs
│   ├── jurisdiction.rs
│   └── calculation.rs
└── budget/           # Budget & Forecast
    ├── mod.rs
    ├── model.rs
    └── variance.rs
```

### 6.3 Causal Ledger Design

A `JournalEntry` is NOT just debits and credits. It carries causal metadata:

```rust
#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub id: Uuid,
    pub date: NaiveDate,
    pub description: String,
    pub lines: Vec<JournalLine>,
    pub causal_parent: Option<Uuid>,  // What caused this entry?
    pub source_document: SourceDocument,
    pub period: FiscalPeriod,
}

#[derive(Debug, Clone)]
pub struct JournalLine {
    pub account: AccountCode,
    pub debit: Dollar,
    pub credit: Dollar,
    pub causal_formula: Option<FormulaNode>, // If computed, which formula?
}
```

The `TrialBalance` is computed via the DAG — NOT by summing a table. When a journal entry is posted, its affected accounts are marked dirty in the DAG, and balances recompute incrementally.

### 6.4 ERP Business Rules

ALL business rules are pure functions:

```rust
/// Determines if a payment can be approved based on liquidity.
/// Pure function. No database access.
pub fn can_approve_payment(
    payment_amount: Dollar,
    current_cash: Dollar,
    current_liabilities: Dollar,
) -> Result<bool, CalculationError> {
    let current_ratio = casiros_core::financial::current_ratio(current_cash, current_liabilities)?;
    Ok(payment_amount <= current_cash && current_ratio > dec!(1.2))
}
```

---

## 7. The API Crate — Infrastructure Layer

### 7.1 `crates/api/Cargo.toml`

```toml
[package]
name = "casiros-api"
version.workspace = true
edition.workspace = true

[dependencies]
casiros-core = { path = "../core" }
casiros-dag = { path = "../dag" }
casiros-simulator = { path = "../simulator" }
casiros-erp = { path = "../erp" }
actix-web = "4"
actix-cors = "0.7"
tokio.workspace = true
serde.workspace = true
tracing.workspace = true
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

### 7.2 Route Structure

```
crates/api/src/
├── main.rs
├── routes/
│   ├── mod.rs
│   ├── calculate.rs      # /api/v1/calculate/{formula}
│   ├── simulate.rs       # /api/v1/simulate
│   ├── ledger.rs         # /api/v1/ledger/*
│   ├── journal.rs        # /api/v1/journal/*
│   ├── ap.rs             # /api/v1/ap/*
│   ├── ar.rs             # /api/v1/ar/*
│   ├── treasury.rs       # /api/v1/treasury/*
│   └── health.rs         # /healthz
└── middleware/
    ├── mod.rs
    ├── tracing.rs
    └── rate_limit.rs
```

### 7.3 API Handler Pattern

EVERY handler MUST:
1. Have `#[instrument(name = "...", skip(...))]`
2. Return `Result<HttpResponse, AppError>`
3. Log at `info!` on success, `error!` on failure
4. Convert `Decimal` results to `String` in JSON (never `f64`)

Example:

```rust
#[derive(Debug, Deserialize)]
pub struct CalculateRequest {
    pub formula: String,
    pub params: HashMap<String, String>, // Parsed to Decimal
}

#[instrument(name = "POST /calculate", skip(req))]
pub async fn handle_calculate(
    req: web::Json<CalculateRequest>,
) -> Result<HttpResponse, AppError> {
    // Parse params to Decimal
    // Match formula string to core function
    // Return JSON with Decimal as string
}
```

### 7.4 WebSocket for Simulation Streaming

Implement `/ws/simulate` that:
1. Accepts a `MonteCarloConfig` JSON
2. Streams progress updates every N universes
3. Sends final `SimulationResults` as the last message
4. Uses `actix-web-actors` or raw WebSocket handlers

---

## 8. The Macros Crate — Narrative Engine

### 8.1 `crates/macros/Cargo.toml`

```toml
[package]
name = "casiros-macros"
version.workspace = true
edition.workspace = true

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
```

### 8.2 `generate_narrative!` Macro

Accepts key-value pairs of financial metrics and expands to a formatted `String` containing a CFO-style analysis memo.

Example expansion:

```rust
let memo = generate_narrative!(
    company: "Acme Corp",
    roe: dec!(0.15),
    debt_to_equity: dec!(0.8),
    current_ratio: dec!(2.0),
);
// Expands to:
// "## Financial Analysis Memo: Acme Corp\n\nReturn on Equity of 15.0% indicates..."
```

---

## 9. Testing Strategy

### 9.1 Test Pyramid

```
         ┌──────┐
         │ E2E  │  API integration tests
         ├──────┤
         │ Int  │  Cross-crate DAG + ERP tests
         ├──────┤
         │ Unit │  Per-function tests + doc-tests (≥2 assertions each)
         └──────┘
```

### 9.2 Property-Based Testing

Use `proptest` in `core` to verify mathematical invariants:

```rust
proptest! {
    #[test]
    fn present_value_is_inverse_of_future_value(
        pv in 0.0f64..1_000_000.0,
        rate in 0.0f64..0.5,
        periods in 1u32..50,
    ) {
        let pv_dec = Decimal::from_f64_retain(pv).unwrap();
        let rate_dec = Decimal::from_f64_retain(rate).unwrap();
        let fv = future_value(pv_dec, rate_dec, periods).unwrap();
        let recovered_pv = present_value(fv, rate_dec, periods).unwrap();
        let diff = (recovered_pv - pv_dec).abs();
        prop_assert!(diff < dec!(0.01));
    }
}
```

### 9.3 Coverage Targets

| Crate | Line Coverage | Branch Coverage |
|---|---|---|
| `core` | ≥95% | ≥90% |
| `dag` | ≥90% | ≥85% |
| `simulator` | ≥85% | ≥80% |
| `erp` | ≥85% | ≥80% |
| `api` | ≥80% | ≥75% |

---

## 10. CI/CD Pipeline

Create `.github/workflows/ci.yml` with these jobs running in parallel:

1. **Check & Lint:** `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`
2. **Test:** `cargo test --workspace --all-features`, `cargo test --doc --workspace`
3. **Coverage:** `cargo tarpaulin --workspace --out Xml` → upload to Codecov
4. **Security:** `cargo audit`, `cargo deny check`
5. **Docs:** `cargo doc --no-deps --document-private-items --workspace`
6. **Benchmarks:** `cargo bench --workspace` (only on `main`)

ALL jobs must pass before merge. No exceptions.

---

## 11. Docker & Deployment

### 11.1 `docker/Dockerfile`

Multi-stage build:
1. `rust:1.82-slim-bookworm` as builder
2. `debian:bookworm-slim` as runtime
3. Run as non-root user (UID 1000)
4. Copy `config/` to `/etc/casiros/`

### 11.2 `docker/docker-compose.yml`

Services:
- `api` (depends on db)
- `db` (PostgreSQL 16 with healthcheck)
- `redis` (for caching)

---

## 12. Configuration

### 12.1 `config/default.toml`

```toml
[server]
bind_addr = "127.0.0.1:8080"
workers = 4
request_timeout_secs = 30
max_body_size_bytes = 1_048_576

[simulation]
default_iterations = 10_000
max_iterations = 1_000_000
convergence_threshold = 0.001
convergence_batch_size = 1_000

[logging]
level = "info"
format = "json"
```

Environment overrides: `CASIROS_SERVER__BIND_ADDR`, `CASIROS_SIMULATION__DEFAULT_ITERATIONS`, etc.

---

## 13. Implementation Order

Build in this EXACT order. Do not skip phases.

### Phase 0: Foundry
1. Workspace scaffolding (`Cargo.toml`, `.cargo/`, `rust-toolchain.toml`)
2. `deny.toml`, `clippy.toml`
3. `crates/core/src/error.rs` and `types.rs`
4. `crates/core/src/lib.rs` with all compiler directives

### Phase 1: Mathematical Kernel
1. `general.rs` — all TVM formulas
2. `financial.rs` — all ratio formulas
3. `corporate.rs` — WACC, FCFF, SGR, etc.
4. `banking.rs`, `markets.rs`, `stocks_bonds.rs`
5. Unit tests and doc-tests for every function
6. Property-based tests for invariants

### Phase 2: Causality Engine
1. `FormulaNode` enum with all variants
2. `CausalityEngine` with graph construction
3. `EvaluationContext` and `evaluate_dag`
4. Dependency edge definitions
5. Cycle detection tests

### Phase 3: Multiverse Simulator
1. `Universe` and `UniverseMetrics` structs
2. `MonteCarloConfig`
3. `generate_scenarios` with distribution perturbation
4. `run_simulation` with Rayon parallelization
5. `SimulationResults` aggregation with percentiles

### Phase 4: ERP Core
1. `ledger/` — Causal journal entries, chart of accounts
2. `ap/` — Invoice parsing, payment proposals, aging
3. `ar/` — Revenue recognition (ASC 606), dunning, credit limits
4. `treasury/` — Cash forecasting, FX exposure
5. `tax/` — Multi-jurisdiction rules, deferred tax
6. `budget/` — Driver-based planning, variance analysis

### Phase 5: API & Infrastructure
1. Actix-Web server with tracing middleware
2. `/calculate/*` endpoints
3. `/simulate` endpoint (sync and WebSocket streaming)
4. `/ledger/*`, `/ap/*`, `/ar/*`, `/treasury/*` endpoints
5. Rate limiting, CORS, request ID propagation

### Phase 6: Macros & Narrative
1. `generate_narrative!` proc macro
2. CFO memo templates
3. Integration with ERP report generation

### Phase 7: Hardening
1. Fuzz testing targets
2. Benchmark suite with Criterion
3. Docker multi-stage build
4. GitHub Actions CI/CD
5. `cargo audit` and `cargo deny` integration

---

## 14. Code Review Checklist (Enforce on Every Commit)

Before declaring ANY task complete, verify:

- [ ] **Documentation:** Every new public function has a doc-comment with ≥1 doc-test containing ≥2 assertions.
- [ ] **Error Handling:** All fallible operations return `Result<T, CalculationError>`. No `.unwrap()` outside tests.
- [ ] **Precision:** All financial values use `Decimal`. `f64` only in random noise generation.
- [ ] **Preconditions:** Every public function validates ALL inputs before computing.
- [ ] **Function Length:** ≤60 lines per function body.
- [ ] **No Unsafe:** `#![forbid(unsafe_code)]` is not violated.
- [ ] **No Recursion:** No recursive functions in `core` crate.
- [ ] **Layer Discipline:** No infrastructure imports in domain/application crates.
- [ ] **Tracing:** Every API handler has `#[instrument]`.
- [ ] **Tests Pass:** `cargo test --workspace` is green.
- [ ] **Clippy Clean:** `cargo clippy --workspace -- -D warnings` is green.
- [ ] **Formatting:** `cargo fmt --all -- --check` passes.
- [ ] **DAG Updated:** If a new formula is added, `FormulaNode` and `CausalityEngine` edges are updated.
- [ ] **ERP Causality:** If a new transaction type is added, its causal parent relationship is documented.

---

## 15. Philosophical Guardrails

When in doubt, ask: *"Would this code survive a rocket launch?"*

- **Prefer explicit over implicit.** Explicit returns. Explicit error handling. Explicit type conversions.
- **Prefer immutability.** No `mut` unless absolutely necessary.
- **Prefer composition over inheritance.** Traits, not structs, define boundaries.
- **Every number tells a story.** If a user sees "15% ROE," they should be able to click through to the exact journal entries that produced it.
- **The multiverse is not a gimmick.** It is a risk management tool. Every material estimate must have a confidence interval.
- **Audit is not an afterthought.** It is the primary design constraint. If an auditor cannot trace a number to its origin in <3 clicks, the design is wrong.

---

## 16. Final Instruction

You are building the financial operating system for civilization. Treat every line of code as if it will be reviewed by a regulator, an auditor, and a rocket scientist — simultaneously.

**Begin with Phase 0. Do not proceed to Phase 1 until Phase 0 compiles with zero warnings. Do not proceed to Phase 2 until Phase 1 has 100% doc-test coverage. Build slowly. Build correctly. Build forever.**

---

*End of Implementation Prompt*
