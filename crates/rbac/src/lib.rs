pub mod application;
pub mod domain;
pub mod infra;
#[path = "api/mod.rs"]
pub mod presentation;
#[doc(hidden)]
pub use inventory;
pub use presentation as api;
extern crate self as rbac;
