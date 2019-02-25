//! Utility wrappers used internally.

use crate::errors::*;
use types::replies::*;
use hyper::{Body, StatusCode, Response};
use serde::de::DeserializeOwned;
use futures::*;
use std::marker::PhantomData;
use futures::stream::Concat2;

pub struct ResponseWrapper<T> {
    inner: Concat2<Body>,
    sc: StatusCode,
    _ph: PhantomData<T>,
}
impl<T: DeserializeOwned> ResponseWrapper<T> {
    pub fn wrap(r: Response<Body>) -> Self {
        let sc = r.status();
        let inner = r.into_body().concat2();
        let _ph = PhantomData;
        Self { sc, inner, _ph, }
    }
    fn _poll(&mut self) -> Poll<::hyper::Chunk, MatrixError> {
        let resp = try_ready!(self.inner.poll());
        if !self.sc.is_success() {
            if let Ok(e) = ::serde_json::from_slice::<BadRequestReply>(&resp) {
                return Err(MatrixError::BadRequest(e));
            }
            else {
                return Err(MatrixError::HttpCode(self.sc.into()));
            }
        }
        Ok(Async::Ready(resp))
    }
}
impl<T: DeserializeOwned> Future for ResponseWrapper<T> {
    type Item = T;
    type Error = MatrixError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let resp = try_ready!(self._poll());
        #[cfg(feature="gitm_show_responses")]
        println!("{:#}", String::from_utf8(resp.to_vec()).unwrap());
        let data = ::serde_json::from_slice::<T>(&resp)?;
        Ok(Async::Ready(data))
    }
}

