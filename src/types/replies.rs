use std::collections::HashMap;
use room::Room;
use ::types::events::{Events};

/// Information about a room's events.
#[derive(Serialize, Deserialize, Debug)]
pub struct RoomEvents {
    #[serde(default)]
    pub state: Events,
    #[serde(default)]
    pub timeline: Events,
}
/// The `rooms` component of a `SyncReply`.
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncRooms {
    #[serde(default)]
    pub join: HashMap<Room<'static>, RoomEvents>,
    #[serde(default)]
    pub invite: HashMap<Room<'static>, RoomEvents>,
    #[serde(default)]
    pub leave: HashMap<Room<'static>, RoomEvents>
}
/// The reply obtained from `sync()`.
#[derive(Deserialize, Debug)]
pub struct SyncReply {
    pub next_batch: String,
    pub rooms: SyncRooms
}
/// The reply obtained from `/send`.
#[derive(Deserialize, Debug)]
pub struct SendReply {
    pub event_id: String
}
/// The reply obtained from `upload()`.
#[derive(Deserialize, Debug)]
pub struct UploadReply {
    pub content_uri: String
}
/// The reply obtained from `/join`.
#[derive(Deserialize, Debug)]
pub struct JoinReply {
    #[serde(rename = "room_id")]
    pub room: Room<'static>
}
/// The reply obtained from `/login`.
#[derive(Deserialize, Debug)]
pub struct LoginReply {
    pub user_id: String,
    pub access_token: String,
    pub home_server: String
}
/// The reply obtained from getting a room alias.
#[derive(Deserialize, Debug)]
pub struct RoomAliasReply {
    #[serde(rename = "room_id")]
    pub room: Room<'static>,
    pub servers: Vec<String>
}
/// The reply obtained when something's gone wrong.
#[derive(Deserialize, Debug)]
pub struct BadRequestReply {
    pub errcode: String,
    pub error: String
}
