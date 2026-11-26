#!/bin/sh
#MISE description="Download the site snapshot of each GitHub release into target/site-snapshots"
#MISE dir="{{config_root}}"
# Each run downloads each snapshot again, so a snapshot replaced on its release
# reaches the next publish. The releases come oldest first: when a patch
# release brings a snapshot of the same name, as v0.1.1 after v0.1.0, the
# newer one is written last and wins. A release with no snapshot is passed
# over.
set -eu
mkdir -p target/site-snapshots
gh release list --limit 200 --json tagName --jq 'reverse | .[].tagName' | while read -r tag; do
  gh release download "$tag" --pattern 'site-v*.tar.gz' --dir target/site-snapshots \
    --clobber 2>/dev/null || true
done
ls target/site-snapshots
