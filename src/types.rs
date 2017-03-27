//! Types used in the Matrix API.
//!
//! Will be better documented in the future; for now,
//! refer to the official API docs for info on what fields mean.
use std::collections::HashMap;

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
        body: String
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
        body: String
    },
    #[serde(rename="m.image")]
    /// This message represents a single image and an optional thumbnail.
    Image {
        /// A textual representation of the image. This could be the alt text of the image,
        /// the filename of the image, or some kind of content description for accessibility
        /// e.g. 'image attachment'.
        body: String,
        /// Metadata about the image referred to in url.
        info: ImageInfo,
        /// Metadata about the image referred to in thumbnail_url.
        thumbnail_info: ImageInfo,
        /// The URL to the thumbnail of the image.
        thumbnail_url: String,
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
        info: FileInfo,
        /// Metadata about the image referred to in thumbnail_url.
        thumbnail_info: ImageInfo,
        /// The URL to the thumbnail of the file.
        thumbnail_url: String,
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
pub enum Presence {
    Online,
    Offline,
    Unavailable
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
/// The content of a room event.
pub enum Content {
    #[serde(rename="m.room.message")]
    RoomMessage(Message),
    #[serde(rename="m.room.member")]
    RoomMember { membership: String },
    #[serde(rename="m.typing")]
    Typing { user_ids: Vec<String> },
    #[serde(rename="m.presence")]
    Presence(Presence),
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
    pub state_key: Option<String>
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
/// The reply obtained from `/join`.
#[derive(Deserialize, Debug)]
pub struct JoinReply {
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
#[derive(Deserialize, Debug)]
pub struct BadRequestReply {
    pub errcode: String,
    pub error: String
}

#[cfg(test)]
#[test]
fn deser_sync() {
    let sync = include_str!("../sync.json");
    ::serde_json::from_str::<SyncReply>(sync).unwrap();
}
