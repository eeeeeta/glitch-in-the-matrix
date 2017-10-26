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
    /// The user has been invited to join a room, but has not yet joined it.
    /// They may not participate in the room until they join.
    Invite,
    /// The user has joined the room (possibly after accepting an invite), and
    /// may participate in it.
    Join,
    /// The user was once joined to the room, but has since left (possibly by
    /// choice, or possibly by being kicked).
    Leave,
    /// The user has been banned from the room, and is no longer allowed to join
    /// it until they are un-banned from the room (by having their membership
    /// state set to a value other than ban).
    Ban,
    // reserved word
    // Knock,
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
