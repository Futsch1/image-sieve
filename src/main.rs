extern crate image_sieve;

use image_sieve::main_window;

fn main() {
    let main_window = main_window::MainWindow::new(get_max_resolution());

    main_window.run();
}

/// Determines the maximum resolution of all available monitors.
fn get_max_resolution() -> (u32, u32) {
    let mut resolution = (0, 0);
    let event_loop = winit::event_loop::EventLoop::new();
    for monitor in event_loop.available_monitors() {
        if monitor.size().width > resolution.0 || monitor.size().height > resolution.1 {
            resolution = (monitor.size().width, monitor.size().height);
        }
    }
    if resolution.0 == 0 || resolution.1 == 0 {
        (1920, 1080)
    } else {
        resolution
    }
}
