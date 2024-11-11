#[macro_use]
extern crate diesel_derive_newtype;

#[cfg(feature = "ssr")]
pub mod backend;
pub mod common;
#[allow(clippy::unwrap_used)]
pub mod frontend;
