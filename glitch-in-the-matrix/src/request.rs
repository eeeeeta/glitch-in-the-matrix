//! Type for making a generic request to the Matrix API.

use std::borrow::Cow;
use std::collections::HashMap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use http::{Request, Response, Method};
use futures::future::Either;
use crate::errors::{MatrixError, MatrixResult};
use types::replies::BadRequestReply;
use serde_json;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use futures::{self, Future, Poll, Async, try_ready};
use std::marker::PhantomData;

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
        use crate::request::ApiType;
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
/// Future representing a response to a Matrix API call that isn't ready yet.
///
/// Returned by the `typed_api_response` method on a `MatrixRequestable`.
pub struct TypedApiResponse<T, U, V> {
    response: Option<U>,
    body: Option<V>,
    parts: Option<::http::response::Parts>,
    _ph: PhantomData<T>,
    discard: bool
}
impl<T, U, RB, V> Future for TypedApiResponse<T, U, V>
    where T: DeserializeOwned + 'static,
          RB: AsRef<[u8]>,
          V: Future<Item = RB, Error = MatrixError>,
          U: Future<Item = Response<V>, Error = MatrixError> {

    type Item = T;
    type Error = MatrixError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use std::any::TypeId;
        use std::ptr;

        if self.response.is_some() {
            let resp = try_ready!(self.response.as_mut().unwrap().poll());
            self.response = None;
            let (parts, body) = resp.into_parts();
            self.parts = Some(parts);
            self.body = Some(body);
        }
        let body = try_ready!(self.body.as_mut().unwrap().poll());
        let parts = self.parts.take().unwrap();
        if !parts.status.is_success() {
            if let Ok(e) = ::serde_json::from_slice::<BadRequestReply>(body.as_ref()) {
                return Err(MatrixError::BadRequest(e));
            }
            else {
                return Err(MatrixError::HttpCode(parts.status));
            }
        }
        let data = if TypeId::of::<T>() == TypeId::of::<()>() && self.discard {
            // If the response type is (), and we have the discard flag set,
            // magic up a () from thin air and use that instead of whatever
            // the JSON response is.
            unsafe {
                ptr::read(&() as *const () as *const T)
            }
        }
        else {
            ::serde_json::from_slice::<T>(body.as_ref())?
        };
        Ok(Async::Ready(data))
    }
}
/// Represents an object that can make requests to the Matrix Client-Server API.
pub trait MatrixRequestable {
    /// The type of the transaction ID returned by `get_txnid()`.
    type Txnid: ::std::fmt::Display;
    /// The type of the HTTP response body.
    type ResponseBody: AsRef<[u8]>;
    /// The type of the future returned as a HTTP response body.
    ///
    /// This is polled in order to get the response body.
    type ResponseBodyFuture: Future<Item = Self::ResponseBody, Error = MatrixError> + 'static;
    /// The type of the future returned by `send_request()`.
    ///
    /// Should return a HTTP `Response` with the body being `Self::ResponseBodyFuture`.
    type SendRequestFuture: Future<Item = Response<Self::ResponseBodyFuture>, Error = MatrixError> + 'static;
    /// Gets the client's URL.
    fn get_url(&self) -> Cow<str>;
    /// Checks whether the client is an Application Service (AS).
    fn is_as(&self) -> bool { false }
    /// Gets the client's access token.
    fn get_access_token(&self) -> Cow<str>;
    /// Gets the client's user ID.
    fn get_user_id(&self) -> Cow<str>;
    /// Gets a new transaction ID.
    ///
    /// Implementors should generate a unique ID, as this will be used by the server to ensure
    /// idempotency of requests.
    fn get_txnid(&mut self) -> Self::Txnid;
    /// Send an arbitrary HTTP request to the Matrix homeserver.
    fn send_request(&mut self, req: Request<Vec<u8>>) -> Self::SendRequestFuture;

    /// Send an arbitrary HTTP request to the Matrix homeserver, and deserialize the JSON response
    /// to a value of type T.
    ///
    /// If T is `()`, and `discard` is true, discards the response and returns `()`, no matter what
    /// the server responds with.
    fn typed_api_call<T>(&mut self, req: Request<Vec<u8>>, discard: bool) -> TypedApiResponse<T, Self::SendRequestFuture, Self::ResponseBodyFuture> where T: DeserializeOwned + 'static {
        TypedApiResponse {
            response: Some(self.send_request(req)),
            body: None,
            parts: None,
            discard,
            _ph: PhantomData
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
    /// Make a new `MatrixRequest`, specifying all possible options.
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
    /// Makes a `MatrixRequest` with the following defaults:
    ///
    /// - `meth` and `endpoint` specified
    /// - `params` set to an empty hashmap
    /// - `body` set to ()
    /// - `typ` set to `apis::r0::ClientApi`
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
    /// Makes a `MatrixRequest` with the following defaults:
    ///
    /// - `meth` and `endpoint` specified
    /// - `body` converted from an iterator over `(T, U)` where T & U implement `Into<Cow<str>>`
    /// - `params` set to an empty hashmap
    /// - `typ` set to `apis::r0::ClientApi`
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
    /// Like `new_with_body`, but takes a serializable object for `body`.
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
    fn body(&self) -> MatrixResult<Vec<u8>> {
        let body = serde_json::to_string(&self.body)?;
        Ok(if body == "{}" {
            vec![]
        }
        else {
            body.into_bytes()
        })
    }
    /// Make this `MatrixRequest` into a HTTP request.
    pub fn make_request<C>(&self, client: &C) -> MatrixResult<Request<Vec<u8>>> where C: MatrixRequestable {
        let body = self.body()?;
        let mut params = format!("access_token={}", client.get_access_token());
        if client.is_as() {
            params += &format!("&user_id={}",
                              utf8_percent_encode(&client.get_user_id(), DEFAULT_ENCODE_SET));
        }
        for (k, v) in self.params.iter() {
            params += &format!("&{}={}",
                              utf8_percent_encode(k.as_ref(), DEFAULT_ENCODE_SET),
                              utf8_percent_encode(v.as_ref(), DEFAULT_ENCODE_SET));
        }
        let url = format!("{}{}{}?{}",
                          client.get_url(),
                          self.typ.get_path(),
                          self.endpoint,
                          params);
        let req = Request::builder()
            .method(self.meth.clone())
            .uri(url)
            .body(body)?;
        Ok(req)
    }
    /// Sends this request to a Matrix homeserver, expecting a deserializable
    /// `R` return type.
    ///
    /// A helpful mix of `make_hyper()` and `MatrixClient::send_request()`.
    pub fn send<C, R>(&self, mxc: &mut C) -> impl Future<Item = R, Error = MatrixError> + 'static where R: DeserializeOwned + 'static, C: MatrixRequestable {
        let req = match self.make_request(mxc) {
            Ok(r) => r,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };
        Either::A(mxc.typed_api_call(req, false))
    }
    /// Like `send()`, but uses `MatrixClient::send_discarding_request()`.
    pub fn discarding_send<'x, 'y, C>(&'x self, mxc: &'y mut C) -> impl Future<Item = (), Error = MatrixError> + 'static where C: MatrixRequestable {
        let req = match self.make_request(mxc) {
            Ok(r) => r,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };
        Either::A(mxc.typed_api_call(req, true))
    }
}
