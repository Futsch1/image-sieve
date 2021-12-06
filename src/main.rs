//! GUI based tool to sort out and categorize images with the help of image similarity classification

#![windows_subsystem = "windows"]
#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unstable_name_collisions,
    unused_import_braces,
    unused_qualifications
)]

mod item_sort_list;
mod main_window;
mod misc;
mod persistence;
mod synchronize;

fn main() {
    let main_window = main_window::MainWindow::new();

    main_window.run();
}
