//! Supplier master data and payment terms.

use casiros_core::error::CalculationError;
use casiros_core::types::{Dollar, Ratio};
use chrono::{Duration, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::ledger::account::AccountCode;

/// A unique identifier for a [`Supplier`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct SupplierId(pub Uuid);

impl SupplierId {
    /// Generates a new, random supplier id.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ap::supplier::SupplierId;
    ///
    /// let a = SupplierId::new();
    /// let b = SupplierId::new();
    /// assert_ne!(a, b);
    /// assert_eq!(a, a);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SupplierId {
    fn default() -> Self {
        Self::new()
    }
}

/// Payment terms: how many days until an invoice is due, and an optional
/// early-payment discount (e.g. "2/10 net 30": 2% off if paid within 10 days,
/// otherwise the full amount is due within 30).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PaymentTerms {
    /// Days from the invoice date until the full amount is due.
    pub net_days: u32,
    /// The early-payment discount rate, if offered.
    #[schema(value_type = Option<Decimal>)]
    pub discount_percent: Option<Ratio>,
    /// Days from the invoice date within which the discount applies.
    pub discount_days: Option<u32>,
}

impl PaymentTerms {
    /// Net-30 terms with no early-payment discount.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ap::supplier::PaymentTerms;
    ///
    /// let terms = PaymentTerms::net(30);
    /// assert_eq!(terms.net_days, 30);
    /// assert!(terms.discount_percent.is_none());
    /// ```
    #[must_use]
    pub fn net(days: u32) -> Self {
        Self {
            net_days: days,
            discount_percent: None,
            discount_days: None,
        }
    }

    /// Terms with an early-payment discount (e.g. `PaymentTerms::with_discount(30, dec!(0.02), 10)`
    /// for "2/10 net 30").
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ap::supplier::PaymentTerms;
    /// use rust_decimal_macros::dec;
    ///
    /// let terms = PaymentTerms::with_discount(30, dec!(0.02), 10);
    /// assert_eq!(terms.discount_percent, Some(dec!(0.02)));
    /// assert_eq!(terms.discount_days, Some(10));
    /// ```
    #[must_use]
    pub fn with_discount(net_days: u32, discount_percent: Ratio, discount_days: u32) -> Self {
        Self {
            net_days,
            discount_percent: Some(discount_percent),
            discount_days: Some(discount_days),
        }
    }

    /// The date by which `invoice_date`'s full balance is due.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the resulting date is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ap::supplier::PaymentTerms;
    /// use chrono::NaiveDate;
    ///
    /// let terms = PaymentTerms::net(30);
    /// let invoice_date = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
    /// let due = terms.due_date(invoice_date).unwrap();
    /// assert_eq!(due, NaiveDate::from_ymd_opt(2026, 8, 31).unwrap());
    /// assert!(due > invoice_date);
    /// ```
    pub fn due_date(self, invoice_date: NaiveDate) -> Result<NaiveDate, CalculationError> {
        invoice_date
            .checked_add_signed(Duration::days(i64::from(self.net_days)))
            .ok_or(CalculationError::Overflow {
                formula: "PaymentTerms::due_date",
            })
    }

    /// The last date on which the early-payment discount still applies, if one is offered.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the resulting date is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ap::supplier::PaymentTerms;
    /// use chrono::NaiveDate;
    /// use rust_decimal_macros::dec;
    ///
    /// let invoice_date = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
    ///
    /// let with_discount = PaymentTerms::with_discount(30, dec!(0.02), 10);
    /// let deadline = with_discount.discount_deadline(invoice_date).unwrap();
    /// assert_eq!(deadline, Some(NaiveDate::from_ymd_opt(2026, 8, 11).unwrap()));
    ///
    /// let net_only = PaymentTerms::net(30);
    /// assert_eq!(net_only.discount_deadline(invoice_date).unwrap(), None);
    /// ```
    pub fn discount_deadline(
        self,
        invoice_date: NaiveDate,
    ) -> Result<Option<NaiveDate>, CalculationError> {
        let Some(discount_days) = self.discount_days else {
            return Ok(None);
        };
        let deadline = invoice_date
            .checked_add_signed(Duration::days(i64::from(discount_days)))
            .ok_or(CalculationError::Overflow {
                formula: "PaymentTerms::discount_deadline",
            })?;
        Ok(Some(deadline))
    }

    /// The amount due if `invoice_amount` is paid on `as_of`: the discounted
    /// amount if `as_of` is still within the discount window, otherwise the
    /// full amount.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if a date or amount computation overflows.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ap::supplier::PaymentTerms;
    /// use chrono::NaiveDate;
    /// use rust_decimal_macros::dec;
    ///
    /// let terms = PaymentTerms::with_discount(30, dec!(0.02), 10);
    /// let invoice_date = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
    ///
    /// // Paid within the discount window: 2% off.
    /// let within_window = NaiveDate::from_ymd_opt(2026, 8, 5).unwrap();
    /// assert_eq!(
    ///     terms.amount_due(dec!(1000), invoice_date, within_window).unwrap(),
    ///     dec!(980)
    /// );
    ///
    /// // Paid after the discount window: full amount.
    /// let after_window = NaiveDate::from_ymd_opt(2026, 8, 20).unwrap();
    /// assert_eq!(
    ///     terms.amount_due(dec!(1000), invoice_date, after_window).unwrap(),
    ///     dec!(1000)
    /// );
    /// ```
    pub fn amount_due(
        self,
        invoice_amount: Dollar,
        invoice_date: NaiveDate,
        as_of: NaiveDate,
    ) -> Result<Dollar, CalculationError> {
        let Some(discount_percent) = self.discount_percent else {
            return Ok(invoice_amount);
        };
        let Some(deadline) = self.discount_deadline(invoice_date)? else {
            return Ok(invoice_amount);
        };
        if as_of > deadline {
            return Ok(invoice_amount);
        }
        let retained_fraction =
            Decimal::ONE
                .checked_sub(discount_percent)
                .ok_or(CalculationError::Overflow {
                    formula: "PaymentTerms::amount_due",
                })?;
        invoice_amount
            .checked_mul(retained_fraction)
            .ok_or(CalculationError::Overflow {
                formula: "PaymentTerms::amount_due",
            })
    }
}

/// A supplier (vendor) master record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Supplier {
    /// This supplier's unique id.
    pub id: SupplierId,
    /// The supplier's name.
    pub name: String,
    /// This supplier's standard payment terms.
    pub payment_terms: PaymentTerms,
    /// The chart-of-accounts liability account this supplier's payables post to.
    pub payable_account: AccountCode,
}
