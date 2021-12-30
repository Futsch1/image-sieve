extern crate ffmpeg_next as ffmpeg;

use image::imageops;

use crate::item_sort_list::FileItem;

use super::images::ImageBuffer;

pub fn get_image_buffer(item: &FileItem, _: u32, _: u32) -> ImageBuffer {
    let mut output_frame = ffmpeg::util::frame::Video::empty();

    // TODO: Error handling
    let mut input_context = ffmpeg::format::input(&item.path).unwrap();
    let video_stream = input_context
        .streams()
        .best(ffmpeg::media::Type::Video)
        .unwrap();
    let stream_index = video_stream.index();
    let mut decoder = video_stream.codec().decoder().video().unwrap();
    let mut buffer = ImageBuffer::new(decoder.width() * 3, decoder.height() * 3);

    let mut i: u32 = 0;
    for (s, packet) in input_context.packets() {
        if stream_index == s.index() && packet.is_key() {
            println!(
                "Frame found: {:?} dts {:?}",
                packet.position(),
                packet.dts()
            );
            if let Some(frame) = get_frame(packet, &mut decoder) {
                println!("Decoded as: {}x{}", frame.width(), frame.height());
                let mut converter = frame
                    .converter(ffmpeg::util::format::pixel::Pixel::RGBA)
                    .ok()
                    .unwrap();
                converter.run(&frame, &mut output_frame).ok();
                let frame_buffer = ImageBuffer::from_raw(
                    output_frame.width(),
                    output_frame.height(),
                    output_frame.data(0).to_vec(),
                )
                .unwrap();
                imageops::overlay(
                    &mut buffer,
                    &frame_buffer,
                    i % 3 * output_frame.width(),
                    i / 3 * output_frame.height(),
                );
                i += 1;
                if i == 9 {
                    break;
                }
            }
        }
    }

    buffer
}

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
