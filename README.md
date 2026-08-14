# CASIROS

A NASA/JPL-grade Financial ERP Operating System, in Rust. Every formula is pure. Every transaction is causal. Every audit trail is mathematically provable.

## Design principles

- **Decimal-only.** All money, rates, and ratios are `rust_decimal::Decimal`. `f64` appears nowhere except Monte Carlo noise sampling, and even there it's converted back to `Decimal` before touching any calculation.
- **No panics in domain code.** Every fallible operation returns `Result`; arithmetic uses `checked_*` throughout. `#![forbid(unsafe_code)]` in every crate.
- **Causality, not just computation.** The DAG engine (`casiros-dag`) tracks which formula's output feeds which formula's input, and the ERP ledger tracks which journal entry causally produced which other entry. A number is never just a number — it has a traceable origin.
- **Layered dependencies.** `core` → `dag`/`simulator`/`erp` → `api`. Inner layers never import outer ones; no infrastructure code leaks into domain logic.

## Workspace

| Crate | Purpose |
|---|---|
| [`casiros-core`](crates/core) | Pure financial formulas — TVM, ratios, banking, markets, bonds, corporate finance. No I/O, no state. |
| [`casiros-dag`](crates/dag) | The causality graph: a generic DAG engine, plus `FormulaNode`/`evaluate_dag` for wiring `core` formulas together by data dependency. |
| [`casiros-simulator`](crates/simulator) | Monte Carlo scenario generation and aggregation over `core` formulas ("the multiverse engine"). |
| [`casiros-erp`](crates/erp) | Enterprise logic: causal general ledger, accounts payable/receivable, treasury (cash forecasting, FX, hedging), multi-jurisdiction tax, driver-based budgeting, and CFO-memo narrative generation. |
| [`casiros-macros`](crates/macros) | The `generate_narrative!` proc macro — compile-time-checked syntax for building a narrative memo from named metrics. |
| [`casiros-api`](crates/api) | The Actix-Web server: REST endpoints, WebSocket streaming, OpenAPI/Swagger docs, rate limiting, request tracing. |

Plus two standalone (non-workspace) crates for tooling, and the frontend:

| Directory | Purpose |
|---|---|
| [`benches/`](benches) | Criterion benchmarks for the hot paths (core formulas, the DAG evaluator, the Monte Carlo engine, ledger posting). |
| [`fuzz/`](fuzz) | `cargo-fuzz` targets proving core formulas, the DAG evaluator, and journal-line validation reject adversarial input via `Result`, never a panic. |
| [`frontend/`](frontend) | Mission control — React + TypeScript UI, including a 3D Monte Carlo scenario visualizer. See its own [README](frontend/README.md). |

See [`ROADMAP.md`](ROADMAP.md) for what's shipped versus planned.

## Quick start

Requires Rust **1.88** (pinned via `rust-toolchain.toml`) or Docker.

```sh
# Run the API server locally
make run
# → http://127.0.0.1:8080/healthz
# → http://127.0.0.1:8080/swagger-ui/

# ...or build and run it in Docker
make docker-run

# ...or bring up the full stack (api + db + redis) via docker compose
make up
```

Run `make help` for the full list of targets. To run the frontend against it:

```sh
make frontend-install   # once
make frontend
# → http://localhost:5173
```

## API

Interactive documentation (Swagger UI) is served at `/swagger-ui/` whenever the server is running, backed by the raw OpenAPI document at `/api-docs/openapi.json`.

| Route | Description |
|---|---|
| `GET /healthz` | Liveness check |
| `POST /api/v1/calculate/{formula}` | Evaluate a single `core` formula by name |
| `POST /api/v1/simulate` | Run a Monte Carlo simulation synchronously |
| `GET /ws/simulate` | Stream a Monte Carlo simulation's progress over WebSocket |
| `POST /api/v1/narrative` | Generate a CFO-style memo from computed metrics |
| `/api/v1/ledger/*` | Chart of accounts, journal posting, trial balance |
| `/api/v1/journal/*` | Journal entry posting and listing |
| `/api/v1/ap/*` | Suppliers, AP invoices, aging, payment proposals |
| `/api/v1/ar/*` | Customers, AR invoices, receipt allocation |
| `/api/v1/treasury/*` | Cash forecasting, FX conversion, hedge effectiveness |

## Development

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo deny check       # license/advisory/dependency policy
cargo audit             # security advisories (config: .cargo/audit.toml)
```

The workspace enforces `-D warnings -D clippy::pedantic -D clippy::cognitive_complexity` globally via `.cargo/config.toml`. Every public function in `casiros-core` and `casiros-erp` carries a doc-test with at least two assertions — these run as part of `cargo test --workspace` alongside ~300 unit and integration tests.

### Benchmarks

```sh
cargo bench --manifest-path benches/Cargo.toml
```

### Fuzzing

```sh
cargo +nightly fuzz run core_formulas
cargo +nightly fuzz run dag_evaluation
cargo +nightly fuzz run erp_journal
```

## Docker

```sh
make docker-build   # build the casiros-api image
make docker-run      # build + run standalone on :8080
make up               # docker compose: api + db + redis
make down
```

`db` and `redis` are scaffolded in `docker/docker-compose.yml` per the target architecture but not yet wired into the app — `casiros-api` is currently fully in-memory (see `crates/api/src/state.rs`).

## Configuration

The server reads `CASIROS_SERVER__BIND_ADDR` directly from the environment (default `127.0.0.1:8080`). `config/*.toml` ship as the documented template for the `CASIROS_<SECTION>__<KEY>` override convention once full config-file loading is wired in.

## License

MIT OR Apache-2.0.
