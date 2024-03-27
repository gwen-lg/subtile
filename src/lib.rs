//! This crate attempt to provide utilities to parse subtitles.
//! Work is started from vobsub [crates.io](https://crates.io/crates/vobsub),
//! [repository](https://github.com/emk/subtitles-rs) which no longer seems to be maintained.
//!
//! ## Contributing
//!
//! Your feedback and contributions are welcome!  Please see
//! [Subtile](https://github.com/gwen-lg/subtile) on GitHub for details.

#![deny(missing_docs)]
#![deny(clippy::bind_instead_of_map)]
#![deny(clippy::doc_markdown)]
#![deny(clippy::missing_fields_in_debug)]
#![deny(clippy::must_use_candidate)]
#![deny(clippy::or_fun_call)]
#![deny(clippy::uninlined_format_args)]
// For error-chain.
#![recursion_limit = "1024"]

mod errors;
pub mod srt;
pub mod time;
mod util;
pub mod vobsub;

pub use errors::SubError;
