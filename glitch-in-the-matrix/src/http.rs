//! Types reexported from `hyper`.
pub use hyper::Method;
pub use hyper::Body;
pub use hyper::header::ContentType;
pub use hyper::StatusCode;
pub use hyper::Client;
pub use hyper_openssl::HttpsConnector;
pub use hyper::client::HttpConnector;
#[allow(missing_docs)]
pub type MatrixHyper = Client<HttpsConnector<HttpConnector>>;
