//! Budget-versus-actual variance analysis.

use crate::error::ErpError;
use crate::ledger::account::{AccountCode, AccountType};
use casiros_core::error::CalculationError;
use casiros_core::types::{Dollar, Ratio};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The result of comparing one account's budgeted amount to its actual amount.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct VarianceResult {
    /// The account this variance applies to.
    pub account: AccountCode,
    /// The budgeted amount.
    #[schema(value_type = Decimal)]
    pub budget: Dollar,
    /// The actual amount.
    #[schema(value_type = Decimal)]
    pub actual: Dollar,
    /// `actual - budget`.
    #[schema(value_type = Decimal)]
    pub variance: Dollar,
    /// `variance / budget`, or `None` if `budget` is zero (the percentage is undefined).
    #[schema(value_type = Option<Decimal>)]
    pub variance_percent: Option<Ratio>,
    /// Whether this variance is favorable, per [`analyze_variance`]'s
    /// account-type convention.
    pub favorable: bool,
}

/// Compares `actual` to `budget` for `account`, classifying the variance as
/// favorable or unfavorable based on `account_type`.
///
/// "More than budgeted" is favorable for accounts that increase the entity's
/// position ([`AccountType::Revenue`], [`AccountType::Asset`],
/// [`AccountType::Equity`]); "less than budgeted" is favorable for accounts
/// that represent an obligation or cost ([`AccountType::Liability`],
/// [`AccountType::Expense`]).
///
/// # Errors
///
/// Returns [`CalculationError::Overflow`] if the variance or its percentage overflows.
///
/// # Examples
///
/// ```
/// use casiros_erp::budget::variance::analyze_variance;
/// use casiros_erp::ledger::account::{AccountCode, AccountType};
/// use rust_decimal_macros::dec;
///
/// // Revenue came in above budget: favorable.
/// let result = analyze_variance(AccountCode(4000), AccountType::Revenue, dec!(100_000), dec!(110_000)).unwrap();
/// assert_eq!(result.variance, dec!(10_000));
/// assert!(result.favorable);
///
/// // Expense came in above budget: unfavorable.
/// let expense = analyze_variance(AccountCode(5000), AccountType::Expense, dec!(50_000), dec!(60_000)).unwrap();
/// assert!(!expense.favorable);
/// ```
pub fn analyze_variance(
    account: AccountCode,
    account_type: AccountType,
    budget: Dollar,
    actual: Dollar,
) -> Result<VarianceResult, ErpError> {
    let formula = "analyze_variance";
    let variance = actual
        .checked_sub(budget)
        .ok_or(CalculationError::Overflow { formula })?;
    let variance_percent = if budget.is_zero() {
        None
    } else {
        Some(
            variance
                .checked_div(budget)
                .ok_or(CalculationError::Overflow { formula })?,
        )
    };
    let favorable = match account_type {
        AccountType::Revenue | AccountType::Asset | AccountType::Equity => {
            variance >= Decimal::ZERO
        }
        AccountType::Liability | AccountType::Expense => variance <= Decimal::ZERO,
    };
    Ok(VarianceResult {
        account,
        budget,
        actual,
        variance,
        variance_percent,
        favorable,
    })
}

/// Runs [`analyze_variance`] across several `(account, account_type, budget,
/// actual)` tuples, e.g. every line item in a budget-to-actuals report.
///
/// # Errors
///
/// Returns whatever [`analyze_variance`] returns for the first failing entry.
///
/// # Examples
///
/// ```
/// use casiros_erp::budget::variance::variance_report;
/// use casiros_erp::ledger::account::{AccountCode, AccountType};
/// use rust_decimal_macros::dec;
///
/// let entries = [
///     (AccountCode(4000), AccountType::Revenue, dec!(100_000), dec!(110_000)),
///     (AccountCode(5000), AccountType::Expense, dec!(50_000), dec!(45_000)),
/// ];
/// let report = variance_report(&entries).unwrap();
/// assert_eq!(report.len(), 2);
/// assert!(report.iter().all(|r| r.favorable));
/// ```
pub fn variance_report(
    entries: &[(AccountCode, AccountType, Dollar, Dollar)],
) -> Result<Vec<VarianceResult>, ErpError> {
    entries
        .iter()
        .map(|&(account, account_type, budget, actual)| {
            analyze_variance(account, account_type, budget, actual)
        })
        .collect()
}
