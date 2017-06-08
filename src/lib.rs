//! **Glitch in the Matrix** is a set of (minimal) [Matrix](http://matrix.org/) bindings for Rust.
//! It has a number of limitations at present, and is not recommended for production use. Still,
//! it is provided in the hope that it might be useful.
//!
//! Licensed under the [Unlicense](http://unlicense.org/).


extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate hyper;
extern crate hyper_native_tls;
#[macro_use] extern crate error_chain;

pub mod errors {
    error_chain! {
        types {
            MatrixError, MatrixErrorKind, ResultExt, MatrixResult;
        }
        foreign_links {
            Hyper(::hyper::error::Error);
            Serde(::serde_json::Error);
            Io(::std::io::Error);
            Ssl(::hyper_native_tls::native_tls::Error);
        }
        errors {
            HttpCode(c: ::hyper::status::StatusCode) {
                display("HTTP error: {}", c.canonical_reason().unwrap_or("unknown"))
            }
            BadRequest(e: super::types::BadRequestReply) {
                display("Bad request: {:?}", e)
            }
        }
    }
}
pub mod http {
    pub use hyper::method::Method;
    pub use hyper::header::ContentType;
}
pub mod types;

use errors::*;
use errors::MatrixErrorKind::*;
use types::*;
use hyper::method::Method;
use Method::*;
use hyper::client::{Response, RequestBuilder};
use hyper::header::ContentType;
use hyper::net::HttpsConnector;
use std::io::Read;
use hyper_native_tls::NativeTlsClient;

