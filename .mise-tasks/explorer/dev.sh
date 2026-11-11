#!/usr/bin/env bun
#MISE description="Serve the compiler explorer with live reload"
#MISE depends=["font:fetch"]
#MISE dir="{{config_root}}/website/zaka_explorer"

trunk serve --cargo-profile web-dev
