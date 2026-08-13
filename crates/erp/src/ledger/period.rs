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
    pub fn new(year: i32, month: u32) -> Result<Self, String> {
        if !(1..=12).contains(&month) {
            return Err(format!("month must be in 1..=12, got {month}"));
        }
        Ok(Self { year, month })
    }

    /// The fiscal period containing `date`.
    #[must_use]
    pub fn containing(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
        }
    }

    /// Whether `date` falls within this fiscal period.
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
