#!/bin/sh
# Install zakac and zakafmt on macOS or Linux:
#
#   curl -fsSL https://zaka-lang.org/install.sh | sh
#
# The executables of the newest release go to ~/.zaka/bin, which then joins the
# PATH through the startup file of the shell. These variables change that:
#
#   ZAKA_VERSION   a release to install in place of the newest, as v0.1.0
#   ZAKA_HOME      the folder to install into, in place of ~/.zaka
#   ZAKA_NO_PATH   when set, the startup files are not changed
#   ZAKA_RELEASES  where the releases are, for tests of this script
set -eu

releases="${ZAKA_RELEASES:-https://github.com/heinthanth/zaka/releases}"
zaka_home="${ZAKA_HOME:-$HOME/.zaka}"
bin="$zaka_home/bin"

fail()
{
  echo "zaka install: $1" >&2
  exit 1
}

# The target of the release for this computer. A shell under Rosetta reports
# x86_64 on Apple silicon, so the CPU of a Mac comes from sysctl.
os="$(uname -s)"
cpu="$(uname -m)"
case "$os" in
  Linux) os=unknown-linux-gnu ;;
  Darwin) os=apple-darwin ;;
  *) fail "no build for $os; on Windows, run install.ps1 in PowerShell" ;;
esac
if [ "$os" = apple-darwin ] && [ "$(sysctl -n hw.optional.arm64 2>/dev/null)" = 1 ]; then
  cpu=arm64
fi
case "$cpu" in
  x86_64 | amd64) cpu=x86_64 ;;
  arm64 | aarch64) cpu=aarch64 ;;
  *) fail "no build for the CPU $cpu" ;;
esac
target="$cpu-$os"
if [ "$target" = x86_64-apple-darwin ]; then
  fail "no build for Intel Macs; build zakac from source: https://zaka-lang.org/learn/get-started/installation/"
fi

asset="zakac-$target.tar.gz"
if [ -n "${ZAKA_VERSION:-}" ]; then
  url="$releases/download/$ZAKA_VERSION/$asset"
else
  url="$releases/latest/download/$asset"
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
echo "Downloading $url"
if command -v curl >/dev/null; then
  curl -fsSL "$url" -o "$work/$asset" || fail "the download failed"
elif command -v wget >/dev/null; then
  wget -q "$url" -O "$work/$asset" || fail "the download failed"
else
  fail "the download needs curl or wget"
fi
tar -xzf "$work/$asset" -C "$work"
mkdir -p "$bin"
cp "$work/zakac-$target/zakac" "$work/zakac-$target/zakafmt" "$bin/"
chmod +x "$bin/zakac" "$bin/zakafmt"
echo "Installed $("$bin/zakac" --version | head -n 1) in $bin"

# The PATH: a shell that finds the folder now needs nothing; else the line
# goes to the startup file of the shell, once.
case ":$PATH:" in
  *":$bin:"*) exit 0 ;;
esac
line="export PATH=\"$bin:\$PATH\""
case "${SHELL:-}" in
  */zsh) profile="$HOME/.zshrc" ;;
  */bash) profile="$HOME/.bashrc" ;;
  */fish)
    profile="$HOME/.config/fish/conf.d/zaka.fish"
    line="fish_add_path \"$bin\""
    ;;
  *) profile="$HOME/.profile" ;;
esac
# A login shell of bash on macOS reads ~/.bash_profile, not ~/.bashrc.
if [ "$os" = apple-darwin ] && [ "$profile" = "$HOME/.bashrc" ]; then
  profile="$HOME/.bash_profile"
fi
if [ -n "${ZAKA_NO_PATH:-}" ]; then
  echo "Add $bin to the PATH: $line"
  exit 0
fi
mkdir -p "$(dirname "$profile")"
if ! grep -qsF "$line" "$profile"; then
  printf '\n# Zaka\n%s\n' "$line" >>"$profile"
fi
echo "Added $bin to the PATH in $profile; open a new terminal, or run: $line"
