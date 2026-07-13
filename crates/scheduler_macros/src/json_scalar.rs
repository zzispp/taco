use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprArray, ExprLit, ExprUnary, Lit, LitFloat, LitInt, UnOp};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonScalarKind {
    String,
    Boolean,
    Number,
}

#[derive(Clone)]
pub struct JsonScalar {
    expression: Expr,
    kind: JsonScalarKind,
}

impl JsonScalar {
    pub fn parse_array(array: ExprArray) -> syn::Result<Vec<Self>> {
        array.elems.into_iter().map(Self::parse).collect()
    }

    pub fn kind(&self) -> JsonScalarKind {
        self.kind
    }

    pub fn expression(&self) -> &Expr {
        &self.expression
    }

    pub fn json_tokens(&self) -> TokenStream {
        let expression = &self.expression;
        quote! { serde_json::json!(#expression) }
    }

    fn parse(expression: Expr) -> syn::Result<Self> {
        let kind = match &expression {
            Expr::Lit(ExprLit { lit: Lit::Str(_), .. }) => JsonScalarKind::String,
            Expr::Lit(ExprLit { lit: Lit::Bool(_), .. }) => JsonScalarKind::Boolean,
            Expr::Lit(ExprLit { lit: Lit::Int(value), .. }) => {
                validate_positive_integer(value)?;
                JsonScalarKind::Number
            }
            Expr::Lit(ExprLit { lit: Lit::Float(value), .. }) => {
                validate_float(value)?;
                JsonScalarKind::Number
            }
            Expr::Unary(unary) => parse_negative_number(unary)?,
            _ => return Err(syn::Error::new_spanned(&expression, scalar_error())),
        };
        Ok(Self { expression, kind })
    }
}

fn parse_negative_number(unary: &ExprUnary) -> syn::Result<JsonScalarKind> {
    if !matches!(unary.op, UnOp::Neg(_)) {
        return Err(syn::Error::new_spanned(unary, scalar_error()));
    }
    match unary.expr.as_ref() {
        Expr::Lit(ExprLit { lit: Lit::Int(value), .. }) => validate_negative_integer(value)?,
        Expr::Lit(ExprLit { lit: Lit::Float(value), .. }) => validate_float(value)?,
        _ => return Err(syn::Error::new_spanned(unary, scalar_error())),
    }
    Ok(JsonScalarKind::Number)
}

fn validate_positive_integer(value: &LitInt) -> syn::Result<()> {
    value
        .base10_parse::<u64>()
        .map(|_| ())
        .map_err(|_| syn::Error::new_spanned(value, "JSON integer literal must fit in u64"))
}

fn validate_negative_integer(value: &LitInt) -> syn::Result<()> {
    const MIN_I64_MAGNITUDE: u64 = i64::MAX as u64 + 1;
    let magnitude = value
        .base10_parse::<u64>()
        .map_err(|_| syn::Error::new_spanned(value, "negative JSON integer literal must fit in i64"))?;
    if magnitude <= MIN_I64_MAGNITUDE {
        return Ok(());
    }
    Err(syn::Error::new_spanned(value, "negative JSON integer literal must fit in i64"))
}

fn validate_float(value: &LitFloat) -> syn::Result<()> {
    let parsed = value
        .base10_parse::<f64>()
        .map_err(|_| syn::Error::new_spanned(value, "invalid JSON number literal"))?;
    if parsed.is_finite() {
        return Ok(());
    }
    Err(syn::Error::new_spanned(value, "JSON number literal must be finite"))
}

const fn scalar_error() -> &'static str {
    "disabled_when_values accepts only String, bool, and numeric literals"
}

#[cfg(test)]
mod tests {
    use super::{JsonScalar, JsonScalarKind};

    #[test]
    fn parses_typed_json_scalars() {
        let values = JsonScalar::parse_array(syn::parse_quote!(["text", false, 3, -1.5])).expect("scalars should parse");
        let kinds = values.iter().map(JsonScalar::kind).collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![JsonScalarKind::String, JsonScalarKind::Boolean, JsonScalarKind::Number, JsonScalarKind::Number]
        );
    }

    #[test]
    fn rejects_expressions_and_non_finite_numbers() {
        assert!(JsonScalar::parse_array(syn::parse_quote!([true || false])).is_err());
        assert!(JsonScalar::parse_array(syn::parse_quote!([1e999])).is_err());
    }
}
