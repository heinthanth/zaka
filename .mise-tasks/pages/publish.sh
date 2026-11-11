#!/usr/bin/env bun
#MISE description="Build dist and publish it to Cloudflare Pages"
#MISE depends=["pages:build"]
#MISE dir="{{config_root}}"

bunx wrangler@4 pages deploy dist --project-name=$CLOUDFLARE_PAGES_PROJECT --branch=main --commit-dirty=true
