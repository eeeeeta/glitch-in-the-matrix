//! Utilities for using the long-polling `/sync` API.

use types::sync::*;
use std::collections::HashMap;
use crate::request::{MatrixRequest, MatrixRequestable, TypedApiResponse};
use crate::request::apis::r0::ClientApi;
use futures::*;
use crate::errors::*;
use http::Method;
use futures::Future;

/// A `Stream` that yields constant replies to `/sync`.
///
/// This calls the long-polling `/sync` API, which will wait until replies come
/// in and send them to the client. If you want to reduce the wait time, use the
/// `set_timeout()` function.
pub struct SyncStream<R> where R: MatrixRequestable {
    pub(crate) rq: R,
    pub(crate) last_batch: Option<String>,
    pub(crate) set_presence: bool,
    pub(crate) timeout: u64,
    pub(crate) cur_req: Option<TypedApiResponse<SyncReply, R::SendRequestFuture, R::ResponseBodyFuture>>
}
impl<R> SyncStream<R> where R: MatrixRequestable {
    /// Make a new `SyncStream` from a given `MatrixRequestable`.
    pub fn new(rq: R) -> Self {
        SyncStream {
            rq,
            last_batch: None,
            set_presence: true,
            timeout: 30_000,
            cur_req: None
        }
    }
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
    fn req(&mut self) -> MatrixRequest<'static, ()> {
        let mut params = HashMap::new();
        params.insert("set_presence".into(), if self.set_presence {
            "online"
        } else { "offline" }.to_string().into());
        if let Some(ref b) = self.last_batch {
            params.insert("since".into(), b.to_string().into());
            params.insert("timeout".into(), self.timeout.to_string().into());
        }
        MatrixRequest {
            meth: Method::GET,
            endpoint: "/sync".into(),
            params,
            body: (),
            typ: ClientApi
        }
    }
}

impl<R> Stream for SyncStream<R> where R: MatrixRequestable {
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
            let req = match req.make_request(&mut self.rq) {
                Ok(r) => r,
                Err(e) => return Err(e.into())
            };
            self.cur_req = Some(self.rq.typed_api_call(req, false));
        }
    }
}

