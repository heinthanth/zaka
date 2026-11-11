#!/usr/bin/env bun
#MISE description="Draw the cover card (og:image) to website/zaka_website/assets/pictures/cover.png"
#MISE depends=["font:fetch"]
#MISE dir="{{config_root}}/website/zaka_website/vendor"

trunk build --release=true --dist ../../../target/cover --public-url ./ cover.html \
  && cargo run -q -p zaka_html2png -- ../../../target/cover/index.html ../assets/pictures/cover.png
