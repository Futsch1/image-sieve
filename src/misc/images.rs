extern crate image;
extern crate slint;

use super::resize::resize_image;
use crate::item_sort_list::FileItem;
use std::cmp::max;

/// Image buffer from the image crate
pub type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

/// Get an image buffer from a FileItem with a width and height constraint. If the image contains
/// an orientation indication, it is rotated accordingly.
pub fn get_image_buffer(item: &FileItem, max_width: u32, max_height: u32) -> ImageBuffer {
    let image_buffer = if item.is_image() {
        load_image_and_rotate(&item.path, get_rotation(item), max_width, max_height)
    } else {
        load_raw_image_and_rotate(&item.path, get_rotation(item), max_width, max_height)
    };

    image_buffer.unwrap_or_else(|| ImageBuffer::new(1, 1))
}

/// Return the rotation in degrees from a file item
pub fn get_rotation(item: &FileItem) -> i32 {
    match item.get_orientation() {
        Some(orientation) => match orientation {
            crate::item_sort_list::Orientation::Landscape => 0,
            crate::item_sort_list::Orientation::Portrait90 => 90,
            crate::item_sort_list::Orientation::Landscape180 => 180,
            crate::item_sort_list::Orientation::Portrait270 => 270,
        },
        None => 0,
    }
}

/// Get an empty image of the size 1x1
pub fn get_empty_image() -> slint::Image {
    let buffer = slint::SharedPixelBuffer::new(1, 1);
    slint::Image::from_rgba8(buffer)
}

/// Convert an image buffer to an image suitable for the slint library
pub fn get_slint_image(buffer: &ImageBuffer) -> slint::Image {
    if buffer.width() > 0 && buffer.height() > 0 {
        let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
            buffer.as_raw(),
            buffer.width(),
            buffer.height(),
        );
        slint::Image::from_rgba8(buffer)
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
) -> Option<ImageBuffer> {
    if let Ok(image) = image::open(path) {
        resize_and_rotate(image.to_rgba8(), rotate, max_width, max_height)
    } else {
        None
    }
}

fn resize_and_rotate(
    cat_image: ImageBuffer,
    rotate: i32,
    max_width: u32,
    max_height: u32,
) -> Option<ImageBuffer> {
    let (new_width, new_height) = get_size(
        (cat_image.width(), cat_image.height()),
        (max_width, max_height),
    );
    if let Ok(cat_image) = resize_image(cat_image, new_width, new_height) {
        Some(match rotate {
            90 => image::imageops::rotate90(&cat_image),
            180 => image::imageops::rotate180(&cat_image),
            270 => image::imageops::rotate270(&cat_image),
            _ => cat_image,
        })
    } else {
        None
    }
}

/// Loads a raw image from a path and rotates it by a given angle in degrees
fn load_raw_image_and_rotate(
    path: &std::path::Path,
    rotate: i32,
    max_width: u32,
    max_height: u32,
) -> Option<ImageBuffer> {
    let raw = match rawloader::decode_file(path) {
        Ok(raw) => raw,
        Err(_) => return None,
    };

    let width = raw.width;
    let height = raw.height;
    let source = imagepipe::ImageSource::Raw(raw);

    let mut pipeline = match imagepipe::Pipeline::new_from_source(source, width, height, true) {
        Ok(pipeline) => pipeline,
        Err(_) => return None,
    };

    pipeline.run(None);
    let image = match pipeline.output_8bit(None) {
        Ok(image) => image,
        Err(_) => return None,
    };

    let image = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(
        image.width as u32,
        image.height as u32,
        image.data,
    );

    let image = match image {
        Some(image) => image,
        None => return None,
    };

    let dyn_img = image::DynamicImage::ImageRgb8(image);
    let rgba_image: ImageBuffer = dyn_img.into_rgba8();
    resize_and_rotate(rgba_image, rotate, max_width, max_height)
}

/// Converts a byte buffer to an image buffer
pub fn image_from_buffer(bytes: &[u8]) -> Result<ImageBuffer, image::ImageError> {
    let cat_image = image::load_from_memory(bytes)?;
    Ok(cat_image.into_rgba8())
}

/// Get the actual size from the current size and the max size
pub fn get_size((width, height): (u32, u32), (max_width, max_height): (u32, u32)) -> (u32, u32) {
    if width > max_width && height > max_height && max_height != 0 && max_width != 0 {
        let wratio = max_width as f32 / width as f32;
        let hratio = max_height as f32 / height as f32;

        let ratio = f32::min(wratio, hratio);

        let new_width = max((width as f32 * ratio).round() as u32, 1);
        let new_height = max((height as f32 * ratio).round() as u32, 1);
        (new_width, new_height)
    } else {
        (width, height)
    }
}
