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

#![deny(missing_docs)]

extern crate serde;
#[macro_use] extern crate serde_json;
extern crate hyper;
extern crate hyper_openssl;
extern crate failure;
#[macro_use] extern crate failure_derive;
extern crate tokio_core;
#[macro_use] extern crate futures;
extern crate percent_encoding;
extern crate uuid;
pub extern crate gm_types as types;

pub mod errors;
pub mod http;
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
use hyper::Method;
use Method::*;
use hyper::client::Request;
use hyper_openssl::HttpsConnector;
use serde::de::DeserializeOwned;
use tokio_core::reactor::Handle;
use futures::*;
use request::{MatrixRequestable, MatrixRequest};
use std::borrow::Cow;
use std::rc::Rc;
use std::cell::RefCell;
use uuid::Uuid;

/// A `Future` with a `MatrixError` error type. Returned by most library
/// functions.
///
/// Yes, I know this is a `Box`, and that sucks a whoole bunch. I'm waiting
/// for `impl Trait` to arrive to save us from this madness.
pub type MatrixFuture<T> = Box<Future<Item=T, Error=MatrixError>>;

/// A connection to a Matrix homeserver, using the `hyper` crate.
#[derive(Clone)]
pub struct MatrixClient {
    hyper: http::MatrixHyper,
    access_token: String,
    hdl: Handle,
    user_id: String,
    url: String,
    is_as: bool
}
impl MatrixClient {
    /// Log in to a Matrix homeserver with a username and password, and return a client object.
    /// 
    /// ## Parameters
    ///
    /// - `username`: the username of the account to use (NB: not a MXID)
    /// - `password`: the password of the account to use
    /// - `url`: the URL of the homeserver
    /// - `hdl`: Tokio reactor handle
    pub fn login_password(username: &str, password: &str, url: &str, hdl: &Handle) -> MatrixFuture<Self> {
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
            }
        }))
    }
    /// (for Application Services) Register a user with the given `user_id`.
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
    /// (for Application Services) Make a new AS client.
    /// 
    /// ## Parameters
    ///
    /// - `url`: homeserver URL
    /// - `user_id`: user ID to impersonate (can be changed later, using `alter_user_id`)
    /// - `as_token`: application service token
    /// - `hdl`: Tokio reactor handle
    pub fn as_new(url: String, user_id: String, as_token: String, hdl: &Handle) -> MatrixResult<Self> {
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
        })
    }
    /// (for Application Services) Alter the user ID which this client is masquerading as.
    pub fn as_alter_user_id(&mut self, user_id: String) {
        self.user_id = user_id;
    }
}
impl MatrixRequestable for Rc<RefCell<MatrixClient>> {
    type Txnid = Uuid;
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
    fn send_request<T>(&mut self, req: Request) -> MatrixFuture<T> where T: DeserializeOwned + 'static {
        Box::new(self.borrow_mut().hyper.request(req)
                 .map_err(|e| e.into())
                 .and_then(ResponseWrapper::<T>::wrap))
    }
    fn send_discarding_request(&mut self, req: Request) -> MatrixFuture<()> {
        Box::new(self.borrow_mut().hyper.request(req)
                 .map_err(|e| e.into())
                 .and_then(UnitaryResponseWrapper::wrap))
    }
}
impl MatrixRequestable for MatrixClient {
    type Txnid = Uuid;

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
    fn send_request<T>(&mut self, req: Request) -> MatrixFuture<T> where T: DeserializeOwned + 'static {
        Box::new(self.hyper.request(req)
                 .map_err(|e| e.into())
                 .and_then(ResponseWrapper::<T>::wrap))
    }
    fn send_discarding_request(&mut self, req: Request) -> MatrixFuture<()> {
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
