// Every rust_decimal::Decimal crosses the wire as a JSON string (see
// crates/api/src/routes/calculate.rs's doc comment on CalculateResponse).
// This alias documents that convention at the type level.
export type DecimalString = string;

export interface CalculateRequest {
  params: Record<string, DecimalString>;
}

export interface CalculateResponse {
  formula: string;
  result: DecimalString;
}

export interface Universe {
  risk_free_rate: DecimalString;
  inflation_rate: DecimalString;
  market_return: DecimalString;
  portfolio_return: DecimalString;
  return_std_dev: DecimalString;
  revenue: DecimalString;
  cogs: DecimalString;
  operating_expenses: DecimalString;
  interest_expense: DecimalString;
  tax_rate: DecimalString;
  beta: DecimalString;
  cost_of_equity: DecimalString;
  cost_of_debt: DecimalString;
  total_assets: DecimalString;
  current_assets: DecimalString;
  inventory: DecimalString;
  current_liabilities: DecimalString;
  total_liabilities: DecimalString;
  total_equity: DecimalString;
  share_price: DecimalString;
  shares_outstanding: DecimalString;
}

export interface MonteCarloConfig {
  iterations: number;
  seed: number;
  track_convergence: boolean;
  convergence_threshold: DecimalString;
  convergence_batch_size: number;
}

export interface SimulateRequest {
  baseline: Universe;
  config: MonteCarloConfig;
}

export interface SimulationResults {
  sample_count: number;
  mean: DecimalString;
  median: DecimalString;
  std_dev: DecimalString;
  min: DecimalString;
  max: DecimalString;
  percentile_5: DecimalString;
  percentile_25: DecimalString;
  percentile_75: DecimalString;
  percentile_95: DecimalString;
}

export const METRIC_KEYS = [
  "ebit",
  "net_income",
  "profit_margin",
  "return_on_equity",
  "return_on_assets",
  "current_ratio",
  "quick_ratio",
  "debt_to_equity",
  "interest_coverage",
  "asset_turnover",
  "wacc",
  "sharpe_ratio",
] as const;
export type MetricKey = (typeof METRIC_KEYS)[number];

export interface SimulateResponse {
  scenarios_requested: number;
  scenarios_evaluated: number;
  scenarios_failed: number;
  metrics: Record<MetricKey, SimulationResults>;
}

// One evaluated scenario's raw metrics, as streamed point-by-point by
// /ws/simulate — see the "Multiverse" 3D view, which plots these directly.
export interface UniverseMetrics {
  ebit: DecimalString;
  net_income: DecimalString;
  profit_margin: DecimalString;
  return_on_equity: DecimalString;
  return_on_assets: DecimalString;
  current_ratio: DecimalString;
  quick_ratio: DecimalString;
  debt_to_equity: DecimalString;
  interest_coverage: DecimalString;
  asset_turnover: DecimalString;
  wacc: DecimalString;
  sharpe_ratio: DecimalString;
}

export type WsOutgoing =
  | { type: "progress"; completed: number; total: number }
  | { type: "final"; metrics: Record<MetricKey, SimulationResults> }
  | { type: "error"; message: string };

export interface NarrativeInputs {
  company: string;
  roe?: DecimalString | null;
  roa?: DecimalString | null;
  debt_to_equity?: DecimalString | null;
  current_ratio?: DecimalString | null;
  quick_ratio?: DecimalString | null;
  profit_margin?: DecimalString | null;
  net_income?: DecimalString | null;
  interest_coverage?: DecimalString | null;
  asset_turnover?: DecimalString | null;
}

export interface NarrativeResponse {
  memo: string;
}

export type AccountType = "Asset" | "Liability" | "Equity" | "Revenue" | "Expense";

export interface Account {
  code: number;
  name: string;
  account_type: AccountType;
  parent: number | null;
}

export interface AccountBalance {
  code: number;
  balance: DecimalString;
}

export type SourceDocument =
  | "ManualEntry"
  | "Accrual"
  | "Consolidation"
  | { Invoice: { id: string } }
  | { Payment: { id: string } }
  | { Receipt: { id: string } };

export interface FiscalPeriod {
  year: number;
  month: number;
}

export interface JournalLine {
  account: number;
  debit: DecimalString;
  credit: DecimalString;
  causal_formula: string | null;
}

export interface PostJournalEntryRequest {
  date: string;
  description: string;
  lines: JournalLine[];
  causal_parent: string | null;
  source_document: SourceDocument;
  period: FiscalPeriod;
}

export interface JournalEntry extends PostJournalEntryRequest {
  id: string;
}
