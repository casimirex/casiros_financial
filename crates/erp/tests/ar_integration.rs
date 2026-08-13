//! Integration tests exercising `casiros-erp`'s `ar` module end-to-end.

use casiros_erp::ap::supplier::PaymentTerms;
use casiros_erp::ar::customer::{CustomerId, can_extend_credit};
use casiros_erp::ar::invoice::{
    ArInvoice, ArInvoiceStatus, DunningLevel, RecognitionMethod, dunning_level,
};
use casiros_erp::ar::receipt::{Receipt, allocate_receipt};
use casiros_erp::error::ErpError;
use chrono::NaiveDate;
use rust_decimal_macros::dec;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn point_in_time_invoice(
    customer: CustomerId,
    amount: rust_decimal::Decimal,
    recognition_date: NaiveDate,
) -> ArInvoice {
    ArInvoice::new(
        customer,
        "INV-1",
        date(2026, 1, 1),
        amount,
        PaymentTerms::net(30),
        RecognitionMethod::PointInTime { recognition_date },
    )
    .unwrap()
}

#[test]
fn point_in_time_recognition_is_all_or_nothing() {
    let recognition_date = date(2026, 1, 15);
    let invoice = point_in_time_invoice(CustomerId::new(), dec!(1000.0), recognition_date);

    assert_eq!(
        invoice.recognized_revenue_as_of(date(2026, 1, 14)).unwrap(),
        dec!(0.0)
    );
    assert_eq!(
        invoice.recognized_revenue_as_of(date(2026, 1, 15)).unwrap(),
        dec!(1000.0)
    );
    assert_eq!(
        invoice.deferred_revenue_as_of(date(2026, 1, 14)).unwrap(),
        dec!(1000.0)
    );
}

#[test]
fn ratably_over_time_recognition_prorates_straight_line() {
    let invoice = ArInvoice::new(
        CustomerId::new(),
        "SUB-1",
        date(2026, 1, 1),
        dec!(1000.0),
        PaymentTerms::net(30),
        RecognitionMethod::RatablyOverTime {
            start: date(2026, 1, 1),
            end: date(2026, 1, 11),
        },
    )
    .unwrap();

    assert_eq!(
        invoice
            .recognized_revenue_as_of(date(2025, 12, 31))
            .unwrap(),
        dec!(0.0)
    );
    assert_eq!(
        invoice.recognized_revenue_as_of(date(2026, 1, 6)).unwrap(),
        dec!(500.0)
    );
    assert_eq!(
        invoice.deferred_revenue_as_of(date(2026, 1, 6)).unwrap(),
        dec!(500.0)
    );
    assert_eq!(
        invoice.recognized_revenue_as_of(date(2026, 1, 11)).unwrap(),
        dec!(1000.0)
    );
    assert_eq!(
        invoice.recognized_revenue_as_of(date(2026, 2, 1)).unwrap(),
        dec!(1000.0)
    );
}

#[test]
fn ratably_over_time_rejects_end_not_after_start() {
    let result = ArInvoice::new(
        CustomerId::new(),
        "SUB-1",
        date(2026, 1, 1),
        dec!(1000.0),
        PaymentTerms::net(30),
        RecognitionMethod::RatablyOverTime {
            start: date(2026, 1, 11),
            end: date(2026, 1, 1),
        },
    );
    assert!(matches!(
        result,
        Err(ErpError::InvalidRecognitionPeriod { .. })
    ));
}

#[test]
fn receipt_lifecycle_tracks_status_correctly() {
    let mut invoice = point_in_time_invoice(CustomerId::new(), dec!(1000.0), date(2026, 1, 1));
    assert_eq!(invoice.status, ArInvoiceStatus::Open);

    invoice.apply_receipt(dec!(400.0)).unwrap();
    assert_eq!(invoice.status, ArInvoiceStatus::PartiallyCollected);

    invoice.apply_receipt(dec!(600.0)).unwrap();
    assert_eq!(invoice.status, ArInvoiceStatus::Collected);
    assert_eq!(invoice.balance_due().unwrap(), dec!(0.0));
}

