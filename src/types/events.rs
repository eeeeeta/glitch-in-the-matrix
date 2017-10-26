//! Types for representing events.
//!
//! For event *content*, see the `content` module.
use super::content::{Content, deserialize_content};
use serde::*;
use serde_json::Value;
use serde::de;
use room::Room;

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsignedData {
    pub age: u64,
    pub prev_content: Option<Content>,
    pub prev_sender: Option<String>,
    pub txn_id: Option<String>,
    pub redacted_because: Option<::serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
/// Metadata for a redacted event.
pub struct MetaRedacted {
    #[serde(rename="type")]
    pub event_type: String,
    pub prev_sender: Option<String>,
    pub prev_content: Option<Content>,
    pub event_id: Option<String>,
    #[serde(rename = "room_id")]
    pub room: Option<Room<'static>>,
    pub sender: Option<String>,
    pub redacted_because: MetaFull
}

#[derive(Serialize, Deserialize, Debug)]
/// Metadata for an ephemeral event (like m.typing).
pub struct MetaMinimal {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "room_id")]
    pub room: Option<Room<'static>>,
    pub event_id: Option<String>,
    pub sender: Option<String>,
    pub state_key: Option<String>,
}

/// An event in a room.
#[derive(Serialize, Deserialize, Debug)]
pub struct MetaFull {
    // event
    #[serde(rename="type")]
    pub event_type: String,
    // room event
    pub event_id: String,
    pub sender: String,
    pub origin_server_ts: u64,
    #[serde(rename = "room_id")]
    pub room: Option<Room<'static>>,
    // can be recursive until we differ between redacted and not redacted events
    pub unsigned: Option<UnsignedData>,
    // state event
    pub state_key: Option<String>,
    pub prev_content: Option<Content>,
    pub prev_sender: Option<String>,
    pub invite_room_state: Option<Vec<MetaMinimal>>,
    // extra
    pub age: Option<u64>,
    pub txn_id: Option<String>,
    pub redacts: Option<String>,
    pub membership: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EventMetadata {
    Full(MetaFull),
    Redacted(MetaRedacted),
    Minimal(MetaMinimal)
}
#[derive(Debug)]
pub struct Event(pub EventMetadata, pub Content);

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(d: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let v = Value::deserialize(d)?;
        let content = {
            let typ = v.get("type").ok_or(de::Error::custom("No event type field"))?;
            let typ = match *typ {
                Value::String(ref s) => s as &str,
                _ => Err(de::Error::custom("Event type is not a string"))?
            };
            let content = v.get("content").ok_or(de::Error::custom("No content field"))?;
            let content = deserialize_content(typ, content.clone())
                .map_err(|e| de::Error::custom(e.to_string()))?;
            content
        };
        let meta: EventMetadata = ::serde_json::from_value(v)
            .map_err(|e| de::Error::custom(e.to_string()))?;
        Ok(Event(meta, content))
    }
}
/// Events in a room.
#[derive(Deserialize, Default, Debug)]
pub struct Events {
    pub events: Vec<Event>
}
