#!/usr/bin/env bun
#MISE description="Build the VS Code extension into a .vsix"
#MISE dir="{{config_root}}/editors/vscode"

wasm-pack build --target nodejs --no-pack --release \
  && bun install \
  && bunx @vscode/vsce package
