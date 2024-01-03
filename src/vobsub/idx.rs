//! Parse a file in `*.idx` format.

use anyhow::{Context, Result};
use log::trace;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use super::{palette, sub, Palette};
use crate::errors::IResultExt;
use crate::errors::SubError;

/// A `*.idx` file describing the subtitles in a `*.sub` file.
#[derive(Debug)]
pub struct Index {
    // Frame size.
    //size: Size,
    /// The colors used for the subtitles.
    palette: Palette,
    /// Our compressed subtitle data.
    sub_data: Vec<u8>,
}

impl Index {
    /// Open an `*.idx` file and the associated `*.sub` file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Index> {
        static KEY_VALUE: Lazy<Regex> = Lazy::new(|| Regex::new("^([A-Za-z/ ]+): (.*)").unwrap());

        let path = path.as_ref();
        let mut sub_path = path.to_owned();
        sub_path.set_extension("sub");

        let mut palette_val: Option<Palette> = None;

        let f = fs::File::open(path).context("Try open idx file")?;
        let input = io::BufReader::new(f);

        for line in input.lines() {
            let line = line.context("Try read line")?;
            if let Some(cap) = KEY_VALUE.captures(&line) {
                let key = cap.get(1).unwrap().as_str();
                let val = cap.get(2).unwrap().as_str();
                match key {
                    "palette" => {
                        palette_val = Some(palette(val.as_bytes()).to_vobsub_result()?);
                    }
                    _ => trace!("Unimplemented idx key: {}", key),
                }
            }
        }

        let mut sub = fs::File::open(sub_path)?;
        let mut sub_data = vec![];
        sub.read_to_end(&mut sub_data)?;

        let palette = palette_val.ok_or(SubError::MissingKey { key: "palette" })?;
        Ok(Index { palette, sub_data })
    }

    /// Get the palette associated with this `*.idx` file.
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// Iterate over the subtitles associated with this `*.idx` file.
    pub fn subtitles(&self) -> sub::Subtitles {
        sub::subtitles(&self.sub_data)
    }
}

#[cfg(test)]
mod tests {
    use image::Rgb;

    use crate::vobsub::Index;

    #[test]
    fn parse_index() {
        env_logger::init();

        let idx = Index::open("./fixtures/example.idx").unwrap();

        //assert_eq!(idx.size(), Size { w: 1920, h: 1080 });
        assert_eq!(idx.palette()[0], Rgb([0x00, 0x00, 0x00]));
        assert_eq!(idx.palette()[15], Rgb([0x11, 0xbb, 0xbb]));
    }
}
