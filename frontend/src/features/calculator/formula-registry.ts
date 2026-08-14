// Mirrors crates/dag/src/evaluator.rs's per-formula parameter names exactly
// (each eval_* function's `resolve(ctx, node, "...")` calls) — the form
// fields sent to /api/v1/calculate/{formula} must match these verbatim.
//
// DiscountedCashFlow, Duration, and Convexity take a cash-flow series rather
// than scalar parameters; /api/v1/calculate rejects them (400) by design
// (see SERIES_FORMULAS in crates/api/src/routes/calculate.rs), so they're
// intentionally not offered here.

export interface FormulaDefinition {
  name: string;
  label: string;
  category: string;
  params: string[];
}

export const FORMULA_CATEGORIES = [
  "Time Value of Money",
  "Financial Ratios",
  "Banking",
  "Markets & Risk",
  "Stocks & Bonds",
  "Corporate Finance",
] as const;

export const FORMULAS: FormulaDefinition[] = [
  // Time Value of Money
  { name: "future_value", label: "Future Value", category: "Time Value of Money", params: ["pv", "rate", "periods"] },
  { name: "present_value", label: "Present Value", category: "Time Value of Money", params: ["fv", "rate", "periods"] },
  { name: "annuity_future_value", label: "Annuity Future Value", category: "Time Value of Money", params: ["pmt", "rate", "periods"] },
  { name: "annuity_present_value", label: "Annuity Present Value", category: "Time Value of Money", params: ["pmt", "rate", "periods"] },
  { name: "perpetuity_present_value", label: "Perpetuity Present Value", category: "Time Value of Money", params: ["pmt", "rate"] },
  { name: "growing_perpetuity", label: "Growing Perpetuity", category: "Time Value of Money", params: ["d1", "rate", "growth"] },
  { name: "effective_annual_rate", label: "Effective Annual Rate", category: "Time Value of Money", params: ["nominal_rate", "compounding_periods"] },
  { name: "continuous_compounding", label: "Continuous Compounding", category: "Time Value of Money", params: ["pv", "rate", "time"] },

  // Financial Ratios
  { name: "return_on_equity", label: "Return on Equity", category: "Financial Ratios", params: ["net_income", "avg_shareholders_equity"] },
  { name: "return_on_assets", label: "Return on Assets", category: "Financial Ratios", params: ["net_income", "avg_total_assets"] },
  { name: "return_on_investment", label: "Return on Investment", category: "Financial Ratios", params: ["current_value", "cost"] },
  { name: "profit_margin", label: "Profit Margin", category: "Financial Ratios", params: ["net_income", "revenue"] },
  { name: "asset_turnover", label: "Asset Turnover", category: "Financial Ratios", params: ["net_sales", "avg_total_assets"] },
  { name: "equity_multiplier", label: "Equity Multiplier", category: "Financial Ratios", params: ["total_assets", "total_equity"] },
  { name: "dupont_roe", label: "DuPont ROE", category: "Financial Ratios", params: ["net_margin", "asset_turnover", "equity_multiplier"] },
  { name: "current_ratio", label: "Current Ratio", category: "Financial Ratios", params: ["current_assets", "current_liabilities"] },
  { name: "quick_ratio", label: "Quick Ratio", category: "Financial Ratios", params: ["current_assets", "inventory", "current_liabilities"] },
  { name: "debt_to_equity", label: "Debt to Equity", category: "Financial Ratios", params: ["total_liabilities", "total_equity"] },
  { name: "interest_coverage", label: "Interest Coverage", category: "Financial Ratios", params: ["ebit", "interest_expense"] },
  { name: "inventory_turnover", label: "Inventory Turnover", category: "Financial Ratios", params: ["cogs", "avg_inventory"] },
  { name: "cash_conversion_cycle", label: "Cash Conversion Cycle", category: "Financial Ratios", params: ["days_inventory_outstanding", "days_sales_outstanding", "days_payable_outstanding"] },

  // Banking
  { name: "net_interest_margin", label: "Net Interest Margin", category: "Banking", params: ["net_interest_income", "avg_earning_assets"] },
  { name: "loan_to_deposit_ratio", label: "Loan to Deposit Ratio", category: "Banking", params: ["total_loans", "total_deposits"] },
  { name: "capital_adequacy_ratio", label: "Capital Adequacy Ratio", category: "Banking", params: ["qualifying_capital", "risk_weighted_assets"] },
  { name: "provision_coverage", label: "Provision Coverage", category: "Banking", params: ["loan_loss_provisions", "non_performing_loans"] },

  // Markets & Risk
  { name: "beta", label: "Beta", category: "Markets & Risk", params: ["covariance", "variance_market"] },
  { name: "sharpe_ratio", label: "Sharpe Ratio", category: "Markets & Risk", params: ["portfolio_return", "risk_free_rate", "std_dev"] },
  { name: "treynor_ratio", label: "Treynor Ratio", category: "Markets & Risk", params: ["portfolio_return", "risk_free_rate", "portfolio_beta"] },
  { name: "jensens_alpha", label: "Jensen's Alpha", category: "Markets & Risk", params: ["portfolio_return", "risk_free_rate", "portfolio_beta", "market_return"] },
  { name: "value_at_risk", label: "Value at Risk", category: "Markets & Risk", params: ["portfolio_value", "z_score", "std_dev"] },
  { name: "expected_shortfall", label: "Expected Shortfall", category: "Markets & Risk", params: ["portfolio_value", "z_score", "std_dev", "confidence"] },

  // Stocks & Bonds
  { name: "dividend_discount_model", label: "Dividend Discount Model", category: "Stocks & Bonds", params: ["next_dividend", "required_return", "growth_rate"] },
  { name: "bond_price", label: "Bond Price", category: "Stocks & Bonds", params: ["face_value", "coupon_rate", "market_rate", "periods"] },
  { name: "yield_to_maturity", label: "Yield to Maturity", category: "Stocks & Bonds", params: ["price", "face_value", "coupon_rate", "periods"] },
  { name: "modified_duration", label: "Modified Duration", category: "Stocks & Bonds", params: ["macaulay_duration", "ytm", "periods_per_year"] },

  // Corporate Finance
  { name: "wacc", label: "WACC", category: "Corporate Finance", params: ["equity_value", "debt_value", "cost_of_equity", "cost_of_debt", "tax_rate"] },
  { name: "free_cash_flow_to_firm", label: "Free Cash Flow to Firm", category: "Corporate Finance", params: ["ebit", "tax_rate", "depreciation_amortization", "capex", "change_in_working_capital"] },
  { name: "free_cash_flow_to_equity", label: "Free Cash Flow to Equity", category: "Corporate Finance", params: ["net_income", "depreciation_amortization", "capex", "change_in_working_capital", "net_borrowing"] },
  { name: "economic_value_added", label: "Economic Value Added", category: "Corporate Finance", params: ["nopat", "invested_capital", "wacc"] },
  { name: "sustainable_growth_rate", label: "Sustainable Growth Rate", category: "Corporate Finance", params: ["roe", "retention_ratio"] },
  { name: "internal_growth_rate", label: "Internal Growth Rate", category: "Corporate Finance", params: ["roa", "retention_ratio"] },
];
