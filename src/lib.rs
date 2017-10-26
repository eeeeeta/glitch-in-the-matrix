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
    #![allow(unused_doc_comment)]
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
pub mod room;
pub mod request;
pub mod sync;
mod util;

use util::*;
use errors::*;
use types::replies::*;
use types::content::root::types::Presence;
use hyper::{Method, Body};
use Method::*;
use hyper::client::{HttpConnector, Request};
use hyper_openssl::HttpsConnector;
use hyper::header::ContentType;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use tokio_core::reactor::Handle;
use futures::*;
use request::MatrixRequest;
use sync::SyncStream;

/// A `Future` with a `MatrixError` error type. Returned by most library
/// functions.
///
/// Yes, I know this is a `Box`, and that sucks a whoole bunch. I'm waiting
/// for `impl Trait` to arrive to save us from this madness.
pub type MatrixFuture<T> = Box<Future<Item=T, Error=MatrixError>>;

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
