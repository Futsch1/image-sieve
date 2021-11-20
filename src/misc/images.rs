extern crate image;
extern crate sixtyfps;

use std::cmp::min;

use image::{imageops, DynamicImage, GenericImageView};

use crate::item_sort_list::FileItem;

/// Image buffer from the image crate
pub type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

/// Get an image buffer from a FileItem with a width and height constraint. If the image contains
/// an orientation indication, it is rotated accordingly.
pub fn get_image_buffer(item: &FileItem, max_width: u32, max_height: u32) -> ImageBuffer {
    let path = item.get_path();
    let rotation = match item.get_orientation() {
        Some(orientation) => match orientation {
            crate::item_sort_list::Orientation::Landscape => 0,
            crate::item_sort_list::Orientation::Portrait90 => 90,
            crate::item_sort_list::Orientation::Landscape180 => 180,
            crate::item_sort_list::Orientation::Portrait270 => 270,
        },
        None => 0,
    };
    load_image_and_rotate(path, rotation, max_width, max_height)
        .unwrap_or_else(|_| ImageBuffer::new(1, 1))
}

/// Get an empty image of the size 1x1
pub fn get_empty_image() -> sixtyfps::Image {
    let buffer = sixtyfps::SharedPixelBuffer::new(1, 1);
    sixtyfps::Image::from_rgba8(buffer)
}

/// Convert an image buffer to an image suitable for the sixtyfps library
pub fn get_sixtyfps_image(buffer: &ImageBuffer) -> sixtyfps::Image {
    if buffer.width() > 0 && buffer.height() > 0 {
        let buffer = sixtyfps::SharedPixelBuffer::<sixtyfps::Rgba8Pixel>::clone_from_slice(
            buffer.as_raw(),
            buffer.width() as _,
            buffer.height() as _,
        );
        sixtyfps::Image::from_rgba8(buffer)
    } else {
        get_empty_image()
    }
}

/// Loads an image from a path and rotates it by a given angle in degrees
fn load_image_and_rotate(
    path: &std::path::Path,
    rotate: i32,
    max_width: u32,
    max_height: u32,
) -> Result<ImageBuffer, image::ImageError> {
    let cat_image = image::open(path)?;
    Ok(process_dynamic_image(
        cat_image, rotate, max_width, max_height,
    ))
}

/// Converts a byte buffer to an image buffer
pub fn image_from_buffer(bytes: &[u8]) -> Result<ImageBuffer, image::ImageError> {
    let cat_image = image::load_from_memory(bytes)?;
    Ok(cat_image.into_rgba8())
}

/// Processes a dynamic image by rotating it and resizing it to the given width and height
fn process_dynamic_image(
    cat_image: DynamicImage,
    rotate: i32,
    max_width: u32,
    max_height: u32,
) -> ImageBuffer {
    let width = cat_image.width();
    let height = cat_image.height();
    let ratio = width as f32 / height as f32;

    let mut new_width = 0;
    let mut new_height = 0;
    if width > height && max_width > 0 {
        new_width = min(width, max_width);
        new_height = (new_width as f32 / ratio) as u32;
    }
    if height > width && max_height > 0 {
        new_height = min(height, max_height);
        new_width = (new_height as f32 * ratio) as u32;
    }

    let cat_image = cat_image.resize(new_width, new_height, imageops::FilterType::Nearest);

    let cat_image = cat_image.into_rgba8();
    match rotate {
        90 => image::imageops::rotate90(&cat_image),
        180 => image::imageops::rotate180(&cat_image),
        270 => image::imageops::rotate270(&cat_image),
        _ => cat_image,
    }
}
