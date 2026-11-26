#!/bin/sh
#MISE description="Serve dist on this machine as Cloudflare Pages serves it, 404 pages included"
#MISE dir="{{config_root}}"
set -eu
bunx wrangler@4 pages dev dist
