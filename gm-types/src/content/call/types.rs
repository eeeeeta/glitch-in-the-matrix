
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Offer {
    /// The type of session description. Must be 'offer'.
    #[serde(rename = "type")]
    session_type: String,
    /// The SDP text of the session description.
    sdp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Candidate {
    /// The SDP media type this candidate is intended for.
    #[serde(rename = "sdpMid")]
    sdp_m_id: String,
    /// The index of the SDP 'm' line this candidate is intended for.
    #[serde(rename = "sdpMLineIndex")]
    sdp_m_line_index: i32,
    /// The SDP 'a' line of the candidate.
    candidate: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Answer {
    /// The type of session description. Must be 'answer'.
    #[serde(rename = "type")]
    session_type: String,
    /// The SDP text of the session description.
    sdp: String,
}

