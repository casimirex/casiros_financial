//! The causality graph: a generic DAG engine, plus [`FormulaNode`] — one node
//! per [`casiros_core`] formula, for edges representing formula data dependency.

use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use utoipa::ToSchema;

/// One variant per public formula in `casiros_core`. Each variant corresponds
/// exactly to a function of the same name (in `snake_case`) in the matching
/// `casiros_core` module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum FormulaNode {
    /// Corresponds to [`casiros_core::general::future_value`].
    FutureValue,
    /// Corresponds to [`casiros_core::general::present_value`].
    PresentValue,
    /// Corresponds to [`casiros_core::general::annuity_future_value`].
    AnnuityFutureValue,
    /// Corresponds to [`casiros_core::general::annuity_present_value`].
    AnnuityPresentValue,
    /// Corresponds to [`casiros_core::general::perpetuity_present_value`].
    PerpetuityPresentValue,
    /// Corresponds to [`casiros_core::general::growing_perpetuity`].
    GrowingPerpetuity,
    /// Corresponds to [`casiros_core::general::effective_annual_rate`].
    EffectiveAnnualRate,
    /// Corresponds to [`casiros_core::general::continuous_compounding`].
    ContinuousCompounding,
    /// Corresponds to [`casiros_core::financial::return_on_equity`].
    ReturnOnEquity,
    /// Corresponds to [`casiros_core::financial::return_on_assets`].
    ReturnOnAssets,
    /// Corresponds to [`casiros_core::financial::return_on_investment`].
    ReturnOnInvestment,
    /// Corresponds to [`casiros_core::financial::profit_margin`].
    ProfitMargin,
    /// Corresponds to [`casiros_core::financial::asset_turnover`].
    AssetTurnover,
    /// Corresponds to [`casiros_core::financial::equity_multiplier`].
    EquityMultiplier,
    /// Corresponds to [`casiros_core::financial::dupont_roe`].
    DupontRoe,
    /// Corresponds to [`casiros_core::financial::current_ratio`].
    CurrentRatio,
    /// Corresponds to [`casiros_core::financial::quick_ratio`].
    QuickRatio,
    /// Corresponds to [`casiros_core::financial::debt_to_equity`].
    DebtToEquity,
    /// Corresponds to [`casiros_core::financial::interest_coverage`].
    InterestCoverage,
    /// Corresponds to [`casiros_core::financial::inventory_turnover`].
    InventoryTurnover,
    /// Corresponds to [`casiros_core::financial::cash_conversion_cycle`].
    CashConversionCycle,
    /// Corresponds to [`casiros_core::banking::net_interest_margin`].
    NetInterestMargin,
    /// Corresponds to [`casiros_core::banking::loan_to_deposit_ratio`].
    LoanToDepositRatio,
    /// Corresponds to [`casiros_core::banking::capital_adequacy_ratio`].
    CapitalAdequacyRatio,
    /// Corresponds to [`casiros_core::banking::provision_coverage`].
    ProvisionCoverage,
    /// Corresponds to [`casiros_core::markets::beta`].
    Beta,
    /// Corresponds to [`casiros_core::markets::sharpe_ratio`].
    SharpeRatio,
    /// Corresponds to [`casiros_core::markets::treynor_ratio`].
    TreynorRatio,
    /// Corresponds to [`casiros_core::markets::jensens_alpha`].
    JensensAlpha,
    /// Corresponds to [`casiros_core::markets::value_at_risk`].
    ValueAtRisk,
    /// Corresponds to [`casiros_core::markets::expected_shortfall`].
    ExpectedShortfall,
    /// Corresponds to [`casiros_core::stocks_bonds::dividend_discount_model`].
    DividendDiscountModel,
    /// Corresponds to [`casiros_core::stocks_bonds::discounted_cash_flow`].
    DiscountedCashFlow,
    /// Corresponds to [`casiros_core::stocks_bonds::bond_price`].
    BondPrice,
    /// Corresponds to [`casiros_core::stocks_bonds::yield_to_maturity`].
    YieldToMaturity,
    /// Corresponds to [`casiros_core::stocks_bonds::duration`].
    Duration,
    /// Corresponds to [`casiros_core::stocks_bonds::modified_duration`].
    ModifiedDuration,
    /// Corresponds to [`casiros_core::stocks_bonds::convexity`].
    Convexity,
    /// Corresponds to [`casiros_core::corporate::wacc`].
    Wacc,
    /// Corresponds to [`casiros_core::corporate::free_cash_flow_to_firm`].
    FreeCashFlowToFirm,
    /// Corresponds to [`casiros_core::corporate::free_cash_flow_to_equity`].
    FreeCashFlowToEquity,
    /// Corresponds to [`casiros_core::corporate::economic_value_added`].
    EconomicValueAdded,
    /// Corresponds to [`casiros_core::corporate::sustainable_growth_rate`].
    SustainableGrowthRate,
    /// Corresponds to [`casiros_core::corporate::internal_growth_rate`].
    InternalGrowthRate,
}

