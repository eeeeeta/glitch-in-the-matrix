//! Types returned from the `/sync` endpoint.
use std::collections::HashMap;
use room::Room;
use events::{Event, Events};
use std::slice;

/// Counts of unread notifications for a room.
#[derive(Deserialize, Debug, Default)]
pub struct UnreadNotificationCounts {
    /// The number of unread notifications for this room with the highlight flag set.
    pub highlight_count: u32,
    /// The total number of unread notifications for this room.
    pub notification_count: u32
}
/// A timeline of messages and state changes in a room.
#[derive(Deserialize, Debug)]
pub struct Timeline {
    /// List of events.
    #[serde(default)]
    pub events: Vec<Event>,
    /// A token that can be supplied as the `from` parameter of the
    /// `/rooms/{roomId}/messages` endpoint.
    pub prev_batch: String,
    /// True if the number of events returned was limited by the `limit` on the
    /// filter.
    #[serde(default)]
    pub limited: bool
}
/// Information about a room the user has joined.
#[derive(Deserialize, Debug)]
pub struct JoinedRoom {
    /// Updates to the state, between the time indicated by the `since`
    /// parameter, and the start of the `timeline` (or all state up to the start
    /// of the `timeline`, if `since` is not given, or `full_state` is true).
    #[serde(default)]
    pub state: Events,
    /// The timeline of messages and state changes in the room.
    pub timeline: Timeline,
    /// The ephemeral events in the room that aren't recorded in the timeline or
    /// state of the room. e.g. typing.
    #[serde(default)]
    pub ephemeral: Events,
    /// The private data that this user has attached to this room.
    #[serde(default)]
    pub account_data: Events,
    /// Counts of unread notifications for this room.
    #[serde(default)]
    pub unread_notifications: UnreadNotificationCounts
}
/// Information about a room the user has left, or been banned from.
#[derive(Deserialize, Debug)]
pub struct LeftRoom {
    /// The state updates for the room up to the start of the timeline.
    #[serde(default)]
    pub state: Events,
    /// The timeline of messages and state changes in the room up to the point
    /// when the user left.
    pub timeline: Timeline
}
/// Information about a room the user has been invited to.
#[derive(Deserialize, Debug)]
pub struct InvitedRoom {
    /// The state of a room that the user has been invited to.
    ///
    /// These state events may only have the `sender`, `type`, `state_key` and
    /// `content` keys present. These events do not replace any state that the
    /// client already has for the room, for example if the client has archived
    /// the room. Instead the client should keep two separate copies of the
    /// state: the one from the `invite_state` and one from the archived state.
    /// If the client joins the room then the current state will be given as a
    /// delta against the archived state not the `invite_state`.
    #[serde(default)]
    pub invite_state: Events
}
/// The `rooms` component of a `SyncReply`.
#[derive(Deserialize, Debug)]
pub struct SyncRooms {
    /// The rooms that the user has joined.
    #[serde(default)]
    pub join: HashMap<Room<'static>, JoinedRoom>,
    /// The rooms that the user has been invited to.
    #[serde(default)]
    pub invite: HashMap<Room<'static>, InvitedRoom>,
    /// The rooms that the user has left, or been banned from.
    #[serde(default)]
    pub leave: HashMap<Room<'static>, LeftRoom>
}
/// The reply obtained from `/sync`.
#[derive(Deserialize, Debug)]
pub struct SyncReply {
    /// The batch token to supply in the `since` param of the next `/sync`
    /// request.
    pub next_batch: String,
    /// Updates to rooms.
    pub rooms: SyncRooms,
    /// The global private data created by this user.
    #[serde(default)]
    pub account_data: Events,
    /// The updates to the presence status of other users.
    #[serde(default)]
    pub presence: Events
}
impl SyncReply {
    /// Iterate over events in this reply.
    pub fn iter_events(&self) -> SyncEventIter {
        let mut rooms = vec![];
        for (id, room) in self.rooms.join.iter() {
            rooms.push((id, room.timeline.events.iter()));
        }
        SyncEventIter { rooms }
    }
}
/// Iterator over events in a `/sync` reply.
pub struct SyncEventIter<'a> {
    rooms: Vec<(&'a Room<'static>, slice::Iter<'a, Event>)>
}
impl<'a> Iterator for SyncEventIter<'a> {
    type Item = (&'a Room<'static>, &'a Event);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(&mut (room, ref mut iter)) = self.rooms.get_mut(0) {
                if let Some(evt) = iter.next() {
                    return Some((room, evt));
                }
            }
            else {
                return None;
            }
            self.rooms.remove(0);
        }
    }
}
