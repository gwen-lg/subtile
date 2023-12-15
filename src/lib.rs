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

extern crate cast;
extern crate common_failures;
#[macro_use]
extern crate failure;
#[cfg(test)]
extern crate env_logger;
extern crate image;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate regex;
extern crate safemem;

mod errors;
mod util;
pub mod vobsub;
