pub(crate) mod utils;

use crate::utils::get_arg;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;

/// Used to define a command
///
/// ```compile_fail
/// #[command(help = "Description")]
/// async fn hello_world<C: mrsbfh::config::Config>(mut tx: mrsbfh::Sender, config: C, sender: String, mut args: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {}
/// ```
#[proc_macro_attribute]
pub fn command(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemFn);

    let args = parse_macro_input!(args as syn::AttributeArgs);

    let help_const_name = syn::Ident::new(
        &format!(
            "{}_HELP",
            input.sig.ident.to_string().to_uppercase().replace("r#", "")
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
                .split('_')
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
        use mrsbfh::const_concat::*;
        const HELP_MARKDOWN: &str = const_concat!(#help_preamble, #(#help_parts,)*);

        async fn help(
            mut tx: mrsbfh::Sender,
        ) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
            let options = mrsbfh::pulldown_cmark::Options::empty();
            let parser = mrsbfh::pulldown_cmark::Parser::new_ext(HELP_MARKDOWN, options);
            let mut html = String::new();
            mrsbfh::pulldown_cmark::html::push_html(&mut html, parser);
            let owned_html = html.to_owned();

            mrsbfh::tokio::spawn(async move {
                let content = mrsbfh::matrix_sdk::events::AnyMessageEventContent::RoomMessage(
                    mrsbfh::matrix_sdk::events::room::message::MessageEventContent::notice_html(
                        HELP_MARKDOWN,
                        owned_html,
                    ),
                );

                if let Err(e) = tx.send(content).await {
                    mrsbfh::tracing::error!("Error: {}",e);
                };
            });

            Ok(())
        }

        pub async fn match_command<C: mrsbfh::config::Config>(cmd: &str, config: C, tx: mrsbfh::Sender, sender: String, args: Vec<&str>,) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
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

#[proc_macro_derive(ConfigDerive)]
pub fn config_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics mrsbfh::config::Config for #name #ty_generics #where_clause {
            fn load<P: AsRef<std::path::Path> + std::fmt::Debug>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
                let contents = std::fs::read_to_string(path).expect("Something went wrong reading the file");
                let config: Self = mrsbfh::serde_yaml::from_str(&contents)?;
                Ok(config)
            }
        }
    };

    TokenStream::from(expanded)
}
