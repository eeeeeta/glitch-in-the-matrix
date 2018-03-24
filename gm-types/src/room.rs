//! Type for Matrix rooms.
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::borrow::Cow;
/// A Matrix room. This object is a thin wrapper over a room ID.
///
/// This is defined in this crate (`gm-types`) in order that it can be
/// deserialized. However, most of the interesting methods are actually defined
/// on `RoomExt`, or are part of the `RoomClient` struct in the main crate.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Room<'a> {
    #[allow(missing_docs)]
    pub id: Cow<'a, str>
}
impl<'a> Serialize for Room<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.id)
    }
}
impl<'de> Deserialize<'de> for Room<'static> {
    fn deserialize<D>(de: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let id: String = Deserialize::deserialize(de)?;
        Ok(Room { id: id.into() })
    }
}
impl<'a> Room<'a> {
    /// Make a `Room` object from a room ID.
    pub fn from_id<T: Into<Cow<'a, str>>>(id: T) -> Self {
        Room {
            id: id.into()
        }
    }
}
