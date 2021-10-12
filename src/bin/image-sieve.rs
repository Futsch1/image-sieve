mod image_cache;
mod images;
mod json_persistence;
mod lru_map;
mod main_window;
mod settings;
mod synchronize;

fn main() {
    let main_window = main_window::MainWindow::new();

    main_window.run();
}
