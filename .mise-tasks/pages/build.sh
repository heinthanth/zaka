#!/usr/bin/env bun
#MISE description="Join the website, the explorer and the web fonts into dist, as Cloudflare Pages serves them"
#MISE depends=["website:build", "font:fetch"]
#MISE dir="{{config_root}}"

rm -rf dist \
  && cp -R target/website dist \
  && cp -R target/vendor dist/vendor \
  && mv dist/vendor/monaco dist/explorer/monaco \
  && cp -R target/fonts dist/fonts
