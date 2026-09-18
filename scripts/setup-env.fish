#!/bin/fish

set TARGET "$argv[1]"
set SETUP "$(cargo cross setup --target "$TARGET" --format fish)"

echo "$SETUP" | source

set TARGET_FIX "$(echo "$TARGET" | string replace -a '-' '_')"
set CC_VAR "CC_$TARGET_FIX"
set CXX_VAR "CXX_$TARGET_FIX"
set AR_VAR "AR_$TARGET_FIX"

set CC "$(eval "echo \"\$$CC_VAR\"" | sed -e 's/-- //g')"
set CXX "$(eval "echo \"\$$CXX_VAR\"" | sed -e 's/-- //g')"
set AR "$(eval "echo \"\$$AR_VAR\"" | sed -e 's/-- //g')"
# set -gx LDFLAGS "$(echo "$RUSTFLAGS" | sed -e 's/^-- -- //g')"

echo "$SETUP"
echo "set -gx CC \"$CC\""
echo "set -gx CXX \"$CXX\""
echo "set -gx AR \"$AR\""
# echo "set -gx LDFLAGS=\"$LDFLAGS\""
