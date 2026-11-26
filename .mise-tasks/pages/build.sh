#!/bin/sh
#MISE description="Join the website, the explorer and the web fonts into dist, as Cloudflare Pages serves them, index the books for search, then add the snapshots of old releases in target/site-snapshots and the list of releases"
#MISE depends=["website:build", "font:fetch"]
#MISE dir="{{config_root}}"
set -eu
/bin/rm -rf dist
cp -R target/website dist
cp -R target/vendor dist/vendor
mv dist/vendor/monaco dist/explorer/monaco
cp -R target/fonts dist/fonts
# The install scripts, at the root: `curl … | sh` and `irm … | iex` read them
# as text.
cp website/zaka_website/install/install.sh website/zaka_website/install/install.ps1 dist/
printf '/install.sh\n  Content-Type: text/plain; charset=utf-8\n/install.ps1\n  Content-Type: text/plain; charset=utf-8\n' \
  > dist/_headers
# The index comes before the snapshots, so the search of the latest lists its
# own pages alone; each snapshot brings an index of its own.
bunx pagefind@1.4.0 --site dist --output-path dist/pagefind
# The list of releases, the newest first: the latest at the root, then each
# snapshot under its base. The snapshot of the latest waits for the next
# release, so each snapshot the site serves is an old one. A snapshot also
# brings the fonts it names into /fonts/; a font folder holds one version, so
# the files it shares with the latest are the same.
latest="$(sed -n 's/^version = "\([0-9]*\.[0-9]*\)\..*/\1/p' compiler/zakac/Cargo.toml | head -n 1)"
printf '[[release]]\nversion = "%s"\npath = "/"\n' "$latest" > dist/versions.toml
snapshots="$(ls target/site-snapshots 2>/dev/null \
  | sed -n 's/^site-v\([0-9]*\.[0-9]*\)\.tar\.gz$/\1/p' \
  | sort -t . -k 1,1nr -k 2,2nr)"
for version in $snapshots; do
  if [ "$version" = "$latest" ]; then continue; fi
  tar -xzf "target/site-snapshots/site-v$version.tar.gz" -C dist
  printf '\n[[release]]\nversion = "%s"\npath = "/v%s/"\n' "$version" "$version" \
    >> dist/versions.toml
done
