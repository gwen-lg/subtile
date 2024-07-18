# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
