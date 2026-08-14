# CASIROS Roadmap

> Phases 0–7 built the physics engine, the causality layer, the multiverse, the ERP, and the API that exposes all of it. Every number in this system can already be traced back to the formula and the scenario that produced it. Phase 8 built the room to *see* that — mission control, not just a mission.

Status legend: `[x]` shipped and verified · `[~]` in progress · `[ ]` planned.

---

## Phases 0–7: Foundation — COMPLETE

The full build described in `CASIROS_BUILD_PROMPT.md` is implemented, tested, and pushed.

- [x] **Phase 0 — Foundry.** Workspace scaffolding, `deny.toml`/`clippy.toml`, the error/types kernel, per-crate compiler directives (`forbid(unsafe_code)`, `deny(missing_docs)`, `deny(clippy::pedantic)`).
- [x] **Phase 1 — Mathematical Kernel.** Every TVM, ratio, banking, market-risk, bond, and corporate-finance formula in `casiros-core`, each with a doc-tested example and `checked_*` arithmetic throughout. Property-based invariant tests via `proptest`.
- [x] **Phase 2 — Causality Engine.** `FormulaNode` (44 variants) + `CausalityEngine` + `evaluate_dag`, with parameter-name-based wiring between formulas and hard cycle detection.
- [x] **Phase 3 — Multiverse Simulator.** `Universe`/`UniverseMetrics`, Rayon-parallel Monte Carlo scenario generation, percentile aggregation.
- [x] **Phase 4 — ERP Core.** Causal general ledger, AP, AR (ASC 606 recognition), treasury (cash forecasting, FX, hedge effectiveness), multi-jurisdiction tax, driver-based budgeting.
- [x] **Phase 5 — API & Infrastructure.** Actix-Web, REST + WebSocket streaming, full OpenAPI/Swagger surface, rate limiting, request tracing.
- [x] **Phase 6 — Macros & Narrative.** `generate_narrative!`, CFO memo generation wired into the API.
- [x] **Phase 7 — Hardening.** Fuzz targets, Criterion benchmarks, multi-stage Docker build, `cargo audit`/`cargo deny`. CI/CD workflow is written and held locally pending explicit go-ahead to push.

Additionally, ahead of Phase 8: a full checklist-and-coverage audit closed every gap against the project's own section-14 discipline — `dag` and `core` are effectively at 100%/94% line coverage, `erp` and `api` both exceed their targets, and every public function in `core`/`erp` carries a doc-tested example.

---

## Phase 8: Mission Control — the frontend

Every phase so far answers *"can the number be computed?"* This phase answers *"can a human see it, trust it, and steer it?"*

- [x] **8.1 — Foundation.** `frontend/`: Vite + React 19 + TypeScript, Tailwind v4, a hand-rolled Radix-based design system (no framework lock-in), TanStack Query against the existing REST surface, a typed API client mirroring every `casiros-api` route.
- [x] **8.2 — The Multiverse view.** The centerpiece. A real-time 3D scenario field (`react-three-fiber`, code-split into its own chunk): axes are independently sampled from the real percentile statistics `/ws/simulate` returns (min/p5/p25/median/p75/p95/max — the API reports the *aggregate* distribution, never raw per-scenario values, so this is stated plainly in-app rather than implied). Orbit controls — turn the universe, look at it from another angle. Caught and fixed a real bug during verification: drei's `<Text>` (SDF font loading via `troika-three-text`) silently crashed the entire scene graph in at least one real browser environment with zero console output — replaced with an HTML overlay for axis captions, found only by bisecting the scene down to a single mesh and rebuilding it piece by piece under a headless browser with direct WebGL-framebuffer verification (not just an OS-level screenshot, which a `toDataURL()` readback proved could itself be misleading).
- [x] **8.3 — Formula Calculator.** 41 of the 44 `casiros-core` formulas (the 3 cash-flow-series formulas — `discounted_cash_flow`, `duration`, `convexity` — aren't exposed by `/api/v1/calculate` itself, by the API's own design), each driven by a form generated from the formula's own parameter names.
- [x] **8.4 — CFO Narrative.** A live-editable metrics panel that renders the generated memo as formatted prose, not raw markdown.
- [x] **8.5 — Ledger.** Chart of accounts, trial balance (correctly excluding roll-up parents from the "nets to zero" check — summing a parent *and* its children would double-count), journal posting.
- [ ] **8.6 — Full ERP surface.** AP/AR/treasury/tax/budget get the same UI depth as the ledger — invoice tables, aging reports, payment proposals, cash forecasting charts, variance analysis. Deliberately deferred out of the initial pass so the shipped pages are complete rather than seven shallow ones.
- [ ] **8.7 — Causality inspector.** A visual DAG explorer over `casiros-dag`'s `execution_order()` — click any computed number in the UI and see the exact chain of formulas and inputs that produced it. This is the literal fulfillment of the ledger's own audit-trail promise ("<3 clicks from a number to its origin," per `CASIROS_BUILD_PROMPT.md` §15) at the UI layer, not just the API layer.

## Phase 9: Persistence

The ERP is currently fully in-memory (`Mutex`-guarded state, gone on restart) by explicit, documented design choice through Phase 8. Real usage needs it to survive a restart.

- [ ] PostgreSQL-backed ledger, chart of accounts, AP/AR, and treasury state (the schema stubs already exist in `docker/docker-compose.yml`'s `db` service).
- [ ] Migration tooling (`sqlx` or `sea-orm`) with the same causal-integrity guarantees the in-memory `Ledger` enforces today.
- [ ] Redis-backed rate limiting and simulation-result caching, replacing the in-process `RateLimiter`.

## Phase 10: Identity & Access

- [ ] Authentication (OIDC) and per-entity authorization — today every route is open.
- [ ] Multi-tenant chart-of-accounts isolation.
- [ ] Audit log of *who* triggered each journal entry, on top of the *why* the causal ledger already tracks.

## Phase 11: Observability & Deployment

- [ ] Structured metrics export (Prometheus) alongside the existing `tracing` spans.
- [ ] The CI/CD pipeline already staged in `.github/workflows/ci.yml` goes live.
<!-- - [ ] Production Kubernetes/ECS deployment manifests on top of the existing Dockerfile. -->

## Phase 12: Looking Back

A system this rigorous about causality should be able to reason about it, not just record it.

<!-- - [ ] Historical replay: re-run a past period's Monte Carlo simulation against what *actually* happened, and surface where reality fell inside — or outside — the simulated distribution. -->
- [ ] Formula-level sensitivity analysis: for any computed number, which upstream input moved it the most?
- [ ] An LLM-assisted narrative layer on top of `generate_narrative!` — the deterministic memo stays the source of truth; the model only explains it in plainer language, never computes a number itself.

---

*Everything from 8.6 onward is a plan, not a promise. Phases 0–8.5 are shipped, verified against a live backend (not just built), and in `main`.*
