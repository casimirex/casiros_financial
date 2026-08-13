//! CFO-style narrative memo generation from computed financial metrics.
//!
//! [`generate_narrative`] takes whichever metrics the caller has computed
//! (each one optional) and produces a markdown memo with a short,
//! threshold-based interpretation of each present metric. This is the
//! runtime half of the `generate_narrative!` proc macro in `casiros-macros`
//! — the macro just builds a [`NarrativeInputs`] value from named
//! arguments and calls this module's pure function.

use casiros_core::types::{Dollar, Ratio};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use utoipa::ToSchema;

/// The financial metrics available to [`generate_narrative`]. Every field
/// except `company` is optional: absent metrics are simply omitted from the
/// generated memo, rather than causing an error.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct NarrativeInputs {
    /// The company or entity name for the memo's header.
    pub company: String,
    /// Return on equity (net income / shareholder equity), e.g. `0.15` for 15%.
    #[schema(value_type = Option<Decimal>)]
    pub roe: Option<Ratio>,
    /// Return on assets (net income / total assets).
    #[schema(value_type = Option<Decimal>)]
    pub roa: Option<Ratio>,
    /// Total liabilities divided by total equity.
    #[schema(value_type = Option<Decimal>)]
    pub debt_to_equity: Option<Ratio>,
    /// Current assets divided by current liabilities.
    #[schema(value_type = Option<Decimal>)]
    pub current_ratio: Option<Ratio>,
    /// (Current assets - inventory) divided by current liabilities.
    #[schema(value_type = Option<Decimal>)]
    pub quick_ratio: Option<Ratio>,
    /// Net income divided by revenue.
    #[schema(value_type = Option<Decimal>)]
    pub profit_margin: Option<Ratio>,
    /// Net income, in dollars.
    #[schema(value_type = Option<Decimal>)]
    pub net_income: Option<Dollar>,
    /// EBIT divided by interest expense.
    #[schema(value_type = Option<Decimal>)]
    pub interest_coverage: Option<Ratio>,
    /// Revenue divided by total assets.
    #[schema(value_type = Option<Decimal>)]
    pub asset_turnover: Option<Ratio>,
}

fn format_percent(ratio: Ratio) -> String {
    let pct = ratio.checked_mul(dec!(100)).unwrap_or(ratio);
    format!("{pct:.1}%")
}

fn format_dollar(amount: Dollar) -> String {
    if amount.is_sign_negative() {
        format!("-${:.2}", amount.abs())
    } else {
        format!("${amount:.2}")
    }
}

fn append_line(memo: &mut String, label: &str, value: &str, interpretation: &str) {
    let _ = writeln!(memo, "**{label}** of {value} is {interpretation}.\n");
}

fn interpret_roe(roe: Ratio) -> &'static str {
    if roe >= dec!(0.20) {
        "excellent, comfortably outperforming typical costs of equity capital"
    } else if roe >= dec!(0.15) {
        "strong, reflecting efficient use of shareholder capital"
    } else if roe >= dec!(0.08) {
        "moderate, roughly in line with typical costs of equity"
    } else if roe >= dec!(0.0) {
        "weak, likely below the company's cost of equity"
    } else {
        "negative, indicating the company destroyed shareholder value in the period"
    }
}

fn append_roe(memo: &mut String, roe: Option<Ratio>) {
    if let Some(roe) = roe {
        append_line(
            memo,
            "Return on Equity",
            &format_percent(roe),
            interpret_roe(roe),
        );
    }
}

fn interpret_roa(roa: Ratio) -> &'static str {
    if roa >= dec!(0.10) {
        "strong, reflecting efficient use of the asset base"
    } else if roa >= dec!(0.05) {
        "moderate asset efficiency"
    } else if roa >= dec!(0.0) {
        "weak asset efficiency"
    } else {
        "negative, indicating unprofitable use of the asset base"
    }
}

fn append_roa(memo: &mut String, roa: Option<Ratio>) {
    if let Some(roa) = roa {
        append_line(
            memo,
            "Return on Assets",
            &format_percent(roa),
            interpret_roa(roa),
        );
    }
}

fn interpret_debt_to_equity(ratio: Ratio) -> &'static str {
    if ratio <= dec!(0.5) {
        "conservative leverage, with ample capacity for additional debt financing"
    } else if ratio <= dec!(1.0) {
        "moderate leverage, within typical industry norms"
    } else if ratio <= dec!(2.0) {
        "elevated leverage, warranting close monitoring of debt covenants"
    } else {
        "high leverage, a material solvency risk"
    }
}

