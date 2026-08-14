//! Fuzzes `casiros-core`'s pure formulas with adversarial `Decimal` inputs.
//! Every one of these must return `Result::Err` rather than panicking, no
//! matter how extreme, negative, or zero the input — that's the whole point
//! of the project's `checked_*`-arithmetic-only discipline.

#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use casiros_core::corporate::wacc;
use casiros_core::financial::current_ratio;
use casiros_core::general::future_value;
use casiros_core::markets::sharpe_ratio;
use casiros_fuzz_shared::arbitrary_decimal;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(a) = arbitrary_decimal(&mut u) else {
        return;
    };
    let Ok(b) = arbitrary_decimal(&mut u) else {
        return;
    };
    let Ok(c) = arbitrary_decimal(&mut u) else {
        return;
    };
    let Ok(periods) = u32::arbitrary(&mut u) else {
        return;
    };

    let _ = future_value(a, b, periods % 1000);
    let _ = current_ratio(a, b);
    let _ = sharpe_ratio(a, b, c);
    let _ = wacc(a, b, a, b, c);
});
