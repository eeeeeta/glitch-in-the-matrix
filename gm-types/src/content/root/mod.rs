//! Content types for `m.*` events.

use std::collections::HashMap;
pub mod types;

/// `m.typing`
///
/// Informs the client of the list of users currently typing.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Typing {
    /// The list of user IDs typing in this room, if any.
    pub user_ids: Vec<String>
}
/// `m.receipt`
///
/// Informs the client of new receipts.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Receipt(HashMap<String, types::Receipts>);
/// `m.presence`
///
/// Informs the client of a user's presence state change.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Presence {
    /// The current avatar URL for this user, if any.
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// The current display name for this user, if any.
    #[serde(default)]
    pub displayname: Option<String>,
    ///	The last time since this used performed some action, in milliseconds.
    #[serde(default)]
    pub last_active_ago: Option<u64>,
    /// The presence state for this user.
    pub presence: types::Presence,
    /// Whether the user is currently active.
    #[serde(default)]
    pub currently_active: bool,
    /// The user's ID.
    pub user_id: String,
}
/// `m.tag`
///
/// Informs the client of tags on a room.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tag {
    /// The tags on the room and their contents.
    pub tags: HashMap<String, types::RoomTag>
}
/// `m.direct`
///
/// A map of which rooms are considered 'direct' rooms for specific users is
/// kept in account_data in an event of type m.direct. The content of this event
/// is an object where the keys are the user IDs and values are lists of room ID
/// strings of the 'direct' rooms for that user ID.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Direct(HashMap<String, Vec<String>>);
