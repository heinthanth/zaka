#!/bin/sh
#MISE description="Fetch the site snapshots of the releases, build dist and publish it to Cloudflare Pages"
#MISE dir="{{config_root}}"
set -eu
mise run pages:snapshot-fetch
mise run pages:build
bunx wrangler@4 pages deploy dist --project-name="$CLOUDFLARE_PAGES_PROJECT" --branch=main --commit-dirty=true
