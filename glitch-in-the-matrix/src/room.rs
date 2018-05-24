//! Abstractions for Matrix rooms.

use types::replies::*;
use types::messages::Message;
use types::events::Event;
use types::content::room::PowerLevels;
use request::{MatrixRequestable, MatrixRequest};
use serde::Serialize;
use serde::de::DeserializeOwned;
use futures::*;
use errors::*;
use http::Method;

pub use types::room::Room;

/// Trait used to implement methods on `Room`.
///
/// This exists because `Room` is defined in another crate (`gm-types`), which
/// has no knowledge of `MatrixClient`. It contains most of the interesting
/// methods for `Room` - most notably, you can make a `RoomClient`, which is how
/// you actually do anything room-related.
pub trait RoomExt<'a> {
    /// Use an implementor of `MatrixRequestable` to make a `RoomClient`, an object used to call
    /// endpoints relating to rooms.
    ///
    /// If you want to do pretty much anything *with* this `Room`, you probably
    /// want to call this at some point.
    fn cli<'b, 'c, T>(&'b self, cli: &'c mut T) -> RoomClient<'b, 'a, 'c, T> where T: MatrixRequestable;
}
/// Contains endpoints relating to creating or joining rooms.
pub struct NewRoom;
impl NewRoom {
    /// Requests that the server resolve a room alias to a room (ID).
    ///
    /// The server will use the federation API to resolve the alias if the
    /// domain part of the alias does not correspond to the server's own domain.
    pub fn from_alias<R: MatrixRequestable>(cli: &mut R, alias: &str) -> impl Future<Item = Room<'static>, Error = MatrixError> {
        MatrixRequest::new_basic(Method::GET, format!("/directory/room/{}", alias))
            .send(cli)
            .map(|RoomAliasReply { room, .. }| room)
    }
    /// Joins a room by identifier or alias.
    pub fn join<R: MatrixRequestable>(cli: &mut R, room: &str) -> impl Future<Item = Room<'static>, Error = MatrixError> {
        MatrixRequest::new_basic(Method::POST, format!("/join/{}", room))
            .send(cli)
            .map(|JoinReply { room }| room)
    }
    /// Creates a room, with given options.
    pub fn create<R: MatrixRequestable>(cli: &mut R, opts: RoomCreationOptions) -> impl Future<Item = Room<'static>, Error = MatrixError> {
        MatrixRequest::new_with_body_ser(Method::POST, "/createRoom", opts)
            .send(cli)
            .map(|JoinReply { room }| room)
    }
}
/// A `Room` with a `MatrixRequestable` type, which you can use to call endpoints relating
/// to rooms.
pub struct RoomClient<'a, 'b: 'a, 'c, T: 'c> {
    /// A reference to a room.
    pub room: &'a Room<'b>,
    /// A reference to a `MatrixRequestable` type.
    pub cli: &'c mut T
}
impl<'a> RoomExt<'a> for Room<'a> {
   fn cli<'b, 'c, T>(&'b self, cli: &'c mut T) -> RoomClient<'b, 'a, 'c, T> where T: MatrixRequestable {
        RoomClient {
            room: self,
            cli
        }
    }
}
impl<'a, 'b, 'c, R> RoomClient<'a, 'b, 'c, R> where R: MatrixRequestable {
    /// Sends a message to this room.
    pub fn send(&mut self, msg: Message) -> impl Future<Item = SendReply, Error = MatrixError> {
        MatrixRequest::new_with_body_ser(
            Method::PUT,
            format!("/rooms/{}/send/m.room.message/{}",
                    self.room.id,
                    self.cli.get_txnid()),
            msg
        ).send(self.cli)
    }
    /// Wrapper function that sends a `Message::Notice` with the specified unformatted text
    /// to this room. Provided for convenience purposes.
    pub fn send_simple<T: Into<String>>(&mut self, msg: T) -> impl Future<Item = SendReply, Error = MatrixError> {
        let msg = Message::Notice {
            body: msg.into(),
            formatted_body: None,
            format: None
        };
        self.send(msg)
    }
    /// Wrapper function that sends a `Message::Notice` with the specified HTML-formatted text
    /// (and accompanying unformatted text, if given) to this room.
    pub fn send_html<T, U>(&mut self, msg: T, unformatted: U) -> impl Future<Item = SendReply, Error = MatrixError>
        where T: Into<String>, U: Into<Option<String>> {
        let m = msg.into();
        let msg = Message::Notice {
            body: unformatted.into().unwrap_or(m.clone()),
            formatted_body: Some(m),
            format: Some("org.matrix.custom.html".into())
        };
        self.send(msg)
    }
    /// Send a read receipt for a given event ID.
    pub fn read_receipt(&mut self, eventid: &str) -> impl Future<Item = (), Error = MatrixError> {
        MatrixRequest::new_basic(Method::POST, format!("/rooms/{}/receipt/m.read/{}", self.room.id, eventid))
            .discarding_send(self.cli)
    }
    /// Looks up the contents of a state event with type `ev_type` and state key
    /// `key` in a room. If the user is joined to the room then the state is
    /// taken from the current state of the room. If the user has left the room
    /// then the state is taken from the state of the room when they left.
    ///
    /// The return value here can be any object that implements `Deserialize`,
    /// allowing you to use the state API to store arbitrary objects. Common
    /// state events, such as `m.room.name`, can be found in the `content`
    /// module (`content::room::Name` for `m.room.name`).
    ///
    /// If the event was not found, an error will be thrown of type
    /// `HttpCode(http::StatusCode::NotFound)`.
    pub fn get_typed_state<T: DeserializeOwned + 'static>(&mut self, ev_type: &str, key: Option<&str>) -> impl Future<Item = T, Error = MatrixError> {
        MatrixRequest::new_basic(Method::GET, format!("/rooms/{}/state/{}/{}",
                                              self.room.id,
                                              ev_type,
                                              key.unwrap_or("")))
            .send(self.cli)
    }
    /// State events can be sent using this endpoint. These events will be
    /// overwritten if the <event type> (`ev_type`) and <state key> (`key`) all
    /// match.
    ///
    /// Like `get_state`, the value here can be any object that implements
    /// `Serialize`, allowing you to use the state API to store arbitrary
    /// objects. See the `get_state` docs for more.
    pub fn set_typed_state<T: Serialize>(&mut self, ev_type: &str, key: Option<&str>, val: T) -> impl Future<Item = SetStateReply, Error = MatrixError> {
        MatrixRequest::new_with_body_ser(
            Method::PUT,
            format!("/rooms/{}/state/{}/{}",
                    self.room.id,
                    ev_type, key.unwrap_or("")),
            val
        ).send(self.cli)
    }
    /// Get the state events for the current state of a room.
    pub fn get_all_state(&mut self) -> impl Future<Item = Vec<Event>, Error = MatrixError> {
        MatrixRequest::new_basic(Method::GET, format!("/rooms/{}/state", self.room.id))
            .send(self.cli)
    }
    /// Get a single event for this room, based on `event_id`.
    ///
    /// You must have permission to retrieve this event (e.g. by being a member of the room for
    /// this event)
    pub fn get_event(&mut self, id: &str) -> impl Future<Item = Event, Error = MatrixError> {
        MatrixRequest::new_basic(Method::GET, format!("/rooms/{}/event/{}", self.room.id, id))
            .send(self.cli)
    }
    /// Get the list of member events for this room.
    pub fn get_members(&mut self) -> impl Future<Item = ChunkReply, Error = MatrixError> {
        MatrixRequest::new_basic(Method::GET, format!("/rooms/{}/members", self.room.id))
            .send(self.cli)
    }
    /// Get the list of joined members in this room.
    ///
    /// The current user must be in the room for it to work, unless it is an
    /// Application Service, in which case any of the AS's users must be in the room.
    ///
    /// This API is primarily for Application Services and should be faster to
    /// respond than `get_members()`, as it can be implemented more efficiently
    /// on the server.
    pub fn get_joined_members(&mut self) -> impl Future<Item = JoinedMembersReply, Error = MatrixError> {
        MatrixRequest::new_basic(Method::GET, format!("/rooms/{}/joined_members", self.room.id))
            .send(self.cli)
    }
    /// This API returns a list of message and state events for a room. It uses
    /// pagination query parameters to paginate history in the room.
    ///
    /// ## Parameters
    ///
    /// - `from`: The token to start returning events from. This token can be
    ///   obtained from a `prev_batch` token returned for each room by the sync
    ///   API, or from a `start` or `end` token returned by a previous request to
    ///   this endpoint.
    /// - `to`: The token to stop returning events at. This token can be
    ///   obtained from a prev_batch token returned for each room by the sync
    ///   endpoint, or from a start or end token returned by a previous request to
    ///   this endpoint.
    /// - `backward`: Whether to paginate backward, or forward; true = back-pagination.
    /// - `limit`: The maximum number of events to return. (default: 10)
    pub fn get_messages(&mut self, from: &str, to: Option<&str>, backward: bool, limit: Option<u32>) -> impl Future<Item = MessagesReply, Error = MatrixError> {
        let mut req = MatrixRequest::new_basic(Method::GET, format!("/rooms/{}/messages", self.room.id));
        req.params.insert("from".into(), from.into());
        if let Some(to) = to {
            req.params.insert("to".into(), to.into());
        }
        let dir = if backward { "b" } else { "f" };
        req.params.insert("dir".into(), dir.into());
        if let Some(limit) = limit {
            req.params.insert("limit".into(), limit.to_string().into());
        }
        req.send(self.cli)
    }
    /// Strips all information out of an event which isn't critical to the
    /// integrity of the server-side representation of the room.
    ///
    /// This cannot be undone.
    ///
    /// Users may redact their own events, and any user with a power level
    /// greater than or equal to the redact power level of the room may redact
    /// events there.
    pub fn redact(&mut self, eventid: &str, reason: Option<&str>) -> impl Future<Item = (), Error = MatrixError> {
        let mut body = vec![];
        body.extend(reason.map(|x| ("reason", x.to_string())));
        MatrixRequest::new_with_body(Method::POST, format!("/rooms/{}/redact/{}/{}",
                                                   self.room.id, eventid, self.cli.get_txnid()),
                                     body)
            .discarding_send(self.cli)
    }
    /// This tells the server that the user is typing for the next N
    /// milliseconds where N is the value specified in the timeout key.
    /// Alternatively, if typing is false, it tells the server that the user has
    /// stopped typing.
    pub fn typing(&mut self, typing: bool, timeout: Option<usize>) -> impl Future<Item = (), Error = MatrixError> {
        let mut body = vec![("typing", typing.to_string())];
        body.extend(timeout.map(|x| ("timeout", x.to_string())));
        MatrixRequest::new_with_body(Method::POST, format!("/rooms/{}/typing/{}",
                                                   self.room.id, self.cli.get_user_id()),
                                     body)
            .discarding_send(self.cli)

    }
    /// This API starts a user participating in a particular room, if that user
    /// is allowed to participate in that room. After this call, the client is
    /// allowed to see all current state events in the room, and all subsequent
    /// events associated with the room until the user leaves the room.
    ///
    /// After a user has joined a room, the room will appear as an entry in the
    /// values in the `SyncStream`.
    pub fn join(&mut self) -> impl Future<Item = (), Error = MatrixError> {
        MatrixRequest::new_basic(Method::POST, format!("/rooms/{}/join", self.room.id))
            .discarding_send(self.cli)
    }
    /// This API stops a user participating in a particular room.
    ///
    /// If the user was already in the room, they will no longer be able to see
    /// new events in the room. If the room requires an invite to join, they
    /// will need to be re-invited before they can re-join.
    ///
    /// If the user was invited to the room, but had not joined, this call
    /// serves to reject the invite.
    ///
    /// The user will still be allowed to retrieve history from the room which
    /// they were previously allowed to see.
    pub fn leave(&mut self) -> impl Future<Item = (), Error = MatrixError> {
        MatrixRequest::new_basic(Method::POST, format!("/rooms/{}/leave", self.room.id))
            .discarding_send(self.cli)
    }
    /// This API stops a user remembering about a particular room.
    ///
    /// In general, history is a first class citizen in Matrix. After this API
    /// is called, however, a user will no longer be able to retrieve history
    /// for this room. If all users on a homeserver forget a room, the room is
    /// eligible for deletion from that homeserver.
    ///
    /// If the user is currently joined to the room, they will implicitly leave
    /// the room as part of this API call.
    pub fn forget(&mut self) -> impl Future<Item = (), Error = MatrixError> {
        MatrixRequest::new_basic(Method::POST, format!("/rooms/{}/forget", self.room.id))
            .discarding_send(self.cli)
    }
    /// Kick a user from the room.
    ///
    /// The caller must have the required power level in order to perform this
    /// operation.
    pub fn kick_user(&mut self, user_id: &str, reason: Option<&str>) -> impl Future<Item = (), Error = MatrixError> {
        let mut body = vec![("user_id", user_id.to_string())];
        body.extend(reason.map(|x| ("reason", x.to_string())));
        MatrixRequest::new_with_body(Method::POST, format!("/rooms/{}/kick", self.room.id),
                                     body)
            .discarding_send(self.cli)
    }
    /// Ban a user in the room. If the user is currently in the room, also kick them.
    ///
    /// When a user is banned from a room, they may not join it or be invited to
    /// it until they are unbanned.
    ///
    /// The caller must have the required power level in order to perform this operation.
    pub fn ban_user(&mut self, user_id: &str, reason: Option<&str>) -> impl Future<Item = (), Error = MatrixError> {
        let mut body = vec![("user_id", user_id.to_string())];
        body.extend(reason.map(|x| ("reason", x.to_string())));
        MatrixRequest::new_with_body(Method::POST, format!("/rooms/{}/ban", self.room.id),
                                     body)
            .discarding_send(self.cli)
    }
    /// Unban a user from the room. This allows them to be invited to the room,
    /// and join if they would otherwise be allowed to join according to its
    /// join rules.
    ///
    /// The caller must have the required power level in order to perform this
    /// operation.
    pub fn unban_user(&mut self, user_id: &str) -> impl Future<Item = (), Error = MatrixError> {
        MatrixRequest::new_with_body(Method::POST, format!("/rooms/{}/unban", self.room.id),
                                     vec![("user_id", user_id.to_string())])
            .discarding_send(self.cli)
    }
    /// This API invites a user to participate in a particular room. They do not
    /// start participating in the room until they actually join the room.
    ///
    /// Only users currently in a particular room can invite other users to join
    /// that room.
    ///
    /// If the user was invited to the room, the homeserver will append a
    /// m.room.member event to the room.
    ///
    /// Note that there are two forms of this API, which are documented
    /// separately. This version of the API requires that the inviter knows the
    /// Matrix identifier of the invitee. The other is documented in the third
    /// party invites section of the Matrix spec, and is not implemented in
    /// *Glitch in the Matrix* (yet!)
    pub fn invite_user(&mut self, user_id: &str) -> impl Future<Item = (), Error = MatrixError> {
        MatrixRequest::new_with_body(Method::POST, format!("/rooms/{}/invite", self.room.id),
                                     vec![("user_id", user_id.to_string())])
            .discarding_send(self.cli)
    }
    /// Get a user's power level, falling back on the default value for the room
    /// if not present.
    ///
    /// The `user_id` is a `String` here, not a `&str`, because it is stored in
    /// a future that outlives this function.
    pub fn get_user_power_level(&mut self, user_id: String) -> impl Future<Item = u32, Error = MatrixError> {
        let fut = self.get_typed_state::<PowerLevels>("m.room.power_levels", None);
        Box::new(fut.map(move |x| {
            if let Some(pl) = x.users.get(&user_id) {
                *pl
            }
            else {
                x.users_default
            }
        }).or_else(|e| {
            if let MatrixError::BadRequest(ref brk) = e {
                if brk.errcode == "M_NOT_FOUND" {
                    return Ok(0)
                }
            }
            Err(e)
        }))
    }
}
