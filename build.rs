extern crate winres;

fn main() {
    println!("slint build");
    slint_build::compile_with_config(
        "ui/image_sieve.slint",
        slint_build::CompilerConfiguration::new().with_style(String::from("fluent")),
    )
    .unwrap();
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("ImageSieve.ico");
        res.compile().unwrap();
    }
}
