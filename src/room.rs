//! Abstraction for Matrix rooms.

use types::replies::*;
use types::messages::Message;
use super::{MatrixClient, MatrixFuture};
use request::MatrixRequest;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use futures::*;
use hyper::Method::*;

/// A Matrix room. This object is a thin wrapper over a room ID.
///
/// It's probably best to read the `RoomClient` documentation as well - after
/// all, that's how you do anything with this room.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Room<'a> {
    pub id: Cow<'a, str>
}
impl<'a> Serialize for Room<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.id)
    }
}
impl<'de> Deserialize<'de> for Room<'static> {
    fn deserialize<D>(de: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let id: String = Deserialize::deserialize(de)?;
        Ok(Room { id: id.into() })
    }
}
/// A `Room` with a `MatrixClient`, which you can use to call endpoints relating
/// to rooms.
pub struct RoomClient<'a, 'b: 'a, 'c> {
    pub room: &'a Room<'b>,
    pub cli: &'c mut MatrixClient
}
impl<'a> Room<'a> {
    /// Requests that the server resolve a room alias to a room (ID).
    ///
    /// The server will use the federation API to resolve the alias if the
    /// domain part of the alias does not correspond to the server's own domain.
    pub fn from_alias(cli: &mut MatrixClient, alias: &str) -> MatrixFuture<Self> {
        Box::new(MatrixRequest::new_basic(Get, format!("/directory/room/{}", alias))
                 .send(cli)
                 .map(|RoomAliasReply { room, .. }| room))
    }
    /// Make a `Room` object from a room ID.
    pub fn from_id<T: Into<Cow<'a, str>>>(id: T) -> Self {
        Room {
            id: id.into()
        }
    }
    /// Use a `MatrixClient` to make a `RoomClient`, an object used to call
    /// endpoints relating to rooms.
    ///
    /// If you want to do pretty much anything *with* this `Room`, you probably
    /// want to call this at some point.
    pub fn cli<'b, 'c>(&'b self, cli: &'c mut MatrixClient) -> RoomClient<'b, 'a, 'c> {
        RoomClient {
            room: self,
            cli
        }
    }
}
impl<'a, 'b, 'c> RoomClient<'a, 'b, 'c> {
    /// Sends a message to this room.
    pub fn send(&mut self, msg: Message) -> MatrixFuture<SendReply> {
        self.cli.txnid += 1;
        MatrixRequest::new_with_body_ser(
            Put,
            format!("/rooms/{}/send/m.room.message/{}",
                    self.room.id,
                    self.cli.txnid),
            msg
        ).send(self.cli)
    }
    /// Wrapper function that sends a `Message::Notice` with the specified unformatted text
    /// to this room. Provided for convenience purposes.
    pub fn send_simple<T: Into<String>>(&mut self, msg: T) -> MatrixFuture<SendReply> {
        let msg = Message::Notice {
            body: msg.into(),
            formatted_body: None,
            format: None
        };
        self.send(msg)
    }
    /// Wrapper function that sends a `Message::Notice` with the specified HTML-formatted text
    /// (and accompanying unformatted text, if given) to this room.
    pub fn send_html<T, U>(&mut self, msg: T, unformatted: U) -> MatrixFuture<SendReply>
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
    pub fn read_receipt(&mut self, eventid: &str) -> MatrixFuture<()> {
        MatrixRequest::new_basic(Post, format!("/rooms/{}/receipt/m.read/{}", self.room.id, eventid))
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
    pub fn get_state<T: DeserializeOwned + 'static>(&mut self, ev_type: &str, key: Option<&str>) -> MatrixFuture<T> {
        MatrixRequest::new_basic(Get, format!("/rooms/{}/state/{}/{}",
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
    pub fn set_state<T: Serialize>(&mut self, ev_type: &str, key: Option<&str>, val: T) -> MatrixFuture<SetStateReply> {
        MatrixRequest::new_with_body_ser(
            Put,
            format!("/rooms/{}/state/{}/{}",
                    self.room.id,
                    ev_type, key.unwrap_or("")),
            val
        ).send(self.cli)
    }
    /// Strips all information out of an event which isn't critical to the
    /// integrity of the server-side representation of the room.
    ///
    /// This cannot be undone.
    ///
    /// Users may redact their own events, and any user with a power level
    /// greater than or equal to the redact power level of the room may redact
    /// events there.
    pub fn redact(&mut self, eventid: &str, reason: Option<&str>) -> MatrixFuture<()> {
        self.cli.txnid += 1;
        let mut body = vec![];
        body.extend(reason.map(|x| ("reason", x)));
        MatrixRequest::new_with_body(Post, format!("/rooms/{}/redact/{}/{}",
                                                   self.room.id, eventid, self.cli.txnid),
                                     body)
            .discarding_send(self.cli)
    }
    /// This tells the server that the user is typing for the next N
    /// milliseconds where N is the value specified in the timeout key.
    /// Alternatively, if typing is false, it tells the server that the user has
    /// stopped typing.
    pub fn typing(&mut self, typing: bool, timeout: Option<usize>) -> MatrixFuture<()> {
        self.cli.txnid += 1;
        let mut body = vec![("typing", typing.to_string())];
        body.extend(timeout.map(|x| ("timeout", x.to_string())));
        MatrixRequest::new_with_body(Post, format!("/rooms/{}/typing/{}",
                                                   self.room.id, self.cli.user_id),
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
    pub fn join(&mut self) -> MatrixFuture<()> {
        MatrixRequest::new_basic(Post, format!("/rooms/{}/join", self.room.id))
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
    pub fn leave(&mut self) -> MatrixFuture<()> {
        MatrixRequest::new_basic(Post, format!("/rooms/{}/leave", self.room.id))
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
    pub fn forget(&mut self) -> MatrixFuture<()> {
        MatrixRequest::new_basic(Post, format!("/rooms/{}/forget", self.room.id))
            .discarding_send(self.cli)
    }
    /// Kick a user from the room.
    ///
    /// The caller must have the required power level in order to perform this
    /// operation.
    pub fn kick_user(&mut self, user_id: &str, reason: Option<&str>) -> MatrixFuture<()> {
        let mut body = vec![("user_id", user_id)];
        body.extend(reason.map(|x| ("reason", x)));
        MatrixRequest::new_with_body(Post, format!("/rooms/{}/kick", self.room.id),
                                     body)
            .discarding_send(self.cli)
    }
    /// Ban a user in the room. If the user is currently in the room, also kick them.
    ///
    /// When a user is banned from a room, they may not join it or be invited to
    /// it until they are unbanned.
    ///
    /// The caller must have the required power level in order to perform this operation.
    pub fn ban_user(&mut self, user_id: &str, reason: Option<&str>) -> MatrixFuture<()> {
        let mut body = vec![("user_id", user_id)];
        body.extend(reason.map(|x| ("reason", x)));
        MatrixRequest::new_with_body(Post, format!("/rooms/{}/ban", self.room.id),
                                     body)
            .discarding_send(self.cli)
    }
    /// Unban a user from the room. This allows them to be invited to the room,
    /// and join if they would otherwise be allowed to join according to its
    /// join rules.
    ///
    /// The caller must have the required power level in order to perform this
    /// operation.
    pub fn unban_user(&mut self, user_id: &str) -> MatrixFuture<()> {
        MatrixRequest::new_with_body(Post, format!("/rooms/{}/unban", self.room.id),
                                     vec![("user_id", user_id)])
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
    pub fn invite_user(&mut self, user_id: &str) -> MatrixFuture<()> {
        MatrixRequest::new_with_body(Post, format!("/rooms/{}/invite", self.room.id),
                                     vec![("user_id", user_id)])
            .discarding_send(self.cli)
    }
}
