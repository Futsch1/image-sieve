# Changelog
All notable changes to this project will be documented in this file.

## Unreleased

### Changed

 - 

### Added

 - 

### Fixed

 - Moving files from one mount point to the other was always failing

## [0.2.2] - 2021-11-14

### Changed

 - All images are now loaded in a background threads increasing GUI responsiveness
 - Improved similarity detection by using longer hashes and taking image orientation into account

### Added

 - Application icon

### Fixed

 - File item date is now the minimum of created and modified date and not only created date
 - Display file item date in local timezone
 - No longer crash when an image with either width or height 0 is loaded
 - Images were cropped in the similar images list
 - Similarities where not calculated when an image was not decodeable


## [0.2.1] - 2021-10-31

### Changed

- Folder selection edit is now disabled, since entering something there had no effect

### Added

 -

### Fixed

 - While the image similarity calculation is running, no other folder can be selected

## [0.2.0] - 2021-10-25

### Changed

 - Now using sixtyfps v0.1.4

### Added

 - Image hashing to calculate similarities in image contents
 - Settings tab for tuning the similarity calculation

### Fixed

 - If an image has many similar images, a maximum of six are displayed at the same time since the GUI was blocked otherwise
 - Fixed showing the correct text when one of the similar images was selected
 - Fixed event scrollview

## [0.1.3] - 2021-10-21

### Added

 - Renamed executable to image_sieve instead of image-sieve

### Fixed

 - Fixed crashes that could occur when an item was deleted or renamed while ImageSieve is open 

## [0.1.2] - 2021-10-10

### Added

 - Added a button to open the current item in an external viewer
 - Release to crates.io

### Fixed

 - Improved overall code style
 - Combined code into single crate

## [0.1.1] - 2021-10-10

### Added

 - Added a confirmation when sieving with deletion

### Fixed

 - Fixed GitHub action for releasing Windows binary
 - Fixed updating events


## [0.1.0] - 2021-10-09

### Added

 - Initial GitHub release