//! Types used in the Matrix API.
//!
//! Will be better documented in the future; for now,
//! refer to the official API docs for info on what fields mean.
use std::collections::HashMap;

fn serde_true() -> bool {
    true
}
/// Information about an image.
#[derive(Serialize, Deserialize, Debug)]
pub struct ImageInfo {
    /// The height of the image in pixels.
    pub h: u32,
    /// MIME type
    pub mimetype: String,
    /// Size, in bytes
    pub size: u32,
    /// The width of the image in pixels.
    pub w: u32
}
/// Information about a file.
#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    /// MIME type
    pub mimetype: String,
    /// Size, in bytes
    pub size: u32
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "msgtype")]
/// A message sent to a room.
pub enum Message {
    #[serde(rename="m.text")]
    /// This message is the most basic message and is used to represent text.
    Text {
        /// The body of the message.
        body: String,
        /// The formatted body of the message (if the message is formatted).
        #[serde(default)]
        formatted_body: Option<String>,
        /// The format of the formatted body (if the message is formatted).
        #[serde(default)]
        format: Option<String>
    },
    #[serde(rename="m.notice")]
    /// A m.notice message should be considered similar to a plain m.text message except
    /// that clients should visually distinguish it in some way.
    /// It is intended to be used by automated clients, such as bots, bridges, and other
    /// entities, rather than humans. Additionally, such automated agents which watch a
    /// room for messages and respond to them ought to ignore m.notice messages. This
    /// helps to prevent infinite-loop situations where two automated clients continuously
    /// exchange messages, as each responds to the other.
    Notice {
        /// The notice text to send.
        body: String,
        /// The formatted body of the message (if the message is formatted).
        #[serde(default)]
        formatted_body: Option<String>,
        /// The format of the formatted body (if the message is formatted).
        #[serde(default)]
        format: Option<String>
    },
    #[serde(rename="m.image")]
    /// This message represents a single image and an optional thumbnail.
    Image {
        /// A textual representation of the image. This could be the alt text of the image,
        /// the filename of the image, or some kind of content description for accessibility
        /// e.g. 'image attachment'.
        body: String,
        /// Metadata about the image referred to in url.
        info: Option<ImageInfo>,
        /// Metadata about the image referred to in thumbnail_url.
        thumbnail_info: Option<ImageInfo>,
        /// The URL to the thumbnail of the image.
        thumbnail_url: Option<String>,
        /// The URL to the image.
        url: String
    },
    #[serde(rename="m.emote")]
    /// This message is similar to m.text except that the sender is 'performing' the action
    /// contained in the body key, similar to /me in IRC. This message should be prefixed by the
    /// name of the sender. This message could also be represented in a different colour to
    /// distinguish it from regular m.text messages.
    Emote {
        /// The emote action to perform.
        body: String
    },
    #[serde(rename="m.file")]
    /// This message represents a generic file.
    File {
        /// A human-readable description of the file. This is recommended to be the filename
        /// of the original upload.
        body: String,
        /// The original filename of the uploaded file.
        filename: String,
        /// Information about the file referred to in url.
        info: Option<FileInfo>,
        /// Metadata about the image referred to in thumbnail_url.
        thumbnail_info: Option<ImageInfo>,
        /// The URL to the thumbnail of the file.
        thumbnail_url: Option<String>,
        /// The URL to the file.
        url: String
    },
    #[serde(rename="m.location")]
    /// This message represents a real-world location.
    Location {
        /// A description of the location e.g. 'Big Ben, London, UK', or some kind of content
        /// description for accessibility e.g. 'location attachment'.
        body: String,
        /// A geo URI representing this location.
        geo_uri: String
    }
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "presence", rename_all="snake_case")]
/// Information about whether people are online or not.
pub enum Presence {
    Online,
    Offline,
    Unavailable
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MembershipState {
    /// The user has been invited to join a room, but has not yet joined it.
    /// They may not participate in the room until they join.
    Invite,
    /// The user has been invited to join a room, but has not yet joined it.
    /// They may not participate in the room until they join.
    Join,
    /// The user was once joined to the room, but has since left (possibly by
    /// choice, or possibly by being kicked).
    Leave,
    /// The user has been banned from the room, and is no longer allowed to join
    /// it until they are un-banned from the room (by having their membership
    /// state set to a value other than this one).
    Ban,
    /// This is a reserved word, which currently has no meaning.
    Knock
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
/// The content of a room event.
pub enum Content {
    #[serde(rename="m.room.message")]
    RoomMessage(Message),
    #[serde(rename="m.room.member")]
    RoomMember {
        membership: MembershipState,
        avatar_url: Option<String>,
        displayname: Option<String>
    },
    #[serde(rename="m.typing")]
    Typing { user_ids: Vec<String> },
    #[serde(rename="m.presence")]
    Presence(Presence),
    #[serde(rename="m.room.aliases")]
    RoomAliases { aliases: Vec<String> },
    #[serde(rename="m.room.canonical_alias")]
    RoomCanonicalAlias { alias: String },
    #[serde(rename="m.room.create")]
    RoomCreationEvent {
        creator: String,
        #[serde(default = "serde_true", rename = "m.federate")]
        federated: bool
    },
    #[serde(rename="m.room.redaction")]
    RoomRedaction { reason: Option<String> },
    Unknown(::serde_json::Value)
}
/// An event in a room.
#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub event_id: String,
    pub sender: String,
    #[serde(rename="type")]
    pub ty: String,
    pub origin_server_ts: u64,
    #[serde(default)]
    pub age: Option<u64>,
    pub content: Content,
    #[serde(default)]
    pub prev_content: Option<Content>,
    #[serde(default)]
    pub state_key: Option<String>,
    #[serde(default)]
    pub redacts: Option<String>,
}
/// Events in a room.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Events {
    pub events: Vec<Event>
}
/// Information about a room's events.
#[derive(Serialize, Deserialize, Debug)]
pub struct Room {
    #[serde(default)]
    pub state: Events,
    #[serde(default)]
    pub timeline: Events,
}
/// The `rooms` component of a `SyncReply`.
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncRooms {
    #[serde(default)]
    pub join: HashMap<String, Room>,
    #[serde(default)]
    pub invite: HashMap<String, Room>,
    #[serde(default)]
    pub leave: HashMap<String, Room>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct StateEvent {
    pub content: Content,
    pub state_key: Option<String>,
    #[serde(rename="type")]
    pub ty: String
}
/// Used for the `preset` parameter of a `NewRoomInfo`.
#[derive(Serialize, Deserialize, Debug)]
pub enum RoomPreset {
    #[serde(rename="private_chat")]
    /// `join_rules` is set to invite. `history_visibility` is set to shared.
    PrivateChat,
    #[serde(rename="trusted_private_chat")]
    /// `join_rules` is set to invite. `history_visibility` is set to shared.
    /// All invitees are given the same power level as the room creator.
    TrustedPrivateChat,
    #[serde(rename="public_chat")]
    /// `join_rules` is set to public. `history_visibility` is set to shared.
    PublicChat
}
/// Something that is either `public` or `private`.
#[derive(Serialize, Deserialize, Debug)]
pub enum PublicPrivate {
    #[serde(rename="public")]
    Public,
    #[serde(rename="private")]
    Private
}
/// Information pertaining to the creation of a new room.
#[derive(Serialize, Default, Debug)]
pub struct NewRoomInfo {
    /// Extra keys to be added to the content of the m.room.create. The server
    /// will clobber the following keys: creator. Future versions of the
    /// specification may allow the server to clobber other keys.
    pub creation_content: HashMap<String, String>,
    /// A list of state events to set in the new room. This allows the user to
    /// override the default state events set in the new room. The expected
    /// format of the state events are an object with type, state_key and
    /// content keys set. Takes precedence over events set by presets, but gets
    /// overridden by name and topic keys.
    pub initial_state: Vec<StateEvent>,
    /// A list of user IDs to invite to the room. This will tell the server to
    /// invite everyone in the list to the newly created room.
    pub invite: Vec<String>,
    /// If this is included, an m.room.name event will be sent into the room to
    /// indicate the name of the room.
    pub name: Option<String>,
    /// Convenience parameter for setting various default state events based on a preset.
    pub preset: Option<RoomPreset>,
    /// The desired room alias **local part**. If this is included, a room alias
    /// will be created and mapped to the newly created room. The alias will
    /// belong on the same homeserver which created the room. For example, if
    /// this was set to "foo" and sent to the homeserver "example.com" the
    /// complete room alias would be #foo:example.com.
    pub room_alias_name: Option<String>,
    /// If this is included, an m.room.topic event will be sent into the room to
    /// indicate the topic for the room.
    pub topic: Option<String>,
    /// A public visibility indicates that the room will be shown in the
    /// published room list. A private visibility will hide the room from the
    /// published room list. Rooms default to private visibility if this key is
    /// not included.
    pub visibility: Option<PublicPrivate>
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
/// The reply obtained from `create_room()` and `join()`, indicating a room ID.
#[derive(Deserialize, Debug)]
pub struct RoomIdReply {
    pub room_id: String
}
/// The reply obtained from `/login`.
#[derive(Deserialize, Debug)]
pub struct LoginReply {
    pub user_id: String,
    pub access_token: String,
    pub home_server: String
}
/// The reply obtained when something's gone wrong.
#[derive(Serialize, Deserialize, Debug)]
pub struct BadRequestReply {
    pub errcode: String,
    #[serde(default)]
    pub error: String
}

#[cfg(test)]
#[test]
fn deser_sync() {
    let sync = include_str!("../sync.json");
    ::serde_json::from_str::<SyncReply>(sync).unwrap();
}
