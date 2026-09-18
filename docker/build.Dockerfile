FROM jrottenberg/ffmpeg:9.0-ubuntu

RUN apt-get update && \
    apt-get -y install \
    mold \
    build-essential \
    pkg-config \
    gcc \
    clang \
    libfontconfig1-dev \
    curl \
    tar \
    gzip \
    bzip2 \
    bash

RUN curl -fsSL https://sh.rustup.rs | sh -s -- --default-toolchain none -y && \
    echo "$HOME/.cargo/bin" >> "$HOME/.bashrc"

SHELL [ "/bin/bash", "-c" ]

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

RUN rustup update --no-self-update nightly && \
    rustup component add --toolchain nightly rustfmt rust-src && \
    rustup default nightly

RUN cargo install cross --force \
    --git https://github.com/cross-rs/cross \
    --rev 29d00c7803f221f1b3f35e561b03792368fb8339

ARG RUST_TARGET

RUN rustup target add "${RUST_TARGET}"

