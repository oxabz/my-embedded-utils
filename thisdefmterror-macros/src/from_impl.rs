use quote::quote_spanned;
use syn::{spanned::Spanned, GenericArgument, Token};

use crate::bail;


pub(crate) fn impl_from(item_ident: &syn::Ident, variant: &syn::Variant, fields: &syn::FieldsUnnamed, generics: &syn::Generics, whereclause: &Option<syn::WhereClause>, args: &syn::punctuated::Punctuated<GenericArgument, Token![::]>) -> proc_macro2::TokenStream {
    if fields.unnamed.len() > 1 {
        bail!(fields, "#[into] only works with one fields variants");
    }

    let field = fields.unnamed.get(0).unwrap();
    let field_path = &field.ty;
    let variant_ident = &variant.ident;
    quote_spanned! {
        field.span() =>
        impl #generics From<#field_path> for #item_ident<#args> #whereclause {
            fn from(this:#field_path) -> Self {
                Self::#variant_ident(this)
            }
        }
    }
}