//! Values for the `m.room.message` event's content.

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
/// Information about an audio clip.
#[derive(Serialize, Deserialize, Debug)]
pub struct AudioInfo {
    ///	The duration of the audio in milliseconds.
    pub duration: u32,
    /// The mimetype of the audio e.g. `audio/aac`.
    pub mimetype: String,
    /// The size of the audio clip in bytes.
    pub size: u32
}
#[derive(Serialize, Deserialize, Debug)]
pub struct VideoInfo {
    /// The duration of the video in milliseconds.
    pub duration: u32,
    /// The height of the video in pixels.
    pub h: u32,
    /// The width of the video in pixels.
    pub w: u32,
    /// The mimetype of the video e.g. `video/mp4`.
    pub mimetype: String,
    /// The size of the video in bytes.
    pub size: u32,
    /// The URL to an image thumbnail of the video clip.
    pub thumbnail_url: String,
    /// Metadata about the image referred to in thumbnail_url.
    pub thumbnail_info: ImageInfo
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag="msgtype")]
/// A message sent to a room.
pub enum Message {
    #[serde(rename="m.text")]
    /// This message is the most basic message and is used to represent text.
    Text{
        /// The body of the message.
        body: String,
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
    /// This message is similar to m.text except that the sender is 'performing'
    /// the action contained in the body key, similar to /me in IRC. This
    /// message should be prefixed by the name of the sender. This message could
    /// also be represented in a different colour to distinguish it from regular
    /// m.text messages.
    Emote{
        /// The emote action to perform.
        body: String,
    },
    #[serde(rename="m.file")]
    /// This message represents a generic file.
    File {
        /// A human-readable description of the file. This is recommended to be
        /// the filename of the original upload.
        body: String,
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
    Location {
        /// A description of the location e.g. 'Big Ben, London, UK', or some
        /// kind of content description for accessibility e.g. 'location
        /// attachment'.
        body: String,
        /// A geo URI representing this location.
        geo_uri: String
    },
    #[serde(rename="m.audio")]
    /// This message represents a single audio clip.
    Audio {
        /// A description of the audio e.g. 'Bee Gees - Stayin' Alive', or some
        /// kind of content description for accessibility e.g. 'audio
        /// attachment'.
        body: String,
        /// The URL to the audio clip.
        url: String,
        /// Metadata for the audio clip referred to in url.
        info: Option<AudioInfo>
    },
    #[serde(rename="m.video")]
    /// This message represents a single video clip.
    Video {
        /// A description of the video e.g. 'Gangnam style', or some kind of
        /// content description for accessibility e.g. 'video attachment'.
        body: String,
        /// The URL to the video clip.
        url: String,
        /// Metadata about the video clip referred to in url.
        info: Option<VideoInfo>
    }
}
