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
        ) -> Result<(), Error> {
            let options = mrsbfh::pulldown_cmark::Options::empty();
            let parser = mrsbfh::pulldown_cmark::Parser::new_ext(HELP_MARKDOWN, options);
            let mut html = String::new();
            mrsbfh::pulldown_cmark::html::push_html(&mut html, parser);
            let owned_html = html.to_owned();

            mrsbfh::tokio::spawn(async move {
                let content = matrix_sdk::events::AnyMessageEventContent::RoomMessage(
                    matrix_sdk::events::room::message::MessageEventContent::notice_html(
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

        pub async fn match_command<C: mrsbfh::config::Config + Clone>(cmd: &str, config: C, tx: mrsbfh::Sender, sender: String, args: Vec<&str>,) -> Result<(), Error> {
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
            fn load<P: AsRef<std::path::Path> + std::fmt::Debug>(path: P) -> Result<Self, mrsbfh::errors::ConfigError> {
                let contents = std::fs::read_to_string(path)?;
                let config: Self = mrsbfh::serde_yaml::from_str(&contents)?;
                Ok(config)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Used to generate code to autojoin when we get a invite for the bot
///
/// Requirements:
///
/// * Tokio
/// * Naming of arguments needs to be EXACTLY like in the example
/// * the async_trait macro needs to be BELOW the autojoin macro
///
/// ```compile_fail
/// #[mrsbfh::utils::autojoin]
/// #[async_trait]
/// impl EventEmitter for Bot {
///
/// async fn on_stripped_state_member(
///         &self,
///         room: SyncRoom,
///         room_member: &StrippedStateEvent<MemberEventContent>,
///         _: Option<MemberEventContent>,
///     ) {
///         // Your own logic. (Executed BEFORE the autojoin)
///     }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn autojoin(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as syn::ItemImpl);
    let items = &mut input.items;

    for item in items {
        if let syn::ImplItem::Method(method) = item {
            if method.sig.ident == "on_stripped_state_member" {
                let original = method.block.clone();
                let new_block = syn::parse_quote! {
                    {
                        #original

                        // Autojoin logic
                        if room_member.state_key != self.client.user_id().await.unwrap() {
                            warn!("Got invite that isn't for us");
                            return;
                        }
                        if let matrix_sdk::SyncRoom::Invited(room) = room {
                            let room_id = {
                                let room = room.read().await;
                                room.room_id.clone()
                            };
                            let client = self.client.clone();

                            tokio::spawn(async move {
                                info!("Autojoining room {}", room_id);
                                let mut delay = 2;

                                while let Err(err) = client.join_room_by_id(&room_id).await {
                                    // retry autojoin due to synapse sending invites, before the
                                    // invited user can join for more information see
                                    // https://github.com/matrix-org/synapse/issues/4345
                                    error!(
                                        "Failed to join room {} ({:?}), retrying in {}s",
                                        room_id, err, delay
                                    );

                                    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                                    delay *= 2;

                                    if delay > 3600 {
                                        error!("Can't join room {} ({:?})", room_id, err);
                                        break;
                                    }
                                }
                                info!("Successfully joined room {}", room_id);
                            });
                        }
                    }
                };
                method.block = new_block;
            }
        }
    }

    TokenStream::from(quote! {#input})
}

/// Used to generate code to detect commands when we get a message for the bot
///
/// Requirements:
///
/// * Tokio
/// * Naming of arguments needs to be EXACTLY like in the example
/// * the async_trait macro needs to be BELOW the commands macro
/// * The match_command MUST be imported
///
/// ```compile_fail
/// use crate::commands::match_command;
///
/// #[mrsbfh::commands::commands]
/// #[async_trait]
/// impl EventEmitter for Bot {
///
/// async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
///         // Your own logic. (Executed BEFORE the commands matching)
///     }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn commands(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as syn::ItemImpl);
    let items = &mut input.items;

    for item in items {
        if let syn::ImplItem::Method(method) = item {
            if method.sig.ident == "on_room_message" {
                let original = method.block.clone();
                let new_block = syn::parse_quote! {
                    {
                        #original

                        // Command matching logic
                        if let matrix_sdk::SyncRoom::Joined(room) = room {
                            let msg_body = if let matrix_sdk::events::SyncMessageEvent {
                                content: matrix_sdk::events::room::message::MessageEventContent::Text(matrix_sdk::events::room::message::TextMessageEventContent { body: msg_body, .. }),
                                ..
                            } = event
                            {
                                msg_body.clone()
                            } else {
                                String::new()
                            };
                            if msg_body.is_empty() {
                                return;
                            }

                            let sender = event.sender.clone().to_string();

                            let (tx, mut rx) = mpsc::channel(100);
                            let room_id = room.read().await.clone().room_id;

                            let cloned_config = self.config.clone();
                            tokio::spawn(async move {
                                let mut split = msg_body.split_whitespace();

                                let command_raw = split.next().expect("This is not a command");
                                let command = command_raw.to_lowercase();
                                info!("Got command: {}", command);

                                // Make sure this is immutable
                                let args: Vec<&str> = split.collect();
                                if let Err(e) = match_command(
                                    command.replace("!", "").as_str(),
                                    cloned_config.clone(),
                                    tx,
                                    sender,
                                    args,
                                )
                                .await
                                {
                                    error!("{}", e);
                                }
                            });

                            while let Some(v) = rx.recv().await {
                                if let Err(e) = self
                                    .client
                                    .clone()
                                    .room_send(&room_id.clone(), v, None)
                                    .await
                                {
                                    error!("{}", e);
                                }
                            }
                        }
                    }
                };
                method.block = new_block;
            }
        }
    }

    TokenStream::from(quote! {#input})
}
