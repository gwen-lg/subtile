//! This crate attempt to provide utilities to parse subtitles.
//! Work is started from vobsub [crates.io](https://crates.io/crates/vobsub),
//! [repository](https://github.com/emk/subtitles-rs) which no longer seems to be maintained.
//!
//! ## Contributing
//!
//! Your feedback and contributions are welcome!  Please see
//! [Subtile](https://github.com/gwen-lg/subtile) on GitHub for details.

#![deny(missing_docs)]
#![deny(unused_imports)]
#![deny(clippy::bind_instead_of_map)]
#![deny(clippy::borrowed_box)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::clone_on_copy)]
#![deny(clippy::derive_partial_eq_without_eq)]
#![deny(clippy::doc_markdown)]
#![deny(clippy::extra_unused_lifetimes)]
#![deny(clippy::if_not_else)]
#![deny(clippy::match_same_arms)]
#![deny(clippy::missing_const_for_fn)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_fields_in_debug)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::must_use_candidate)]
#![deny(clippy::or_fun_call)]
#![deny(clippy::trivially_copy_pass_by_ref)]
#![deny(clippy::uninlined_format_args)]
#![deny(clippy::use_self)]
#![deny(clippy::unreadable_literal)]
#![deny(clippy::useless_conversion)]
// For error-chain.
#![recursion_limit = "1024"]

pub mod content;
mod errors;
pub mod srt;
pub mod time;
mod util;
pub mod vobsub;

pub use errors::SubError;
