![Build](https://github.com/Futsch1/image-sieve/workflows/Build/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/image_sieve.svg)](https://crates.io/crates/image_sieve)

# ImageSieve
GUI based tool to sort images based on taken date and similarity, categorize them according to their creation date and archive them in a target folder.

![Screenshot](doc/walkthrough.gif?raw=true "ImageSieve")

ImageSieve aims to help in the process of sorting through images and videos taken with several devices and collect the images and videos worth keeping in a folder structure suitable for archiving or to eliminate potential duplicates or irrelevant images .

ImageSieve also includes a mechanism to automatically categorize images according to user defined events based on their taken dates. The sorting progress is stored on a per-folder basis, so that it can be resumed later. Images can be analyzed for similarity based on the taken date and on an image similarity algorithm.

The idea of working with ImageSieve is as follows: Copy all the images and videos taken in a period of time with various devices to a single folder or folder structure. Open this folder with ImageSieve and select which items to discard. Create events for the sorting period and finally move the selected items to your image archive folder.

## Features
- Supports plenty of image formats (jpg, tiff, gif, bmp, webp, png), many raw image formats and videos in mp4, avi, mov and mts format
- Browse images and videos from a folder structure in the order of their creation
- Select which images to keep and which to discard
- Images which resemble each other and images being taken within a customizable number of seconds are considered similar and are highlighted to support sorting
- Manage events with a name, a start and an end date for the images to sort to automatically assign the images to an event
- Discarded images and events are saved so that the sorting process can resume later
- Sieve the images by either deleting discarded ones, copying or moving kept images to a target folder

## Installation
A Windows installer or a portable zip is available for every release for [download](https://github.com/Futsch1/image-sieve/releases) or the app can be installed via the [Microsoft Store](https://www.microsoft.com/en-us/p/imagesieve/9nwlt9phl39d). For Linux, ImageSieve is
available on the [Snap Store](https://snapcraft.io/image-sieve).

On Windows, Linux or macOS, it is also possible to install [Rust](https://rustup.rs/), clone the repository and run

``` cargo install image_sieve ```

After the compilation, you can run the tool by typing

``` image_sieve ```

## Operation

### 📷 📹 Images
To get started, first open a folder containing images and videos in the "📷 📹  Images" tab. A folder can be selected by pressing the "📂 Browse..." button. All images and videos from the folder and from all subfolders will be analyzed. Depending on the amount of images, this might take a while. Note that the last selected folder will be re-opened when ImageSieve is started for the next time.

Once the folder has been processed, a list of file names will appear in the box to the right. This list contains the files that have been found in the folder and that will be considered in the sieving process. Each file has a set of icons that indicate its state. 

The following icons are used (exact rendering depends on platform/font):

- 📷: The file is an image
- 📹: The file is a video
- 🗑: The file is discarded
- 🔀: There are similar files to this one
- 📅: File is in the date range of an event

To select a file, click it and it will be shown in the image area. Below the image, some details about the file are listed. In order to discard an image, just click the upper part of it and it will be displayed in a translucent way. As an alternative, you can hit the space bar to toggle between discarded and kept state. To navigate between images, click on the left or right side of the image or hit the left and right key on your keyboard.
If you want to open an image or a video with the default application in your OS, click the lower part of the image or press the "Enter" key.

If an image belongs to a group of similar images, all these similar images are displayed below the current image. The currently selected one is highlighted in blue. To navigate between similar images, you can use the up and down key.
![Screenshot](doc/screenshot.png?raw=true "ImageSieve")

Note that video files are also displayed in the list of images and previewed as a 3x3 matrix of screenshots. Similiarities are not calculated for video files.
![Screenshot](doc/screenshot2.png?raw=true "ImageSieve")

### 📅 Events
Per default, the images will be sorted in folders corresponding to the months they were taken, like "09-2021", "10-2021" etc. To be able to find images more quickly in an archive, ImageSieve supports grouping pictures with the help of events in the "Events" tab. Events are named date spans that will provide a target folder name during the sieve process, like "2021-10-07 - 2021-10-10 Cool trip". All images taken in the given period of time will be put into that folder. You can specify an arbitrary number of events, but be aware that in case of overlapping dates, an image is put into the folder of the first matching event.

To add an event, fill the start date, end date and name text box and click the "➕ Add" button. Valid date formats are YYYY-MM-DD or DD.MM.YYYY. You can edit existing events by modifying their fields and pressing enter - the updated values will be taken over and be visible in the event's caption when you click the "💾 Update" button. To remove an event, click the "🗑 Remove" button.
The time spans of events must not overlap.

Be aware that the events are saved in the currently selected folder along with the selection of images.
![Screenshot](doc/screenshot3.png?raw=true "ImageSieve")

### 💾 Sieve
When you are done sorting the images, the sieving process can be started. Go to the "💾  Sieve" tab and select a sieving mode. The following modes are supported:

- Copy to target directory: Copies only the kept items to the target directory creating folders for the items, the source directory will be left untouched.
- Move to target directory: Moves the kept items to the target directory creating folders for the items, effectively removing them from the source directory. Discarded items will stay in the source directory.
- Move to target directory and delete in source directory: Moves the kept items to the target directory creating folders for the items and deletes discarded items in the source directory. If the source directory contained only images and videos, it will be empty afterwards (except for sub folders).
- Delete in source directory: Deletes all discarded items in the source directory.

Depending on the mode, you need to indicate a target directory that is used for the result of the sieving process. Once you are done, click the "✅ Start" button and the sieve process will start.
![Screenshot](doc/screenshot4.png?raw=true "ImageSieve")

### ⚙ Settings
In the settings tab, you can specify the behavior of the similarity detection process. You can turn on and off both the use of the file/capture date as an indicator for similarity and the similarity calculation.

Note that the similarity calculation takes some time and will not be available right from the start of the tool, especially if the number of files is huge. The similarity can be tweaked in order to provide better results.

## Misc
ImageSieve is published under [GPL-3.0](https://github.com/Futsch1/image-sieve/blob/main/LICENSE).

If you want to help, clone and pull-request. The tool uses the [slint](https://github.com/slint-ui/slint) GUI framework and a few of the great [bootstrap icons](https://icons.getbootstrap.com/). For previewing video files, [FFmpeg](https://ffmpeg.org) is used.

## Disclaimer
This tool is free software. The author does not take any responsibility or liability for data lost due to bugs or faulty use of the software. Note that the software is in constant development and may contain bugs. Use at your own risk!
