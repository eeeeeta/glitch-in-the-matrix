//! Replies obtained from calling various API endpoints.
use room::Room;
use events::Event;
use std::collections::HashMap;
use serde_json::Value;
use content::room::Member;

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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DisplaynameReply {
    /// A unique identifier for the event.
    pub displayname: String
}
/// The reply obtained when calling `Room::set_state`.
#[derive(Deserialize, Clone, Debug)]
pub struct SetStateReply {
    /// A unique identifier for the event.
    pub event_id: String
}
/// The reply obtained when something's gone wrong.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BadRequestReply {
    pub errcode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>
}
/// The reply obtained from `Room::messages`.
#[derive(Deserialize, Debug)]
pub struct MessagesReply {
    /// The token the pagination starts from.
    ///
    /// If back-paginating, this will be the token supplied in `from`.
    pub start: String,
    /// The token the pagination ends at.
    ///
    /// If back-paginating, use this to request earlier events.
    pub end: String,
    /// A list of room events.
    pub chunk: Vec<Event>
}
#[derive(Deserialize, Debug)]
pub struct ChunkReply {
    pub chunk: Vec<Event>
}
#[derive(Deserialize, Debug)]
pub struct JoinedMembersReply {
    /// A map of MXID to room member objects.
    pub joined: HashMap<String, Member>
}
#[derive(Deserialize, Clone, Debug)]
pub struct RoomIdReply {
    /// The room ID for this room alias.
    pub room_id: String,
    /// A list of servers that are aware of this room alias.
    pub servers: Vec<String>
}
#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomVisibility {
    Public,
    Private
}
#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomPreset {
    PrivateChat,
    TrustedPrivateChat,
    PublicChat
}
#[derive(Serialize, Clone, Debug, Default)]
pub struct RoomCreationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<RoomVisibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_alias_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub invite: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub creation_content: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<RoomPreset>,
    pub is_direct: bool
}
