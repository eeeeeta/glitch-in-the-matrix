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

#![warn(missing_docs)]

extern crate serde;
extern crate serde_json;
pub extern crate http;
extern crate hyper;
extern crate hyper_openssl;
extern crate failure;
extern crate failure_derive;
extern crate tokio_core;
extern crate futures;
extern crate percent_encoding;
extern crate uuid;
pub extern crate gm_types as types;

pub mod errors;
pub mod room;
pub mod request;
pub mod sync;
pub mod profile;
pub mod media;
pub mod presence;
mod util;

use util::*;
use errors::*;
use types::replies::*;
use futures::future::Either;
use hyper::client::{Client, HttpConnector};
use http::{Request, Response, Method};
use hyper_openssl::HttpsConnector;
use tokio_core::reactor::Handle;
use futures::*;
use request::{MatrixRequestable, MatrixRequest};
use std::borrow::Cow;
use uuid::Uuid;
use std::cell::RefCell;
use std::rc::Rc;
use serde_json::json;

#[allow(missing_docs)]
pub type MatrixHyper = Client<HttpsConnector<HttpConnector>>;
/// A connection to a Matrix homeserver, using the `hyper` crate.
#[derive(Clone)]
pub struct MatrixClient {
    hyper: MatrixHyper,
    access_token: String,
    hdl: Handle,
    user_id: String,
    url: String,
    is_as: bool
}
impl MatrixClient {
    pub fn new_from_access_token(token: &str, url: &str, hdl: &Handle) -> impl Future<Item = Self, Error = MatrixError> {
        let conn = match HttpsConnector::new(4) {
            Ok(c) => c,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };
        let client = Client::builder()
            .build(conn);

        let uri: hyper::Uri = match format!("{}/_matrix/client/r0/account/whoami?access_token={}", url, token).parse() {
            Ok(u) => u,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };

        // Build the request
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(hyper::Body::empty());
        let req = match req {
            Ok(r) => r,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };

        // Send request
        let resp = client.request(req).map_err(|e| e.into()).and_then(ResponseWrapper::<WhoamiReply>::wrap);
        let hdl = hdl.clone();
        let url = url.to_string();
        let token = token.to_string();
        Either::A(resp.map(move |rpl| {
            MatrixClient {
                hyper: client,
                access_token: token,
                user_id: rpl.user_id,
                url: url,
                hdl: hdl,
                is_as: false,
            }
        }))
    }
    /// Log in to a Matrix homeserver with a username and password, and return a client object.
    ///
    /// ## Parameters
    ///
    /// - `username`: the username of the account to use (NB: not a MXID)
    /// - `password`: the password of the account to use
    /// - `url`: the URL of the homeserver
    /// - `hdl`: Tokio reactor handle
    pub fn login_password(username: &str, password: &str, url: &str, hdl: &Handle) -> impl Future<Item = Self, Error = MatrixError> {
        let conn = match HttpsConnector::new(4) {
            Ok(c) => c,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };
        let client = Client::builder()
            .build(conn);
        let uri: hyper::Uri = match format!("{}/_matrix/client/r0/login", url).parse() {
            Ok(u) => u,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };

        // Build the request
        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .body(json!({
                "type": "m.login.password",
                "user": username,
                "password": password
            }).to_string().into());
        let req = match req {
            Ok(r) => r,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };

        // Send request
        let resp = client.request(req).map_err(|e| e.into()).and_then(ResponseWrapper::<LoginReply>::wrap);
        let hdl = hdl.clone();
        let url = url.to_string();
        Either::A(resp.map(move |rpl| {
            MatrixClient {
                hyper: client,
                access_token: rpl.access_token,
                user_id: rpl.user_id,
                url: url,
                hdl: hdl,
                is_as: false,
            }
        }))
    }
    /// (for Application Services) Register a user with the given `user_id`.
    pub fn as_register_user(&mut self, user_id: String) -> impl Future<Item = (), Error = MatrixError> {
        let body = json!({
            "type": "m.login.application_service",
            "user": user_id
        }).to_string().into_bytes();
        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("{}/_matrix/client/r0/register?access_token={}", self.url, self.access_token))
            .body(body);
        let req = match req {
            Ok(u) => u,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };
        Either::A(self.typed_api_call(req, true))
    }
    /// (for Application Services) Make a new AS client.
    ///
    /// ## Parameters
    ///
    /// - `url`: homeserver URL
    /// - `user_id`: user ID to impersonate (can be changed later, using `alter_user_id`)
    /// - `as_token`: application service token
    /// - `hdl`: Tokio reactor handle
    pub fn as_new(url: String, user_id: String, as_token: String, hdl: &Handle) -> MatrixResult<Self> {
        let conn = HttpsConnector::new(4)?;
        let client = hyper::Client::builder()
            .build(conn);
        let hdl = hdl.clone();
        Ok(MatrixClient {
            hyper: client,
            access_token: as_token,
            user_id: user_id,
            url: url,
            hdl: hdl,
            is_as: true,
        })
    }
    /// (for Application Services) Alter the user ID which this client is masquerading as.
    pub fn as_alter_user_id(&mut self, user_id: String) {
        self.user_id = user_id;
    }
    /// Convenience method that clones the MatrixClient's underlying HTTP client.
    /// Useful if you don't want to have to make your own HTTP client, and need one
    /// for some quick'n'dirty task.
    pub fn get_hyper(&self) -> MatrixHyper {
        self.hyper.clone()
    }
}
impl MatrixRequestable for Rc<RefCell<MatrixClient>> {
    type Txnid = Uuid;
    type ResponseBody = hyper::Chunk;
    type ResponseBodyFuture = MxClientResponseBodyFuture;
    type SendRequestFuture = MxClientSendRequestFuture;