fn append_debt_to_equity(memo: &mut String, ratio: Option<Ratio>) {
    if let Some(ratio) = ratio {
        append_line(
            memo,
            "Debt-to-Equity",
            &format!("{ratio:.2}"),
            interpret_debt_to_equity(ratio),
        );
    }
}

fn interpret_current_ratio(ratio: Ratio) -> &'static str {
    if ratio >= dec!(2.0) {
        "a strong liquidity buffer"
    } else if ratio >= dec!(1.5) {
        "adequate short-term liquidity"
    } else if ratio >= dec!(1.0) {
        "thin, but sufficient, short-term liquidity"
    } else {
        "a liquidity shortfall — current liabilities exceed current assets"
    }
}

fn append_current_ratio(memo: &mut String, ratio: Option<Ratio>) {
    if let Some(ratio) = ratio {
        append_line(
            memo,
            "Current Ratio",
            &format!("{ratio:.2}"),
            interpret_current_ratio(ratio),
        );
    }
}

fn interpret_quick_ratio(ratio: Ratio) -> &'static str {
    if ratio >= dec!(1.5) {
        "a strong liquidity position even excluding inventory"
    } else if ratio >= dec!(1.0) {
        "adequate liquidity without relying on inventory liquidation"
    } else if ratio >= dec!(0.5) {
        "a liquidity position that depends partly on inventory turnover"
    } else {
        "a liquidity shortfall once inventory is excluded"
    }
}

fn append_quick_ratio(memo: &mut String, ratio: Option<Ratio>) {
    if let Some(ratio) = ratio {
        append_line(
            memo,
            "Quick Ratio",
            &format!("{ratio:.2}"),
            interpret_quick_ratio(ratio),
        );
    }
}

fn interpret_profit_margin(margin: Ratio) -> &'static str {
    if margin >= dec!(0.15) {
        "highly profitable operations"
    } else if margin >= dec!(0.05) {
        "healthy operating profitability"
    } else if margin >= dec!(0.0) {
        "thin operating margins"
    } else {
        "operations running at a loss"
    }
}

fn append_profit_margin(memo: &mut String, margin: Option<Ratio>) {
    if let Some(margin) = margin {
        append_line(
            memo,
            "Profit Margin",
            &format_percent(margin),
            interpret_profit_margin(margin),
        );
    }
}

fn interpret_net_income(amount: Dollar) -> &'static str {
    if amount.is_zero() {
        "exactly breaking even for the period"
    } else if amount.is_sign_positive() {
        "adding to retained earnings"
    } else {
        "reducing retained earnings"
    }
}

fn append_net_income(memo: &mut String, net_income: Option<Dollar>) {
    if let Some(amount) = net_income {
        let _ = writeln!(
            memo,
            "**Net Income** for the period was {}, {}.\n",
            format_dollar(amount),
            interpret_net_income(amount)
        );
    }
}

fn interpret_interest_coverage(coverage: Ratio) -> &'static str {
    if coverage >= dec!(5.0) {
        "very comfortable debt service capacity"
    } else if coverage >= dec!(3.0) {
        "adequate earnings coverage of interest obligations"
    } else if coverage >= dec!(1.5) {
        "coverage that leaves limited room for an earnings downturn"
    } else {
        "coverage that raises going-concern-level solvency concerns"
    }
}

fn append_interest_coverage(memo: &mut String, coverage: Option<Ratio>) {
    if let Some(coverage) = coverage {
        append_line(
            memo,
            "Interest Coverage",
            &format!("{coverage:.2}x"),
            interpret_interest_coverage(coverage),
        );
    }
}

fn interpret_asset_turnover(turnover: Ratio) -> &'static str {
    if turnover >= dec!(1.5) {
        "highly efficient use of the asset base to generate revenue"
    } else if turnover >= dec!(0.75) {
        "moderate asset utilization efficiency"
    } else {
        "low asset utilization, generating comparatively little revenue per dollar of assets"
    }
}

fn append_asset_turnover(memo: &mut String, turnover: Option<Ratio>) {
    if let Some(turnover) = turnover {
        append_line(
            memo,
            "Asset Turnover",
            &format!("{turnover:.2}x"),
            interpret_asset_turnover(turnover),
        );
    }
}

