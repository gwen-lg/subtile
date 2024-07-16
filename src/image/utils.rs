use crate::SubtileError;
use image::{EncodableLayout, Pixel, PixelWithColorType};
use std::{
    borrow::Borrow,
    fs::create_dir_all,
    io,
    ops::Deref,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Handle Error for image dump.
#[derive(Error, Debug)]
pub enum DumpError {
    /// Error with path creation
    #[error("Could not create path for dump images '{}'", path.display())]
    Folder {
        /// Path of the folder
        path: PathBuf,
        /// Error source
        source: io::Error,
    },

    /// Error during file dump
    #[error("Could not write image dump file '{}'", filename.display())]
    DumpImage {
        /// Path of the file write failed
        filename: PathBuf,
        /// Error source
        source: image::ImageError,
    },
}

/// Dump some images in a folder specified by the path.
#[profiling::function]
pub fn dump_images<'a, Iter, Img, P, Container>(
    path: &str,
    images: Iter,
) -> Result<(), SubtileError>
where
    P: Pixel + PixelWithColorType + 'a,
    [P::Subpixel]: EncodableLayout,
    Container: Deref<Target = [P::Subpixel]> + 'a,
    Img: Borrow<image::ImageBuffer<P, Container>>,
    Iter: IntoIterator<Item = Img>,
{
    let folder_path = PathBuf::from(path);

    // create path if not exist
    if !folder_path.is_dir() {
        create_dir_all(folder_path.as_path()).map_err(|source| DumpError::Folder {
            path: folder_path.clone(),
            source,
        })?;
    }

    images
        .into_iter()
        .enumerate()
        .try_for_each(move |(i, img)| {
            let mut filepath = folder_path.clone();
            filepath.push(format!("{i:06}.png"));
            dump_image(&filepath, img.borrow()).map_err(|source| DumpError::DumpImage {
                filename: filepath,
                source,
            })
        })?;

    Ok(())
}

/// Dump one image
#[profiling::function]
fn dump_image<P, Pix, Container>(
    filename: P,
    image: &image::ImageBuffer<Pix, Container>, // image::Luma<u8>, Vec<u8>
) -> Result<(), image::ImageError>
where
    P: AsRef<Path>,
    Pix: Pixel + PixelWithColorType,
    [Pix::Subpixel]: EncodableLayout,
    Container: Deref<Target = [Pix::Subpixel]>,
{
    image.save(filename)
}
