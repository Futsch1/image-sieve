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
        println!("cargo::rerun-if-changed=ImageSieve.ico");

        println!("cargo::rustc-link-lib=strmiids");
        println!("cargo::rustc-link-lib=mfuuid");
        println!("cargo::rustc-link-lib=mfplat");
    }
}
