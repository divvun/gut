pub mod graphql;
pub mod models;
pub mod rest;

pub use graphql::*;
pub use models::*;
pub use rest::*;

pub(crate) static USER_AGENT: &str = concat!("gut ", env!("CARGO_PKG_VERSION"));
