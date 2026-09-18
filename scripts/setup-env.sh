#!/bin/bash

TARGET="$1"

TMPF="$(mktemp)"
cargo cross setup --target "$TARGET" --format bash > "$TMPF"
source "$TMPF"
rm "$TMPF"

TARGET_FIX="${TARGET//-/_}"
CC_VAR="CC_$TARGET_FIX"
CXX_VAR="CXX_$TARGET_FIX"
AR_VAR="AR_$TARGET_FIX"

export CC="$(eval "echo \"\$$CC_VAR\"" | sed -e 's/^-- //g')"
export CXX="$(eval "echo \"\$$CXX_VAR\"" | sed -e 's/^-- //g')"
export AR="$(eval "echo \"\$$AR_VAR\"" | sed -e 's/^-- //g')"
# export LDFLAGS="$(echo "$RUSTFLAGS" | sed -e 's/^-- //g')"