impl FormulaNode {
    /// The `snake_case` name of the `casiros_core` function this node represents.
    ///
    /// This is also the convention [`crate::evaluator::evaluate_dag`] uses to wire
    /// one node's output into another node's input: a downstream formula's
    /// parameter named `"wacc"` will resolve against a computed [`FormulaNode::Wacc`]
    /// result before falling back to raw scalar inputs.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::FutureValue => "future_value",
            Self::PresentValue => "present_value",
            Self::AnnuityFutureValue => "annuity_future_value",
            Self::AnnuityPresentValue => "annuity_present_value",
            Self::PerpetuityPresentValue => "perpetuity_present_value",
            Self::GrowingPerpetuity => "growing_perpetuity",
            Self::EffectiveAnnualRate => "effective_annual_rate",
            Self::ContinuousCompounding => "continuous_compounding",
            Self::ReturnOnEquity => "return_on_equity",
            Self::ReturnOnAssets => "return_on_assets",
            Self::ReturnOnInvestment => "return_on_investment",
            Self::ProfitMargin => "profit_margin",
            Self::AssetTurnover => "asset_turnover",
            Self::EquityMultiplier => "equity_multiplier",
            Self::DupontRoe => "dupont_roe",
            Self::CurrentRatio => "current_ratio",
            Self::QuickRatio => "quick_ratio",
            Self::DebtToEquity => "debt_to_equity",
            Self::InterestCoverage => "interest_coverage",
            Self::InventoryTurnover => "inventory_turnover",
            Self::CashConversionCycle => "cash_conversion_cycle",
            Self::NetInterestMargin => "net_interest_margin",
            Self::LoanToDepositRatio => "loan_to_deposit_ratio",
            Self::CapitalAdequacyRatio => "capital_adequacy_ratio",
            Self::ProvisionCoverage => "provision_coverage",
            Self::Beta => "beta",
            Self::SharpeRatio => "sharpe_ratio",
            Self::TreynorRatio => "treynor_ratio",
            Self::JensensAlpha => "jensens_alpha",
            Self::ValueAtRisk => "value_at_risk",
            Self::ExpectedShortfall => "expected_shortfall",
            Self::DividendDiscountModel => "dividend_discount_model",
            Self::DiscountedCashFlow => "discounted_cash_flow",
            Self::BondPrice => "bond_price",
            Self::YieldToMaturity => "yield_to_maturity",
            Self::Duration => "duration",
            Self::ModifiedDuration => "modified_duration",
            Self::Convexity => "convexity",
            Self::Wacc => "wacc",
            Self::FreeCashFlowToFirm => "free_cash_flow_to_firm",
            Self::FreeCashFlowToEquity => "free_cash_flow_to_equity",
            Self::EconomicValueAdded => "economic_value_added",
            Self::SustainableGrowthRate => "sustainable_growth_rate",
            Self::InternalGrowthRate => "internal_growth_rate",
        }
    }

    /// Parses a `snake_case` formula name (as produced by [`Self::name`]) back
    /// into a [`FormulaNode`]. Used by `casiros-api` to resolve a formula name
    /// supplied in a request path into the node to evaluate.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "future_value" => Some(Self::FutureValue),
            "present_value" => Some(Self::PresentValue),
            "annuity_future_value" => Some(Self::AnnuityFutureValue),
            "annuity_present_value" => Some(Self::AnnuityPresentValue),
            "perpetuity_present_value" => Some(Self::PerpetuityPresentValue),
            "growing_perpetuity" => Some(Self::GrowingPerpetuity),
            "effective_annual_rate" => Some(Self::EffectiveAnnualRate),
            "continuous_compounding" => Some(Self::ContinuousCompounding),
            "return_on_equity" => Some(Self::ReturnOnEquity),
            "return_on_assets" => Some(Self::ReturnOnAssets),
            "return_on_investment" => Some(Self::ReturnOnInvestment),
            "profit_margin" => Some(Self::ProfitMargin),
            "asset_turnover" => Some(Self::AssetTurnover),
            "equity_multiplier" => Some(Self::EquityMultiplier),
            "dupont_roe" => Some(Self::DupontRoe),
            "current_ratio" => Some(Self::CurrentRatio),
            "quick_ratio" => Some(Self::QuickRatio),
            "debt_to_equity" => Some(Self::DebtToEquity),
            "interest_coverage" => Some(Self::InterestCoverage),
            "inventory_turnover" => Some(Self::InventoryTurnover),
            "cash_conversion_cycle" => Some(Self::CashConversionCycle),
            "net_interest_margin" => Some(Self::NetInterestMargin),
            "loan_to_deposit_ratio" => Some(Self::LoanToDepositRatio),
            "capital_adequacy_ratio" => Some(Self::CapitalAdequacyRatio),
            "provision_coverage" => Some(Self::ProvisionCoverage),
            "beta" => Some(Self::Beta),
            "sharpe_ratio" => Some(Self::SharpeRatio),
            "treynor_ratio" => Some(Self::TreynorRatio),
            "jensens_alpha" => Some(Self::JensensAlpha),
            "value_at_risk" => Some(Self::ValueAtRisk),
            "expected_shortfall" => Some(Self::ExpectedShortfall),
            "dividend_discount_model" => Some(Self::DividendDiscountModel),
            "discounted_cash_flow" => Some(Self::DiscountedCashFlow),
            "bond_price" => Some(Self::BondPrice),
            "yield_to_maturity" => Some(Self::YieldToMaturity),
            "duration" => Some(Self::Duration),
            "modified_duration" => Some(Self::ModifiedDuration),
            "convexity" => Some(Self::Convexity),
            "wacc" => Some(Self::Wacc),
            "free_cash_flow_to_firm" => Some(Self::FreeCashFlowToFirm),
            "free_cash_flow_to_equity" => Some(Self::FreeCashFlowToEquity),
            "economic_value_added" => Some(Self::EconomicValueAdded),
            "sustainable_growth_rate" => Some(Self::SustainableGrowthRate),
            "internal_growth_rate" => Some(Self::InternalGrowthRate),
            _ => None,
        }
    }

    /// Every [`FormulaNode`] variant, in declaration order — the single
    /// source of truth other crates should enumerate against rather than
    /// hand-writing their own copy of the variant list.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_dag::graph::FormulaNode;
    ///
    /// assert_eq!(FormulaNode::all().len(), 44);
    /// assert!(FormulaNode::all().contains(&FormulaNode::Wacc));
    /// ```
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::FutureValue,
            Self::PresentValue,
            Self::AnnuityFutureValue,
            Self::AnnuityPresentValue,
            Self::PerpetuityPresentValue,
            Self::GrowingPerpetuity,
            Self::EffectiveAnnualRate,
            Self::ContinuousCompounding,
            Self::ReturnOnEquity,
            Self::ReturnOnAssets,
            Self::ReturnOnInvestment,
            Self::ProfitMargin,
            Self::AssetTurnover,
            Self::EquityMultiplier,
            Self::DupontRoe,
            Self::CurrentRatio,
            Self::QuickRatio,
            Self::DebtToEquity,
            Self::InterestCoverage,
            Self::InventoryTurnover,
            Self::CashConversionCycle,
            Self::NetInterestMargin,
            Self::LoanToDepositRatio,
            Self::CapitalAdequacyRatio,
            Self::ProvisionCoverage,
            Self::Beta,
            Self::SharpeRatio,
            Self::TreynorRatio,
            Self::JensensAlpha,
            Self::ValueAtRisk,
            Self::ExpectedShortfall,
            Self::DividendDiscountModel,
            Self::DiscountedCashFlow,
            Self::BondPrice,
            Self::YieldToMaturity,
            Self::Duration,
            Self::ModifiedDuration,
            Self::Convexity,
            Self::Wacc,
            Self::FreeCashFlowToFirm,
            Self::FreeCashFlowToEquity,
            Self::EconomicValueAdded,
            Self::SustainableGrowthRate,
            Self::InternalGrowthRate,
        ]
    }

    /// The parameter names this formula's evaluator resolves, in the order
    /// [`crate::evaluator`]'s `eval_*` function for this node calls `resolve`
    /// (or `resolve_periods`/`resolve_series`) for them. Kept hand-in-hand
    /// with `crates/dag/src/evaluator.rs` — a mismatch here would misreport
    /// a formula's dependency wiring without ever failing evaluation itself,
    /// so `crate::evaluator`'s tests cross-check every entry against a real
    /// [`crate::evaluator::evaluate_dag`] run.
    ///
    /// [`FormulaNode::DiscountedCashFlow`], [`FormulaNode::Duration`], and
    /// [`FormulaNode::Convexity`] each list `"cash_flows"` here even though
    /// it names a cash-flow *series*, not a scalar — those three are the only
    /// nodes whose evaluator calls `resolve_series` rather than `resolve`.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_dag::graph::FormulaNode;
    ///
    /// assert_eq!(FormulaNode::FutureValue.parameters(), ["pv", "rate", "periods"]);
    /// assert_eq!(FormulaNode::PerpetuityPresentValue.parameters(), ["pmt", "rate"]);
    /// ```
    #[must_use]
    pub fn parameters(self) -> &'static [&'static str] {
        match self {
            Self::FutureValue => &["pv", "rate", "periods"],
            Self::PresentValue => &["fv", "rate", "periods"],
            Self::AnnuityFutureValue | Self::AnnuityPresentValue => &["pmt", "rate", "periods"],
            Self::PerpetuityPresentValue => &["pmt", "rate"],
            Self::GrowingPerpetuity => &["d1", "rate", "growth"],
            Self::EffectiveAnnualRate => &["nominal_rate", "compounding_periods"],
            Self::ContinuousCompounding => &["pv", "rate", "time"],
            Self::ReturnOnEquity => &["net_income", "avg_shareholders_equity"],
            Self::ReturnOnAssets => &["net_income", "avg_total_assets"],
            Self::ReturnOnInvestment => &["current_value", "cost"],
            Self::ProfitMargin => &["net_income", "revenue"],
            Self::AssetTurnover => &["net_sales", "avg_total_assets"],
            Self::EquityMultiplier => &["total_assets", "total_equity"],
            Self::DupontRoe => &["net_margin", "asset_turnover", "equity_multiplier"],
            Self::CurrentRatio => &["current_assets", "current_liabilities"],
            Self::QuickRatio => &["current_assets", "inventory", "current_liabilities"],
            Self::DebtToEquity => &["total_liabilities", "total_equity"],
            Self::InterestCoverage => &["ebit", "interest_expense"],
            Self::InventoryTurnover => &["cogs", "avg_inventory"],
            Self::CashConversionCycle => &[
                "days_inventory_outstanding",
                "days_sales_outstanding",
                "days_payable_outstanding",
            ],
            Self::NetInterestMargin => &["net_interest_income", "avg_earning_assets"],
            Self::LoanToDepositRatio => &["total_loans", "total_deposits"],
            Self::CapitalAdequacyRatio => &["qualifying_capital", "risk_weighted_assets"],
            Self::ProvisionCoverage => &["loan_loss_provisions", "non_performing_loans"],
            Self::Beta => &["covariance", "variance_market"],
            Self::SharpeRatio => &["portfolio_return", "risk_free_rate", "std_dev"],
            Self::TreynorRatio => &["portfolio_return", "risk_free_rate", "portfolio_beta"],
            Self::JensensAlpha => &[
                "portfolio_return",
                "risk_free_rate",
                "portfolio_beta",
                "market_return",
            ],
            Self::ValueAtRisk => &["portfolio_value", "z_score", "std_dev"],
            Self::ExpectedShortfall => &["portfolio_value", "z_score", "std_dev", "confidence"],
            Self::DividendDiscountModel => &["next_dividend", "required_return", "growth_rate"],
            Self::DiscountedCashFlow | Self::Duration | Self::Convexity => &["cash_flows", "rate"],
            Self::BondPrice => &["face_value", "coupon_rate", "market_rate", "periods"],
            Self::YieldToMaturity => &["price", "face_value", "coupon_rate", "periods"],
            Self::ModifiedDuration => &["macaulay_duration", "ytm", "periods_per_year"],
            Self::Wacc => &[
                "equity_value",
                "debt_value",
                "cost_of_equity",
                "cost_of_debt",
                "tax_rate",
            ],
            Self::FreeCashFlowToFirm => &[
                "ebit",
                "tax_rate",
                "depreciation_amortization",
                "capex",
                "change_in_working_capital",
            ],
            Self::FreeCashFlowToEquity => &[
                "net_income",
                "depreciation_amortization",
                "capex",
                "change_in_working_capital",
                "net_borrowing",
            ],
            Self::EconomicValueAdded => &["nopat", "invested_capital", "wacc"],
            Self::SustainableGrowthRate => &["roe", "retention_ratio"],
            Self::InternalGrowthRate => &["roa", "retention_ratio"],
        }
    }

    /// The other [`FormulaNode`]s this formula could directly draw an input
    /// from: for each of [`Self::parameters`], whether that name matches
    /// another node's [`Self::name`] (the exact-name wiring convention
    /// [`crate::evaluator::resolve`] uses to prefer a prior computed result
    /// over a raw scalar input). A parameter with no such match is a raw
    /// input — this formula's own [`Self::name`] is always excluded, so a
    /// coincidentally self-named parameter is never reported as depending on
    /// itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_dag::graph::FormulaNode;
    ///
    /// // "asset_turnover" and "equity_multiplier" both match another
    /// // formula's name, so DuPont ROE has two real upstream dependencies.
    /// let upstream = FormulaNode::DupontRoe.upstream_dependencies();
    /// assert_eq!(upstream.len(), 2);
    /// assert!(upstream.contains(&FormulaNode::AssetTurnover));
    /// assert!(upstream.contains(&FormulaNode::EquityMultiplier));
    ///
    /// // Cash conversion cycle's parameters are raw day-counts with no
    /// // producing formula: no upstream dependencies.
    /// assert!(FormulaNode::CashConversionCycle.upstream_dependencies().is_empty());
    /// ```
    #[must_use]
    pub fn upstream_dependencies(self) -> Vec<Self> {
        self.parameters()
            .iter()
            .filter_map(|parameter| Self::from_name(parameter))
            .filter(|&upstream| upstream != self)
            .collect()
    }
}

