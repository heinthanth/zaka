#!/bin/sh
#MISE description="Build the runtime lib for another target, for zakac build --runtime-lib"
#MISE dir="{{config_root}}"
#USAGE arg "<triple>" help="The Rust target triple, such as x86_64-unknown-linux-gnu"

# Writes target/runtime/<triple>/libzaka_runtime.a (zaka_runtime.lib for MSVC)
# and native-libs.txt beside it, the OS libs the lib needs, one flag a line, as
# the build of zakac does for the host. Needs the target of rustup:
# `rustup target add <triple>`.
set -eu

triple="$usage_triple"
out="target/runtime/$triple"
mkdir -p "$out"

cargo rustc -p zaka_runtime --lib --crate-type staticlib --profile zaka-runtime \
  --target "$triple" -- --print native-static-libs 2> "$out/build.log" \
  || { cat "$out/build.log"; exit 1; }

sed -n 's/.*native-static-libs: //p' "$out/build.log" | tr ' ' '\n' | sed '/^$/d' > "$out/native-libs.txt"

case "$triple" in *-msvc) lib=zaka_runtime.lib ;; *) lib=libzaka_runtime.a ;; esac
cp "${CARGO_TARGET_DIR:-target}/$triple/zaka-runtime/$lib" "$out/$lib"
echo "$out/$lib"
