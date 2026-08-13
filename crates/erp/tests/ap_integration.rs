//! Integration tests exercising `casiros-erp`'s `ap` module end-to-end.

use casiros_erp::ap::invoice::{
    AgingBucket, ApInvoice, ApInvoiceStatus, aging_bucket, aging_report,
};
use casiros_erp::ap::payment::propose_payments;
use casiros_erp::ap::supplier::{PaymentTerms, SupplierId};
use casiros_erp::error::ErpError;
use chrono::NaiveDate;
use rust_decimal_macros::dec;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn net_terms_compute_due_date_with_no_discount() {
    let terms = PaymentTerms::net(30);
    let invoice_date = date(2026, 1, 1);
    assert_eq!(terms.due_date(invoice_date).unwrap(), date(2026, 1, 31));
    assert_eq!(
        terms
            .amount_due(dec!(1000.0), invoice_date, invoice_date)
            .unwrap(),
        dec!(1000.0)
    );
}

#[test]
fn discount_terms_apply_within_window_and_expire_after() {
    // "2/10 net 30"
    let terms = PaymentTerms::with_discount(30, dec!(0.02), 10);
    let invoice_date = date(2026, 1, 1);

    // The deadline (invoice_date + discount_days) is inclusive: Jan 1 + 10 days = Jan 11.
    let on_deadline = terms
        .amount_due(dec!(1000.0), invoice_date, date(2026, 1, 11))
        .unwrap();
    assert_eq!(on_deadline, dec!(980.0));

    let after_window = terms
        .amount_due(dec!(1000.0), invoice_date, date(2026, 1, 12))
        .unwrap();
    assert_eq!(after_window, dec!(1000.0));
}

#[test]
fn invoice_rejects_non_positive_amount() {
    let result = ApInvoice::new(
        SupplierId::new(),
        "INV-1",
        date(2026, 1, 1),
        dec!(0.0),
        PaymentTerms::net(30),
    );
    assert!(result.is_err());
}

#[test]
fn invoice_payment_lifecycle_tracks_status_correctly() {
    let mut invoice = ApInvoice::new(
        SupplierId::new(),
        "INV-1",
        date(2026, 1, 1),
        dec!(1000.0),
        PaymentTerms::net(30),
    )
    .unwrap();
    assert_eq!(invoice.status, ApInvoiceStatus::Open);

    invoice.apply_payment(dec!(400.0)).unwrap();
    assert_eq!(invoice.status, ApInvoiceStatus::PartiallyPaid);
    assert_eq!(invoice.balance_due().unwrap(), dec!(600.0));

    invoice.apply_payment(dec!(600.0)).unwrap();
    assert_eq!(invoice.status, ApInvoiceStatus::Paid);
    assert_eq!(invoice.balance_due().unwrap(), dec!(0.0));
}

#[test]
fn invoice_payment_cannot_exceed_balance_due() {
    let mut invoice = ApInvoice::new(
        SupplierId::new(),
        "INV-1",
        date(2026, 1, 1),
        dec!(1000.0),
        PaymentTerms::net(30),
    )
    .unwrap();
    let result = invoice.apply_payment(dec!(1000.01));
    assert!(matches!(
        result,
        Err(ErpError::PaymentExceedsBalance { .. })
    ));
}

#[test]
fn aging_bucket_classifies_by_days_past_due() {
    let terms = PaymentTerms::net(30);
    let invoice = ApInvoice::new(
        SupplierId::new(),
        "INV-1",
        date(2026, 1, 1),
        dec!(1000.0),
        terms,
    )
    .unwrap();
    // Due date is 2026-01-31.
    assert_eq!(
        aging_bucket(&invoice, date(2026, 1, 31)).unwrap(),
        AgingBucket::Current
    );
    assert_eq!(
        aging_bucket(&invoice, date(2026, 2, 10)).unwrap(),
        AgingBucket::Days1To30
    );
    assert_eq!(
        aging_bucket(&invoice, date(2026, 3, 15)).unwrap(),
        AgingBucket::Days31To60
    );
    assert_eq!(
        aging_bucket(&invoice, date(2026, 4, 15)).unwrap(),
        AgingBucket::Days61To90
    );
    assert_eq!(
        aging_bucket(&invoice, date(2026, 6, 1)).unwrap(),
        AgingBucket::Over90
    );
}

