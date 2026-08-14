//! Fuzzes `JournalLine::new`: it must reject any invalid debit/credit
//! combination via `Result`, never panic — including on huge, negative, or
//! maximum-precision `Decimal` values that could overflow the checked
//! arithmetic in `casiros_erp::ledger::journal`'s debit/credit summation.

#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use casiros_erp::ledger::account::AccountCode;
use casiros_erp::ledger::journal::JournalLine;
use casiros_fuzz_shared::arbitrary_decimal;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(code) = u32::arbitrary(&mut u) else {
        return;
    };
    let Ok(debit) = arbitrary_decimal(&mut u) else {
        return;
    };
    let Ok(credit) = arbitrary_decimal(&mut u) else {
        return;
    };

    let _ = JournalLine::new(AccountCode(code), debit, credit, None);
});
