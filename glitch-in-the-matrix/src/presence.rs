//! Presence management.

use types::content::root::types::Presence;
use crate::request::{MatrixRequest, MatrixRequestable};
use http::Method;
use crate::errors::MatrixError;
use futures::Future;
use serde_json::json;

/// Contains methods relating to `/presence/` endpoints.
pub struct PresenceManagement;

impl PresenceManagement {
    /// Update our presence status.
    pub fn update_presence<R: MatrixRequestable>(rq: &mut R, p: Presence) -> impl Future<Item = (), Error = MatrixError> {
        MatrixRequest::new_with_body_ser(
            Method::PUT,
            format!("/presence/{}/status", rq.get_user_id()),
            json!({
                "presence": p
            })
        ).discarding_send(rq)
    }
}
