//! Type for making a generic request to the Matrix API.

use std::borrow::Cow;
use hyper::{Body, Method};
use std::collections::HashMap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use hyper::client::Request;
use super::{MatrixFuture, MatrixClient};
use errors::MatrixResult;
use serde_json;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use futures;

/// Describes the type of a Matrix API.
pub trait ApiType {
    /// Get the base path which all requests to this API should contain.
    ///
    /// For example, `ClientApi`, the struct for the client-server API, sets
    /// this method to return `/_matrix/client/r0`
    fn get_path<'a>(&'a self) -> Cow<'a, str>;
}
/// Types of Matrix APIs.
pub mod apis {
    /// APIs at version r0.
    pub mod r0 {
        use request::ApiType;
        use std::borrow::Cow;
        /// `/_matrix/client/r0`
        pub struct ClientApi;
        impl ApiType for ClientApi {
            fn get_path(&self) -> Cow<'static, str> {
                "/_matrix/client/r0".into()
            }
        }
        /// `/_matrix/media/r0`
        pub struct MediaApi;
        impl ApiType for MediaApi {
            fn get_path(&self) -> Cow<'static, str> {
                "/_matrix/media/r0".into()
            }
        }
    }
}
use self::apis::r0::*;
/// A arbitrary request to an endpoint in the Matrix API.
///
/// To actually determine what URL is used for the request, two things are
/// consulted: the request type, and the request endpoint. The request type
/// specifies what Matrix API is being used (for example, the client-server API
/// revision 0, under `/_matrix/client/r0`), while the endpoint determines what
/// method is being called on that API.
///
/// This type has Super `Cow` Powers.
pub struct MatrixRequest<'a, T, U = ClientApi> {
    /// Request method (exported in the `http` module)
    pub meth: Method,
    /// API endpoint (e.g. `/sync`)
    pub endpoint: Cow<'a, str>,
    /// Query-string parameters.
    pub params: HashMap<Cow<'a, str>, Cow<'a, str>>,
    /// Request body (some type implementing `Serialize`).
    ///
    /// If this is empty (serialises to `{}`), it will not be sent. Therefore,
    /// requests with no body should use `()` here.
    pub body: T,
    /// Request type.
    pub typ: U
}
impl<'a, T, U> MatrixRequest<'a, T, U> where T: Serialize, U: ApiType {
    pub fn new<S: Into<Cow<'a, str>>>(meth: Method, endpoint: S, body: T, typ: U) -> Self {
        Self {
            meth,
            endpoint: endpoint.into(),
            params: HashMap::new(),
            body,
            typ
        }
    }
}
impl<'a> MatrixRequest<'a, ()> {
    /// Convenience method for making a `MatrixRequest` from a method and
    /// endpoint.
    pub fn new_basic<S: Into<Cow<'a, str>>>(meth: Method, endpoint: S) -> Self {
        Self {
            meth,
            endpoint: endpoint.into(),
            params: HashMap::new(),
            body: (),
            typ: ClientApi
        }
    }
}
impl<'a, 'b, 'c> MatrixRequest<'a, HashMap<Cow<'b, str>, Cow<'c, str>>> {
    pub fn new_with_body<S, T, U, V>(meth: Method, endpoint: S, body: V) -> Self
        where S: Into<Cow<'a, str>>,
              T: Into<Cow<'b, str>>,
              U: Into<Cow<'c, str>>,
              V: IntoIterator<Item=(T, U)> {
        let body = body.into_iter().map(|(t, u)| (t.into(), u.into()))
            .collect();
        Self {
            meth,
            endpoint: endpoint.into(),
            params: HashMap::new(),
            body,
            typ: ClientApi
        }
    }
}
impl<'a, T> MatrixRequest<'a, T> where T: Serialize {
    pub fn new_with_body_ser<S>(meth: Method, endpoint: S, body: T) -> Self
        where S: Into<Cow<'a, str>> {
        Self {
            meth,
            endpoint: endpoint.into(),
            params: HashMap::new(),
            body,
            typ: ClientApi
        }
    }
}
impl<'a, T, U> MatrixRequest<'a, T, U> where T: Serialize, U: ApiType {
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
        if client.is_as {
            params += &format!("&user_id={}",
                              utf8_percent_encode(&client.user_id, DEFAULT_ENCODE_SET));
        }
        for (k, v) in self.params.iter() {
            params += &format!("&{}={}",
                              utf8_percent_encode(k.as_ref(), DEFAULT_ENCODE_SET),
                              utf8_percent_encode(v.as_ref(), DEFAULT_ENCODE_SET));
        }
        let url = format!("{}{}{}?{}",
                          client.url,
                          self.typ.get_path(),
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
