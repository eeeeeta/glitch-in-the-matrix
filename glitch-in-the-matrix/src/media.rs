//! Media repository management.

use futures;
use super::MatrixFuture;
use request::{self, MatrixRequest, MatrixRequestable};
use hyper::Method::*;
use std::collections::HashMap;
use hyper::Body;
use hyper::header::ContentType;
use types::replies::UploadReply;

pub struct Media;

impl Media {
    /// Upload some data (convertible to a `Body`) of a given `ContentType`, like an image.
    ///
    /// `Body` is accessible via the `http` module. See the documentation there
    /// for a complete reference of what implements `Into<Body>` - a quick
    /// shortlist: `Vec<u8>`, `&'static [u8]` (not `&'a [u8]`, sadly), `String`,
    /// `&'static str`.
    ///
    /// `ContentType` is accessible via the `http` module. See the documentation
    /// there for more information on how to use it.
    pub fn upload<T: Into<Body>, R: MatrixRequestable>(rq: &mut R, data: T, ct: ContentType) -> MatrixFuture<UploadReply> {
        let req = MatrixRequest {
            meth: Post,
            endpoint: "/upload".into(),
            params: HashMap::new(),
            body: (),
            typ: request::apis::r0::MediaApi
        }.make_hyper(rq);
        let mut req = match req {
            Ok(r) => r,
            Err(e) => return Box::new(futures::future::err(e.into()))
        };
        req.set_body(data.into());
        req.headers_mut().set(ct);
        rq.send_request(req)
    }

}
