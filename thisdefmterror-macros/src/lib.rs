mod format_impl;
mod display_impl;
mod from_impl;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, spanned::Spanned, Attribute, DeriveInput, GenericArgument, GenericParam, LitStr, PathSegment, Token};


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

fn generic_param_to_arg(param: &GenericParam) -> GenericArgument{
    match param {
        GenericParam::Lifetime(lifetime_param) =>
            GenericArgument::Lifetime(lifetime_param.lifetime.clone()),
        GenericParam::Type(type_param) => {
            let mut segments: Punctuated<PathSegment, syn::token::PathSep> = Punctuated::new();
            segments.push(PathSegment{
                ident: type_param.ident.clone(),
                arguments: syn::PathArguments::None,
            });
            let typ = syn::TypePath{
                qself: None,
                path: syn::Path{
                    leading_colon: None,
                    segments,
                },
            };

            GenericArgument::Type(syn::Type::Path(typ))
        },
        GenericParam::Const(const_param) => {
            let mut segments: Punctuated<PathSegment, syn::token::PathSep> = Punctuated::new();
            segments.push(PathSegment{
                ident: const_param.ident.clone(),
                arguments: syn::PathArguments::None,
            });
            let expr_path = syn::ExprPath{
                attrs: vec![],
                qself: None,
                path: syn::Path { leading_colon: None, segments},
            };
            GenericArgument::Const(syn::Expr::Path(expr_path))
        },
    }
}

#[proc_macro_derive(DefmtError, attributes(error, from, display))]
pub fn derive_helper_attr(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let data_enum = match input.data {
        syn::Data::Enum(data_enum) => data_enum,
        _ => bail!(input, "DefmtError derive is only implemented for enums")
    };

    let ident = input.ident;
    let mut generics = input.generics.clone();
    let generics_where = generics.where_clause.take();
    let generics_arg: syn::punctuated::Punctuated<_, Token![::]> = input.generics.params.iter().map(generic_param_to_arg).collect();
    
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
                    from_impls.push(from_impl::impl_from(&ident, &variant, fields_unnamed, &generics, &generics_where, &generics_arg));
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
            impl #generics defmt::Format for  #ident<#generics_arg> #generics_where {
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
        impl #generics DefmtError for #ident<#generics_arg> #generics_where {}
        impl #generics core::error::Error for  #ident<#generics_arg> #generics_where {}

        impl #generics core::fmt::Display for #ident<#generics_arg> #generics_where {
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
