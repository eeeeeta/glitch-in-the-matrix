//! Types for representing events.
//!
//! For event *content*, see the `content` module.
use super::content::{Content, deserialize_content};
use serde::*;
use serde_json::Value;
use serde::de;
use room::Room;

/// Contains optional extra information about the event.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnsignedData {
    /// The time in milliseconds that has elapsed since the event was sent.
    ///
    /// This field is generated by the local homeserver, and may be incorrect if the local time on at
    /// least one of the two servers is out of sync, which can cause the age to either be negative or
    /// greater than it actually is.
    pub age: u64,
    /// The event that recated this event, if any.
    pub redacted_because: Option<Box<Event>>,
    /// The client-supplied transaction ID, if the client being given the event is the same one which sent it.
    pub transaction_id: Option<String>
}
/// Additional fields, specific to a Room Event.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomEventData {
    /// The globally unique event identifier.
    pub event_id: String,
    /// The room associated with this event.
    #[serde(rename = "room_id")]
    pub room: Room<'static>,
    /// The fully-qualified user ID of the user who sent this event.
    pub sender: String,
    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: u64,
    /// Optional extra information.
    pub unsigned: Option<UnsignedData>
}
/// Additional fields, specific to a State Event.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StateEventData {
    /// The previous `content` for this event.
    ///
    /// If there is no previous content, this key will be missing.
    pub prev_content: Option<Content>,
    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This value is often a zero-length string. The presence of this key makes this event a State Event.
    /// The key MUST NOT start with '_'.
    pub state_key: String
}
/// A Matrix event.
///
/// All communication in Matrix is expressed in the form of data objects called events.
/// These are the fundamental building blocks common to the client-server, server-server and 
/// application-service APIs.
#[derive(Serialize, Clone, Debug)]
pub struct Event {
    /// Event content.
    pub content: Content,
    #[serde(rename = "type")]
    /// The type of this event.
    ///
    /// This SHOULD be namespaced similar to Java package naming conventions,
    /// e.g. 'com.example.subdomain.event.type'
    pub event_type: String,
    /// If this event is a Room Event, this struct stores Room Event-specific fields.
    pub room_data: Option<RoomEventData>,
    /// If this event is a State Event, this struct stores Room Event-specific fields.
    pub state_data: Option<StateEventData>
}
impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(d: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let v = Value::deserialize(d)?;
        let typ = v.get("type").ok_or(de::Error::custom("No event type field"))?;
        let typ = match *typ {
            Value::String(ref s) => s as &str,
            _ => Err(de::Error::custom("Event type is not a string"))?
        };
        let c = v.get("content").ok_or(de::Error::custom("No content field"))?;
        let content = deserialize_content(typ, c.clone());
        let room_data: Option<RoomEventData> = ::serde_json::from_value(v.clone()).ok();
        let state_data: Option<StateEventData> = ::serde_json::from_value(v.clone()).ok();
        Ok(Event {
            event_type: typ.into(),
            content,
            room_data,
            state_data
        })
    }
}
/// Events in a room.
#[derive(Deserialize, Default, Debug)]
pub struct Events {
    /// A list of events.
    pub events: Vec<Event>
}
