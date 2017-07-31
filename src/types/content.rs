use std::collections::HashMap;
use types::messages::{ImageInfo};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="snake_case")]
/// Information about whether people are online or not.
pub enum Presence {
    Online,
    Offline,
    Unavailable
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="snake_case")]
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
/// Possible membership states of a user
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
/// Possible types for a m.room.message.feedback event
pub enum FeedbackType {
    Read,
    Delivered,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="snake_case")]
/// Who can see the room history
pub enum HistoryVisibility {
    Invited,
    Joined,
    Shared,
    WorldReadable,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="snake_case")]
/// Whether guests can join a room
pub enum GuestAccess {
    CanJoin,
    Forbidden,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Receipts {
    #[serde(rename="m.read")]
    pub read: HashMap<String,Receipt>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Receipt {
    pub ts: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicKey {
    pub public_key: String,
    pub key_validity_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomTag {
    // can be a number or a string
    pub order: Option<::serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
/// The content of an event.
pub enum Content {
    #[serde(rename="m.room.aliases")]
    Aliases { aliases: Vec<String> },
    #[serde(rename="m.room.canonical_alias")]
    CanonicalAlias { alias: String },
    #[serde(rename="m.room.create")]
    Create { creator: String },
    #[serde(rename="m.room.join_rules")]
    JoinRule{ join_rule: JoinRule },
    #[serde(rename="m.room.member")]
    Member {
        avatar_url: Option<String>,
        displayname: Option<String>,
        membership: Membership,
        is_direct: Option<bool>,
        third_party_invite: Option<::serde_json::Value>,
    },
    #[serde(rename="m.room.power_levels")]
    PowerLevels {
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
    Redaction { reason: Option<String>, },
    #[serde(rename="m.room.message")]
    Message(super::messages::Message),
    #[serde(rename="m.room.message.feedback")]
    Feedback {
        target_event_id: String,
        #[serde(rename="type")]
        feedback_type: FeedbackType,
    },
    #[serde(rename="m.room.name")]
    Name { name: String, },
    #[serde(rename="m.room.avatar")]
    Avatar {
        info: ImageInfo,
        url: String,
        thumbnail_url: Option<String>,
        thumbnail_info: Option<ImageInfo>,
    },
    #[serde(rename="m.room.topic")]
    Topic { topic: String, },
    #[serde(rename="m.typing")]
    Typing { user_ids: Vec<String> },
    #[serde(rename="m.receipt")]
    Receipt( HashMap<String,Receipts>),
    #[serde(rename="m.presence")]
    Presence {
        avatar_url: Option<String>,
        displayname: Option<String>,
        last_active_ago: Option<i32>,
        presence: Presence,
        currently_active: bool,
        user_id: String,
    },
    #[serde(rename="m.history_visibility")]
    HistoryVisibility { history_visibility:HistoryVisibility },
    #[serde(rename="m.room.guest_access")]
    GuestAccess { guest_access: GuestAccess },
    #[serde(rename="m.room.third_party_invite")]
    ThirdPartyInvite {
        key_validity_url: String,
        public_key: String,
        display_name: String,
        public_keys: Vec<PublicKey>,
    },
    #[serde(rename="m.tag")]
    Tag { tags: HashMap<String,RoomTag> },
    #[serde(rename="m.direct")]
    Direct( HashMap<String,Vec<String>> ),
    #[cfg(not(feature="gitm_deny_unknown"))]
    Unknown(::serde_json::Value),
}
