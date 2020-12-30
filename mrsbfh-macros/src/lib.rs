use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;

fn get_arg<'a>(
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

/// Used to define a command
///
/// ```compile_fail
/// #[command(help = "Description")]
/// async fn r#in(mut tx: tokio::sync::mpsc::Sender<matrix_sdk::events::AnyMessageEventContent>, sender: String, mut args: Vec<&str>) -> Result<(), ParseErrors> {}
/// ```
#[proc_macro_attribute]
pub fn command(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemFn);

    let args = parse_macro_input!(args as syn::AttributeArgs);

    let help_const_name = syn::Ident::new(
        &format!(
            "{}_HELP",
            input.sig.ident.to_string().to_uppercase().replace("R#", "")
        ),
        input.sig.span(),
    );
    let help_description = match get_arg(
        input.span(),
        args,
        "help",
        "#[command(help = \"<description>\")]",
        1,
    ) {
        Ok(v) => syn::LitStr::new(&format!("* {}\n", v.value()), v.span()),
        Err(e) => return e,
    };

    let code = quote! {
        #input
        pub(crate) const #help_const_name: &str = #help_description;

    };
    code.into()
}

/// Used to generate the match case and help text
///
/// ```compile_fail
/// #[command_generate(bot_name = "botless", description = "Is it a bot or is it not?")]
/// enum Commands {
///     In,
///     Out
/// }
/// ```
///
/// Note: The defined enum will NOT be present at runtime. It gets replaced fully
#[proc_macro_attribute]
pub fn command_generate(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemEnum);

    let args = parse_macro_input!(args as syn::AttributeArgs);

    let commands = input.variants.iter().map(|v| {
        let command_string = v.ident.to_string().to_lowercase();
        let command_short = {
            let chars: Vec<String> = v
                .ident
                .to_string()
                .to_case(Case::Snake)
                .split("_")
                .map(|x| x.chars().next().unwrap().to_string().to_lowercase())
                .collect();
            chars.join("")
        };
        let command = quote::format_ident!(
            "r#{}",
            syn::Ident::new(&v.ident.to_string().to_case(Case::Snake), v.span())
        );

        quote! {
            #command_string => {
                #command::#command(tx, config, sender, args).await
            },
            #command_short => {
                #command::#command(tx, config, sender, args).await
            },
        }
    });

    let help_parts = input.variants.iter().map(|v| {
        let command_string = v.ident.to_string().to_case(Case::Snake);
        let help_command =
            syn::Ident::new(&format!("{}_HELP", command_string.to_uppercase()), v.span());
        let command = quote::format_ident!(
            "r#{}",
            syn::Ident::new(&v.ident.to_string().to_case(Case::Snake), v.span())
        );

        quote! {
            #command::#help_command
        }
    });

    let bot_name = match get_arg(
        input.span(),
        args.clone(),
        "bot_name",
        "#[command_generate(bot_name = \"<bot name>\", description = \"<bot description>\")]",
        2,
    ) {
        Ok(v) => v.value(),
        Err(e) => return e,
    };
    let description = match get_arg(
        input.span(),
        args,
        "description",
        "#[command_generate(bot_name = \"<bot name>\", description = \"<bot description>\")]",
        2,
    ) {
        Ok(v) => format!("{}\n\n", v.value()),
        Err(e) => return e,
    };

    let help_title = format!("# Help for the {} Bot\n\n", bot_name);
    let commands_title = "## Commands\n";
    let help_preamble = help_title + &description + commands_title;

    let code = quote! {
        use const_concat::*;
        const HELP_MARKDOWN: &str = const_concat!(#help_preamble, #(#help_parts,)*);

        async fn help(
            mut tx: tokio::sync::mpsc::Sender<matrix_sdk::events::AnyMessageEventContent>,
        ) -> Result<(), crate::errors::ParseErrors> {
            let options = pulldown_cmark::Options::empty();
            let parser = pulldown_cmark::Parser::new_ext(HELP_MARKDOWN, options);
            let mut html = String::new();
            pulldown_cmark::html::push_html(&mut html, parser);
            let owned_html = html.to_owned();

            tokio::spawn(async move {
                let content = matrix_sdk::events::AnyMessageEventContent::RoomMessage(
                    matrix_sdk::events::room::message::MessageEventContent::notice_html(
                        HELP_MARKDOWN,
                        owned_html,
                    ),
                );

                if let Err(e) = tx.send(content).await {
                    tracing::error!("Error: {}",e);
                };
            });

            Ok(())
        }
        pub async fn match_command(cmd: &str, config: crate::config::Config<'_>, tx: tokio::sync::mpsc::Sender<matrix_sdk::events::AnyMessageEventContent>, sender: String, args: Vec<&str>,) -> Result<(), crate::errors::ParseErrors> {
                match cmd {
                    #(#commands)*
                    "help" => {
                        help(tx).await
                    },
                    "h" => {
                        help(tx).await
                    },
                    _ => {Ok(())}
                }
            }

    };
    code.into()
}