/// A connection to a Matrix homeserver.
pub struct MatrixClient {
    hyper: hyper::Client,
    access_token: String,
    user_id: String,
    url: String,
    last_batch: Option<String>,
    set_presence: bool,
    txnid: u32
}
impl MatrixClient {
    /// Log in to a Matrix homeserver, and return a client object.
    pub fn login(username: &str, password: &str, url: &str) -> MatrixResult<Self> {
        let ssl = NativeTlsClient::new()?;
        let conn = HttpsConnector::new(ssl);
        let client = hyper::Client::with_connector(conn);
        let mut resp = client.post(&format!("{}/_matrix/client/r0/login", url))
            .body(&json!({
                "type": "m.login.password",
                "user": username,
                "password": password
            }).to_string())
            .send()?;
        Self::handle_errs(&mut resp)?;
        let rpl = serde_json::from_reader::<_, LoginReply>(resp)?;
        Ok(MatrixClient {
            hyper: client,
            access_token: rpl.access_token,
            user_id: rpl.user_id,
            url: url.to_string(),
            last_batch: None,
            set_presence: true,
            txnid: 0
        })
    }
    fn handle_errs(resp: &mut Response) -> MatrixResult<()> {
        if !resp.status.is_success() {
            let st = resp.status.clone();
            if let Ok(e) = serde_json::from_reader::<_, BadRequestReply>(resp) {
                bail!(BadRequest(e));
            }
            else {
                bail!(HttpCode(st));
            }
        }
        Ok(())
    }
    /// Join a room by identifier or alias.
    pub fn join(&mut self, roomid: &str) -> MatrixResult<JoinReply> {
        let mut resp = self.req(Post, &format!("/join/{}", roomid), vec![])
            .send()?;
        Self::handle_errs(&mut resp)?;
        let rpl = serde_json::from_reader(resp)?;
        Ok(rpl)
    }
    /// Update our presence status.
    pub fn update_presence(&mut self, p: Presence) -> MatrixResult<()> {
        let uri = format!("/presence/{}/status", self.user_id);
        let mut resp = self.req(Put, &uri, vec![])
            .body(&serde_json::to_string(&p)?)
            .send()?;
        Self::handle_errs(&mut resp)?;
        Ok(())
    }
    /// Send a read receipt for a given event ID.
    pub fn read_receipt(&mut self, roomid: &str, eventid: &str) -> MatrixResult<()> {
        let uri = format!("/rooms/{}/receipt/m.read/{}", roomid, eventid);
        let mut resp = self.req(Post, &uri, vec![])
            .send()?;
        Self::handle_errs(&mut resp)?;
        Ok(())
    }
    /// Send a message to a room ID.
    pub fn send(&mut self, roomid: &str, msg: Message) -> MatrixResult<SendReply> {
        let body = serde_json::to_value(&msg)?;
        let uri = format!("/rooms/{}/send/m.room.message/{}",
                          roomid,
                          self.txnid);
        self.txnid += 1;
        let mut resp = self.req(Put, &uri, vec![])
            .body(&body.to_string())
            .send()?;
        Self::handle_errs(&mut resp)?;
        let rpl = serde_json::from_reader(resp)?;
        Ok(rpl)
    }
    /// Wrapper function that sends a `Message::Notice` with the specified unformatted text
    /// to the given room ID. Provided for convenience purposes.
    pub fn send_simple<T: Into<String>>(&mut self, roomid: &str, msg: T) -> MatrixResult<SendReply> {
        let msg = Message::Notice { body: msg.into(), formatted_body: None, format: None };
        self.send(roomid, msg)
    }
    /// Wrapper function that sends a `Message::Notice` with the specified HTML-formatted text
    /// (and accompanying unformatted text, if given) to the given room ID.
    pub fn send_html<T: Into<String>, U: Into<Option<String>>>(&mut self, roomid: &str, msg: T, unformatted: U) -> MatrixResult<SendReply> {
        let m = msg.into();
        let msg = Message::Notice { body: unformatted.into().unwrap_or(m.clone()), formatted_body: Some(m), format: Some("org.matrix.custom.html".into()) };
        self.send(roomid, msg)
    }
    pub fn upload<T: Read>(&mut self, data: &mut T, ct: ContentType) -> MatrixResult<UploadReply> {
        let req = self.hyper.request(Post, &format!("{}/_matrix/media/r0/upload?access_token={}",
                                                    self.url, self.access_token))
            .header(ct)
            .body(data);
        let mut resp = req.send()?;
        Self::handle_errs(&mut resp)?;
        let rpl = serde_json::from_reader(resp)?;
        Ok(rpl)
    }
    /// Get the client's MXID.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    /// Set whether polling the `sync` API marks us as online.
    pub fn set_set_presence(&mut self, v: bool) {
        self.set_presence = v;
    }
    /// Ascertain whether polling the `sync` API marks us as online.
    pub fn set_presence(&self) -> bool {
        self.set_presence
    }
    /// Poll the Matrix server for new events since the last call to `sync()`.
    ///
    /// It is recommended to call this in a loop. The Matrix server will block
    /// until new events arrive, up to a given timeout value.
    pub fn sync(&mut self, timeout: u64) -> MatrixResult<SyncReply> {
        let mut params = vec![];
        params.push(format!("set_presence={}", if self.set_presence {
            "online"
        } else { "offline" }));
        if let Some(ref b) = self.last_batch {
            params.push(format!("since={}", b));
            params.push(format!("timeout={}", timeout));
        }
        let mut resp = self.req(Get, "/sync", params).send()?;
        Self::handle_errs(&mut resp)?;
        let rpl = serde_json::from_reader::<_, SyncReply>(resp)?;
        self.last_batch = Some(rpl.next_batch.clone());
        Ok(rpl)
    }
    /// Make an arbitrary HTTP request to the Matrix API.
    ///
    /// - `/_matrix/client/r0` is filled in for you, so your `endpoint` is something like `/sync`.
    /// - `params` is a list of `key=value` HTTP parameters, like `timeout=30`.
    ///
    /// Returns a `RequestBuilder` which you can use for your own nefarious ends.
    pub fn req(&mut self, meth: Method, endpoint: &str, params: Vec<String>) -> RequestBuilder {
        self.hyper.request(meth,
                           &format!("{}/_matrix/client/r0{}?access_token={}{}{}",
                                    self.url,
                                    endpoint,
                                    &self.access_token,
                                    if params.len() == 0 { "" } else { "&" },
                                    params.join("&")
                           ))
    }
}
impl Drop for MatrixClient {
    /// Invalidates our access token, so we don't have millions of devices.
    /// Also sets us as offline.
    fn drop(&mut self) {
        let _ = self.update_presence(Presence::Offline);
        let _ = self.req(Post, "/logout", vec![]).send();
    }
}