/// Builds the transitive dependency graph rooted at `target`: `target` plus
/// every [`FormulaNode`] reachable by repeatedly following
/// [`FormulaNode::upstream_dependencies`], as a [`CausalityEngine`] with edges
/// "upstream -> downstream" — ready for [`CausalityEngine::execution_order`].
///
/// # Examples
///
/// ```
/// use casiros_dag::graph::{FormulaNode, transitive_dependencies};
///
/// let engine = transitive_dependencies(FormulaNode::DupontRoe);
/// let order = engine.execution_order().unwrap();
/// assert_eq!(order.len(), 3);
/// // Both dependencies evaluate before the formula that consumes them.
/// let position = |node: FormulaNode| order.iter().position(|&n| n == node).unwrap();
/// assert!(position(FormulaNode::AssetTurnover) < position(FormulaNode::DupontRoe));
/// assert!(position(FormulaNode::EquityMultiplier) < position(FormulaNode::DupontRoe));
/// ```
#[must_use]
pub fn transitive_dependencies(target: FormulaNode) -> CausalityEngine<FormulaNode> {
    let mut engine = CausalityEngine::new();
    engine.add_node(target);
    let mut visited = HashSet::new();
    visited.insert(target);
    let mut frontier = vec![target];
    while let Some(node) = frontier.pop() {
        for upstream in node.upstream_dependencies() {
            engine.add_dependency(upstream, node);
            if visited.insert(upstream) {
                frontier.push(upstream);
            }
        }
    }
    engine
}

