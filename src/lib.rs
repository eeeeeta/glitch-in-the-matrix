//! **Glitch in the Matrix** is a set of (minimal) [Matrix](http://matrix.org/) bindings for Rust.
//! It has a number of limitations at present, and is not recommended for production use. Still,
//! it is provided in the hope that it might be useful.
//!
//! See the `examples/` subdirectory for a simple echo client example.
//!
//! Licensed under CC0.


extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate hyper;
extern crate hyper_openssl;
#[macro_use] extern crate error_chain;
extern crate tokio_core;
#[macro_use] extern crate futures;
extern crate percent_encoding;

pub mod errors {
    //! Error handling, using `error_chain`.
    error_chain! {
        types {
            MatrixError, MatrixErrorKind, ResultExt, MatrixResult;
        }
        foreign_links {
            Hyper(::hyper::error::Error);
            Serde(::serde_json::Error);
            UriError(::hyper::error::UriError);
            Io(::std::io::Error);
            Openssl(::hyper_openssl::openssl::error::ErrorStack);
        }
        errors {
            HttpCode(c: ::hyper::StatusCode) {
                display("HTTP error: {}", c.canonical_reason().unwrap_or("unknown"))
            }
            BadRequest(e: super::types::replies::BadRequestReply) {
                display("Bad request: {:?}", e)
            }
        }
    }
}
pub mod http {
    //! Types reexported from `hyper`.
    pub use hyper::Method;
    pub use hyper::Body;
    pub use hyper::header::ContentType;
}
pub mod types;
mod util;

use util::*;
use errors::*;
use types::replies::*;
use types::content::{Presence};
use types::messages::{Message};
use hyper::{Method, Body};
use Method::*;
use hyper::client::{HttpConnector, Request};
use hyper_openssl::HttpsConnector;
use hyper::header::ContentType;
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use serde::Serialize;
use std::collections::HashMap;
use tokio_core::reactor::Handle;
use futures::*;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

/// A `Future` with a `MatrixError` error type. Returned by most library
/// functions.
///
/// Yes, I know this is a `Box`, and that sucks a whoole bunch. I'm waiting
/// for `impl Trait` to arrive to save us from this madness.
pub type MatrixFuture<T> = Box<Future<Item=T, Error=MatrixError>>;

/// A `Stream` that yields constant replies to `/sync`.
///
/// This calls the long-polling `/sync` API, which will wait until replies come
/// in and send them to the client. If you want to reduce the wait time, use the
/// `set_timeout()` function.
pub struct SyncStream {
    hyper: hyper::Client<HttpsConnector<HttpConnector>>,
    last_batch: Option<String>,
    set_presence: bool,
    access_token: String,
    url: String,
    timeout: u64,
    cur_req: Option<MatrixFuture<SyncReply>>
}
impl SyncStream {
    /// Set whether polling the `/sync` API marks us as online.
    pub fn set_sync_sets_presence(&mut self, v: bool) {
        self.set_presence = v;
    }
    /// Ascertain whether polling the `/sync` API marks us as online.
    ///
    /// The default value is `true`; `/sync` sets presence.
    pub fn sync_sets_presence(&self) -> bool {
        self.set_presence
    }
    /// Get the current long-polling timeout.
    pub fn timeout(&self) -> u64 {
        self.timeout
    }
    /// Set a timeout (in milliseconds) for the server long-polling, after which
    /// the homeserver should return a blank reply instead of continuing to wait
    /// for new events.
    ///
    /// The default value is `30000` (30 seconds).
    ///
    /// This does not guard against other problems, such as connection loss;
    /// this merely *asks* the HS for a given timeout.
    pub fn set_timeout(&mut self, timeout: u64) {
        self.timeout = timeout;
    }
    fn req(&mut self) -> Request {
        let mut params = vec![];
        params.push(format!("set_presence={}", if self.set_presence {
            "online"
        } else { "offline" }));
        if let Some(ref b) = self.last_batch {
            params.push(format!("since={}", b));
            params.push(format!("timeout={}", self.timeout));
        }
        Request::new(Get, format!("{}/_matrix/client/r0/sync?access_token={}&{}",
                                  self.url,
                                  &self.access_token,
                                  params.join("&")
        ).parse().unwrap())
    }
}

