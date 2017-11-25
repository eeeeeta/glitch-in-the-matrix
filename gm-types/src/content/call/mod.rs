//! Content types for `m.call.*` events.

pub mod types;
/// `m.call.invite`
///
/// This event is sent by the caller when they wish to establish a call.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Invite {
    /// A unique identifer for the call.
    call_id: String,
    /// The session description object
    offer: types::Offer,
    /// The version of the VoIP specification this message adheres to. This specification is version 0.
    version: i32,
    /// The time in milliseconds that the invite is valid for. Once the
    /// invite age exceeds this value, clients should discard it. They
    /// should also no longer show the call as awaiting an answer in the UI.
    lifetime: i32,
}
/// `m.call.candidates`
///
/// This event is sent by callers after sending an invite and by the callee
/// after answering. Its purpose is to give the other party additional ICE
/// candidates to try using to communicate.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Candidates {
    /// The ID of the call this event relates to.
    call_id: String,
    /// Array of objects describing the candidates.
    candidates: Vec<types::Candidate>,
    /// The version of the VoIP specification this messages adheres to. This specification is version 0.
    version: i32,
}
/// `m.call.answer`
///
/// This event is sent by the callee when they wish to answer the call.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Answer {
    /// The ID of the call this event relates to.
    call_id: String,
    /// The session description object
    answer: types::Answer,
    /// The version of the VoIP specification this message adheres to. This
    /// specification is version 0.
    version: i32,
}
/// `m.call.hangup`
///
/// Sent by either party to signal their termination of the call. This can be
/// sent either once the call has has been established or before to abort the
/// call.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Hangup {
    /// The ID of the call this event relates to.
    call_id: String,
    /// The version of the VoIP specification this message adheres to. This specification is version 0.
    version: i32,
}
