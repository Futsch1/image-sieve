extern crate winres;

fn main() {
    println!("sixtyfps build");
    sixtyfps_build::compile("ui/image_sieve.60").unwrap();
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("ImageSieve.ico");
        res.compile().unwrap();
    }
}
