extern crate ffmpeg_next as ffmpeg;

use image::imageops;

use crate::item_sort_list::{FileItem, Orientation};

use super::images::ImageBuffer;
const VIDEO_PNG: &[u8; 2900] = include_bytes!("video.png");

/// Construct an image for a video by combining 9 frames from the video.
pub fn get_image_buffer(item: &FileItem, _: u32, _: u32) -> ImageBuffer {
    create_image_from_video(item).unwrap_or_else(|_| get_alternative_image())
}

/// Get the alternative image of a video camera
fn get_alternative_image() -> ImageBuffer {
    crate::misc::images::image_from_buffer(VIDEO_PNG).unwrap()
}

/// Get the position of a frame in the 3x3 matrix depending on the orientation of the video
fn get_position(orientation: Option<&Orientation>, i: u32, width: u32, height: u32) -> (u32, u32) {
    if let Some(orientation) = orientation {
        match orientation {
            crate::item_sort_list::Orientation::Landscape => (i % 3 * width, i / 3 * height),
            crate::item_sort_list::Orientation::Portrait90 => (i / 3 * width, (2 - i % 3) * height),
            crate::item_sort_list::Orientation::Landscape180 => {
                ((2 - i % 3) * width, (2 - i / 3) * height)
            }
            crate::item_sort_list::Orientation::Portrait270 => {
                ((2 - i / 3) * width, i % 3 * height)
            }
        }
    } else {
        (i % 3 * width, i / 3 * height)
    }
}

/// Create the 3x3 frames image from a video
fn create_image_from_video(item: &FileItem) -> Result<ImageBuffer, ffmpeg::Error> {
    let mut input_context = ffmpeg::format::input(&item.path)?;
    if let Some(video_stream) = input_context.streams().best(ffmpeg::media::Type::Video) {
        let stream_index = video_stream.index();
        let mut decoder = video_stream.codec().decoder().video()?;
        let mut buffer = ImageBuffer::new(decoder.width() * 3, decoder.height() * 3);
        let orientation = item.get_orientation();
        let seek_step_us = input_context.duration() / 9;

        let mut i: u32 = 0;
        let mut last_packet_position: isize = isize::MIN;

        // Make nine steps in the video file
        for step in 0..9 {
            let seek_ts = step * seek_step_us;

            if input_context.seek(seek_ts, seek_ts..).is_err() {
                break;
            }
            for (s, packet) in input_context.packets() {
                // Read packets until a key frame was found
                if stream_index == s.index() && packet.is_key() {
                    // Only decode the packet if it is not the same as the last one
                    if packet.position() > last_packet_position {
                        last_packet_position = packet.position();
                        // Try to decode the packet to a frame
                        if let Some(frame) = get_frame(packet, &mut decoder) {
                            let (x, y) =
                                get_position(orientation, i, frame.width(), frame.height());
                            // And finally put the frame into the output buffer
                            frame_to_buffer(&frame, &mut buffer, (x, y));
                            i += 1;
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        // Rotate the image if necessary
        if let Some(orientation) = orientation {
            match orientation {
                crate::item_sort_list::Orientation::Landscape => {}
                crate::item_sort_list::Orientation::Portrait90 => {
                    buffer = image::imageops::rotate90(&buffer);
                }
                crate::item_sort_list::Orientation::Landscape180 => {
                    buffer = image::imageops::rotate180(&buffer);
                }
                crate::item_sort_list::Orientation::Portrait270 => {
                    buffer = image::imageops::rotate270(&buffer);
                }
            };
        }

        Ok(buffer)
    } else {
        Err(ffmpeg::Error::StreamNotFound)
    }
}

/// Gets a frame from a packet.
fn get_frame(
    packet: ffmpeg::Packet,
    decoder: &mut ffmpeg::decoder::Video,
) -> Option<ffmpeg::util::frame::Video> {
    let mut frame = ffmpeg::util::frame::Video::empty();
    decoder.send_packet(&packet).ok();
    while decoder.receive_frame(&mut *frame).is_ok() {
        if frame.width() > 0 && frame.height() > 0 {
            return Some(frame);
        }
    }
    None
}

/// Put a frame into the 3x3 matrix image buffer
fn frame_to_buffer(
    frame: &ffmpeg::util::frame::Video,
    buffer: &mut ImageBuffer,
    position: (u32, u32),
) {
    let mut output_frame = ffmpeg::util::frame::Video::empty();
    let mut converter = frame
        .converter(ffmpeg::util::format::pixel::Pixel::RGBA)
        .ok()
        .unwrap();
    converter.run(frame, &mut output_frame).ok();
    let frame_buffer = ImageBuffer::from_raw(
        output_frame.width(),
        output_frame.height(),
        output_frame.data(0).to_vec(),
    )
    .unwrap();
    imageops::overlay(buffer, &frame_buffer, position.0, position.1);
}
