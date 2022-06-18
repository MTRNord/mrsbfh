//! # Helpers for the sync process

use matrix_sdk::{room::Room, ruma::events::room::member::StrippedRoomMemberEvent, Client};
use tracing::*;

/// A small helper to auto join any incitation
///
/// To join just do this:
/// ```compile_fail
/// client.register_event_handler(mrsbfh::sync::autojoin).await;
/// ```
/// This will also automatically retry to join if that failed with increasing
/// delay between tries (numeber_of_tries*2) starting with a delay of 2.
/// It will print an error with the room id if the delay exceeds 3600s.
///
pub async fn autojoin(room_member: StrippedRoomMemberEvent, client: Client, room: Room) {
    // Autojoin logic
    if room_member.state_key != client.user_id().await.unwrap() {
        debug!("Got invite that isn't for us");
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
