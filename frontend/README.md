# CASIROS Mission Control

The frontend for CASIROS: five screens over the `casiros-api` REST/WebSocket surface, built around one idea — a number without its origin isn't trustworthy, so nothing here shows a value without also showing where it came from.

## Screens

- **Overview** — entry points into the rest of the app.
- **The Multiverse** — runs a Monte Carlo simulation via `/ws/simulate` and renders the result as a rotating 3D point field. Since the API reports the *aggregate* distribution (min/percentiles/mean/max), not raw per-scenario values, each axis is independently reconstructed from its own real percentile breakpoints — exact per-axis shape, illustrative cross-axis correlation, stated plainly in the UI rather than implied. See `src/features/multiverse/percentile-sampling.ts`.
- **Calculator** — evaluates any of the 41 scalar `casiros-core` formulas exposed by `/api/v1/calculate/{formula}` (`discounted_cash_flow`/`duration`/`convexity` take a cash-flow series, which that endpoint doesn't accept, so they're not offered here).
- **Narrative** — drives `/api/v1/narrative`, rendering the returned markdown memo as formatted prose.
- **Ledger** — chart of accounts, journal entry posting, and the trial balance, against `/api/v1/ledger/*` and `/api/v1/journal/*`.

## Stack

Vite + React 19 + TypeScript, Tailwind CSS v4, a small hand-rolled design system on top of Radix primitives (no shadcn CLI dependency — the primitives in `src/components/ui/` are the entire thing), TanStack Query for server state, React Router, and `@react-three/fiber` (code-split into its own chunk — it's the only screen that needs three.js) for the Multiverse view.

## Architecture

```
src/
  api/          typed client + one module per casiros-api route group
  components/
    ui/          design system primitives (button, card, input, select, ...)
    layout/      sidebar, topbar, shell
  features/
    <domain>/    one folder per screen: page component + its own hooks/helpers
  routes/        router wiring
  lib/           cn(), Decimal-string formatting
```

Every `Decimal` the API returns crosses the wire as a JSON string (see `casiros-api`'s own doc comments on why) — `api/types.ts` documents this via a `DecimalString` alias, and `lib/format.ts` is the only place that parses one for display.

## Running it

```sh
npm install
npm run dev
```

Requires `casiros-api` running and reachable — `make run` or `make docker-run` from the repo root. Defaults to `http://127.0.0.1:8080`; override via `VITE_API_BASE_URL` (see `.env.example`).

```sh
npm run build   # type-checks (tsc -b) then builds
npm run lint
```
