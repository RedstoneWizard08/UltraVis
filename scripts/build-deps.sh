#!/bin/bash

set -euo pipefail

if [[ -z "$1" ]]; then
    echo "Usage: $0 [target]"
    exit 1
fi

TARGET="$1"
CWD="$(pwd)"
PREFIX="$CWD/cross/prefixes/$TARGET"

[[ ! -d "$CWD/cross/source/freetype" ]] && \
    git clone --depth 1 -b VER-2-14-3 https://gitlab.freedesktop.org/freetype/freetype "$CWD/cross/source/freetype"

meson setup \
    --reconfigure \
    --prefix "$PREFIX" \
    --buildtype release \
    --cross-file "$CWD/cross/$TARGET.cross" \
    --strip \
    --default-library both \
    "$CWD/cross/build/$TARGET/freetype" \
    "$CWD/cross/source/freetype"

ninja -C "$CWD/cross/build/$TARGET/freetype" install

if [[ "$TARGET" = "aarch64-unknown-linux-gnu" ]]; then
    [[ ! -d "$CWD/cross/source/fontconfig" ]] && \
        git clone --depth 1 -b 2.18.3 https://gitlab.freedesktop.org/fontconfig/fontconfig "$CWD/cross/source/fontconfig"

    meson setup \
        --reconfigure \
        --prefix "$PREFIX" \
        --buildtype release \
        --cross-file "$CWD/cross/$TARGET.cross" \
        --strip \
        --default-library both \
        "$CWD/cross/build/$TARGET/fontconfig" \
        "$CWD/cross/source/fontconfig"

    ninja -C "$CWD/cross/build/$TARGET/fontconfig" install
fi

[[ ! -d "$CWD/cross/source/ffmpeg" ]] && \
    git clone --depth 1 -b n9.0.1 https://github.com/ffmpeg/ffmpeg "$CWD/cross/source/ffmpeg"

FFMPEG="$CWD/cross/source/ffmpeg"
FFMPEG_BUILD="$CWD/cross/build/$TARGET/ffmpeg"

[[ ! -d "$FFMPEG_BUILD" ]] && mkdir -p "$FFMPEG_BUILD"

cd "$FFMPEG_BUILD" || exit 1

FFMPEG_ARGS=(
    --disable-programs
    --disable-doc
    --extra-cxxflags='-D_GLIBCXX_USE_CXX11_ABI=0'
)

[[ ! -f "$FFMPEG_BUILD/Makefile" ]] && case "$TARGET" in
    x86_64-pc-windows-*) "$FFMPEG/configure" --target-os=mingw32 --cross-prefix=x86_64-w64-mingw32- --arch=x86_64 $FFMPEG_ARGS --prefix="$PREFIX" ;;
    i686-pc-windows-*) "$FFMPEG/configure" --target-os=mingw32 --cross-prefix=i686-w64-mingw32- --arch=x86 $FFMPEG_ARGS --prefix="$PREFIX" ;;
    x86_64-unknown-linux-gnu) "$FFMPEG/configure" --target-os=linux --cross-prefix=x86_64-linux-gnu- --arch=x86_64 $FFMPEG_ARGS --prefix="$PREFIX" ;;
    aarch64-unknown-linux-gnu) "$FFMPEG/configure" --target-os=linux --cross-prefix=aarch64-linux-gnu- --arch=aarch64 $FFMPEG_ARGS --prefix="$PREFIX" ;;
    *) echo "Invalid target platform: $TARGET"; exit 1 ;;
esac

make "-j$(nproc)" install
cd "$CWD" || exit 1
