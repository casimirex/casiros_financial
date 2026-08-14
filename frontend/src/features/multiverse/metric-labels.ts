import type { MetricKey } from "@/api/types";

export const METRIC_LABELS: Record<MetricKey, string> = {
  ebit: "EBIT",
  net_income: "Net Income",
  profit_margin: "Profit Margin",
  return_on_equity: "Return on Equity",
  return_on_assets: "Return on Assets",
  current_ratio: "Current Ratio",
  quick_ratio: "Quick Ratio",
  debt_to_equity: "Debt / Equity",
  interest_coverage: "Interest Coverage",
  asset_turnover: "Asset Turnover",
  wacc: "WACC",
  sharpe_ratio: "Sharpe Ratio",
};