    fn get_url(&self) -> Cow<str> {
        self.borrow().url.clone().into()
    }
    fn get_access_token(&self) -> Cow<str> {
        self.borrow().access_token.clone().into()
    }
    fn get_txnid(&mut self) -> Uuid {
        Uuid::new_v4()
    }
    fn get_user_id(&self) -> Cow<str> {
        self.borrow().user_id.clone().into()
    }
    fn is_as(&self) -> bool {
        self.borrow().is_as
    }
    fn send_request(&mut self, req: http::Request<Vec<u8>>) -> Self::SendRequestFuture {
        let (parts, body) = req.into_parts();
        let body = hyper::Body::from(body);
        let req = Request::from_parts(parts, body);

        MxClientSendRequestFuture {
            inner: self.borrow_mut().hyper.request(req.into())
        }
    }
}
/// The `ResponseBodyFuture` of a `MatrixClient`.
pub struct MxClientResponseBodyFuture {
    inner: futures::stream::Concat2<hyper::Body>
}
impl Future for MxClientResponseBodyFuture {
    type Item = hyper::Chunk;
    type Error = MatrixError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let resp = try_ready!(self.inner.poll());
        Ok(Async::Ready(resp))
    }
}
/// The `SendRequestFuture` of a `MatrixClient`.
pub struct MxClientSendRequestFuture {
    inner: hyper::client::ResponseFuture
}
impl Future for MxClientSendRequestFuture {
    type Item = Response<MxClientResponseBodyFuture>;
    type Error = MatrixError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let req: http::Response<_> = try_ready!(self.inner.poll()).into();
        let (parts, body) = req.into_parts();
        let body = MxClientResponseBodyFuture {
            inner: body.concat2()
        };
        let resp = Response::from_parts(parts, body);
        Ok(Async::Ready(resp))
    }
}
impl MatrixRequestable for MatrixClient {
    type Txnid = Uuid;
    type ResponseBody = hyper::Chunk;
    type ResponseBodyFuture = MxClientResponseBodyFuture;
    type SendRequestFuture = MxClientSendRequestFuture;

    fn get_url(&self) -> Cow<str> {
        (&self.url as &str).into()
    }
    fn get_access_token(&self) -> Cow<str> {
        (&self.access_token as &str).into()
    }
    fn get_txnid(&mut self) -> Uuid {
        Uuid::new_v4()
    }
    fn get_user_id(&self) -> Cow<str> {
        (&self.user_id as &str).into()
    }
    fn is_as(&self) -> bool {
        self.is_as
    }
    fn send_request(&mut self, req: http::Request<Vec<u8>>) -> Self::SendRequestFuture {
        let (parts, body) = req.into_parts();
        let body = hyper::Body::from(body);
        let req = Request::from_parts(parts, body);

        MxClientSendRequestFuture {
            inner: self.hyper.request(req.into())
        }
    }
}
impl Drop for MatrixClient {
    /// Invalidates our access token, so we don't have millions of devices.
    /// Also sets us as offline.
    fn drop(&mut self) {
        let fut = MatrixRequest::new_basic(Method::POST, "/logout")
            .discarding_send(self).map_err(|_| ()).map(|_| ());
        self.hdl.spawn(fut);
    }
}
