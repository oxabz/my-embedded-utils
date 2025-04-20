mod format_impl;
mod display_impl;
mod from_impl;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Attribute, DeriveInput,  LitStr};


macro_rules! bail {
    ($e: expr, $msg: literal) => {
        {
            return syn::Error::new($e.span(), $msg).into_compile_error().into()
        }
    };
}
pub(crate) use bail;

fn is_error_attr(attr: &&Attribute) -> bool{
    attr.path().get_ident().map(|ident| ident == "error").unwrap_or(false)
}
fn is_from_attr(attr: &&Attribute) -> bool{
    attr.path().get_ident().map(|ident| ident == "from").unwrap_or(false)
}

#[proc_macro_derive(DefmtError, attributes(error, from, display))]
pub fn derive_helper_attr(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let data_enum = match input.data {
        syn::Data::Enum(data_enum) => data_enum,
        _ => bail!(input, "DefmtError derive is only implemented for enums")
    };

    let ident = input.ident;

    let mut variants_display_impl = vec![];
    let mut variants_format_impl = vec![];
    let mut from_impls = vec![];
    for variant in data_enum.variants{
        let Some(attr) = variant.attrs.iter().find(is_error_attr) else {
            bail!(variant, "Variant need to have an associated error attribute");
        };

        let Ok(fmt_str) = attr.parse_args::<LitStr>() else {
            bail!(attr, "error only accept a single format string");
        };

        let (display_impl, format_impl) = match &variant.fields {
            syn::Fields::Named(_fields_named) => todo!("Named fields variant are not yet supported"),
            syn::Fields::Unnamed(fields_unnamed) => {
                let is_into_variant = fields_unnamed.unnamed
                    .iter()
                    .flat_map(|field|field.attrs.iter())
                    .find(is_from_attr).is_some();
                if is_into_variant{
                    from_impls.push(from_impl::impl_from(&ident, &variant, fields_unnamed));
                }

                (display_impl::impl_unamed_variant(&variant, &fmt_str, fields_unnamed), format_impl::impl_unamed_variant(&variant, &fmt_str, fields_unnamed))
            },
            syn::Fields::Unit => (display_impl::impl_unit_variant(&variant, &fmt_str), format_impl::impl_unit_variant(&variant, &fmt_str)),
        };
        
        variants_format_impl.push(format_impl);
        variants_display_impl.push(display_impl);
    }

    let defmt_impl = if cfg!(feature = "defmt") {
        Some(quote! {
            impl defmt::Format for  #ident {
                fn format(&self, fmt: defmt::Formatter) {
                    match self {
                        #(#variants_format_impl)*
                    }
                }
            }    
        })
    } else {
        None
    };

    quote! {
        impl DefmtError for #ident {}
        impl core::error::Error for  #ident {}

        impl core::fmt::Display for #ident  {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #(#variants_display_impl)*
                }
            }
        }

        #defmt_impl

        #(#from_impls)*
    }.into()
}
