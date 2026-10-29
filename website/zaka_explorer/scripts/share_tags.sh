#!/bin/sh
# Fill in the addresses of the share tags in the built index.html: the page
# and the cover card, from ZAKA_SITE_ORIGIN and the public URL Trunk builds
# for. Chat apps and feeds need whole addresses; without the origin the tags
# hold bare paths, which serve a build on this machine.
set -eu

origin="$(printf %s "${ZAKA_SITE_ORIGIN:-}" | tr -d "[:space:]")"
[ -n "$origin" ] || echo "scripts/share_tags.sh: ZAKA_SITE_ORIGIN is not set; the share tags hold bare paths" >&2
origin="${origin%/}"
page="$origin$TRUNK_PUBLIC_URL"
cover="${page}assets/$(cd target/assets && ls cover.*.png)"
html="$TRUNK_STAGING_DIR/index.html"
sed -e "s|__PAGE__|$page|g" -e "s|__COVER__|$cover|g" "$html" > "$html.part"
mv "$html.part" "$html"
