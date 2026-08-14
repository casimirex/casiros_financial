//! Shared helpers for CASIROS fuzz targets.

use arbitrary::{Arbitrary, Unstructured};
use rust_decimal::Decimal;

/// Reads an arbitrary, always-valid `Decimal` from `u`: an `i64` mantissa
/// and a scale clamped into rust_decimal's supported `0..=28` range, so
/// constructing it can never itself panic — a fuzz-harness panic isn't a bug
/// in the code under test.
pub fn arbitrary_decimal(u: &mut Unstructured<'_>) -> arbitrary::Result<Decimal> {
    let mantissa = i64::arbitrary(u)?;
    let raw_scale = u8::arbitrary(u)?;
    let scale = u32::from(raw_scale % 29);
    Ok(Decimal::new(mantissa, scale))
}
