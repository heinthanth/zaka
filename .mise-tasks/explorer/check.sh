#!/usr/bin/env bun
#MISE description="Check the compiler explorer for the wasm target"
#MISE dir="{{config_root}}"

cargo check -p zaka_explorer --features csr --target wasm32-unknown-unknown
