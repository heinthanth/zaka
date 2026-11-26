#!/bin/sh
#MISE description="Build the site of a release under /v<major>.<minor> into target/site-snapshots: pass a git ref (the working tree when left out or '.'), then a version to name it with in place of the one of its zakac crate"
#MISE dir="{{config_root}}"
# The working tree builds in place: ZAKA_SITE_BASE changes the web crates
# alone, and the next build of the latest builds them back. Another ref builds
# in a worktree with a target folder of its own, a full build. Its code must
# know ZAKA_SITE_BASE, which puts each address of its pages under the base.
set -eu
ref="${1:-.}"
repo="$(pwd)"
work="$(mktemp -d)"
tree="$repo"
trap '/bin/rm -rf "$work"' EXIT
if [ "$ref" != "." ]; then
  tree="$work/tree"
  git worktree add --detach "$tree" "$ref"
  trap 'git -C "$repo" worktree remove --force "$tree"; /bin/rm -rf "$work"' EXIT
  if [ -f .env ]; then cp .env "$tree/.env"; fi
fi
crate_version="$(sed -n 's/^version = "\([0-9]*\.[0-9]*\)\..*/\1/p' "$tree/compiler/zakac/Cargo.toml" | head -n 1)"
version="${2:-$crate_version}"
base="/v$version"
(cd "$tree" && MISE_TRUSTED_CONFIG_PATHS="$tree" ZAKA_SITE_BASE="$base" mise run website:build)
(cd "$tree" && MISE_TRUSTED_CONFIG_PATHS="$tree" mise run font:fetch)
site="$work/site/v$version"
mkdir -p "$work/site"
cp -R "$tree/target/website" "$site"
# The fonts go beside the base, to /fonts/ of the site, where the snapshot
# names them. Each folder holds one version of one font, so the fonts of the
# snapshots and of the latest merge in dist, and a font version the latest
# drops is kept for the snapshots that name it.
cp -R "$tree/target/fonts" "$work/site/fonts"
/bin/rm -f "$work/site/fonts/index.html" "$work/site/fonts/.trunk-lock"
cp -R "$tree/target/vendor" "$site/vendor"
mv "$site/vendor/monaco" "$site/explorer/monaco"
# Pagefind puts the folder above its bundle, the base, in front of each page
# it finds, so it indexes the snapshot from its own folder.
bunx pagefind@1.4.0 --site "$site" --output-path "$site/pagefind"
# Each address of the snapshot points under its base; one that points at the
# root of the site would open a page of the latest.
rooted='(href|src)="/(learn|docs|pkg|assets|vendor|pagefind|explorer)/'
if grep -rEl --include='*.html' "$rooted" "$site"; then
  echo "pages:snapshot: the files above name addresses outside $base" >&2
  exit 1
fi
mkdir -p target/site-snapshots
tar -czf "target/site-snapshots/site-v$version.tar.gz" -C "$work/site" "v$version" fonts
echo "target/site-snapshots/site-v$version.tar.gz"
