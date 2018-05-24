//! Profile management.

use types::replies::DisplaynameReply;
use request::{MatrixRequest, MatrixRequestable};
use http::Method;
use futures::Future;
use errors::MatrixError;

/// Contains methods relating to `/profile/` endpoints.
pub struct Profile;

impl Profile {
    /// Get the displayname of a given user ID. This API may be used to fetch the user's own displayname or 
    /// to query the name of other users; either locally or on remote homeservers. 
    pub fn get_displayname<R: MatrixRequestable>(rq: &mut R, user_id: &str) -> impl Future<Item = DisplaynameReply, Error = MatrixError> {
        MatrixRequest::new_basic(Method::GET, format!("/profile/{}/displayname", user_id))
            .send(rq)
    }
    /// Sets the user's displayname.
    pub fn set_displayname<R: MatrixRequestable>(rq: &mut R, name: String) -> impl Future<Item = (), Error = MatrixError> {
        MatrixRequest::new_with_body_ser(
            Method::PUT,
            format!("/profile/{}/displayname", rq.get_user_id()),
            DisplaynameReply { displayname: name }
        ).discarding_send(rq)
    }

}
