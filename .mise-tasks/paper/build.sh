#!/usr/bin/env bun
#MISE description="Build the papers as main.pdf in docs/paper/design, compiler and combined"
#MISE dir="{{config_root}}"

fonts=website/zaka_web_ui_common/assets/fonts

typst compile --root . --font-path $fonts docs/paper/design/main.typ \
  && typst compile --root . --font-path $fonts docs/paper/compiler/main.typ \
  && typst compile --root . --font-path $fonts docs/paper/combined/main.typ
