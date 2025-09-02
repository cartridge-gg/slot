#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod account;
pub mod bigint;
pub mod credentials;
pub(crate) mod error;
pub mod preset;
pub mod version;

pub use error::Error;
use update_informer as _;
