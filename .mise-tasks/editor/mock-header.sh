#!/bin/sh
#MISE description="Write the C headers of a build and of a lib into target/c-headers, which .clangd of the C runtime reads"
#MISE dir="{{config_root}}"

# The headers of the C runtime include three that a build writes: zaka_checks.h,
# zaka_runtime.h and, for a lib, zaka_lib_names.h. A build of hello writes the
# first two; a build of a lib adds the third. Run it again after a change to
# zaka_runtime or to the glue of a lib.
set -eu

out="target/c-headers"
mkdir -p "$out"

cargo run -q -p zakac -- build --backend c --emit c --cache "$out" \
  samples/basics/hello.zaka
cargo run -q -p zakac -- build --backend c --emit lib --cache "$out" \
  -o "$out/mathlib" compiler/zakac_codegen_target_c/tests/inputs/libs/mathlib.zaka
