//! Presence management.

use super::MatrixFuture;
use types::content::root::types::Presence;
use request::{MatrixRequest, MatrixRequestable};
use hyper::Method::*;

pub struct PresenceManagement;
impl PresenceManagement {
    /// Update our presence status.
    pub fn update_presence<R: MatrixRequestable>(rq: &mut R, p: Presence) -> MatrixFuture<()> {
        MatrixRequest::new_with_body_ser(
            Put,
            format!("/presence/{}/status", rq.get_user_id()),
            json!({
                "presence": p
            })
        ).discarding_send(rq)
    }
}
