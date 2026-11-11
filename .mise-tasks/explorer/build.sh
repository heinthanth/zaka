#!/usr/bin/env bun
#MISE description="Build the compiler explorer"
#MISE depends=["font:fetch"]
#MISE dir="{{config_root}}/website/zaka_explorer"

trunk build --release --cargo-profile wasm-release
