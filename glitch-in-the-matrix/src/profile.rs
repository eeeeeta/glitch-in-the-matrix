//! Profile management.

use super::MatrixFuture;
use types::replies::DisplaynameReply;
use request::{MatrixRequest, MatrixRequestable};
use hyper::Method::*;

pub struct Profile;
impl Profile {
    pub fn get_displayname<R: MatrixRequestable>(rq: &mut R, user_id: &str) -> MatrixFuture<DisplaynameReply> {
        MatrixRequest::new_basic(Get, format!("/profile/{}/displayname", user_id))
            .send(rq)
    }
    pub fn set_displayname<R: MatrixRequestable>(rq: &mut R, name: String) -> MatrixFuture<()> {
        MatrixRequest::new_with_body_ser(
            Put,
            format!("/profile/{}/displayname", rq.get_user_id()),
            DisplaynameReply { displayname: name }
        ).discarding_send(rq)
    }

}
