use std::cmp::max;

use fast_image_resize::{
    images::Image, ImageBufferError, MulDivImagesError, PixelType, Resizer
};

use super::images::ImageBuffer;

/// We do not really care about the underlying error, so wrap all fast_image_resize errors to a single type
#[derive(Debug)]
pub enum ResizeImageError {
    Error,
}

impl From<ImageBufferError> for ResizeImageError {
    fn from(_: ImageBufferError) -> Self {
        ResizeImageError::Error
    }
}

impl From<MulDivImagesError> for ResizeImageError {
    fn from(_: MulDivImagesError) -> Self {
        ResizeImageError::Error
    }
}

/// Resize an image buffer with the nearest neighbor method
pub fn resize_image(
    src_image: ImageBuffer,
    new_width: u32,
    new_height: u32,
) -> Result<ImageBuffer, ResizeImageError> {
    let src_image = Image::from_vec_u8(
        src_image.width(),
        src_image.height(),
        src_image.to_vec(),
        PixelType::U8x4,
    )?;

    let mut dst_image = Image::new(
        new_width,
        new_height,
        src_image.pixel_type(),
    );
    let mut fast_resizer = Resizer::new();

    let result = fast_resizer.resize(&src_image, &mut dst_image, None);

    if result.is_err() {
        return Err(ResizeImageError::Error);
    }
    Ok(ImageBuffer::from_raw(new_width, new_height, dst_image.buffer().to_vec()).unwrap())
}

/// Get the actual size from the current size and the max size
/// If either width nor height is smaller or equal to max_width and max_height, the new size is
/// reduced to the larger of the two. If one of the max values is set to 0, the size in that dimension
/// is not restricted
pub fn restrict_size(
    (width, height): (u32, u32),
    (max_width, max_height): (u32, u32),
) -> (u32, u32) {
    if (width > max_width || height > max_height) && (max_width != 0 || max_height != 0) {
        let wratio = if max_width > 0 {
            max_width as f32 / width as f32
        } else {
            f32::MAX
        };
        let hratio = if max_height > 0 {
            max_height as f32 / height as f32
        } else {
            f32::MAX
        };

        let ratio = f32::min(wratio, hratio);

        let new_width = max((width as f32 * ratio).round() as u32, 1);
        let new_height = max((height as f32 * ratio).round() as u32, 1);
        (new_width, new_height)
    } else {
        (width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize() {
        let image_buffer = ImageBuffer::new(100, 100);
        let result = resize_image(image_buffer, 200, 100);
        assert!(result.is_ok());
        let resized_image = result.unwrap();
        assert_eq!(resized_image.width(), 200);
        assert_eq!(resized_image.height(), 100);

        let image_buffer = ImageBuffer::new(100, 100);
        let result = resize_image(image_buffer, 100, 200);
        assert!(result.is_ok());
        let resized_image = result.unwrap();
        assert_eq!(resized_image.width(), 100);
        assert_eq!(resized_image.height(), 200);
    }

    #[test]
    fn test_get_size() {
        let size = restrict_size((100, 100), (100, 100));
        assert_eq!(size, (100, 100));

        let size = restrict_size((1000, 1000), (100, 100));
        assert_eq!(size, (100, 100));

        let size = restrict_size((10, 10), (100, 100));
        assert_eq!(size, (10, 10));

        let size = restrict_size((100, 50), (100, 100));
        assert_eq!(size, (100, 50));

        let size = restrict_size((50, 100), (100, 100));
        assert_eq!(size, (50, 100));

        let size = restrict_size((200, 60), (100, 100));
        assert_eq!(size, (100, 30));

        let size = restrict_size((200, 60), (100, 0));
        assert_eq!(size, (100, 30));

        let size = restrict_size((60, 150), (100, 100));
        assert_eq!(size, (40, 100));

        let size = restrict_size((60, 150), (0, 100));
        assert_eq!(size, (40, 100));

        let size = restrict_size((200, 400), (100, 100));
        assert_eq!(size, (50, 100));

        let size = restrict_size((400, 200), (100, 100));
        assert_eq!(size, (100, 50));

        let size = restrict_size((400, 200), (0, 0));
        assert_eq!(size, (400, 200));
    }
}