impl Stream for SyncStream {
    type Item = SyncReply;
    type Error = MatrixError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if self.cur_req.is_some() {
                match self.cur_req.as_mut().unwrap().poll() {
                    Ok(Async::Ready(rpl)) => {
                        self.last_batch = Some(rpl.next_batch.clone());
                        self.cur_req = None;
                        return Ok(Async::Ready(Some(rpl)));
                    },
                    Ok(Async::NotReady) => {
                        return Ok(Async::NotReady);
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            let req = self.req();
            self.cur_req = Some(Box::new(self.hyper.request(req)
                                         .map_err(|e| e.into())
                                         .and_then(ResponseWrapper::<SyncReply>::wrap)))
        }
    }
}

/// A arbitrary request to an endpoint in the Matrix API.
///
/// This type has Super `Cow` Powers.
pub struct MatrixRequest<'a, T> {
    /// Request method (exported in the `http` module)
    pub meth: Method,
    /// API endpoint, without `/_matrix/client/r0` (e.g. `/sync`)
    pub endpoint: Cow<'a, str>,
    /// Query-string parameters.
    pub params: HashMap<Cow<'a, str>, Cow<'a, str>>,
    /// Request body (some type implementing `Serialize`).
    ///
    /// If this is empty (serialises to `{}`), it will not be sent. Therefore,
    /// requests with no body should use `()` here.
    pub body: T
}
impl<'a> MatrixRequest<'a, ()> {
    /// Convenience method for making a `MatrixRequest` from a method and
    /// endpoint.
    pub fn new_basic<S: Into<Cow<'a, str>>>(meth: Method, endpoint: S) -> Self {
        Self {
            meth,
            endpoint: endpoint.into(),
            params: HashMap::new(),
            body: ()
        }
    }
}

impl<'a, T> MatrixRequest<'a, T> where T: Serialize {
    fn body(&self) -> MatrixResult<Option<Body>> {
        let body = serde_json::to_string(&self.body)?;
        Ok(if body == "{}" {
            None
        }
        else {
            Some(body.into())
        })
    }
    /// Makes a hyper `Request` from this type.
    ///
    /// The generated `Request` can then be sent to some unsuspecting Matrix
    /// homeserver using the `send_request()` or `send_discarding_request()`
    /// methods on `MatrixClient`.
    pub fn make_hyper(&self, client: &MatrixClient) -> MatrixResult<Request> {
        let body = self.body()?;
        let mut params = format!("access_token={}", client.access_token);
        for (k, v) in self.params.iter() {
            params += &format!("&{}={}",
                              utf8_percent_encode(k.as_ref(), DEFAULT_ENCODE_SET),
                              utf8_percent_encode(v.as_ref(), DEFAULT_ENCODE_SET));
        }
        let url = format!("{}/_matrix/client/r0{}?{}",
                          client.url,
                          self.endpoint,
                          params);
        let mut req = Request::new(self.meth.clone(), url.parse()?);
        if let Some(b) = body {
            req.set_body(b);
        }
        Ok(req)
    }
    /// Sends this request to a Matrix homeserver, expecting a deserializable
    /// `R` return type.
    ///
    /// A helpful mix of `make_hyper()` and `MatrixClient::send_request()`.
    pub fn send<R>(&self, mxc: &mut MatrixClient) -> MatrixFuture<R> where R: DeserializeOwned + 'static {
        let req = match self.make_hyper(mxc) {
            Ok(r) => r,
            Err(e) => return Box::new(futures::future::err(e.into()))
        };
        mxc.send_request(req)
    }
    /// Like `send()`, but uses `MatrixClient::send_discarding_request()`.
    pub fn discarding_send(&self, mxc: &mut MatrixClient) -> MatrixFuture<()> {
        let req = match self.make_hyper(mxc) {
            Ok(r) => r,
            Err(e) => return Box::new(futures::future::err(e.into()))
        };
        mxc.send_discarding_request(req)
    }
    // incredibly useful and relevant method
    pub fn moo() -> &'static str {
        r#"(__)
         (oo)
   /------\/
  / |    ||
 *  /\---/\
    ~~   ~~
