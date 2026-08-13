//! Parsing and code generation for `generate_narrative!`.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, Token};

/// Every field on `NarrativeInputs` besides `company`, in declaration order.
const METRIC_FIELDS: &[&str] = &[
    "roe",
    "roa",
    "debt_to_equity",
    "current_ratio",
    "quick_ratio",
    "profit_margin",
    "net_income",
    "interest_coverage",
    "asset_turnover",
];

/// One `key: value` pair from the macro's input.
struct MetricArg {
    key: Ident,
    value: Expr,
}

impl Parse for MetricArg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: Expr = input.parse()?;
        Ok(Self { key, value })
    }
}

/// Sorts `args` into a required `company` expression and the metric
/// expressions keyed by field name, rejecting unknown or duplicate keys.
fn classify_args(
    args: Punctuated<MetricArg, Token![,]>,
) -> syn::Result<(Expr, Vec<(String, Expr)>)> {
    let mut company: Option<Expr> = None;
    let mut metrics: Vec<(String, Expr)> = Vec::new();
    let mut seen = HashSet::new();

    for arg in args {
        let key = arg.key.to_string();
        if !seen.insert(key.clone()) {
            return Err(syn::Error::new(
                arg.key.span(),
                format!("duplicate key '{key}'"),
            ));
        }
        if key == "company" {
            company = Some(arg.value);
        } else if METRIC_FIELDS.contains(&key.as_str()) {
            metrics.push((key, arg.value));
        } else {
            return Err(syn::Error::new(
                arg.key.span(),
                format!(
                    "unknown narrative metric '{key}'; expected 'company' or one of {}",
                    METRIC_FIELDS.join(", ")
                ),
            ));
        }
    }

    let company = company.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "generate_narrative! requires a 'company' key",
        )
    })?;
    Ok((company, metrics))
}

/// Parses `input` as a `key: value, ...` list and expands it into a call
/// building a `casiros_erp::narrative::NarrativeInputs` and generating its
/// memo.
pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let args = Punctuated::<MetricArg, Token![,]>::parse_terminated.parse2(input)?;
    let (company, metrics) = classify_args(args)?;

    let field_inits = METRIC_FIELDS.iter().map(|field| {
        let field_ident = Ident::new(field, Span::call_site());
        if let Some((_, value)) = metrics.iter().find(|(key, _)| key.as_str() == *field) {
            quote! { #field_ident: Some(#value) }
        } else {
            quote! { #field_ident: None }
        }
    });

    Ok(quote! {
        ::casiros_erp::narrative::generate_narrative(&::casiros_erp::narrative::NarrativeInputs {
            company: (#company).to_string(),
            #(#field_inits),*
        })
    })
}

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::quote;

    #[test]
    fn valid_input_expands_without_error() {
        let input = quote! { company: "Acme Corp", roe: dec!(0.15) };
        let result = expand(input);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("NarrativeInputs"));
        assert!(tokens.contains("roe : Some"));
    }

    #[test]
    fn missing_company_is_an_error() {
        let input = quote! { roe: dec!(0.15) };
        let result = expand(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("'company'"));
    }

    #[test]
    fn unknown_key_is_an_error() {
        let input = quote! { company: "Acme", not_a_metric: dec!(1) };
        let result = expand(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unknown narrative metric")
        );
    }

    #[test]
    fn duplicate_key_is_an_error() {
        let input = quote! { company: "Acme", roe: dec!(0.1), roe: dec!(0.2) };
        let result = expand(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duplicate key"));
    }
}
