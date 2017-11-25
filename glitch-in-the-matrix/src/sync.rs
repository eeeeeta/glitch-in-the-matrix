//! Utilities for using the long-polling `/sync` API.

use hyper;
use hyper::Method::*;
use hyper::client::{HttpConnector, Request};
use hyper_openssl::HttpsConnector;
use types::sync::*;
use super::MatrixFuture;
use util::ResponseWrapper;
use futures::*;
use errors::*;

/// A `Stream` that yields constant replies to `/sync`.
///
/// This calls the long-polling `/sync` API, which will wait until replies come
/// in and send them to the client. If you want to reduce the wait time, use the
/// `set_timeout()` function.
pub struct SyncStream {
    pub(crate) hyper: hyper::Client<HttpsConnector<HttpConnector>>,
    pub(crate) last_batch: Option<String>,
    pub(crate) set_presence: bool,
    pub(crate) access_token: String,
    pub(crate) url: String,
    pub(crate) timeout: u64,
    pub(crate) cur_req: Option<MatrixFuture<SyncReply>>
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

