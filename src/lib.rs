//! `subtile` is a Rust library which aims to propose a set of operations
//! for working on subtitles. Example: parsing from and export in different formats,
//! transform, adjust, correct, ...
//!
//! # Project
//! ## start
//! The project started with the fork of [vobsub](https://crates.io/crates/vobsub)
//! crate which no longer seems to be maintained.
//! Beyond the simple recovery, I want to take the opportunity to improve the code
//! and extend the provided features.
//!
//! ## Name
//! `Subtile` is a french word than fit well as contraction of Subtitles Utils.
//!
//! ## Contributing
//!
//! Your feedback and contributions are welcome!  Please see
//! [Subtile](https://github.com/gwen-lg/subtile) on GitHub for details.

// For error-chain.
#![recursion_limit = "1024"]

pub mod content;
mod errors;
pub mod image;
pub mod srt;
pub mod time;
mod util;
pub mod vobsub;

pub use errors::SubtileError;
