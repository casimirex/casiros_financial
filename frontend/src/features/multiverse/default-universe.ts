import type { MonteCarloConfig, Universe } from "@/api/types";

// A representative mid-size company, used to pre-fill the form so a first
// run only requires pressing "Run" — every field remains editable.
export const DEFAULT_UNIVERSE: Universe = {
  risk_free_rate: "0.03",
  inflation_rate: "0.02",
  market_return: "0.08",
  portfolio_return: "0.10",
  return_std_dev: "0.15",
  revenue: "1000000",
  cogs: "600000",
  operating_expenses: "200000",
  interest_expense: "50000",
  tax_rate: "0.25",
  beta: "1.2",
  cost_of_equity: "0.11",
  cost_of_debt: "0.06",
  total_assets: "1500000",
  current_assets: "400000",
  inventory: "100000",
  current_liabilities: "200000",
  total_liabilities: "750000",
  total_equity: "750000",
  share_price: "50",
  shares_outstanding: "20000",
};

export const DEFAULT_CONFIG: MonteCarloConfig = {
  iterations: 5000,
  seed: 42,
  track_convergence: false,
  convergence_threshold: "0.0001",
  convergence_batch_size: 1000,
};

export const UNIVERSE_FIELD_GROUPS: { title: string; fields: (keyof Universe)[] }[] = [
  {
    title: "Macroeconomic",
    fields: ["risk_free_rate", "inflation_rate", "market_return", "portfolio_return", "return_std_dev"],
  },
  {
    title: "Income Statement",
    fields: ["revenue", "cogs", "operating_expenses", "interest_expense", "tax_rate"],
  },
  {
    title: "Cost of Capital",
    fields: ["beta", "cost_of_equity", "cost_of_debt"],
  },
  {
    title: "Balance Sheet",
    fields: [
      "total_assets",
      "current_assets",
      "inventory",
      "current_liabilities",
      "total_liabilities",
      "total_equity",
    ],
  },
  {
    title: "Market",
    fields: ["share_price", "shares_outstanding"],
  },
];

export const FIELD_LABELS: Record<keyof Universe, string> = {
  risk_free_rate: "Risk-Free Rate",
  inflation_rate: "Inflation Rate",
  market_return: "Market Return",
  portfolio_return: "Portfolio Return",
  return_std_dev: "Return Std Dev",
  revenue: "Revenue",
  cogs: "COGS",
  operating_expenses: "Operating Expenses",
  interest_expense: "Interest Expense",
  tax_rate: "Tax Rate",
  beta: "Beta",
  cost_of_equity: "Cost of Equity",
  cost_of_debt: "Cost of Debt",
  total_assets: "Total Assets",
  current_assets: "Current Assets",
  inventory: "Inventory",
  current_liabilities: "Current Liabilities",
  total_liabilities: "Total Liabilities",
  total_equity: "Total Equity",
  share_price: "Share Price",
  shares_outstanding: "Shares Outstanding",
};
