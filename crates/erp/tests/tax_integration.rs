//! Integration tests exercising `casiros-erp`'s `tax` module end-to-end.

use casiros_erp::error::ErpError;
use casiros_erp::tax::calculation::{
    DeferredTaxPosition, TemporaryDifference, calculate_multi_jurisdiction_tax, calculate_tax,
};
use casiros_erp::tax::jurisdiction::{JurisdictionCode, TaxBracket, TaxJurisdiction};
use rust_decimal_macros::dec;

fn federal() -> TaxJurisdiction {
    TaxJurisdiction::new(
        JurisdictionCode("US-FEDERAL".to_string()),
        "US Federal",
        vec![
            TaxBracket {
                upper_bound: Some(dec!(10_000.0)),
                rate: dec!(0.10),
            },
            TaxBracket {
                upper_bound: Some(dec!(40_000.0)),
                rate: dec!(0.20),
            },
            TaxBracket {
                upper_bound: None,
                rate: dec!(0.30),
            },
        ],
    )
    .unwrap()
}

fn flat_state() -> TaxJurisdiction {
    TaxJurisdiction::new(
        JurisdictionCode("US-XX".to_string()),
        "Flat State",
        vec![TaxBracket {
            upper_bound: None,
            rate: dec!(0.05),
        }],
    )
    .unwrap()
}

#[test]
fn progressive_tax_spans_multiple_brackets() {
    let tax = calculate_tax(&federal(), dec!(50_000.0)).unwrap();
    // 10,000*0.10 + 30,000*0.20 + 10,000*0.30 = 1,000 + 6,000 + 3,000
    assert_eq!(tax, dec!(10_000.0));
}

#[test]
fn progressive_tax_stays_within_first_bracket_for_low_income() {
    let tax = calculate_tax(&federal(), dec!(5_000.0)).unwrap();
    assert_eq!(tax, dec!(500.0));
}

#[test]
fn zero_income_owes_zero_tax() {
    assert_eq!(calculate_tax(&federal(), dec!(0.0)).unwrap(), dec!(0.0));
}

#[test]
fn negative_income_is_rejected() {
    assert!(calculate_tax(&federal(), dec!(-1.0)).is_err());
}

#[test]
fn jurisdiction_rejects_empty_brackets() {
    let result = TaxJurisdiction::new(JurisdictionCode("EMPTY".to_string()), "Empty", vec![]);
    assert!(matches!(result, Err(ErpError::InvalidTaxBrackets(_))));
}

#[test]
fn jurisdiction_rejects_non_final_unbounded_bracket() {
    let result = TaxJurisdiction::new(
        JurisdictionCode("BAD".to_string()),
        "Bad",
        vec![
            TaxBracket {
                upper_bound: None,
                rate: dec!(0.10),
            },
            TaxBracket {
                upper_bound: Some(dec!(10_000.0)),
                rate: dec!(0.20),
            },
        ],
    );
    assert!(matches!(result, Err(ErpError::InvalidTaxBrackets(_))));
}

#[test]
fn jurisdiction_rejects_bounded_final_bracket() {
    let result = TaxJurisdiction::new(
        JurisdictionCode("BAD".to_string()),
        "Bad",
        vec![TaxBracket {
            upper_bound: Some(dec!(10_000.0)),
            rate: dec!(0.10),
        }],
    );
    assert!(matches!(result, Err(ErpError::InvalidTaxBrackets(_))));
}

#[test]
fn multi_jurisdiction_tax_sums_across_jurisdictions() {
    let federal = federal();
    let state = flat_state();
    let total =
        calculate_multi_jurisdiction_tax(&[(&federal, dec!(50_000.0)), (&state, dec!(50_000.0))])
            .unwrap();
    // Federal 10,000 + state 50,000*0.05=2,500.
    assert_eq!(total, dec!(12_500.0));
}

#[test]
fn deferred_tax_liability_when_book_exceeds_tax_basis() {
    let difference = TemporaryDifference {
        description: "fixed asset".into(),
        book_basis: dec!(100_000.0),
        tax_basis: dec!(80_000.0),
        tax_rate: dec!(0.25),
    };
    assert_eq!(
        difference.deferred_tax_position().unwrap(),
        DeferredTaxPosition::Liability(dec!(5_000.0))
    );
}

#[test]
fn deferred_tax_asset_when_tax_exceeds_book_basis() {
    let difference = TemporaryDifference {
        description: "warranty reserve".into(),
        book_basis: dec!(80_000.0),
        tax_basis: dec!(100_000.0),
        tax_rate: dec!(0.25),
    };
    assert_eq!(
        difference.deferred_tax_position().unwrap(),
        DeferredTaxPosition::Asset(dec!(5_000.0))
    );
}

#[test]
fn no_deferred_tax_when_bases_are_equal() {
    let difference = TemporaryDifference {
        description: "cash".into(),
        book_basis: dec!(100_000.0),
        tax_basis: dec!(100_000.0),
        tax_rate: dec!(0.25),
    };
    assert_eq!(
        difference.deferred_tax_position().unwrap(),
        DeferredTaxPosition::None
    );
}
