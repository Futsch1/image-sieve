#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate image_sieve;
extern crate backtrace;

use std::io::Write;
use std::{panic, fs::File};
use backtrace::Backtrace;

use image_sieve::main_window;

fn main() {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let backtrace = Backtrace::new();
        let trace_filename = main_window::get_trace_filename();
        if let Ok(mut trace_file) = File::create(trace_filename) {
            let mut buf = Vec::new();
            write!(buf, "{}\n\n", panic_info).ok();
            write!(buf, "{:?}", backtrace).ok();
            trace_file.write_all(&buf).ok();
        }
        
        prev(panic_info);
    }));
    let main_window = main_window::MainWindow::new();

    main_window.run();
}
