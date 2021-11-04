use matrix_sdk::{
    room::Room,
    ruma::events::{room::member::MemberEventContent, StrippedStateEvent},
    Client,
};
use tracing::*;

pub async fn autojoin(
    room_member: StrippedStateEvent<MemberEventContent>,
    client: Client,
    room: Room,
) {
    // Autojoin logic
    if room_member.state_key != client.user_id().await.unwrap() {
        warn!("Got invite that isn't for us");
        return;
    }
    if let matrix_sdk::room::Room::Invited(room) = room {
        info!("Autojoining room {}", room.room_id());
        let mut delay = 2;

        while let Err(err) = room.accept_invitation().await {
            // retry autojoin due to synapse sending invites, before the
            // invited user can join for more information see
            // https://github.com/matrix-org/synapse/issues/4345
            error!(
                "Failed to join room {} ({:?}), retrying in {}s",
                room.room_id(),
                err,
                delay
            );

            tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                error!("Can't join room {} ({:?})", room.room_id(), err);
                break;
            }
        }
        info!("Successfully joined room {}", room.room_id());
    }
}
