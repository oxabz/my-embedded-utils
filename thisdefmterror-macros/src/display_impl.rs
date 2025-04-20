use proc_macro2::Span;
use quote::{quote, quote_spanned};

use syn::{spanned::Spanned, Ident};
pub(crate) fn impl_unit_variant(variant: &syn::Variant, fmt_str: &syn::LitStr) -> proc_macro2::TokenStream{
    let variant_ident = &variant.ident;    
    let variant_span = variant.span();
    quote_spanned! {
        variant_span =>
        Self::#variant_ident => {
            f.write_str(#fmt_str)
        }
    }
}

pub(crate) fn impl_unamed_variant(variant: &syn::Variant, fmt_str: &syn::LitStr, fields: &syn::FieldsUnnamed) -> proc_macro2::TokenStream{
    let mut variant_fields_names = vec![];
    if fields.unnamed.len() > 26 {
        super::bail!(variant, "This macro only support 26 fields for tuple variants, also may I ask what the fuck are you even doing???");
    }
    for (ident, _) in fields.unnamed.iter().enumerate() {
        let ident = char::from_u32('a' as u32 + ident as u32).unwrap();
        let ident = Ident::new(&ident.to_string(), Span::call_site());
        variant_fields_names.push(
            quote! {#ident, }
        );
    }

    let variant_ident = &variant.ident;
    let variant_span = variant.span();
    quote_spanned! {
        variant_span =>
        Self::#variant_ident(#(#variant_fields_names)*) => {
            f.write_fmt(format_args!(fmt, #fmt_str, #(#variant_fields_names)*))
        }
    }
}