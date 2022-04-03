# Building instructions

## Build on Ubuntu

To build the software on Ubuntu, you have to install Rust first following the recommended procedure [here](https://rustup.rs/).

``` curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh ```

After the installation is complete, you need to install the following packages:
``` sudo apt install libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev libavcodec-dev libavformat-dev libavutil-dev libswscale-dev llvm ```

Clone the repository using

``` git clone https://github.com/Futsch1/image-sieve.git ```

Then run ImageSieve via

``` cargo run ```

## Build on Windows

First, install Rust following the instructions [here](https://rustup.rs/).

As a next step, you have to download the latest ffmpeg build from [here](https://github.com/Futsch1/FFmpeg-Builds/releases/download/latest/ffmpeg-n5.0-latest-win64-gpl-shared-5.0.zip). Extract the archive and set the environment variable "FFMPEG_DIR" to the extracted folder.

Clone the repository using

``` git clone https://github.com/Futsch1/image-sieve.git ```

Copy the dll files from FFMPEG_DIR/bin to the root of the checked out repository. Then run ImageSieve via

``` cargo run ```
