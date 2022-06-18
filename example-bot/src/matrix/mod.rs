use crate::config::Config;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::UserId;
use matrix_sdk::{Client, Session as SDKSession};
use mrsbfh::url::Url;
use mrsbfh::utils::Session;
use std::sync::Arc;
use std::{convert::TryFrom, error::Error, fs, path::Path};
use tokio::sync::Mutex;
use tracing::*;

mod sync;

pub async fn setup(config: Config<'_>) -> Result<Client, Box<dyn Error>> {
    info!("Beginning Matrix Setup");
    let store_path_string = config.store_path.to_string();
    let store_path = Path::new(&store_path_string);
    if !store_path.exists() {
        fs::create_dir_all(store_path)?;
    }

    let homeserver_url =
        Url::parse(&config.homeserver_url).expect("Couldn't parse the homeserver URL");

    let state_store = matrix_sdk::store::StateStore::open_with_path(store_path)?;
    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .state_store(Box::new(state_store))
        .build()
        .await?;

    if let Some(session) = Session::load(config.session_path.parse().unwrap()) {
        info!("Starting relogin");

        let session = SDKSession {
            access_token: session.access_token,
            device_id: session.device_id.into(),
            user_id: <&UserId>::try_from(session.user_id.as_str())
                .unwrap()
                .to_owned(),
        };

        if let Err(e) = client.restore_login(session).await {
            error!("{}", e);
        };
        info!("Finished relogin");
    } else {
        info!("Starting login");
        let login_response = client
            .login(
                &config.mxid,
                &config.password,
                None,
                Some("timetracking-bot"),
            )
            .await;
        match login_response {
            Ok(login_response) => {
                info!("Session: {:#?}", login_response);
                let session = Session {
                    homeserver: client.homeserver().await.to_string(),
                    user_id: login_response.user_id.to_string(),
                    access_token: login_response.access_token,
                    device_id: login_response.device_id.to_string(),
                };
                session.save(config.session_path.parse().unwrap())?;
            }
            Err(e) => error!("Error while login: {}", e),
        }
        info!("Finished login");
    }

    info!("logged in as {}", config.mxid);

    Ok(client)
}

pub async fn start_sync(
    client: &mut Client,
    config: Config<'static>,
) -> Result<(), Box<dyn Error>> {
    client.register_event_handler(mrsbfh::sync::autojoin).await;

    let config = Arc::new(Mutex::new(config));
    let cloned_config = Arc::clone(&config);
    client
        .register_event_handler(move |ev, room, client| {
            let cloned_config = Arc::clone(&cloned_config);
            sync::on_room_message(ev, room, client, cloned_config)
        })
        .await;

    info!("Starting full Sync...");
    client.sync(SyncSettings::default()).await;

    Ok(())
}
