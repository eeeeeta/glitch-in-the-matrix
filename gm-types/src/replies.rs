//! Replies obtained from calling various API endpoints.
use room::Room;

/// The reply obtained from `/send`.
#[derive(Deserialize, Clone, Debug)]
pub struct SendReply {
    pub event_id: String
}
/// The reply obtained from `upload()`.
#[derive(Deserialize, Clone, Debug)]
pub struct UploadReply {
    pub content_uri: String
}
/// The reply obtained from `/join`.
#[derive(Deserialize, Clone, Debug)]
pub struct JoinReply {
    #[serde(rename = "room_id")]
    pub room: Room<'static>
}
/// The reply obtained from `/login`.
#[derive(Deserialize, Clone, Debug)]
pub struct LoginReply {
    pub user_id: String,
    pub access_token: String,
    pub home_server: String
}
/// The reply obtained from getting a room alias.
#[derive(Deserialize, Clone, Debug)]
pub struct RoomAliasReply {
    #[serde(rename = "room_id")]
    pub room: Room<'static>,
    pub servers: Vec<String>
}
/// The reply obtained when calling `Room::set_state`.
#[derive(Deserialize, Clone, Debug)]
pub struct SetStateReply {
    /// A unique identifier for the event.
    pub event_id: String
}
/// The reply obtained when something's gone wrong.
#[derive(Deserialize, Clone, Debug)]
pub struct BadRequestReply {
    pub errcode: String,
    pub error: String
}
