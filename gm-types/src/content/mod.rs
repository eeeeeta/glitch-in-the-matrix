//! Values for the `content` key of an event.
//!
//! These are organised by event type: events with type `m.room.*` are found
//! under the `room` module, while events with type `m.*` are found under the
//! `root` module.
//!
//! The name of a `struct` in this module (and submodules) mirrors its event
//! type, modulo case conversion. That is, `m.room.join_rules` would be found in
//! `content::room::JoinRules`.
#![allow(missing_docs)]
use serde_json::Value;

pub mod room;
pub mod root;
pub mod call;


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
#[allow(missing_docs)]
/// The content of an event.
///
/// NB: This crate's API does *not* guarantee exhaustive matching on this enum.
pub enum Content {
    RoomAliases(room::Aliases),
    RoomAvatar(room::Avatar),
    RoomCanonicalAlias(room::CanonicalAlias),
    RoomCreate(room::Create),
    RoomGuestAccess(room::GuestAccess),
    RoomHistoryVisibility(room::HistoryVisibility),
    RoomJoinRules(room::JoinRules),
    RoomMember(room::Member),
    RoomName(room::Name),
    RoomPowerLevels(room::PowerLevels),
    RoomRedaction(room::Redaction),
    RoomTopic(room::Topic),
    RoomMessage(room::Message),
    RootDirect(root::Direct),
    RootPresence(root::Presence),
    RootReceipt(root::Receipt),
    RootTag(root::Tag),
    RootTyping(root::Typing),
    CallInvite(call::Invite),
    CallCandidates(call::Candidates),
    CallAnswer(call::Answer),
    CallHangup(call::Hangup),
    Unknown(::serde_json::Value),
    #[doc(hidden)]
    __Nonexhaustive
}
// Generates a `match` expression to map event types to the correct content enum
// variant.
//
// The name is an entry for the "Most Ridiculous Macro Name 2017" contest.
macro_rules! matchy_matchy {
    ($in:ident, $val:ident, $($a:pat, $t:ident),*) => {
        match $in {
            $(
                $a => {
                    if let Ok(val) = ::serde_json::from_value($val.clone()) {
                        Content::$t(val)
                    }
                    else {
                        Content::Unknown($val)
                    }
                },
            )*
            _ => Content::Unknown($val),
        }
    }
}
/// Deserialize a JSON `Value` of given event type into some event `Content`.
pub fn deserialize_content(typ: &str, val: Value) -> Content {
    matchy_matchy! {
        typ, val,
        "m.room.aliases", RoomAliases,
        "m.room.avatar", RoomAvatar,
        "m.room.canonical_alias", RoomCanonicalAlias,
        "m.room.create", RoomCreate,
        "m.room.guest_access", RoomGuestAccess,
        "m.room.history_visibility", RoomHistoryVisibility,
        "m.room.join_rules", RoomJoinRules,
        "m.room.member", RoomMember,
        "m.room.name", RoomName,
        "m.room.power_levels", RoomPowerLevels,
        "m.room.redaction", RoomRedaction,
        "m.room.topic", RoomTopic,
        "m.room.message", RoomMessage,
        "m.direct", RootDirect,
        "m.presence", RootPresence,
        "m.receipt", RootReceipt,
        "m.tag", RootTag,
        "m.typing", RootTyping,
        "m.call.invite", CallInvite,
        "m.call.candidates", CallCandidates,
        "m.call.answer", CallAnswer,
        "m.call.hangup", CallHangup
    }
}
