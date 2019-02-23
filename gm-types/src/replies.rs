//! Replies obtained from calling various API endpoints.
use crate::room::Room;
use crate::events::Event;
use std::collections::HashMap;
use serde_json::Value;
use crate::content::room::Member;

/// The reply obtained from `/send`.
#[derive(Deserialize, Clone, Debug)]
pub struct SendReply {
    /// An ID for the sent event.
    pub event_id: String
}
/// The reply obtained from `/upload`.
#[derive(Deserialize, Clone, Debug)]
pub struct UploadReply {
    /// The MXC URI for the uploaded conten.
    pub content_uri: String
}
/// The reply obtained from `/join`.
#[derive(Deserialize, Clone, Debug)]
pub struct JoinReply {
    /// The room that was just joined.
    #[serde(rename = "room_id")]
    pub room: Room<'static>
}
/// The reply obtained from `/login`.
#[derive(Deserialize, Clone, Debug)]
pub struct LoginReply {
    /// The fully-qualified Matrix ID that has been registered. 
    pub user_id: String,
    /// An access token for the account.
    ///
    /// This access token can then be used to authorize other requests.
    pub access_token: String,
    /// ID of the logged-in device. 
    ///
    /// Will be the same as the corresponding parameter in the request, if one was specified.
    pub device_id: String
}
/// The reply obtained from the `/whoami` API.
#[derive(Deserialize, Clone, Debug)]
pub struct WhoamiReply {
    /// The user the access token belongs to.
    pub user_id: String
}
/// The reply obtained from getting a room alias.
#[derive(Deserialize, Clone, Debug)]
pub struct RoomAliasReply {
    /// The room for this room alias.
    #[serde(rename = "room_id")]
    pub room: Room<'static>,
    /// A list of servers that are aware of this room alias.
    pub servers: Vec<String>
}
/// Data about a user's display name.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DisplaynameReply {
    /// The display name for this user.
    pub displayname: String
}
/// The reply obtained when calling `Room::set_state`.
#[derive(Deserialize, Clone, Debug)]
pub struct SetStateReply {
    /// A unique identifier for the event.
    pub event_id: String
}
/// A 'standard error response' from a Matrix homeserver.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BadRequestReply {
    /// Unique string describing this kind of error.
    ///
    /// These error codes should have their namespace first in ALL CAPS, followed by a
    /// single _ to ease separating the namespace from the error code. For example, if there was
    /// a custom namespace com.mydomain.here, and a FORBIDDEN code, the error code should look
    /// like COM.MYDOMAIN.HERE_FORBIDDEN.
    pub errcode: String,
    /// Human-readable error message.
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
/// A chunk of events.
#[derive(Deserialize, Debug)]
pub struct ChunkReply {
    /// A list of room events.
    pub chunk: Vec<Event>
}
/// The reply obtained from `/joined_members`.
#[derive(Deserialize, Debug)]
pub struct JoinedMembersReply {
    /// A map of MXID to room member objects.
    pub joined: HashMap<String, Member>
}
#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
/// Controls room visibility in the published room list.
pub enum RoomVisibility {
    Public,
    Private
}
#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// A list of convenience presets used to make a room.
pub enum RoomPreset {
    /// `join_rules` is set to `invite`, `history_visibility` is set to `shared`
    PrivateChat,
    /// `join_rules` is set to `invite`, `history_visibility` is set to `shared`.
    /// All invitees are given the same power level as the room creator.
    TrustedPrivateChat,
    /// `join_rules` is set to `public`, `history_visibility` is set to `shared`
    PublicChat
}
/// Options used to create a room.
#[derive(Serialize, Clone, Debug, Default)]
pub struct RoomCreationOptions {
    /// Controls whether the room will be shown in the published room list.
    /// NB. **Not** the same thing as `join_rules`.
    ///
    /// Rooms default to `private` visibility if this key is not included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<RoomVisibility>,
    /// The desired room alias **local part**.
    ///
    /// If this is included, a room alias will be created and mapped to the newly created room. The alias will belong
    /// on the same homeserver which created the room.
    ///
    /// For example, if this was set to "foo" and sent to the homeserver "example.com", the complete room alias
    /// would be `#foo:example.com`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_alias_name: Option<String>,
    /// If this is included, an `m.room.name` event will be sent into the room to indicate the name of the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// If this is included, an `m.room.topic` event will be sent into the room to indicate the topic for the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// A list of user IDs to invite to the room. This will tell the server to invite everyone in the list to the newly created room.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub invite: Vec<String>,
    /// Extra keys to be added to the content of the `m.room.create`.
    ///
    /// The server will clobber the following keys: `creator`.
    /// Future versions of the specification may allow the server to clobber other keys.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub creation_content: HashMap<String, Value>,
    /// Convenience parameter for setting various default state events based on a preset. 
    ///
    /// See the `RoomPreset` docs for more.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<RoomPreset>,
    /// This flag makes the server set the `is_direct` flag on the `m.room.member` events sent to the users
    /// in `invite` and `invite_3pid`.
    pub is_direct: bool
}
