//! Types used in the Matrix API.
//!
//! Will be better documented in the future; for now,
//! refer to the official API docs for info on what fields mean.

extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

pub mod room;
pub mod messages;
pub mod content;
pub mod events;
pub mod replies;
pub mod sync;
