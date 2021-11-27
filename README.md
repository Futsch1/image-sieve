![Build](https://github.com/Futsch1/image-sieve/workflows/Build/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/image_sieve.svg)](https://crates.io/crates/image_sieve)

# ImageSieve
GUI based tool to sort out images based on similarity, categorize them according to their creation date and archive them in a target folder.

![Screenshot](doc/screenshot.png?raw=true "ImageSieve")

ImageSieve aims to help in the process of sorting through images taken by several people with several devices and collect the images worth keeping in a folder structure suitable for achiving or to eliminate potential duplicates or irrelevant images from a folder structure. The tool runs on Windows, Linux and macOS.

## Features
- Browse images in jpg, tiff or png format and videos in mp4, avi and mts format from a folder structure in the order of their creation
- Select which images to keep and which to discard
- Images which resemble each other and images being taken within a customizable number of seconds are considered similar and are highlighted to support sorting
- Manage events with a name, a start and an end date for the images to sort to automatically assign the images to an event
- Discarded images and events are saved so that the sorting process can resume later
- Sieve the images by either deleting discarded ones, copying or moving kept images to a target folder

## Installation
Precompiled Windows binaries are available for every release for [download](https://github.com/Futsch1/image-sieve/releases). Linux and macOS users need to install [Rust](https://rustup.rs/), clone the repository and run

``` cargo install image_sieve ```

After the compilation, you can run the tool by typing

``` image_sieve ```

## Operation

### Sorting
To get started, first select a folder containing images in the "Sort" tab using the upper right "Browse" button. ImageSieve will now search for files and order them according to their creation date. Depending on the amount of images, this might take a while.
Once the folder has been processed, a list of file names will appear in the box to the right. This list contains the files that have been found in the folder and that will be considered in the sieving process. Each file has a set of icons that indicate its state. 
The following icons are used (exact rendering depends on platform/font):
- &#x1F4F7;: The file is an image
- &#x1F4F9;: The file is a video
- &#x1F5D1;: The file is discarded
- &#x1F500;: There are similar files to this one
- &#x1F4C5;: File is in the date range of an event

To select a file, click it and it will be shown in the image area. Below the image, some details about the file are listed. In order to discard an image, just click it and it will be displayed in a translucent way. As an alternative, you can hit the space bar to toggle between discarded and kept state. To navigate between images, you can use the "Previous" and "Next" buttons or hit the left and right key on your keyboard.
If you want to open an image or a video with the default application in your OS, click the "Open" button.
If an image belongs to a group of similar images, all these similar images are displayed below the current image. The currently selected one is highlighted in blue.
Note that video files are also displayed in the list of images, but cannot be previewed. However, they are also kept or discarded in the sieving process as are the image files.

### Events
Per default, the images will be sorted in folders corresponding to the months they were taken, like "09-2021", "10-2021" etc. To be able to find images more quickly in an archive, ImageSieve supports grouping pictures with the help of events in the "Events" tab. Events are named date spans that will provide a target folder name during the sieve process, like "2021-10-07 - 2021-10-10 Cool trip". All images taken in the given period of time will be put into that folder. You can specify an arbitrary number of events, but be aware that in case of overlapping dates, an image is put into the folder of the first matching event.
To add an event, fill the start date, end date and name text box and click the "Add" button. Valid date formats are YYYY-MM-DD or DD.MM.YYYY. You can edit existing events by modifying their fields and pressing enter - the updated values will be visible in the event's caption. To remove an event, click the "Remove" button.
The time spans of events must not overlap.
Be aware that the events are saved in the currently selected folder along with the selection of images.

### Settings
In the settings tab, you can specify the behavior of the similarity detection process. You can turn on and off both the use of the file/capture
date as an indicator for similarity and the similarity calculation.
Note that the similarity calculation takes some time and will not be available right from the start of the tool, especially if the number of files is huge. The similarity can be tweaked in order to provide better results.

### Sieve
When you are done sorting the images, the sieving process can be started. Go to the "Sieve" tab and select a sieving mode. The following modes are supported:
- Copy to target directory: Copies only the kept items to the target directory creating folders for the items, the source directory will be left untouched.
- Move to target directory: Moves the kept items to the target directory creating folders for the items, effectively removing them from the source directory. Discarded items will stay in the source directory.
- Move to target directory and delete in source directory: Moves the kept items to the target directory creating folders for the items and deletes discarded items in the source directory. If the source directory contained only images and videos, it will be empty afterwards (except for subfolders).
- Delete in source directory: Deletes all discarded items in the source directory.

Depending on the mode, you need to indicate a target directory that is used for the result of the sieving process. Once you are done, click the start button and the sieve process will start.

## Known issues and TODOs
- The indication of problems during sieving is not implemented
- Navigation with previous/next through similar items can be improved

## Details
This is the first software I've ever written in Rust, so there might be room for improvement. If you want to help, clone and pull-request.
The tool uses the [sixtyfps](https://github.com/sixtyfpsui/sixtyfps) GUI framework and a few of the great [bootstrap icons](https://icons.getbootstrap.com/).

## Disclaimer
This tool is free software. The author does not take any responsibility or liability for data lost due to bugs or faulty use of the software. Note that the software is in constant development and may contain bugs. Use at your own risk!