#[test]
fn receipt_cannot_exceed_balance_due() {
    let mut invoice = point_in_time_invoice(CustomerId::new(), dec!(1000.0), date(2026, 1, 1));
    let result = invoice.apply_receipt(dec!(1000.01));
    assert!(matches!(
        result,
        Err(ErpError::PaymentExceedsBalance { .. })
    ));
}

#[test]
fn dunning_level_escalates_with_days_overdue() {
    let invoice = point_in_time_invoice(CustomerId::new(), dec!(1000.0), date(2026, 1, 1));
    // Due date is 2026-01-31 (net 30 from 2026-01-01).
    assert_eq!(
        dunning_level(&invoice, date(2026, 1, 31)).unwrap(),
        DunningLevel::None
    );
    assert_eq!(
        dunning_level(&invoice, date(2026, 2, 10)).unwrap(),
        DunningLevel::Reminder
    );
    assert_eq!(
        dunning_level(&invoice, date(2026, 3, 1)).unwrap(),
        DunningLevel::FirstNotice
    );
    assert_eq!(
        dunning_level(&invoice, date(2026, 4, 15)).unwrap(),
        DunningLevel::FinalNotice
    );
    assert_eq!(
        dunning_level(&invoice, date(2026, 6, 1)).unwrap(),
        DunningLevel::Collections
    );
}

#[test]
fn dunning_level_is_none_once_fully_collected_even_if_overdue() {
    let mut invoice = point_in_time_invoice(CustomerId::new(), dec!(1000.0), date(2026, 1, 1));
    invoice.apply_receipt(dec!(1000.0)).unwrap();
    assert_eq!(
        dunning_level(&invoice, date(2026, 6, 1)).unwrap(),
        DunningLevel::None
    );
}

#[test]
fn credit_limit_boundary_is_inclusive() {
    assert!(can_extend_credit(dec!(2_000.0), dec!(8_000.0), dec!(10_000.0)).unwrap());
    assert!(!can_extend_credit(dec!(2_000.01), dec!(8_000.0), dec!(10_000.0)).unwrap());
}

#[test]
fn allocate_receipt_applies_oldest_invoice_first_and_ignores_other_customers() {
    let customer = CustomerId::new();
    let other_customer = CustomerId::new();

    let older = ArInvoice::new(
        customer,
        "OLD",
        date(2025, 12, 1),
        dec!(300.0),
        PaymentTerms::net(30),
        RecognitionMethod::PointInTime {
            recognition_date: date(2025, 12, 1),
        },
    )
    .unwrap();
    let newer = ArInvoice::new(
        customer,
        "NEW",
        date(2026, 1, 1),
        dec!(500.0),
        PaymentTerms::net(30),
        RecognitionMethod::PointInTime {
            recognition_date: date(2026, 1, 1),
        },
    )
    .unwrap();
    let unrelated = ArInvoice::new(
        other_customer,
        "OTHER",
        date(2025, 11, 1),
        dec!(1000.0),
        PaymentTerms::net(30),
        RecognitionMethod::PointInTime {
            recognition_date: date(2025, 11, 1),
        },
    )
    .unwrap();

    let receipt = Receipt::new(customer, dec!(400.0), date(2026, 2, 1)).unwrap();
    let mut invoices = vec![older, newer, unrelated.clone()];
    let allocations = allocate_receipt(&receipt, &mut invoices).unwrap();

    // The 300 older invoice is fully settled first, then 100 flows to newer.
    assert_eq!(allocations.len(), 2);
    assert_eq!(allocations[0].amount_applied, dec!(300.0));
    assert_eq!(allocations[1].amount_applied, dec!(100.0));
    assert_eq!(invoices[0].status, ArInvoiceStatus::Collected);
    assert_eq!(invoices[1].balance_due().unwrap(), dec!(400.0));
    // The unrelated customer's invoice is untouched.
    assert_eq!(invoices[2], unrelated);
}
