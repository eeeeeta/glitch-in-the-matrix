//! **Glitch in the Matrix** is a set of (minimal) [Matrix](http://matrix.org/) bindings for Rust.
//! It has a number of limitations at present, and is not recommended for production use. Still,
//! it is provided in the hope that it might be useful.
//!
//! This crate re-exports a subcrate, `gm-types`, which contains most of the API
//! types used for deserialization - mostly to reduce compile times.
//!
//! See the `examples/` subdirectory for a simple echo client example.
//!
//! Licensed under CC0.


extern crate serde;
#[macro_use] extern crate serde_json;
extern crate hyper;
extern crate hyper_openssl;
#[macro_use] extern crate error_chain;
extern crate tokio_core;
#[macro_use] extern crate futures;
extern crate percent_encoding;
pub extern crate gm_types as types;

pub mod errors {
    #![allow(unused_doc_comment)]
    //! Error handling, using `error_chain`.
    //!
    //! The `StatusCode` enum is re-exported in the `http` module.
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
    pub use hyper::StatusCode;
    pub use hyper::Client;
    pub use hyper_openssl::HttpsConnector;
    pub use hyper::client::HttpConnector;
    pub type MatrixHyper = Client<HttpsConnector<HttpConnector>>;
}
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
use hyper::client::Request;
use hyper_openssl::HttpsConnector;
use hyper::header::ContentType;
use serde::de::DeserializeOwned;
use tokio_core::reactor::Handle;
use futures::*;
use request::MatrixRequest;
use sync::SyncStream;
use std::collections::HashMap;

/// A `Future` with a `MatrixError` error type. Returned by most library
/// functions.
///
/// Yes, I know this is a `Box`, and that sucks a whoole bunch. I'm waiting
/// for `impl Trait` to arrive to save us from this madness.
pub type MatrixFuture<T> = Box<Future<Item=T, Error=MatrixError>>;

/// A connection to a Matrix homeserver.
pub struct MatrixClient {
    hyper: http::MatrixHyper,
    access_token: String,
    hdl: Handle,
    user_id: String,
    url: String,
    is_as: bool,
    txnid: u32
}
impl MatrixClient {
    pub fn as_register_user(&mut self, user_id: String) -> MatrixFuture<()> {
        let uri: hyper::Uri = match format!("{}/_matrix/client/r0/register?access_token={}", self.url, self.access_token).parse() {
            Ok(u) => u,
            Err(e) => return Box::new(futures::future::err(e.into()))
        };
        let mut req = Request::new(Post, uri);
        req.set_body(json!({
            "type": "m.login.application_service",
            "user": user_id
        }).to_string());
        self.send_discarding_request(req)
    }
    pub fn new_appservice(url: String, user_id: String, as_token: String, hdl: &Handle) -> MatrixResult<Self> {
        let conn = HttpsConnector::new(4, hdl)?;
        let client = hyper::Client::configure()
            .connector(conn)
            .build(hdl);
        let hdl = hdl.clone();
        Ok(MatrixClient {
            hyper: client,
            access_token: as_token,
            user_id: user_id,
            url: url,
            hdl: hdl,
            is_as: true,
            txnid: 0
        })
    }
    pub fn alter_user_id(&mut self, user_id: String) {
        self.user_id = user_id;
    }
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
                is_as: false,
                txnid: 0
            }
        }))
    }
    pub fn create_room(&mut self, opts: RoomCreationOptions) -> MatrixFuture<JoinReply> {
        MatrixRequest::new_with_body_ser(Post, "/createRoom", opts)
            .send(self)
    }
    pub fn get_displayname(&mut self, user_id: &str) -> MatrixFuture<DisplaynameReply> {
        MatrixRequest::new_basic(Get, format!("/profile/{}/displayname", user_id))
            .send(self)
    }
    pub fn set_displayname(&mut self, name: String) -> MatrixFuture<()> {
        MatrixRequest::new_with_body_ser(
            Put,
            format!("/profile/{}/displayname", self.user_id),
            DisplaynameReply { displayname: name }
        ).discarding_send(self)
    }
    /// Join a room by identifier or alias.
    pub fn join(&mut self, roomid: &str) -> MatrixFuture<JoinReply> {
        MatrixRequest::new_basic(Post, format!("/join/{}", roomid))
            .send(self)
    }
    /// Update our presence status.
    pub fn update_presence(&mut self, p: Presence) -> MatrixFuture<()> {
        MatrixRequest::new_with_body_ser(
            Put,
            format!("/presence/{}/status", self.user_id),
            json!({
                "presence": p
            })
        ).discarding_send(self)
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
            endpoint: "/upload".into(),
            params: HashMap::new(),
            body: (),
            typ: request::apis::r0::MediaApi
        }.make_hyper(self);
        let mut req = match req {
            Ok(r) => r,
            Err(e) => return Box::new(futures::future::err(e.into()))
        };
        req.set_body(data.into());
        req.headers_mut().set(ct);
        self.send_request(req)
    }
    /// Requests that the server resolve a room alias to a room ID.
    ///
    /// The server will use the federation API to resolve the alias if the
    /// domain part of the alias does not correspond to the server's own domain.
    pub fn resolve_room_id(&mut self, rid: &str) -> MatrixFuture<RoomIdReply> {
        MatrixRequest::new_basic(Get, format!("/directory/room/{}", rid))
            .send(self)
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
    /// Get this `MatrixClient`'s underlying `hyper::Client`.
    pub fn get_hyper(&mut self) -> &mut http::MatrixHyper {
        &mut self.hyper
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
