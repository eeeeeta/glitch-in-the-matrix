//! Content types for `m.room.*` events.

use std::collections::HashMap;

pub mod types;
use messages;

fn tru() -> bool {
    true
}
fn fifty() -> u32 {
    50
}
fn zero() -> u32 {
    0
}
/// `m.room.aliases`
///
/// - `state_key`: The homeserver domain which owns these room aliases.
///
/// This event is sent by a homeserver directly to inform of changes to the list
/// of aliases it knows about for that room. The state_key for this event is set
/// to the homeserver which owns the room alias. The entire set of known aliases
/// for the room is the union of all the m.room.aliases events, one for each
/// homeserver. Clients should check the validity of any room alias given in
/// this list before presenting it to the user as trusted fact. The lists given
/// by this event should be considered simply as advice on which aliases might
/// exist, for which the client can perform the lookup to confirm whether it
/// receives the correct room ID.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Aliases {
    /// A list of room aliases.
    pub aliases: Vec<String>
}
/// `m.room.canonical_alias`
///
/// This event is used to inform the room about which alias should be considered
/// the canonical one. This could be for display purposes or as suggestion to
/// users which alias to use to advertise the room.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CanonicalAlias {
    pub alias: String
}
/// `m.room.create`
///
/// This is the first event in a room and cannot be changed. It acts as the root
/// of all other events.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Create {
    /// The user_id of the room creator. This is set by the homeserver.
    pub creator: String,
    /// Whether users on other servers can join this room. Defaults to true if
    /// key does not exist.
    #[serde(default = "tru", rename = "m.federate")]
    pub m_federate: bool
}
/// `m.room.join_rules`
///
/// A room may be public meaning anyone can join the room without any prior
/// action. Alternatively, it can be invite meaning that a user who wishes to
/// join the room must first receive an invite to the room from someone already
/// inside of the room. Currently, knock and private are reserved keywords which
/// are not implemented.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JoinRules {
    /// The type of rules used for users wishing to join this room.
    pub join_rule: types::JoinRule
}
/// `m.room.member`
///
/// - `state_key`: The user_id this membership event relates to.
///
/// Adjusts the membership state for a user in a room. It is preferable to use
/// the membership APIs (/rooms/<room id>/invite etc) when performing membership
/// actions rather than adjusting the state directly as there are a restricted
/// set of valid transformations. For example, user A cannot force user B to
/// join a room, and trying to force this state change directly will fail.
///
/// The third_party_invite property will be set if this invite is an invite
/// event and is the successor of an m.room.third_party_invite event, and absent
/// otherwise.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Member {
    ///	The avatar URL for this user, if any. This is added by the homeserver.
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// The display name for this user, if any. This is added by the homeserver.
    #[serde(default)]
    pub displayname: Option<String>,
    /// The membership state of the user.
    pub membership: types::Membership,
    /// Flag indicating if the room containing this event was created with the
    /// intention of being a direct chat.
    pub is_direct: Option<bool>,
    /// [undocumented in the spec]
    pub third_party_invite: Option<::serde_json::Value>,
}
/// `m.room.power_levels`
///
/// This event specifies the minimum level a user must have in order to perform
/// a certain action. It also specifies the levels of each user in the room.
///
/// If a user_id is in the users list, then that user_id has the associated
/// power level. Otherwise they have the default level users_default. If
/// users_default is not supplied, it is assumed to be 0.
///
/// The level required to send a certain event is governed by events,
/// state_default and events_default. If an event type is specified in events,
/// then the user must have at least the level specified in order to send that
/// event. If the event type is not supplied, it defaults to events_default for
/// Message Events and state_default for State Events.
///
/// If there is no state_default in the m.room.power_levels event, the
/// state_default is 50. If there is no events_default in the
/// m.room.power_levels event, the events_default is 0. If the room contains no
/// m.room.power_levels event, both the state_default and events_default are 0.
///
/// The power level required to invite a user to the room, kick a user from the
/// room, ban a user from the room, or redact an event, is defined by invite,
/// kick, ban, and redact, respectively. Each of these levels defaults to 50 if
/// they are not specified in the m.room.power_levels event, or if the room
/// contains no m.room.power_levels event.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PowerLevels {
    /// The level required to ban a user. Defaults to 50 if unspecified.
    #[serde(default = "fifty")]
    pub ban: u32,
    /// The level required to send specific event types. This is a mapping from
    /// event type to power level required.
    #[serde(default)]
    pub events: HashMap<String, u32>,
    /// The default level required to send message events. Can be overridden by
    /// the events key. Defaults to 0 if unspecified.
    #[serde(default = "zero")]
    pub events_default: u32,
    /// The level required to invite a user. Defaults to 50 if unspecified.
    #[serde(default = "fifty")]
    pub invite: u32,
    /// The level required to kick a user. Defaults to 50 if unspecified.
    #[serde(default = "fifty")]
    pub kick: u32,
    /// The level required to redact an event. Defaults to 50 if unspecified.
    #[serde(default = "fifty")]
    pub redact: u32,
    /// The default level required to send state events. Can be overridden by
    /// the events key. Defaults to 50 if unspecified, but 0 if there is no
    /// m.room.power_levels event at all.
    #[serde(default = "fifty")]
    pub state_default: u32,
    /// The power levels for specific users. This is a mapping from user_id to
    /// power level for that user.
    #[serde(default)]
    pub users: HashMap<String,u32>,
    /// The default power level for every user in the room, unless their user_id
    /// is mentioned in the users key. Defaults to 0 if unspecified.
    #[serde(default = "zero")]
    pub users_default: u32,
}
/// `m.room.redaction`
///
/// Events can be redacted by either room or server admins. Redacting an event
/// means that all keys not required by the protocol are stripped off, allowing
/// admins to remove offensive or illegal content that may have been attached to
/// any event. This cannot be undone, allowing server owners to physically
/// delete the offending data. There is also a concept of a moderator hiding a
/// message event, which can be undone, but cannot be applied to state events.
/// The event that has been redacted is specified in the `redacts` event level
/// key.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Redaction {
    ///	The reason for the redaction, if any.
    #[serde(default)]
    pub reason: Option<String>
}
/// `m.room.message`
///
/// Yeah, this is just a re-export of the message type from the other module.
pub type Message = messages::Message;
/// `m.room.name`
///
/// A room has an opaque room ID which is not human-friendly to read. A room
/// alias is human-friendly, but not all rooms have room aliases. The room name
/// is a human-friendly string designed to be displayed to the end-user. The
/// room name is not unique, as multiple rooms can have the same room name set.
///
/// A room with an m.room.name event with an absent, null, or empty name field
/// should be treated the same as a room with no m.room.name event.
///
/// An event of this type is automatically created when creating a room using
/// /createRoom with the name key.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Name {
    /// The name of the room. This MUST NOT exceed 255 bytes.
    pub name: String
}
/// `m.room.avatar`
///
/// A picture that is associated with the room. This can be displayed alongside
/// the room information.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Avatar {
    /// Metadata about the image referred to in `url`.
    #[serde(default)]
    pub info: Option<messages::ImageInfo>,
    /// The URL to the image.
    pub url: String,
}
/// `m.room.topic`
///
/// A topic is a short message detailing what is currently being discussed in
/// the room. It can also be used as a way to display extra information about
/// the room, which may not be suitable for the room name. The room topic can
/// also be set when creating a room using /createRoom with the topic key.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Topic {
    /// The topic text.
    pub topic: String
}
/// `m.room.history_visibility`
///
/// This event controls whether a user can see the events that happened in a
/// room from before they joined.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryVisibility {
    /// Who can see the room history.
    pub history_visibility: types::HistoryVisibility
}
/// `m.room.guest_access`
///
/// This event controls whether guest users are allowed to join rooms. If this
/// event is absent, servers should act as if it is present and has the
/// guest_access value "forbidden".
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuestAccess {
    /// Whether guests can join the room.
    pub guest_access: types::GuestAccess
}
