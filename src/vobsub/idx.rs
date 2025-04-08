//! Parse a file in `*.idx` format.

use log::trace;
use regex::Regex;
use std::{
    fs,
    io::{self, prelude::*, BufReader},
    path::Path,
    sync::LazyLock,
};

use crate::vobsub::IResultExt;

use super::{
    palette::{palette, DEFAULT_PALETTE},
    sub, Palette, VobSubError,
};

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

const PALETTE_KEY: &str = "palette";

impl Index {
    /// Open an `*.idx` file and the associated `*.sub` file.
    ///
    /// # Errors
    /// Will return VobSubError::Io if failed to open of read `.idx` or ``.sub`` file.
    #[profiling::function]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, VobSubError> {
        let path = path.as_ref();
        let mkerr_idx = |source| VobSubError::Io {
            source,
            path: path.into(),
        };

        let f = fs::File::open(path).map_err(mkerr_idx)?;
        let input = io::BufReader::new(f);
        let palette = read_palette(input, &mkerr_idx).or_else(|err| {
            if let VobSubError::MissingKey(PALETTE_KEY) = err {
                Ok(DEFAULT_PALETTE)
            } else {
                Err(err)
            }
        })?;

        let mut sub_path = path.to_owned();
        sub_path.set_extension("sub");

        let sub_path = sub_path.as_path();
        let mut sub = fs::File::open(sub_path).map_err(|source| VobSubError::Io {
            source,
            path: sub_path.into(),
        })?;
        let mut sub_data = vec![];
        sub.read_to_end(&mut sub_data)
            .map_err(|source| VobSubError::Io {
                source,
                path: sub_path.into(),
            })?;

        Ok(Self { palette, sub_data })
    }

    /// Create an Index from a palette and sub data
    #[must_use]
    pub const fn init(palette: Palette, sub_data: Vec<u8>) -> Self {
        Self { palette, sub_data }
    }

    /// Get the palette associated with this `*.idx` file.
    #[must_use]
    pub const fn palette(&self) -> &Palette {
        &self.palette
    }

    /// Iterate over the subtitles associated with this `*.idx` file.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn subtitles<D>(&self) -> sub::VobsubParser<D> {
        sub::VobsubParser::new(&self.sub_data)
    }
}

/// Read the palette in `*.idx` file content
///
/// # Errors
/// Will return `VobSubError::MissingKey` if the palette key/value is not present
/// Will return `VobSubError::PaletteError` if failed to read and parse palette value.
///
/// # Panics
/// Panic if the Regex creation failed
#[profiling::function]
pub fn read_palette<T, Err>(mut input: BufReader<T>, mkerr: &Err) -> Result<Palette, VobSubError>
where
    T: std::io::Read,
    Err: Fn(io::Error) -> VobSubError,
{
    static KEY_VALUE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^([A-Za-z/ ]+): (.*)").unwrap());

    let mut palette_val: Option<Palette> = None;
    let mut buf = String::with_capacity(256);
    while input.read_line(&mut buf).map_err(mkerr)? > 0 {
        let line = buf.trim_end();
        if let Some(cap) = KEY_VALUE.captures(line) {
            let key = cap.get(1).unwrap().as_str();
            let val = cap.get(2).unwrap().as_str();
            match key {
                PALETTE_KEY => {
                    palette_val = Some(
                        palette(val.as_bytes())
                            .to_result_no_rest()
                            .map_err(VobSubError::PaletteError)?,
                    );
                }
                _ => trace!("Unimplemented idx key: {key}"),
            }
        }
        buf.clear();
    }

    palette_val.ok_or(VobSubError::MissingKey(PALETTE_KEY))
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
