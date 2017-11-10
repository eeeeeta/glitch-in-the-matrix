//! Utility wrappers used internally.

use errors::*;
use errors::MatrixErrorKind::*;
use types::replies::*;
use hyper::{Body, StatusCode};
use hyper::client::Response;
use serde::de::DeserializeOwned;
use futures::*;
use std::marker::PhantomData;
use futures::stream::Concat2;

pub struct ResponseWrapper<T> {
    inner: Concat2<Body>,
    sc: StatusCode,
    _ph: PhantomData<T>,
}
pub struct UnitaryResponseWrapper {
    inner: ResponseWrapper<()>
}
impl<T: DeserializeOwned> ResponseWrapper<T> {
    pub fn wrap(r: Response) -> Self {
        let sc = r.status();
        let inner = r.body().concat2();
        let _ph = PhantomData;
        Self { sc, inner, _ph, }
    }
    fn _poll(&mut self) -> Poll<::hyper::Chunk, MatrixError> {
        let resp = try_ready!(self.inner.poll());
        if !self.sc.is_success() {
            if let Ok(e) = ::serde_json::from_slice::<BadRequestReply>(&resp) {
                bail!(BadRequest(e));
            }
            else {
                bail!(HttpCode(self.sc.clone()));
            }
        }
        Ok(Async::Ready(resp))
    }
}
impl UnitaryResponseWrapper {
    pub fn wrap(r: Response) -> Self {
        Self {
            inner: ResponseWrapper::<()>::wrap(r)
        }
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
impl Future for UnitaryResponseWrapper {
    type Item = ();
    type Error = MatrixError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        try_ready!(self.inner._poll());
        Ok(Async::Ready(()))
    }
}
