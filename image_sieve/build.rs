extern crate embed_resource;

fn main() {
    println!("sixtyfps build");
    sixtyfps_build::compile("ui/image_sieve.60").unwrap();
    println!("embed resource");
    embed_resource::compile("ImageSieve.rc");
}
