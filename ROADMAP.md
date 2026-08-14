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
- [x] **8.6 — Full ERP surface.** Found a real backend gap first: `casiros-erp::tax` and `::budget` had full business logic since Phase 4 but zero HTTP routes — Phase 5 never exposed them. Added `/api/v1/tax/*` (progressive calculation, multi-jurisdiction aggregation, deferred tax position) and `/api/v1/budget/*` (drivers, driver-based line items, variance analysis) to `casiros-api`, each fully `#[utoipa::path]`-documented and wired into the OpenAPI surface, then built the frontend on top: AP (suppliers, invoices, an aging-bucket bar chart, payment proposals), AR (customers, invoices with ASC 606 recognition-method selection, receipt allocation), Treasury (cash forecast items + shortfall projection, FX conversion, hedge effectiveness), Tax (a reusable progressive-bracket editor driving both single- and multi-jurisdiction calculation, plus deferred tax), and Budget (drivers, driver-based line items, budget-vs-actual variance) — five new pages at the same UI depth as the Ledger page. Verified for real: every new route smoke-tested via `curl` against a rebuilt Docker container, then all 15 end-to-end flows (register → create → list → compute) driven through a headless Chromium session against the live backend, not just typechecked.
- [x] **8.7 — Causality inspector.** Two honest halves, not one glossed-over feature. **Formula graph:** `casiros-dag` had no way to introspect a formula's parameter wiring without re-running its evaluator, so added `FormulaNode::parameters()` (hand-transcribed from every `eval_*` function's `resolve` calls, cross-checked by a new test that runs real `evaluate_dag` calls and fails if any declared parameter turns out to be missing — a transcription omission would show up as a test failure, not a silent gap), `upstream_dependencies()` (which parameters match another formula's name, per the exact-name wiring convention), and `transitive_dependencies()`, which builds a real `CausalityEngine` and calls its actual `execution_order()` — the literal thing this line promised, not a reimplementation. Exposed via `/api/v1/causality/formulas` and `/formulas/{name}`. The result is genuinely uneven and reported as such: only `DupontRoe` (← `AssetTurnover`, `EquityMultiplier`) and `EconomicValueAdded` (← `Wacc`) have real formula-to-formula edges under the exact-name convention; every other formula's parameters are raw inputs, shown plainly rather than inventing conceptual edges the wiring doesn't actually support. **Journal lineage:** the causal ledger already carried `causal_parent`/`causal_formula` on every entry and line since Phase 4 — Phase 5 exposed them over the API but the frontend never gave a user any way to *set* them, so every entry was silently posted with both null. Added a causal-parent picker and a per-line causal-formula picker to the Ledger's journal-posting form, then built a lineage view that walks `causal_parent` back to its origin (client-side, over the same `/api/v1/journal/entries` the Ledger page already fetches — no new backend endpoint needed) and flags, rather than silently swallows, a parent id that doesn't resolve to any posted entry (the domain model doesn't validate `causal_parent` against existing entries at post time, so this is a real failure mode, not a hypothetical one). Verified for real: `curl` against the rebuilt container for both new formula endpoints, then a 10-check headless Chromium run that posts two causally-linked journal entries through the actual UI and confirms the lineage view reconstructs the real chain.

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

*Everything from Phase 9 onward is a plan, not a promise. Phase 8 in full is shipped, verified against a live backend (not just built), and in `main`.*
