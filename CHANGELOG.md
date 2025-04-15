# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/gwen-lg/subtile/compare/v0.3.2...v0.4.0) - 2025-04-15

### Added

- *(vobsub)* add lang parsing in Index file
- *(vobsub)* [**breaking**] make Index optional.
- *(vobsub)* add TimePointIdx to implement Display
- *(time)* set msecs public
- *(webvtt)* add write_line public function
- *(time)* add TimePointVtt to handle display of time for WebVTT
- *(time)* add TimePointSrt to implement display on TimePoint
- implement FusedIterator for SubParser & VobsubParser

### Fixed

- *(image)* re-export GrayImage & Luma from image crate

### Other

- *(srt)* add write_line in srt module
- *(release-plz)* update release-plz action
- *(release-plz)* disable `release-plz` workflow for fork
- add dependabot.yml for github
- use LazyLock from std instead of OnceCell
- *(clippy)* enable additional lints
- *(clippy)* add missing semicolon to the last statement
- *(vobsub)* Use Luma<u8> from image for palette.
- *(decoder)* rework decoder loop to handle parsing errors cleaner
- *(pgs)* rename `seg_header` var in `DecodeTimeImage`
- *(clippy)* directly use variables in the `format!` string
- *(cargo)* run cargo update
- *(clippy)* add parenthesis on mixed usage of arithmetic and bit shifting/combining operators
- *(lint)* allow for clippy lint `missing_const_for_fn` invalide report
- *(github)* add FUNDING.yml file with liberapay account
- *(commits)* add check-commits on push
- *(commits)* add workflow to check individual commits
- *(code_check)* use locked flag for reduce time
- *(code_check)* add rust-cache action
- *(code_check)* reorder steps with typos/fmt at first.

## [0.3.2](https://github.com/gwen-lg/subtile/compare/v0.3.1...v0.3.2) - 2025-01-12

### Added

- *(ods)* manage object data split in multiple segment

### Fixed

- *(pgs)* skip_segment content in DecodeTimeOnly

### Other

- *(release-plz)* update github action from v0.3.112
- *(ods)* create read_object_data function
- *(ods)* move ObjectDataLength reading in dedicated function
- *(ods)* move image size fields reading in a dedicated function
- *(ods)* move last_in_sequence_flag field handling into a method
- *(ods)* move object fields handling into a function
- *(pgs)* add only_one.sup fixtures and associate test

## [0.3.1](https://github.com/gwen-lg/subtile/compare/v0.3.0...v0.3.1) - 2025-01-06

### Fixed

- setup a default palette if missing in index file
- *(const)* add missing const keyword

### Other

- use secrets.GITHUB_TOKEN not RELEASE_PLZ_TOKENT
- remove useless intermediate variable in `read_palette`
- add const for `palette` key
- fix wokflows file path for triggering
- *(clippy)* remove unnecessary closure for error creation
- *(clippy)* remove explicit lifetimes than could be elided
- *(cargo)* update crate thiserror to 2.0
- *(cargo)* update dependendies
- include cargo readme management in release-plz workflow

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
