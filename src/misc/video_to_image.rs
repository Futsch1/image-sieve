extern crate ffmpeg_next as ffmpeg;

use crate::item_sort_list::FileItem;

use super::images::ImageBuffer;

pub fn get_image_buffer(item: &FileItem, _: u32, _: u32) -> ImageBuffer {
    let mut frame = ffmpeg::util::frame::Video::empty();
    let mut output_frame = ffmpeg::util::frame::Video::empty();

    // TODO: Error handling
    let mut input_context = ffmpeg::format::input(&item.path).unwrap();
    let stream_index = input_context
        .streams()
        .best(ffmpeg::media::Type::Video)
        .unwrap()
        .index();

    let mut decoder = input_context
        .streams()
        .best(ffmpeg::media::Type::Video)
        .unwrap()
        .codec()
        .decoder()
        .video()
        .unwrap();
    for (s, packet) in input_context.packets() {
        if stream_index == s.index() && packet.is_key() {
            decoder.send_packet(&packet).ok();
            decoder.receive_frame(&mut *frame).ok();
            break;
        }
    }

    let mut converter = frame
        .converter(ffmpeg::util::format::pixel::Pixel::RGBA)
        .ok()
        .unwrap();
    converter.run(&frame, &mut output_frame).ok();

    ImageBuffer::from_raw(
        output_frame.width(),
        output_frame.height(),
        output_frame.data(0).to_vec(),
    )
    .unwrap()
}
