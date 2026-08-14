//! Fiscal periods and their open/closed posting status.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A calendar-month fiscal period, identified by year and month number (`1..=12`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
pub struct FiscalPeriod {
    /// The calendar year.
    pub year: i32,
    /// The calendar month, `1..=12`.
    pub month: u32,
}

impl FiscalPeriod {
    /// Creates a fiscal period for `year`/`month`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `month` is not in `1..=12`.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::period::FiscalPeriod;
    ///
    /// let period = FiscalPeriod::new(2026, 8).unwrap();
    /// assert_eq!(period.year, 2026);
    /// assert_eq!(period.month, 8);
    /// assert!(FiscalPeriod::new(2026, 13).is_err());
    /// ```
    pub fn new(year: i32, month: u32) -> Result<Self, String> {
        if !(1..=12).contains(&month) {
            return Err(format!("month must be in 1..=12, got {month}"));
        }
        Ok(Self { year, month })
    }

    /// The fiscal period containing `date`.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::period::FiscalPeriod;
    /// use chrono::NaiveDate;
    ///
    /// let date = NaiveDate::from_ymd_opt(2026, 8, 13).unwrap();
    /// let period = FiscalPeriod::containing(date);
    /// assert_eq!(period.year, 2026);
    /// assert_eq!(period.month, 8);
    /// ```
    #[must_use]
    pub fn containing(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
        }
    }

    /// Whether `date` falls within this fiscal period.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::period::FiscalPeriod;
    /// use chrono::NaiveDate;
    ///
    /// let period = FiscalPeriod::new(2026, 8).unwrap();
    /// let in_august = NaiveDate::from_ymd_opt(2026, 8, 13).unwrap();
    /// let in_september = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
    /// assert!(period.contains(in_august));
    /// assert!(!period.contains(in_september));
    /// ```
    #[must_use]
    pub fn contains(self, date: NaiveDate) -> bool {
        Self::containing(date) == self
    }
}

/// Whether a fiscal period is still open for posting or has been closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum PeriodStatus {
    /// New journal entries may be posted to this period.
    Open,
    /// This period has been closed; no further postings are accepted.
    Closed,
}
