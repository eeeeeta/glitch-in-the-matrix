//! Error handling.
macro_rules! derive_from {
    ($err:ident, $($var:ident, $ty:ty),*) => {
        $(
            impl From<$ty> for $err {
                fn from(e: $ty) -> $err {
                    $err::$var(e)
                }
            }
         )*
    }
}
/// Something Matrixy that can go wrong.
#[derive(Fail, Debug)]
#[allow(missing_docs)]
pub enum MatrixError {
    #[fail(display = "HTTP error: {}", _0)]
    Hyper(#[cause] ::hyper::error::Error),
    #[fail(display = "Serialization error: {}", _0)]
    Serde(#[cause] ::serde_json::Error),
    #[fail(display = "Error decoding URI: {}", _0)]
    UriError(#[cause] ::hyper::error::UriError),
    #[fail(display = "I/O error: {}", _0)]
    Io(#[cause] ::std::io::Error),
    #[fail(display = "OpenSSL error: {}", _0)]
    Openssl(#[cause] ::hyper_openssl::openssl::error::ErrorStack),
    /// A request failed with a non-OK HTTP status.
    ///
    /// If the body contained a valid `BadRequestReply`, the `BadRequest` variant will be used
    /// instead of this one.
    #[fail(display = "Request failed with HTTP status: {}", _0)]
    HttpCode(::hyper::StatusCode),
    /// A request failed with an error from the homeserver.
    #[fail(display = "Error from homeserver: {:?}", _0)]
    BadRequest(super::types::replies::BadRequestReply)
}
derive_from!(MatrixError,
             Hyper, ::hyper::error::Error,
             Serde, ::serde_json::Error,
             UriError, ::hyper::error::UriError,
             Io, ::std::io::Error,
             Openssl, ::hyper_openssl::openssl::error::ErrorStack
            );
/// Bog-standard result newtype. You know the drill.
pub type MatrixResult<T> = Result<T, MatrixError>;

