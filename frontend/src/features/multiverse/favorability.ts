import type { MetricKey } from "@/api/types";

// Whether a *higher* value of this metric is the favorable direction. Debt
// load and cost of capital are the two "lower is better" metrics here.
const HIGHER_IS_BETTER: Record<MetricKey, boolean> = {
  ebit: true,
  net_income: true,
  profit_margin: true,
  return_on_equity: true,
  return_on_assets: true,
  current_ratio: true,
  quick_ratio: true,
  debt_to_equity: false,
  interest_coverage: true,
  asset_turnover: true,
  wacc: false,
  sharpe_ratio: true,
};

/** Maps `values` to a `[0, 1]` favorability score via their rank within the sample. */
export function favorabilityScores(values: number[], metric: MetricKey): number[] {
  const sortedIndices = values
    .map((v, i) => [v, i] as const)
    .sort((a, b) => a[0] - b[0])
    .map(([, i]) => i);

  const ranks = new Array<number>(values.length);
  sortedIndices.forEach((originalIndex, rank) => {
    ranks[originalIndex] = values.length <= 1 ? 0.5 : rank / (values.length - 1);
  });

  const higherIsBetter = HIGHER_IS_BETTER[metric];
  return ranks.map((r) => (higherIsBetter ? r : 1 - r));
}
