
/// Information about an image.
#[derive(Serialize, Deserialize, Debug)]
pub struct ImageInfo {
    /// The height of the image in pixels.
    pub h: u32,
    /// MIME type
    pub mimetype: String,
    /// Size, in bytes
    pub size: u32,
    /// The width of the image in pixels.
    pub w: u32,
}

/// Information about a file.
#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    /// MIME type
    pub mimetype: String,
    /// Size, in bytes
    pub size: u32
}

#[derive(Serialize, Deserialize, Debug)]
/// This message represents a single image and an optional thumbnail.
/// Information about an image and it's thumbnail.
pub struct Image {
    /// A textual representation of the image. This could be the alt text of the image,
    /// the filename of the image, or some kind of content description for accessibility
    /// e.g. 'image attachment'.
    pub body: String,
    /// must be m.image
    pub msgtype: String,
    /// The URL to the image.
    pub url: String,
    /// Metadata about the image referred to in url.
    pub info: Option<ImageInfo>,
    /// The URL to the thumbnail of the image.
    pub thumbnail_url: Option<String>,
    /// Metadata about the image referred to in thumbnail_url.
    pub thumbnail_info: Option<ImageInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Offer {
    /// The type of session description. Must be 'offer'.
    session_type: String,
    /// The SDP text of the session description.
    sdp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Candidate {
    /// Required. The SDP media type this candidate is intended for.
    sdp_m_id: String,
    /// Required. The index of the SDP 'm' line this candidate is intended for.
    sdp_m_line_index: i32,
    /// Required. The SDP 'a' line of the candidate.
    candidate: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Answer {
    // Required. The type of session description. Must be 'answer'.
    session_type: String,
    // Required. The SDP text of the session description.
    sdp: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
/// A message sent to a room.
pub enum Message {
    #[serde(rename="m.text")]
    /// This message is the most basic message and is used to represent text.
    Text{
        /// The body of the message.
        body: String,
        /// must be m.text
        msgtype: String,
        /// The formatted body of the message (if the message is formatted).
        #[serde(default)]
        formatted_body: Option<String>,
        /// The format of the formatted body (if the message is formatted).
        #[serde(default)]
        format: Option<String>
    },
    #[serde(rename="m.notice")]
    /// A m.notice message should be considered similar to a plain m.text message except
    /// that clients should visually distinguish it in some way.
    /// It is intended to be used by automated clients, such as bots, bridges, and other
    /// entities, rather than humans. Additionally, such automated agents which watch a
    /// room for messages and respond to them ought to ignore m.notice messages. This
    /// helps to prevent infinite-loop situations where two automated clients continuously
    /// exchange messages, as each responds to the other.
    Notice{
        /// The notice text to send.
        body: String,
        /// must be m.notice
        msgtype: String,
        /// The formatted body of the message (if the message is formatted).
        #[serde(default)]
        formatted_body: Option<String>,
        /// The format of the formatted body (if the message is formatted).
        #[serde(default)]
        format: Option<String>
    },
    #[serde(rename="m.image")]
    /// This message represents a single image and an optional thumbnail.
    /// Information about an image and it's thumbnail.
    Image{
        /// A textual representation of the image. This could be the alt text of the image,
        /// the filename of the image, or some kind of content description for accessibility
        /// e.g. 'image attachment'.
        body: String,
        /// must be m.image
        msgtype: String,
        /// The URL to the image.
        url: String,
        /// Metadata about the image referred to in url.
        info: Option<ImageInfo>,
        /// The URL to the thumbnail of the image.
        thumbnail_url: Option<String>,
        /// Metadata about the image referred to in thumbnail_url.
        thumbnail_info: Option<ImageInfo>,
    },
    #[serde(rename="m.emote")]
    /// This message is similar to m.text except that the sender is 'performing' the action
    /// contained in the body key, similar to /me in IRC. This message should be prefixed by the
    /// name of the sender. This message could also be represented in a different colour to
    /// distinguish it from regular m.text messages.
    Emote{
        /// The emote action to perform.
        body: String,
        /// must be m.emote
        msgtype: String,
    },
    #[serde(rename="m.file")]
    /// This message represents a generic file.
    File{
        /// A human-readable description of the file. This is recommended to be the filename
        /// of the original upload.
        body: String,
        /// must be m.string
        msgtype: String,
        /// The original filename of the uploaded file.
        filename: String,
        /// Information about the file referred to in url.
        info: Option<FileInfo>,
        /// Metadata about the image referred to in thumbnail_url.
        thumbnail_info: Option<ImageInfo>,
        /// The URL to the thumbnail of the file.
        thumbnail_url: Option<String>,
        /// The URL to the file.
        url: String
    },
    #[serde(rename="m.location")]
    /// This message represents a real-world location.
    /// A description of the location e.g. 'Big Ben, London, UK', or some kind of content
    Location{
        /// description for accessibility e.g. 'location attachment'.
        body: String,
        /// must be m.location
        msgtype: String,
        /// A geo URI representing this location.
        geo_uri: String
    },
    #[serde(rename="m.call.invite")]
    /// This event is sent by the caller when they wish to establish a call.
    CallInvite {
        /// Required. A unique identifer for the call.
        call_id: String,
        /// Required. The session description object
        offer: Offer,
        /// Required. The version of the VoIP specification this message adheres to. This specification is version 0.
        version: i32,
        /// Required. The time in milliseconds that the invite is valid for. Once the invite age exceeds this value, clients should discard it. They should also no longer show the call as awaiting an answer in the UI.
        lifetime: i32,
    },
    #[serde(rename="m.call.candidates")]
    /// This event is sent by callers after sending an invite and by the callee after answering. Its purpose is to give the other party additional ICE candidates to try using to communicate.
    CallCandidates {
        ///Required. The ID of the call this event relates to.
        call_id: String,
        /// Required. Array of objects describing the candidates.
        candidates: Vec<Candidate>,
        /// Required. The version of the VoIP specification this messages adheres to. This specification is version 0.
        version: i32,
    },
    #[serde(rename="m.call.answer")]
    /// This event is sent by the callee when they wish to answer the call.
    CallAnswer {
        /// Required. The ID of the call this event relates to.
        call_id: String,
        /// Required. The session description object
        answer: Answer,
        /// Required. The version of the VoIP specification this message adheres to. This specification is version 0.
        version: i32,
    },
    #[serde(rename="m.call.hangup")]
    /// Sent by either party to signal their termination of the call. This can be sent either once the call has has been established or before to abort the call.
    CallHangup {
        /// Required. The ID of the call this event relates to.
        call_id: String,
        /// Required. The version of the VoIP specification this message adheres to. This specification is version 0.
        version: i32,
    },
}

































//
