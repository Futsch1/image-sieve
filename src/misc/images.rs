extern crate image;
extern crate slint;

use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};

use super::resize::{resize_image, restrict_size};
use crate::item_sort_list::{FileItem, ItemType};

/// Image buffer from the image crate
pub type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

/// Get an image buffer from a FileItem with a width and height constraint. If the image contains
/// an orientation indication, it is rotated accordingly.
pub fn get_image_buffer(item: &FileItem, max_width: u32, max_height: u32) -> ImageBuffer {
    let image_buffer = match item.get_item_type() {
        ItemType::Image => {
            load_image_and_rotate(&item.path, get_rotation(item), max_width, max_height)
        }
        ItemType::RawImage => {
            load_raw_image_and_rotate(&item.path, get_rotation(item), max_width, max_height)
        }
        ItemType::HeifImage => {
            load_heif_image_and_rotate(&item.path, get_rotation(item), max_width, max_height)
        }
        _ => None,
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
        let decoder =
            jxl_oxide::integration::JxlDecoder::new(std::fs::File::open(path).ok()?).ok()?;
        if let Ok(image) = image::DynamicImage::from_decoder(decoder) {
            resize_and_rotate(image.to_rgba8(), rotate, max_width, max_height)
        } else {
            None
        }
    }
}

fn resize_and_rotate(
    cat_image: ImageBuffer,
    rotate: i32,
    max_width: u32,
    max_height: u32,
) -> Option<ImageBuffer> {
    let (new_width, new_height) = restrict_size(
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

    let source = imagepipe::ImageSource::Raw(raw);

    let mut pipeline = match imagepipe::Pipeline::new_from_source(source) {
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

    let image = image?;

    let dyn_img = image::DynamicImage::ImageRgb8(image);
    let rgba_image: ImageBuffer = dyn_img.into_rgba8();
    resize_and_rotate(rgba_image, rotate, max_width, max_height)
}

/// Loads a heif image from a path and rotates it by a given angle in degrees
fn load_heif_image_and_rotate(
    path: &std::path::Path,
    rotate: i32,
    max_width: u32,
    max_height: u32,
) -> Option<ImageBuffer> {
    let lib_heif = LibHeif::new();
    let ctx = match HeifContext::read_from_file(path.to_str().unwrap()) {
        Ok(ctx) => ctx,
        Err(_) => return None,
    };
    let handle = match ctx.primary_image_handle() {
        Ok(handle) => handle,
        Err(_) => return None,
    };

    // Decode the image
    let image = match lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None) {
        Ok(image) => image,
        Err(_) => return None,
    };

    let planes = image.planes();
    let interleaved = planes
        .interleaved.unwrap();

    let data = interleaved.data;
    let width = interleaved.width;
    let height = interleaved.height;
    let stride = interleaved.stride;

    let mut res: Vec<u8> = Vec::new();
    for y in 0..height {
        let mut step = y as usize * stride;

        for _ in 0..width {
            res.extend_from_slice(&[data[step], data[step + 1], data[step + 2], data[step + 3]]);
            step += 4;
        }
    }
    let buf = match image::ImageBuffer::from_vec(width, height, res) {
        Some(buf) => buf,
        None => return None,
    };
    return resize_and_rotate(buf, rotate, max_width, max_height);
}

/// Converts a byte buffer to an image buffer
pub fn image_from_buffer(bytes: &[u8]) -> Result<ImageBuffer, image::ImageError> {
    let cat_image = image::load_from_memory(bytes)?;
    Ok(cat_image.into_rgba8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_image() {
        let img = load_image_and_rotate(
            std::path::Path::new("tests/test.jpg"),
            0,
            1000,
            1000,
        );
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);

        let img = load_image_and_rotate(
            std::path::Path::new("tests/test.jxl"),
            0,
            1000,
            1000,
        );
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width(), 50);
        assert_eq!(img.height(), 34);
    }

    #[test]
    fn test_load_heif_image() {
        let img = load_heif_image_and_rotate(
            std::path::Path::new("tests/test.heif"),
            0,
            1000,
            1000,
        );
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width(), 50);
        assert_eq!(img.height(), 34);

        let img = load_heif_image_and_rotate(
            std::path::Path::new("tests/test.heif"),
            90,
            1000,
            1000,
        );
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width(), 34);
        assert_eq!(img.height(), 50);
    }

    #[test]
    fn test_load_raw_image() {    
        let img = load_raw_image_and_rotate(
            std::path::Path::new("tests/test.nef"),
            180,
            1000,
            1000,
        );
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width(), 1000);
        assert_eq!(img.height(), 656);
    }
}
