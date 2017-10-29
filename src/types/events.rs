//! Types for representing events.
//!
//! For event *content*, see the `content` module.
use super::content::{Content, deserialize_content};
use serde::*;
use errors::*;
use serde_json::Value;
use serde::de;
use room::Room;

/// The `unsigned` field in many event types.
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
    pub unsigned: UnsignedData
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
    pub unsigned: Option<UnsignedData>
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
    pub unsigned: Option<UnsignedData>,
    // state event
    pub state_key: Option<String>,
    pub prev_content: Option<Content>,
    pub prev_sender: Option<String>,
    pub invite_room_state: Option<Vec<MetaMinimal>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum EventMetadata {
    Full(MetaFull),
    Minimal(MetaMinimal),
    Redacted(MetaRedacted)
}
/// A Matrix event.
#[derive(Debug)]
pub enum Event {
    /// A full event.
    Full(MetaFull, Content),
    /// A minimal or ephemeral event.
    Minimal(MetaMinimal, Content),
    /// An event that has been redacted.
    Redacted(MetaRedacted),
    /// A full event where we couldn't deserialize the event content.
    FullError(MetaFull, MatrixError),
    /// A minimal event where we couldn't deserialize the event content.
    MinimalError(MetaMinimal, MatrixError)
}

fn parse_event_content(v: &Value) -> MatrixResult<Content> {
    let typ = v.get("type").ok_or("No event type field")?;
    let typ = match *typ {
        Value::String(ref s) => s as &str,
        _ => Err("Event type is not a string")?
    };
    let content = v.get("content").ok_or("No content field")?;
    let content = deserialize_content(typ, content.clone())?;
    Ok(content)
}
impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(d: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let v = Value::deserialize(d)?;
        let meta: EventMetadata = if v.pointer("/unsigned/redacted_because").is_some() {
            EventMetadata::Redacted(::serde_json::from_value(v.clone())
                                    .map_err(|e| de::Error::custom(e.to_string()))?)
        }
        else {
            ::serde_json::from_value(v.clone())
                .map_err(|e| de::Error::custom(e.to_string()))?
        };
        if let EventMetadata::Redacted(mr) = meta {
            return Ok(Event::Redacted(mr));
        }
        match parse_event_content(&v) {
            Ok(content) => Ok(match meta {
                EventMetadata::Full(m) => Event::Full(m, content),
                EventMetadata::Minimal(m) => Event::Minimal(m, content),
                _ => unreachable!()
            }),
            Err(e) => Ok(match meta {
                EventMetadata::Full(m) => Event::FullError(m, e),
                EventMetadata::Minimal(m) => Event::MinimalError(m, e),
                _ => unreachable!()
            })
        }

    }
}
/// Events in a room.
#[derive(Deserialize, Default, Debug)]
pub struct Events {
    pub events: Vec<Event>
}
