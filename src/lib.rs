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

/// Re-export `failure::Error` for convenience.
pub type Error = failure::Error;

/// A short alias for `Result<T, failure::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Import this module to get a useful error-handling API.
pub mod prelude {
    //    pub use display::DisplayCausesAndBacktraceExt;
    pub use failure::ResultExt;
    //    pub use io::{IoContextErrorExt, IoContextExt};
    //  pub use {Error, Result};
}
