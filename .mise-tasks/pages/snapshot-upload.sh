#!/bin/sh
#MISE description="Attach the site snapshot of a release to its GitHub release: pass the tag, as v0.1.0"
#MISE dir="{{config_root}}"
# Run pages:snapshot on the tag first.
set -eu
tag="$1"
version="$(echo "$tag" | sed 's/^v\([0-9]*\.[0-9]*\).*/\1/')"
gh release upload "$tag" "target/site-snapshots/site-v$version.tar.gz" --clobber
