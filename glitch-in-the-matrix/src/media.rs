//! Media repository management.

use futures::{self, Future};
use request::{self, MatrixRequest, MatrixRequestable};
use http::Method;
use std::collections::HashMap;
use http::header::{HeaderValue, CONTENT_TYPE};
use types::replies::UploadReply;
use futures::future::Either;
use errors::MatrixError;

/// Contains media repository endpoints.
pub struct Media;

impl Media {
    /// Upload some data (convertible to a `Body`) of a given `ContentType`, like an image.
    pub fn upload<T: Into<Vec<u8>>, R: MatrixRequestable>(rq: &mut R, data: T, content_type: &str) -> impl Future<Item = UploadReply, Error = MatrixError> {
        let req = MatrixRequest {
            meth: Method::POST,
            endpoint: "/upload".into(),
            params: HashMap::new(),
            body: (),
            typ: request::apis::r0::MediaApi
        }.make_request(rq);
        let mut req = match req {
            Ok(r) => r,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };
        *req.body_mut() = data.into();
        let hv = match HeaderValue::from_str(content_type) {
            Ok(r) => r,
            Err(e) => return Either::B(futures::future::err(e.into()))
        };
        req.headers_mut().insert(CONTENT_TYPE, hv);
        Either::A(rq.typed_api_call(req, false))
    }

}
