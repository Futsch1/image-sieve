## update and install some things we should probably have
apt-get update
apt-get install -y \
  curl \
  git \
  gnupg2 \
  jq \
  sudo \
  zsh \
  vim \
  build-essential \
  openssl \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libgtk-3-dev \
  libavcodec-dev \
  libavformat-dev \
  libavutil-dev \
  libswscale-dev \
  clang \
  dbus-x11 \
  fonts-noto-color-emoji

## Install rustup and common components
curl https://sh.rustup.rs -sSf | sh -s -- -y 
source "$HOME/.cargo/env"
rustup component add rustfmt
rustup component add clippy 
rustup component add llvm-tools-preview
