//! This crate attempt to provide utilities to parse subtitles.
//! Work is started from vobsub [crates.io](https://crates.io/crates/vobsub),
//! [repository](https://github.com/emk/subtitles-rs) which no longer seems to be maintained.
//!
//! ## Contributing
//!
//! Your feedback and contributions are welcome!  Please see
//! [FramaGit](https://framagit.org/gwenlg/subtitles-utils) for details.

#![warn(missing_docs)]
// For error-chain.
#![recursion_limit = "1024"]

mod errors;
/// Module for SubRip/Srt fonctionnalities
pub mod srt;
/// Module for time management
pub mod time;
mod util;
pub mod vobsub;

pub use errors::SubError;
