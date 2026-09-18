FROM jrottenberg/ffmpeg:9.0-ubuntu

# Build deps

RUN apt-get update && \
    apt-get -y install \
    mold \
    build-essential \
    pkg-config \
    gcc \
    clang \
    curl \
    tar \
    gzip \
    bzip2 \
    bash

# Rust

RUN curl -fsSL https://sh.rustup.rs | sh -s -- --default-toolchain none -y && \
    echo "export PATH=\"\$PATH:$HOME/.cargo/bin\"" >> "$HOME/.bashrc"

SHELL [ "/bin/bash", "-c" ]

# Binaries

ARG CARGO_CHEF="0.1.78"
ARG ZIG="0.16.0"

RUN mkdir -p "/opt/chef" && \
    curl -fsSL "https://github.com/LukeMathWalker/cargo-chef/releases/download/v${CARGO_CHEF}/cargo-chef-$(uname -m)-unknown-linux-gnu.tar.xz" | \
    tar -C "/opt/chef" -Jx --strip-components 1 && \
    chmod a+rx "/opt/chef/cargo-chef" && \
    ln -sf "/opt/chef/cargo-chef" "/usr/local/bin/cargo-chef"

RUN mkdir -p "/opt/zig" && \
    curl -fsSL "https://ziglang.org/download/${ZIG}/zig-$(uname -m)-linux-${ZIG}.tar.xz" | \
    tar -C "/opt/zig" -Jx --strip-components 1 && \
    chmod a+rx "/opt/zig/zig" && \
    ln -sf "/opt/zig/zig" "/usr/local/bin/zig"

# Rust stuff

ENV PATH="$PATH:/root/.cargo/bin"

RUN rustup update --no-self-update nightly && \
    rustup component add --toolchain nightly rustfmt rust-src && \
    rustup default nightly

RUN cargo install cross --force \
    --git https://github.com/cross-rs/cross \
    --rev 29d00c7803f221f1b3f35e561b03792368fb8339

RUN curl -fsSL https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --force cargo-cross

RUN apt-get -y install \
    libfreetype6-dev \
    fontconfig libfontconfig-dev libfontconfig1 fontconfig-config fonts-dejavu-core fonts-dejavu-mono
#     libvidstab-dev libvidstab1.1 \
#     libfribidi-dev libfribidi0 \
#     libass-dev libass9 \
#     libaom-dev libaom3 \
#     libsvtav1-dev libsvtav1enc-dev libsvtav1enc1d1 libsvtav1dec-dev libsvtav1dec0 \
#     libdav1d-dev libdav1d7 \
#     libxcb-shm0-dev libxcb-shm0 libxcb-xfixes0 libxcb-xfixes0-dev \
#     x11proto-core-dev x11proto-dev \
#     libxau-dev libxau6 \
#     libpthread-stubs0-dev \
#     libxml2-dev libxml2 \
#     libbluray-dev libbluray2 \
#     libzmq3-dev libzmq5 \
#     libpng-dev libpng16-16t64 \
#     libaribb24-dev \
#     libzimg-dev libzimg2 \
#     libtheora-dev libtheora0 \
#     libssl-dev libsrt-openssl-dev libsrt1.5-openssl \
#     libbsd0 libdrm-dev libdrm2 libxcb1-dev libxcb1

# App

RUN mkdir -p /app && \
    mkdir -p /app/thirdparty/video-rs && \
    mkdir -p /output

COPY Cargo.toml /app/
COPY Cargo.lock /app/

COPY thirdparty/video-rs/Cargo.toml /app/thirdparty/video-rs/
COPY thirdparty/video-rs/Cargo.lock /app/thirdparty/video-rs/

RUN mkdir -p /app/src && \
    mkdir -p /app/thirdparty/video-rs/src && \
    touch /app/src/lib.rs && \
    touch /app/thirdparty/video-rs/src/lib.rs && \
    echo "fn main() {}" > /app/src/main.rs

WORKDIR /app

RUN cargo chef prepare --bin ultravis --recipe-path /app/recipe.json

# Target

ARG RUST_TARGET

RUN rustup target add "${RUST_TARGET}"
RUN cargo chef cook --release --bin ultravis --target "${RUST_TARGET}" --recipe-path /app/recipe.json

COPY . /app/

RUN cargo cross build --release --bin ultravis --target "${RUST_TARGET}"

RUN EXT="" && \
    if [[ "${RUST_TARGET}" = *"windows"* ]]; then EXT=".exe"; fi && \
    cp \
    "/app/target/${RUST_TARGET}/release/ultravis$EXT" \
    "/output/ultravis-${RUST_TARGET}$EXT"
