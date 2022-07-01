#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate image_sieve;

use image_sieve::main_window;

fn main() {
    let main_window = main_window::MainWindow::new();

    main_window.run();
}