#[test]
fn aging_report_sums_balances_into_correct_buckets() {
    let terms = PaymentTerms::net(30);
    let supplier = SupplierId::new();
    let current = ApInvoice::new(supplier, "INV-1", date(2026, 1, 1), dec!(100.0), terms).unwrap();
    let overdue_45 =
        ApInvoice::new(supplier, "INV-2", date(2025, 12, 1), dec!(200.0), terms).unwrap();
    // Fully paid invoices should not contribute to the report.
    let mut fully_paid =
        ApInvoice::new(supplier, "INV-3", date(2025, 11, 1), dec!(300.0), terms).unwrap();
    fully_paid.apply_payment(dec!(300.0)).unwrap();

    let as_of = date(2026, 1, 31);
    // INV-2 due 2025-12-31, as_of 2026-01-31 => 31 days overdue => 31-60 bucket.
    assert_eq!(overdue_45.balance_due().unwrap(), dec!(200.0));

    let report = aging_report(&[current, overdue_45, fully_paid], as_of).unwrap();
    assert_eq!(report.current, dec!(100.0));
    assert_eq!(report.days_31_to_60, dec!(200.0));
    assert_eq!(report.days_1_to_30, dec!(0.0));
}

#[test]
fn propose_payments_prioritizes_expiring_discount_over_older_overdue_invoice() {
    let supplier = SupplierId::new();
    let as_of = date(2026, 2, 1);

    // Discount expires exactly today.
    let discount_terms = PaymentTerms::with_discount(60, dec!(0.02), 31);
    let discount_invoice = ApInvoice::new(
        supplier,
        "DISC-1",
        date(2026, 1, 1),
        dec!(1000.0),
        discount_terms,
    )
    .unwrap();

    // Older, overdue invoice with no discount.
    let overdue_invoice = ApInvoice::new(
        supplier,
        "OLD-1",
        date(2025, 11, 1),
        dec!(1000.0),
        PaymentTerms::net(30),
    )
    .unwrap();

    // Enough cash for exactly one of the two (after the 2% discount, the
    // discounted invoice costs 980; only one can be approved before the
    // second would push cumulative spend, and thus the liquidity check, too far).
    let available_cash = dec!(980.0);
    let current_liabilities = dec!(100.0); // ratio 9.8, always > 1.2

    let proposals = propose_payments(
        &[overdue_invoice, discount_invoice.clone()],
        as_of,
        available_cash,
        current_liabilities,
    )
    .unwrap();

    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].supplier, supplier);
    assert_eq!(proposals[0].invoices, vec![discount_invoice.id]);
    assert_eq!(proposals[0].total_amount, dec!(980.0));
}

#[test]
fn propose_payments_groups_multiple_invoices_per_supplier() {
    let supplier = SupplierId::new();
    let as_of = date(2026, 1, 1);
    let terms = PaymentTerms::net(30);
    let invoice_a = ApInvoice::new(supplier, "A", date(2025, 12, 1), dec!(300.0), terms).unwrap();
    let invoice_b = ApInvoice::new(supplier, "B", date(2025, 12, 2), dec!(400.0), terms).unwrap();

    let proposals = propose_payments(
        &[invoice_a, invoice_b],
        as_of,
        dec!(1_000_000.0),
        dec!(100.0),
    )
    .unwrap();

    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].invoices.len(), 2);
    assert_eq!(proposals[0].total_amount, dec!(700.0));
}
