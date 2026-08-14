//! Driver-based budget planning: line items computed from named business
//! drivers rather than hardcoded numbers, so changing one driver's value
//! propagates to every line item that references it.

use crate::error::ErpError;
use crate::ledger::account::AccountCode;
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A budget line item, computed as the product of one or more named drivers
/// (e.g. `revenue = units_sold * average_price`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriverBasedLineItem {
    /// The chart-of-accounts account this line item budgets for.
    pub account: AccountCode,
    /// A human-readable description.
    pub description: String,
    /// The names of the drivers whose values are multiplied together to
    /// compute this line item's amount. Must be non-empty.
    pub driver_names: Vec<String>,
}

/// A driver-based budget model: a set of named drivers, plus line items that
/// compute their budgeted amount from those drivers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BudgetModel {
    drivers: HashMap<String, Decimal>,
    line_items: Vec<DriverBasedLineItem>,
}

impl BudgetModel {
    /// Creates an empty budget model.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::budget::model::BudgetModel;
    ///
    /// let model = BudgetModel::new();
    /// assert!(model.line_items().is_empty());
    /// assert_eq!(model.driver("revenue"), None);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets (or overwrites) a driver's value.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::budget::model::BudgetModel;
    /// use rust_decimal_macros::dec;
    ///
    /// let mut model = BudgetModel::new();
    /// model.set_driver("units_sold", dec!(1000));
    /// assert_eq!(model.driver("units_sold"), Some(dec!(1000)));
    /// model.set_driver("units_sold", dec!(1200));
    /// assert_eq!(model.driver("units_sold"), Some(dec!(1200)));
    /// ```
    pub fn set_driver(&mut self, name: impl Into<String>, value: Decimal) {
        self.drivers.insert(name.into(), value);
    }

    /// Looks up a driver's current value.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::budget::model::BudgetModel;
    /// use rust_decimal_macros::dec;
    ///
    /// let mut model = BudgetModel::new();
    /// assert_eq!(model.driver("units_sold"), None);
    /// model.set_driver("units_sold", dec!(1000));
    /// assert_eq!(model.driver("units_sold"), Some(dec!(1000)));
    /// ```
    #[must_use]
    pub fn driver(&self, name: &str) -> Option<Decimal> {
        self.drivers.get(name).copied()
    }

    /// Adds a line item to the model.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::budget::model::{BudgetModel, DriverBasedLineItem};
    /// use casiros_erp::ledger::account::AccountCode;
    ///
    /// let mut model = BudgetModel::new();
    /// assert_eq!(model.line_items().len(), 0);
    /// model.add_line_item(DriverBasedLineItem {
    ///     account: AccountCode(4000),
    ///     description: "Revenue".to_string(),
    ///     driver_names: vec!["units_sold".to_string(), "average_price".to_string()],
    /// });
    /// assert_eq!(model.line_items().len(), 1);
    /// ```
    pub fn add_line_item(&mut self, item: DriverBasedLineItem) {
        self.line_items.push(item);
    }

    /// Every line item in the model, in insertion order.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::budget::model::{BudgetModel, DriverBasedLineItem};
    /// use casiros_erp::ledger::account::AccountCode;
    ///
    /// let mut model = BudgetModel::new();
    /// model.add_line_item(DriverBasedLineItem {
    ///     account: AccountCode(4000),
    ///     description: "Revenue".to_string(),
    ///     driver_names: vec!["units_sold".to_string()],
    /// });
    /// assert_eq!(model.line_items().len(), 1);
    /// assert_eq!(model.line_items()[0].description, "Revenue");
    /// ```
    #[must_use]
    pub fn line_items(&self) -> &[DriverBasedLineItem] {
        &self.line_items
    }

    /// Computes `item`'s budgeted amount as the product of its named
    /// drivers' current values.
    ///
    /// # Errors
    ///
    /// Returns an error (via [`CalculationError::MissingInput`]) if
    /// `item.driver_names` is empty. Returns [`ErpError::UnknownDriver`] if
    /// any named driver has not been set. Returns [`CalculationError::Overflow`]
    /// if the product overflows.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::budget::model::{BudgetModel, DriverBasedLineItem};
    /// use casiros_erp::ledger::account::AccountCode;
    /// use rust_decimal_macros::dec;
    ///
    /// let mut model = BudgetModel::new();
    /// model.set_driver("units_sold", dec!(1000));
    /// model.set_driver("average_price", dec!(25));
    /// let item = DriverBasedLineItem {
    ///     account: AccountCode(4000),
    ///     description: "Revenue".to_string(),
    ///     driver_names: vec!["units_sold".to_string(), "average_price".to_string()],
    /// };
    /// assert_eq!(model.compute_line_item(&item).unwrap(), dec!(25_000));
    ///
    /// let unknown = DriverBasedLineItem {
    ///     driver_names: vec!["missing_driver".to_string()],
    ///     ..item
    /// };
    /// assert!(model.compute_line_item(&unknown).is_err());
    /// ```
    pub fn compute_line_item(&self, item: &DriverBasedLineItem) -> Result<Dollar, ErpError> {
        let Some((first, rest)) = item.driver_names.split_first() else {
            return Err(CalculationError::MissingInput {
                formula: "BudgetModel::compute_line_item",
                parameter: "driver_names",
            }
            .into());
        };
        let mut product = self
            .drivers
            .get(first)
            .copied()
            .ok_or_else(|| ErpError::UnknownDriver(first.clone()))?;
        for name in rest {
            let value = self
                .drivers
                .get(name)
                .copied()
                .ok_or_else(|| ErpError::UnknownDriver(name.clone()))?;
            product = product
                .checked_mul(value)
                .ok_or(CalculationError::Overflow {
                    formula: "BudgetModel::compute_line_item",
                })?;
        }
        Ok(product)
    }

    /// The total budget: the sum of every line item's computed amount.
    ///
    /// # Errors
    ///
    /// Returns whatever [`Self::compute_line_item`] returns for the first
    /// failing line item, or [`CalculationError::Overflow`] if the total overflows.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::budget::model::{BudgetModel, DriverBasedLineItem};
    /// use casiros_erp::ledger::account::AccountCode;
    /// use rust_decimal_macros::dec;
    ///
    /// let mut model = BudgetModel::new();
    /// model.set_driver("units_sold", dec!(1000));
    /// model.set_driver("average_price", dec!(25));
    /// model.add_line_item(DriverBasedLineItem {
    ///     account: AccountCode(4000),
    ///     description: "Revenue".to_string(),
    ///     driver_names: vec!["units_sold".to_string(), "average_price".to_string()],
    /// });
    /// model.add_line_item(DriverBasedLineItem {
    ///     account: AccountCode(4100),
    ///     description: "Other Income".to_string(),
    ///     driver_names: vec!["units_sold".to_string()],
    /// });
    /// assert_eq!(model.total_budget().unwrap(), dec!(26_000));
    /// assert!(model.total_budget().unwrap() > dec!(0));
    /// ```
    pub fn total_budget(&self) -> Result<Dollar, ErpError> {
        let mut total = Decimal::ZERO;
        for item in &self.line_items {
            let amount = self.compute_line_item(item)?;
            total = total
                .checked_add(amount)
                .ok_or(CalculationError::Overflow {
                    formula: "BudgetModel::total_budget",
                })?;
        }
        Ok(total)
    }
}
