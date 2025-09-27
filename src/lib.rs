#![deny(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unstable_name_collisions,
    unused_import_braces,
    unused_qualifications
)]

//! Image_sieve crate providing a GUI based tool to sort out images based on similarity, categorize them according
//! to their creation date and archive them in a target folder.
mod controller;
mod item_sort_list;
pub mod main_window;
mod misc;
mod persistence;
mod synchronize;