/// A directed acyclic graph over any `Copy + Eq + Hash` node type, wrapping
/// `petgraph` for topological ordering and cycle detection.
///
/// Originally specific to [`FormulaNode`]; genericized once the causal ledger
/// (`casiros-erp`) needed the same topological-ordering and cycle-detection
/// machinery for account roll-up dependencies, which are not `FormulaNode`s.
/// [`FormulaNode`]-specific behavior (parameter-name resolution) stays on
/// `FormulaNode` itself; this type only knows about graph structure.
#[derive(Debug)]
pub struct CausalityEngine<N> {
    /// The underlying graph. Edge `A -> B` means "B depends on A".
    graph: DiGraph<N, ()>,
    /// Reverse lookup from node to its graph index, so repeated inserts of the
    /// same node are idempotent.
    indices: HashMap<N, NodeIndex>,
}

impl<N> Default for CausalityEngine<N> {
    fn default() -> Self {
        Self {
            graph: DiGraph::new(),
            indices: HashMap::new(),
        }
    }
}

impl<N: Copy + Eq + Hash> CausalityEngine<N> {
    /// Creates an empty causality graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensures `node` is present in the graph, returning its index. Calling this
    /// twice for the same node returns the same index rather than duplicating it.
    pub fn add_node(&mut self, node: N) -> NodeIndex {
        if let Some(&index) = self.indices.get(&node) {
            return index;
        }
        let index = self.graph.add_node(node);
        self.indices.insert(node, index);
        index
    }

    /// Declares that `to` depends on `from`: `from` must be evaluated first, and
    /// its result is available for `to` to consume. Both nodes are added to the
    /// graph if not already present.
    pub fn add_dependency(&mut self, from: N, to: N) {
        let from_index = self.add_node(from);
        let to_index = self.add_node(to);
        self.graph.update_edge(from_index, to_index, ());
    }
}

impl<N: Copy + Eq + Hash + Debug> CausalityEngine<N> {
    /// Returns a valid evaluation order: every node appears after all the nodes
    /// it depends on.
    ///
    /// # Errors
    ///
    /// Returns `Err` describing the cycle if the graph is not a DAG. Cycle
    /// detection is a hard error — a financial model (or a ledger roll-up
    /// hierarchy) that depends on itself cannot be computed.
    pub fn execution_order(&self) -> Result<Vec<N>, String> {
        toposort(&self.graph, None).map_or_else(
            |cycle| {
                let node = self.graph[cycle.node_id()];
                Err(format!("cyclic dependency detected at {node:?}"))
            },
            |indices| Ok(indices.into_iter().map(|index| self.graph[index]).collect()),
        )
    }
}