....Cow::Borrowed("Have you mooed today?")..."#
    }
}
/// A connection to a Matrix homeserver.
pub struct MatrixClient {
    hyper: hyper::Client<HttpsConnector<HttpConnector>>,
    access_token: String,
    hdl: Handle,
    user_id: String,
    url: String,
    txnid: u32
}
impl MatrixClient {
    /// Log in to a Matrix homeserver, and return a client object.
    pub fn login(username: &str, password: &str, url: &str, hdl: &Handle) -> MatrixFuture<Self> {
        let conn = match HttpsConnector::new(4, hdl) {
            Ok(c) => c,
            Err(e) => return Box::new(futures::future::err(e.into()))
        };
        let client = hyper::Client::configure()
            .connector(conn)
            .build(hdl);
        let uri: hyper::Uri = match format!("{}/_matrix/client/r0/login", url).parse() {
            Ok(u) => u,
            Err(e) => return Box::new(futures::future::err(e.into()))
        };
        let mut req = Request::new(Post, uri);
        req.set_body(json!({
            "type": "m.login.password",
            "user": username,
            "password": password
        }).to_string());
        let resp = client.request(req).map_err(|e| e.into()).and_then(ResponseWrapper::<LoginReply>::wrap);
        let hdl = hdl.clone();
        let url = url.to_string();
        Box::new(resp.map(move |rpl| {
            MatrixClient {
                hyper: client,
                access_token: rpl.access_token,
                user_id: rpl.user_id,
                url: url,
                hdl: hdl,
                txnid: 0
            }
        }))
    }
    /// Join a room by identifier or alias.
    pub fn join(&mut self, roomid: &str) -> MatrixFuture<JoinReply> {
        MatrixRequest::new_basic(Post, format!("/join/{}", roomid))
            .send(self)
    }
    /// Update our presence status.
    pub fn update_presence(&mut self, p: Presence) -> MatrixFuture<()> {
        MatrixRequest {
            meth: Put,
            endpoint: format!("/presence/{}/status", self.user_id).into(),
            params: HashMap::new(),
            body: json!({
                "presence": p
            })
        }.discarding_send(self)
    }
    /// Send a read receipt for a given event ID.
    pub fn read_receipt(&mut self, roomid: &str, eventid: &str) -> MatrixFuture<()> {
        MatrixRequest::new_basic(Post, format!("/rooms/{}/receipt/m.read/{}", roomid, eventid))
            .discarding_send(self)
    }
    /// Send a message to a room ID.
    pub fn send(&mut self, roomid: &str, msg: Message) -> MatrixFuture<SendReply> {
        self.txnid += 1;
        MatrixRequest {
            meth: Put,
            endpoint: format!("/rooms/{}/send/m.room.message/{}",
                              roomid,
                              self.txnid).into(),
            params: HashMap::new(),
            body: msg
        }.send(self)
    }
    /// Wrapper function that sends a `Message::Notice` with the specified unformatted text
    /// to the given room ID. Provided for convenience purposes.
    pub fn send_simple<T: Into<String>>(&mut self, roomid: &str, msg: T) -> MatrixFuture<SendReply> {
        let msg = Message::Notice {
            body: msg.into(),
            formatted_body: None,
            format: None
        };
        self.send(roomid, msg)
    }
    /// Wrapper function that sends a `Message::Notice` with the specified HTML-formatted text
    /// (and accompanying unformatted text, if given) to the given room ID.
    pub fn send_html<T: Into<String>, U: Into<Option<String>>>(&mut self, roomid: &str, msg: T, unformatted: U) -> MatrixFuture<SendReply> {
        let m = msg.into();
        let msg = Message::Notice {
            body: unformatted.into().unwrap_or(m.clone()),
            formatted_body: Some(m),
            format: Some("org.matrix.custom.html".into())
        };
        self.send(roomid, msg)
    }
    /// Upload some data (convertible to a `Body`) of a given `ContentType`, like an image.
    ///
    /// `Body` is accessible via the `http` module. See the documentation there
    /// for a complete reference of what implements `Into<Body>` - a quick
    /// shortlist: `Vec<u8>`, `&'static [u8]` (not `&'a [u8]`, sadly), `String`,
    /// `&'static str`.
    ///
    /// `ContentType` is accessible via the `http` module. See the documentation
    /// there for more information on how to use it.
    pub fn upload<T: Into<Body>>(&mut self, data: T, ct: ContentType) -> MatrixFuture<UploadReply> {
        let req = MatrixRequest {
            meth: Post,
            endpoint: format!("{}/_matrix/media/r0/upload?access_token={}",
                              self.url, self.access_token).into(),
            params: HashMap::new(),
            body: ()
        }.make_hyper(self);
        let mut req = match req {
            Ok(r) => r,
            Err(e) => return Box::new(futures::future::err(e.into()))
        };
        req.set_body(data.into());
        req.headers_mut().set(ct);
        self.send_request(req)
    }
    /// Get the client's MXID.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    /// Get a `SyncStream`, a `Stream` used to obtain replies to the `/sync`
    /// API.
    ///
    /// This `SyncStream` is independent from the original `MatrixClient`, and
    /// does not borrow from it in any way.
    pub fn get_sync_stream(&self) -> SyncStream {
        SyncStream {
            hyper: self.hyper.clone(),
            last_batch: None,
            set_presence: true,
            access_token: self.access_token.clone(),
            url: self.url.clone(),
            timeout: 30000,
            cur_req: None
        }
    }
    /// Sends an arbitrary `Request` to the Matrix homeserver, like one
    /// generated by `get_request_for()`.
    pub fn send_request<T>(&mut self, req: Request) -> MatrixFuture<T> where T: DeserializeOwned + 'static {
        Box::new(self.hyper.request(req)
                 .map_err(|e| e.into())
                 .and_then(ResponseWrapper::<T>::wrap))
    }
    /// Like `send_request()`, but discards the return value that the Matrix
    /// homeserver sends back.
    pub fn send_discarding_request(&mut self, req: Request) -> MatrixFuture<()> {
        Box::new(self.hyper.request(req)
                 .map_err(|e| e.into())
                 .and_then(UnitaryResponseWrapper::wrap))
    }
}

impl Drop for MatrixClient {
    /// Invalidates our access token, so we don't have millions of devices.
    /// Also sets us as offline.
    fn drop(&mut self) {
        let fut = MatrixRequest::new_basic(Post, "/logout")
            .discarding_send(self).map_err(|_| ()).map(|_| ());
        self.hdl.spawn(fut);
    }
}
