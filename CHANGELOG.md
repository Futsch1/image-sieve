# Changelog
All notable changes to this project will be documented in this file.

## [0.6.0] - Pending

### Changed

 - Save and restore window position and size on certain platforms
 
## [0.5.11] - 2022-12-22

### Fixed

 - Similarity calculation was broken for some time, fixed it

## [0.5.10] - 2022-12-21

### Fixed

 - Build automation fixes

## [0.5.9] - 2022-12-21

### Changed

 - Updated to slint 0.3.3, updated further dependencies
 - Removed setting for dark mode, only automatic is working now (restriction from slint)
 - Dark mode has a differnt color set now

## [0.5.8] - 2022-09-10

### Changed

 - Updated to slint 0.2.5
 - Updated snap package
 - Minor GUI tweaks

## [0.5.7] - 2022-07-01

### Fixed

 - When an file name would exist twice in the sieve target directory but the files are different, the files are now automatically
   renamed instead of not copied.

## [0.5.6] - 2022-05-15

### Changed

 - Updated to slint 0.2.4

### Added

 - Added dark mode

## [0.5.5] - 2022-05-08

### Changed

 - Updated to slint 0.2.2, supporting tab focus

### Added

 - Added a button to speed up creating an event from an image filling the event date to the image date.

## [0.5.4] - 2022-04-03

### Fixed

 - Reverted getting max image size from largest connected monitor - this caused a crash in the Windows build

## [0.5.3] - 2022-04-02

### Changed

 - Max image size for caching now depends on the resolution of the largest connected monitor
 - Updated to ffmpeg 5.0
 - Updated to slint 0.2.1

### Fixed

 - Fixed crash when reading file with invalid EXIF date

## [0.5.2] - 2022-02-14

### Added

 - Support for mpeg video files
 - Option to sort as year and month in subdirectories

### Fixed

 - Always force fluent style to avoid issues with Qt backend

## [0.5.1] - 2022-02-13

### Changed

 - Changed trash icon

### Added

 - Similars icon in similar images list

### Fixed

 - Removed animated gif from README.md due to size constraints of cargo.io

## [0.5.0] - 2022-02-13

### Changed

 - Renamed all sixtyfps instances to slint
 - Reworked UI and replaced next, open and previous button with on-image buttons, added animations

### Added

 - Support for many raw formats
 - Up and down keys navigate in similar images
 - Enter key opens item

### Fixed

 - Key commands are now working properly after switching tabs, also they have no effect on other tabs than the sort tab
 - Several crashes or weird behavior when images list is empty
 - Discarded status is now properly updated in image caption

## [0.4.2] - 2022-02-05

### Changed

 - Resize video preview image internally to save memory
 - Optimized image loading time

### Added

 - Snap support for removable media and network if permissions are set

### Fixed

 - Do not pass default folder to dir picker if it does not exist
 - Fixed crash when selecting folder without images

## [0.4.2] - 2022-01-09

### Changed

 - Now reading EXIF files for all image formats, not only for JPEGs
 - Optimized loading of similar images
 - Events now have an update button to be able to move an event after or before another event

### Added

 - Filters and sort options to the item list
 - Some symbols in the GUI

## [0.4.1] - 2022-01-05

### Changed

 - Target directory input field is now always disabled

### Fixed

 - Fixed layout in events view

## [0.4.0] - 2022-01-03

### Changed

 - Layout of commit result list improved

### Added

 - Preview for video files
 - Get taken date from video files metadata
 - Showing also mov files now

### Fixed

 - Fixed selected image after changing folder

## [0.3.3] - 2021-12-29

### Changed

 - C runtime is now linked statically for Windows

### Added

 - Setting to select the target directory name pattern

### Fixed

 - Only able to commit when the target directory is set

## [0.3.2] - 2021-12-23

### Changed

 - Performance and robustness improvements
 - Improved layout of events tab

### Added

 - About information in settings tab
 - Help tab

### Fixed

 - Added a space between file info and event name

## [0.3.1] - 2021-12-12

### Changed

 - No directory is now the default
 - Performance checking for files significantly improved

### Added

 - Checking for images can now be cancelled
 - Events are now sorted by date
 - Error message is shown when an event was edited

### Fixed

 - When selecting a folder without images, the similar images model is now cleared
 - Fixed a crash when a folder was selected with insufficient rights to access

## [0.3.0] - 2021-12-06

### Changed

 - Settings are now stored in the home directory. As a consequence, settings from previous versions are lost

### Added

 - Sieving operations are now displayed in detail
 - Generate a MSI installer package for Windows
 - Generate a snap package for Linux

### Fixed

 - Console window is hidden in Windows version


## [0.2.4] - 2021-11-27

### Added

 - Events are now checked for overlapping dates

### Fixed

 - Start date of an event must now be before or equal to the end date

## [0.2.3] - 2021-11-21

### Added

 - Showing result of commit operation now

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