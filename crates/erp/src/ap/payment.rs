//! Payment proposals: which open invoices to pay today, and how much.

use super::invoice::{ApInvoice, ApInvoiceId};
use super::supplier::SupplierId;
use crate::business_rules::can_approve_payment;
use crate::error::ErpError;
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// A proposed batch payment to one supplier, covering one or more of its
/// open invoices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PaymentProposal {
    /// The supplier to be paid.
    pub supplier: SupplierId,
    /// The invoices this payment would settle (fully or partially).
    pub invoices: Vec<ApInvoiceId>,
    /// The total amount proposed for this supplier.
    #[schema(value_type = Decimal)]
    pub total_amount: Dollar,
}

/// One invoice, annotated with its sort priority for [`propose_payments`].
struct Candidate<'a> {
    discount_expires_today: bool,
    due_date: NaiveDate,
    invoice: &'a ApInvoice,
    balance_due: Dollar,
}

fn collect_candidates(
    invoices: &[ApInvoice],
    as_of: NaiveDate,
) -> Result<Vec<Candidate<'_>>, CalculationError> {
    let mut candidates = Vec::new();
    for invoice in invoices {
        let balance_due = invoice.balance_due()?;
        if balance_due <= Decimal::ZERO {
            continue;
        }
        let due_date = invoice.due_date()?;
        let discount_expires_today =
            invoice.terms.discount_deadline(invoice.invoice_date)? == Some(as_of);
        candidates.push(Candidate {
            discount_expires_today,
            due_date,
            invoice,
            balance_due,
        });
    }
    // Invoices whose early-payment discount expires today are prioritized (use
    // it or lose it), then the most overdue (earliest due date) invoices.
    candidates.sort_by(|a, b| {
        b.discount_expires_today
            .cmp(&a.discount_expires_today)
            .then(a.due_date.cmp(&b.due_date))
    });
    Ok(candidates)
}

/// Proposes which open invoices to pay `as_of` a given date, given the cash
/// currently available and the entity's current liabilities.
///
/// Invoices are considered in priority order (expiring discounts first, then
/// oldest due date), and each is included only if [`can_approve_payment`]
/// still approves the *cumulative* proposed total against `available_cash`
/// and `current_liabilities` — invoices that would breach that liquidity gate
/// are skipped (not fatal), so smaller invoices further down the list can
/// still be proposed.
///
/// # Errors
///
/// Returns [`CalculationError::Overflow`] if a running total overflows, or
/// whatever error [`can_approve_payment`] produces for degenerate inputs
/// (e.g. zero `current_liabilities`).
pub fn propose_payments(
    invoices: &[ApInvoice],
    as_of: NaiveDate,
    available_cash: Dollar,
    current_liabilities: Dollar,
) -> Result<Vec<PaymentProposal>, ErpError> {
    let candidates = collect_candidates(invoices, as_of)?;
    let mut running_total = Decimal::ZERO;
    let mut by_supplier: HashMap<SupplierId, PaymentProposal> = HashMap::new();

    for candidate in candidates {
        let amount = candidate.invoice.terms.amount_due(
            candidate.balance_due,
            candidate.invoice.invoice_date,
            as_of,
        )?;
        let candidate_total =
            running_total
                .checked_add(amount)
                .ok_or(CalculationError::Overflow {
                    formula: "propose_payments",
                })?;
        if !can_approve_payment(candidate_total, available_cash, current_liabilities)? {
            continue;
        }
        running_total = candidate_total;
        let proposal = by_supplier
            .entry(candidate.invoice.supplier)
            .or_insert_with(|| PaymentProposal {
                supplier: candidate.invoice.supplier,
                invoices: Vec::new(),
                total_amount: Decimal::ZERO,
            });
        proposal.invoices.push(candidate.invoice.id);
        proposal.total_amount =
            proposal
                .total_amount
                .checked_add(amount)
                .ok_or(CalculationError::Overflow {
                    formula: "propose_payments",
                })?;
    }
    Ok(by_supplier.into_values().collect())
}
