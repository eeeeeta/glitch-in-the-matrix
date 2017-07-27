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
    pub w: u32,
}

#[derive(Serialize, Deserialize, Debug)]
/// Information about an image and it's thumbnail.
pub struct Image {
    /// A textual representation of the image. This could be the alt text of the image,
    /// the filename of the image, or some kind of content description for accessibility
    /// e.g. 'image attachment'.
    pub body: String,
    /// Metadata about the image referred to in url.
    pub info: Option<ImageInfo>,
    /// Metadata about the image referred to in thumbnail_url.
    pub thumbnail_info: Option<ImageInfo>,
    /// The URL to the thumbnail of the image.
    pub thumbnail_url: Option<String>,
    /// The URL to the image.
    pub url: String
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
    Image( Image ),
    //  {
    //     /// A textual representation of the image. This could be the alt text of the image,
    //     /// the filename of the image, or some kind of content description for accessibility
    //     /// e.g. 'image attachment'.
    //     body: String,
    //     /// Metadata about the image referred to in url.
    //     info: Option<ImageInfo>,
    //     /// Metadata about the image referred to in thumbnail_url.
    //     thumbnail_info: Option<ImageInfo>,
    //     /// The URL to the thumbnail of the image.
    //     thumbnail_url: Option<String>,
    //     /// The URL to the image.
    //     url: String
    // },
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
#[serde(tag="join_rules", rename_all="snake_case")]
/// Defines who can join a room
pub enum JoinRule {
    Public,
    Invite,
    // reserved keywords
    // Knock,
    // Private,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="snake_case")]
/// Defines what Membership a user in a room
pub enum Membership {
    Invite,
    Join,
    Leave,
    Ban,
    // reserved word
    // Knock,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="snake_case")]
/// Defines what Membership a user in a room
pub enum FeedbackType {
    Read,
    Delivered,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
/// The content of a room event.
pub enum Content {
    #[serde(rename="m.room.aliases")]
    RoomAliases { aliases: Vec<String> },
    #[serde(rename="m.room.canonical_alias")]
    RoomCanonicalAlias { alias: String },
    #[serde(rename="m.room.create")]
    RoomCreate { creator: String },
    #[serde(rename="m.room.join_rules")]
    RoomJoinRule(JoinRule),
    #[serde(rename="m.room.member")]
    RoomMember {
        avatar_url: Option<String>,
        displayname: Option<String>,
        membership: Membership,
        is_direct: 	Option<bool>,
        third_party_invite: Option<::serde_json::Value>,
    },
    #[serde(rename="m.room.power_levels")]
    RoomPowerLevels {
        ban: u32,
        events: HashMap<String,u32>,
        events_default: u32,
        invite: u32,
        kick: u32,
        redact: u32,
        state_default: u32,
        users: HashMap<String,u32>,
        users_default: u32,
    },
    #[serde(rename="m.room.redaction")]
    RoomRedaction { readon: String, },
    #[serde(rename="m.room.message")]
    RoomMessage(Message),
    #[serde(rename="m.room.message.feedback")]
    RoomFeedback {
        target_event_id: String,
        #[serde(rename="type")]
        feedback_type: FeedbackType,
    },
    #[serde(rename="m.room.name")]
    RoomName { name: String, },
    #[serde(rename="m.room.topic")]
    RoomTopic { topic: String, },
    #[serde(rename="m.room.")]
    Room { },
    // #[serde(rename="m.room.")]
    // Room { },
    #[serde(rename="m.typing")]
    Typing { user_ids: Vec<String> },
    #[serde(rename="m.presence")]
    Presence(Presence),
    // in debug mode, every event must be known
    #[cfg(not(debug_assertions))]
    Unknown(::serde_json::Value),
}


#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
/// basic Event
pub struct BasicEvent {
    #[serde(rename="type")]
    pub event_type: String,
    pub content: HashMap<String,::serde_json::Value>,
    pub room_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
// in debug mode, every field must be parsed
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
/// Event in invite_room_state
pub struct MinimalEvent {
    #[serde(rename="type")]
    pub event_type: String,
    pub content: Content,
    pub sender: Option<String>,
    #[serde(default)]
    pub state_key: Option<String>,
    /// optional. The event that redacted this event, if any.
    pub redacted_because: Option<Event>,

}

/// An event in a room.
#[derive(Serialize, Deserialize, Debug)]
// in debug mode, every field must be parsed
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct Event {
    #[serde(default)]
    pub event_id: String,
    #[serde(default)]
    pub sender: String,
    pub content: Content,
    #[serde(rename="type")]
    pub event_type: String,
    pub room_id: Option<String>,
    // #[serde(default)]
    pub origin_server_ts: u64,
    // #[serde(default)]
    pub state_key: Option<String>,
    // #[serde(default)]
    pub age: u64,
    #[serde(default)]
    pub prev_content: Option<Content>,
    #[serde(default,rename="txn_id")]
    pub transaction_id: Option<String>,
    #[serde(default)]
    pub invite_room_state: Vec<MinimalEvent>,
    pub unsigned: Option<::serde_json::Value>,
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
/// The reply obtained from `upload()`.
#[derive(Deserialize, Debug)]
pub struct UploadReply {
    pub content_uri: String
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
fn test_deser() {
    // let event_list: &str = include_str!("../events.json");
    let event_list: &str = "";
    let event_json = ::json::parse(event_list).unwrap();
    for event in event_json["events"].members() {
        let e_json = ::json::stringify_pretty(event.clone(),4);
        println!("trying to parse event of type {}:",event["type"]);
        if let Err(error) = ::serde_json::from_str::<Event>(&e_json[..]) {
            println!("Error parsing json: {}",error);
            println!("{:#}",e_json);
        }
        // if event.has_key("invite_room_state") {
        //     println!("{:?}", e.invite_room_state);
        // }
    }
}

#[cfg(test)]
#[test]
fn deser_sync() {
    let sync = include_str!("../sync.json");
    ::serde_json::from_str::<SyncReply>(sync).unwrap();
}
