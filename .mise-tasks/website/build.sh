#!/bin/sh
#MISE description="Build the website and write its pages to target/website"
#MISE depends=["website:vendor"]
#MISE dir="{{config_root}}"

# cargo-leptos names each pkg file from a hash taken before it rewrites the
# imports of the split chunks, so a changed wasm can keep its old name and a
# cached one pairs with the glue of another build. The pkg files move into a
# folder named from a digest of their final content; their own references are
# relative, so the pages alone learn the folder.
set -eu

cargo leptos build --project zaka_website --release --split
digest="$(cat target/website/pkg/* | shasum -a 256 | cut -c1-16)"
mv target/website/pkg "target/website/$digest"
mkdir target/website/pkg
mv "target/website/$digest" "target/website/pkg/$digest"
LEPTOS_OUTPUT_NAME=zaka_website LEPTOS_SITE_ROOT=target/website \
  LEPTOS_SITE_PKG_DIR="pkg/$digest" LEPTOS_HASH_FILES=true ZAKA_WEBSITE_ONCE=1 \
  target/release/zaka_website
