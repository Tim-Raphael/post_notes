//! Public library API for the `post-notes` binary.
//!
//! Both `main.rs` and integration tests use this library. It exposes the
//! read phase ([`notes`]), the build/write phase ([`website`]), and the
//! configuration surface ([`Settings`]).

pub mod notes;
pub mod settings;
pub mod website;

pub use notes::Notes;
pub use settings::Settings;
