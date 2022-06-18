pub(crate) mod utils;
use crate::utils::get_arg;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Ident};

/// Used to define a command
///
/// ```compile_fail
/// use mrsbfh::commands::extract::Extension;
///
/// #[command(help = "Description")]
/// async fn hello_world(Extension(tx): Extension<mrsbfh::Sender>,) -> Result<(), Box<dyn std::error::Error>> where Config: mrsbfh::config::Loader + Clone {}
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
/// **Note**: The defined enum will NOT be present at runtime. It gets replaced fully
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
                mrsbfh::commands::Command::call(#command::#command, msg).await
            },
            #command_short => {
                mrsbfh::commands::Command::call(#command::#command, msg).await
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
    let mut help_format_string = String::from("{}");
    input.variants.iter().for_each(|_| {
        help_format_string = format!("{}{}", help_format_string, "{}");
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
        async fn help(
           mrsbfh::commands::extract::Extension(tx): mrsbfh::commands::extract::Extension<mrsbfh::Sender>,
        ) -> Result<(), mrsbfh::errors::Errors> {
            let options = mrsbfh::pulldown_cmark::Options::empty();
            let help_markdown = format!(#help_format_string, #help_preamble, #(#help_parts,)*);
            let parser = mrsbfh::pulldown_cmark::Parser::new_ext(&help_markdown, options);
            let mut html = String::new();
            mrsbfh::pulldown_cmark::html::push_html(&mut html, parser);
            let owned_html = html.to_owned();

            mrsbfh::tokio::spawn(async move {
                let content = matrix_sdk::ruma::events::room::message::RoomMessageEventContent::notice_html(
                    &help_markdown,
                    owned_html,
                );

                if let Err(e) = tx.lock().await.send(content).await {
                    mrsbfh::tracing::error!("Error: {}",e);
                };
            });

            Ok(())
        }

        pub async fn match_command<'a>(cmd: &str, msg: mrsbfh::commands::Message) -> Result<(), impl std::error::Error> {
            match cmd {
                #(#commands)*
                "help" => {
                    mrsbfh::commands::Command::call(help, msg).await
                },
                "h" => {
                    mrsbfh::commands::Command::call(help, msg).await
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
        impl #impl_generics mrsbfh::config::Loader for #name #ty_generics #where_clause {
            fn load<P: AsRef<std::path::Path> + std::fmt::Debug>(path: P) -> Result<Self, mrsbfh::errors::ConfigError> {
                let contents = std::fs::read_to_string(path)?;
                let config: Self = mrsbfh::serde_yaml::from_str(&contents)?;
                Ok(config)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Used to generate code to detect commands when we get a message for the bot
///
/// Requirements:
///
/// * Tokio
/// * Tokio tracing
/// * Naming of arguments needs to be EXACTLY like in the example
/// * the async_trait macro needs to be BELOW the commands macro
/// * The match_command MUST be imported
///
/// ```compile_fail
/// use crate::commands::match_command;
///
/// #[mrsbfh::commands::commands]
/// async fn on_room_message(event: SyncMessageEvent<MessageEventContent>, room: Room) {
///         // Your own logic. (Executed BEFORE the commands matching)
/// }
/// ```
///
#[proc_macro_attribute]
pub fn commands(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut method = parse_macro_input!(input as syn::ItemFn);
    let arguments = method.sig.inputs.clone();

    let message_magic = arguments.iter().map(|argument| {
        if let syn::FnArg::Typed(arg_type) = argument {
            if let syn::Pat::Ident(ref raw_ident) = *arg_type.pat {
                if let syn::Type::Path(ref path) = *arg_type.ty {
                    let ident = &raw_ident.ident;
                    if path
                        .path
                        .segments
                        .iter()
                        .any(|x| x.ident == Ident::new("Arc", Span::call_site())) {
                        quote! {
                            msg.extensions_mut().insert(std::sync::Arc::clone(&#ident));
                        }
                    } else if path
                        .path
                        .segments
                        .iter()
                        .any(|x| x.ident == Ident::new("Client", Span::call_site()))
                        || path
                        .path
                        .segments
                        .iter()
                        .any(|x| x.ident == Ident::new("Room", Span::call_site())) 
                        || path
                        .path
                        .segments
                        .iter()
                        .any(|x| x.ident == Ident::new("OriginalSyncRoomMessageEvent", Span::call_site()))
                    {
                        quote! {
                            msg.extensions_mut().insert(std::sync::Arc::new(mrsbfh::tokio::sync::Mutex::new(#ident.clone())));
                        }
                    } else {
                        quote! {
                            msg.extensions_mut().insert(std::sync::Arc::new(#ident.clone()));
                        }
                    }
                } else {
                    panic!("Unexpected type for argument");
                }
            } else {
                panic!("Unexpected type for argument");
            }
        } else {
            panic!("Unexpected type for argument");
        }
    });

    if method.sig.ident == "on_room_message" {
        let original = method.block.clone();
        let new_block = syn::parse_quote! {
            {
                #original

                // Command matching logic
                if let matrix_sdk::room::Room::Joined(room) = room {
                    let msg_body = match event.content.msgtype {
                        matrix_sdk::ruma::events::room::message::MessageType::Text(matrix_sdk::ruma::events::room::message::TextMessageEventContent { ref body, .. }) => body.clone(),
                        _ => return,
                    };
                    if msg_body.is_empty() {
                        return;
                    }

                    let sender = event.sender.clone().to_string();

                    let (tx, mut rx): (mrsbfh::Sender, _) = tokio::sync::mpsc::channel(100);
                    let room_id = room.room_id().clone();

                    let cloned_client = client.clone();
                    let cloned_room = room.clone();
                    tokio::spawn(async move {
                        let normalized_body = mrsbfh::commands::command_utils::WHITESPACE_DEDUPLICATOR_MAGIC.replace_all(&msg_body, " ");
                        let cloned_body = dbg!(normalized_body).clone();
                        let mut split = cloned_body.split_whitespace().map(|x|x.to_string());

                        let command_raw = split.next().expect("This is not a command").to_lowercase();
                        let command = mrsbfh::commands::command_utils::COMMAND_MATCHER_MAGIC.captures(command_raw.as_str())
                                                           .map_or(String::new(), |caps| {
                                                                caps.get(1)
                                                                    .map_or(String::new(),
                                                                            |m| String::from(m.as_str()))
                                                           });
                        if !command.is_empty() {
                           tracing::info!("Got command: {}", command);
                        }
                        // Make sure this is immutable
                        let args_raw: Vec<String> = split.collect();
                        let args: std::sync::Arc<mrsbfh::tokio::sync::Mutex<Vec<String>>> = std::sync::Arc::new(mrsbfh::tokio::sync::Mutex::new(args_raw.clone()));
                        let tx = std::sync::Arc::new(mrsbfh::tokio::sync::Mutex::new(tx));

                        let mut msg = mrsbfh::commands::Message::new();
                        #(#message_magic)*
                        msg.extensions_mut().insert(std::sync::Arc::clone(&args));
                        msg.extensions_mut().insert(std::sync::Arc::clone(&tx));
                        if let Err(e) = match_command(
                            command.as_str(),
                            msg
                        )
                        .await
                        {
                            tracing::error!("{}", e);
                        }

                    });

                    while let Some(v) = rx.recv().await {
                        if let Err(e) = cloned_room.send(v, None)
                            .await
                        {
                            tracing::error!("{}", e);
                        }
                    }
                }
            }
        };
        method.block = new_block;
    } else {
        panic!("Function needs to be called `on_room_message`");
    }

    TokenStream::from(quote_spanned! {Span::call_site()=>#method})
}
