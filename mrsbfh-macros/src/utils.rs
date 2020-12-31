use proc_macro::TokenStream;
use syn::spanned::Spanned;
use quote::quote;

pub(crate) fn get_arg<'a>(
    input_span: proc_macro2::Span,
    args: syn::AttributeArgs,
    arg: &'a str,
    expected: &'a str,
    expected_args: usize,
) -> Result<syn::LitStr, TokenStream> {
    if args.len() == expected_args {
        let meta = args
            .iter()
            .filter_map(|x| {
                if let syn::NestedMeta::Meta(ref meta) = x {
                    if meta.path().is_ident(&arg) {
                        return Some(meta);
                    }
                }
                None
            })
            .next();
        if let Some(meta) = meta {
            if let syn::Meta::NameValue(ref meta) = meta {
                let meta_lit = meta.lit.clone();
                return match meta_lit {
                    syn::Lit::Str(s) => Ok(s),
                    _ => {
                        let error = syn::Error::new(
                            meta.lit.span(),
                            format!(
                                "expected `{}`\n\nThe field '{}' needs to be a str literal!",
                                expected, arg
                            ),
                        )
                            .to_compile_error();
                        Err(quote! {#error}.into())
                    }
                };
            }
        } else {
            let error = syn::Error::new(
                meta.span(),
                format!(
                    "1expected `{}`\n\nThe field '{}' is required!",
                    expected, arg
                ),
            )
                .to_compile_error();
            return Err(quote! {#error}.into());
        }
    }
    let error = syn::Error::new(
        input_span,
        format!(
            "expected `{}` but not enough arguments where provided.",
            expected
        ),
    )
        .to_compile_error();
    Err(quote! {#error}.into())
}