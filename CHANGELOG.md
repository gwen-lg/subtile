# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1](https://github.com/gwen-lg/subtile/compare/v0.3.0...v0.3.1) - 2024-08-19

### Fixed
- fixup commit command
- fixup remove test of step
- fixup! ci: add cargo readme management in release-plz workflow

### Other
- add force push
- test change in lib.rs to see cargo readme run
- setup current branch for trigger release plz
- add cargo readme management in release-plz workflow

## [0.3.0](https://github.com/gwen-lg/subtile/compare/v0.2.0...v0.3.0) - 2024-08-11

### Added
- *(pgs)* implement ToOcrImage for RleIndexedImage
- *(pgs)* Add implementation of ToImage for pgs image
- feat(): use `Borrow` for more generic pixel convert functions
- *(image)* add pixel convert functions
- *(pgs)* add size_hint implementation for SupParser
- *(pgs)* add size_hint and implement ExactSizeIterator
- *(pgs)* add pixels method on RleEncodedImage
- *(pgs)* add pixel color convertion with genericity
- *(pgs)* manage unexpected eof error in read_next_pixel
- *(pgs)* add decoding of Rle PGS image
- *(pgs)* set Palette in RleEncodedImage
- *(pgs)* handle custom offset in Palette for PaletteEntries
- *(pgs)* Add Palette struct to better handle PaletteEntries
- *(pgs)* add PaletteDefinitionSegment parsing.
- *(pgs)* add RleEncodedImage & impl SubtitleImage
- *(pgs)* add ODS parsing
- *(pgs)* add u24 type
- *(pgs)* add DecoderTimeImage
- *(pgs)* add segment header parsing
- *(pgs)* add ReadExt extension trait.
- *(pgs)* add SegmentTypeCode struct
- *(pgs)* add blank implementation of Iterator for SupParser
- *(pgs)* add from_file on SupParser
- *(pgs)* add PgsDecoder trait for use by SupParser
- *(pgs)* create SupParser struct
- *(typos)* add .typos.toml conf

### Fixed
- *(typos)* fix somes typos in doc, func name and data files

### Other
- add 'pgs' as keyword for crate
- *(pgs)* [**breaking**] use `seek_relative` to avoid buffer discard
- cargo update
- *(cargo)* move dependencies before lints setup
- *(error)* add error and panic documentation
- *(typos)* add `typos` step in code_check ci workflow
- *(github)* fix typo in job name of code_check.yml
- add space between `90` and `kHz`
- use word image instead of img
- fix link to VobsubParser struct
- add backticks for specifics some term

## [0.2.0](https://github.com/gwen-lg/subtile/compare/v0.1.9...v0.2.0) - 2024-07-18

### Added
- *(image)* add trait ToImage for ImageBuffer generation
- *(vobsub)* add genericity to `VobSubParser`
- *(vobsub)* add a `VobSubDecoder` impl to keep only the TimeSpan
- *(image)* add trait ToOcrImage and struct ToOcrImageOpt.
- *(image)* add trait ImageArea and impl for ImageSize types
- *(image)* add ImageSize trait and use it for VobSubIndexedImage
- *(vobsub)* move image data from Subtile struct in a dedicated
- *(vobsub)* add VobSubDecoder trait and use it ...
- *(vobsub)* [**breaking**] create VobsubParser struct
- add Default impl (derive) for time structs

### Other
- add release-plz github workflow
- *(vobsub)* remove useless `to_image` from VobSubIndexedImage
- *(vobsub)* use `VobSubToImage` in vobsub example
- *(vobsub)* create `VobSubToImage` struct who implement ToImage
- *(vobsub)* add a test to parse only subtitles times
- *(vobsub)* [**breaking**] remove Subtitle struct,
- *(vobsub)* invert order of palette and alpha value after parsing
- *(vobsub)* add VobSubOcrImage to create image addapted to OCR
- *(vobsub)* add VobSubRleImage to be used by VobSub decoders
- *(vobsub)* add struct VobSubRleImageData to ...
- *(vobsub)* create a dedicated method for sub packet reading
- *(vobsub)* move missing end_time out of iterator
- some typo fixes and backticks added
- make dump_images accept iterator of value
- remove some useless use of cast
- [**breaking**] rename SubError to SubtileError
- Add Changelog file with only header