/// Generates a CFO-style markdown analysis memo from whichever metrics in
/// `inputs` are present. Absent metrics are simply omitted — this function
/// never fails.
///
/// # Examples
///
/// ```
/// use casiros_erp::narrative::{NarrativeInputs, generate_narrative};
/// use rust_decimal_macros::dec;
///
/// let memo = generate_narrative(&NarrativeInputs {
///     company: "Acme Corp".to_string(),
///     roe: Some(dec!(0.15)),
///     debt_to_equity: Some(dec!(0.8)),
///     current_ratio: Some(dec!(2.0)),
///     ..Default::default()
/// });
///
/// assert!(memo.starts_with("## Financial Analysis Memo: Acme Corp"));
/// assert!(memo.contains("Return on Equity"));
/// assert!(memo.contains("15.0%"));
/// assert!(!memo.contains("Return on Assets"));
/// ```
#[must_use]
pub fn generate_narrative(inputs: &NarrativeInputs) -> String {
    let company = if inputs.company.trim().is_empty() {
        "Unnamed Entity"
    } else {
        inputs.company.as_str()
    };
    let mut memo = format!("## Financial Analysis Memo: {company}\n\n");
    append_roe(&mut memo, inputs.roe);
    append_roa(&mut memo, inputs.roa);
    append_debt_to_equity(&mut memo, inputs.debt_to_equity);
    append_current_ratio(&mut memo, inputs.current_ratio);
    append_quick_ratio(&mut memo, inputs.quick_ratio);
    append_profit_margin(&mut memo, inputs.profit_margin);
    append_net_income(&mut memo, inputs.net_income);
    append_interest_coverage(&mut memo, inputs.interest_coverage);
    append_asset_turnover(&mut memo, inputs.asset_turnover);
    memo
}

#[cfg(test)]
mod tests {
    use super::{NarrativeInputs, generate_narrative};
    use rust_decimal_macros::dec;

    #[test]
    fn empty_inputs_produce_only_the_header() {
        let memo = generate_narrative(&NarrativeInputs {
            company: "Acme Corp".to_string(),
            ..Default::default()
        });
        assert_eq!(memo, "## Financial Analysis Memo: Acme Corp\n\n");
    }

    #[test]
    fn blank_company_falls_back_to_unnamed_entity() {
        let memo = generate_narrative(&NarrativeInputs::default());
        assert!(memo.starts_with("## Financial Analysis Memo: Unnamed Entity"));
    }

    #[test]
    fn negative_roe_is_flagged_as_value_destructive() {
        let memo = generate_narrative(&NarrativeInputs {
            company: "Acme".to_string(),
            roe: Some(dec!(-0.05)),
            ..Default::default()
        });
        assert!(memo.contains("-5.0%"));
        assert!(memo.contains("destroyed shareholder value"));
    }

    #[test]
    fn negative_net_income_reduces_retained_earnings() {
        let memo = generate_narrative(&NarrativeInputs {
            company: "Acme".to_string(),
            net_income: Some(dec!(-1000)),
            ..Default::default()
        });
        assert!(memo.contains("-$1000.00"));
        assert!(memo.contains("reducing retained earnings"));
    }

    #[test]
    fn zero_net_income_breaks_even() {
        let memo = generate_narrative(&NarrativeInputs {
            company: "Acme".to_string(),
            net_income: Some(dec!(0)),
            ..Default::default()
        });
        assert!(memo.contains("exactly breaking even"));
    }

    #[test]
    fn high_leverage_is_flagged_as_a_solvency_risk() {
        let memo = generate_narrative(&NarrativeInputs {
            company: "Acme".to_string(),
            debt_to_equity: Some(dec!(3.5)),
            ..Default::default()
        });
        assert!(memo.contains("high leverage, a material solvency risk"));
    }

    #[test]
    fn every_metric_present_produces_nine_sections() {
        let memo = generate_narrative(&NarrativeInputs {
            company: "Acme".to_string(),
            roe: Some(dec!(0.15)),
            roa: Some(dec!(0.08)),
            debt_to_equity: Some(dec!(0.8)),
            current_ratio: Some(dec!(2.0)),
            quick_ratio: Some(dec!(1.2)),
            profit_margin: Some(dec!(0.10)),
            net_income: Some(dec!(50000)),
            interest_coverage: Some(dec!(4.0)),
            asset_turnover: Some(dec!(1.0)),
        });
        assert_eq!(memo.matches("**").count(), 18);
        assert!(memo.contains("Asset Turnover"));
    }
}
