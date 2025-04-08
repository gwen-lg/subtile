//! Parse a file in `*.idx` format.

use compact_str::CompactString;
use log::trace;
use regex::Regex;
use std::{
    fmt, fs,
    io::{self, prelude::*, BufReader},
    path::Path,
    sync::LazyLock,
};

use super::{
    palette::{palette, DEFAULT_PALETTE},
    Palette, VobSubError,
};
use crate::{time::TimePoint, vobsub::IResultExt};

/// Lang of a subtitle as reported in `VobSub` idx file.
#[derive(Debug, Clone)]
pub struct Lang(CompactString);

impl Lang {
    #[allow(clippy::missing_const_for_fn)]
    pub fn lang(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Lang {
    type Error = VobSubError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static KEY_VALUE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new("^([a-z]+), index: (.*)").unwrap());
        if let Some(cap) = KEY_VALUE.captures(value) {
            let lang = cap.get(1).unwrap().as_str();
            Ok(Self(lang.into()))
        } else {
            Err(VobSubError::LangParsing)
        }
    }
}

/// Extend `TimePoint` to implement `idx` specific `Display`.
#[repr(transparent)]
pub struct TimePointIdx(TimePoint);

impl From<TimePoint> for TimePointIdx {
    fn from(value: TimePoint) -> Self {
        Self(value)
    }
}

impl fmt::Display for TimePointIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_separator(f, ':')
    }
}

/// A `*.idx` file describing the subtitles in a `*.sub` file.
#[derive(Debug)]
pub struct Index {
    // Frame size.
    //size: Size,
    /// The colors used for the subtitles.
    palette: Palette,
    /// Lang of the subtitles
    lang: Option<Lang>,
}

const PALETTE_KEY: &str = "palette";
const LANG_KEY: &str = "id";

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
        Self::read_index(input, &mkerr_idx)
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
    pub fn read_index<T, Err>(mut input: BufReader<T>, mkerr: &Err) -> Result<Self, VobSubError>
    where
        T: std::io::Read,
        Err: Fn(io::Error) -> VobSubError,
    {
        static KEY_VALUE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new("^([A-Za-z/ ]+): (.*)").unwrap());

        let mut palette_val = None;
        let mut lang = None;
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
                    LANG_KEY => {
                        //TODO: reporte missing lang ?
                        lang = Lang::try_from(val).ok();
                    }
                    _ => trace!("Unimplemented idx key: {key}"),
                }
            }
            buf.clear();
        }

        //TODO: report missing palette ?
        let palette = match palette_val {
            Some(palette) => palette,
            None => DEFAULT_PALETTE,
        };

        Ok(Self { palette, lang })
    }

    /// Create an Index from a palette and sub data
    #[must_use]
    pub const fn init(palette: Palette, lang: Option<Lang>) -> Self {
        Self { palette, lang }
    }

    /// Get the palette associated with this `*.idx` file.
    #[must_use]
    pub const fn palette(&self) -> &Palette {
        &self.palette
    }
    /// Get the lang associated with this `*.idx` file.
    #[must_use]
    pub const fn lang(&self) -> &Option<Lang> {
        &self.lang
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
