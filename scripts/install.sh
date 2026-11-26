#!/bin/sh
# Install zakac and zakafmt, the Zaka compiler and formatter, on macOS or Linux.
# `--help` lists the options; each has a variable as well, and the option wins.
set -eu

usage()
{
  cat <<'HELP'
Install zakac and zakafmt, the Zaka compiler and formatter, on macOS or Linux.

Usage:
  curl -fsSL https://zaka-lang.org/install.sh | sh
  curl -fsSL https://zaka-lang.org/install.sh | sh -s -- [options]
  sh install.sh [options]

Options:
  --version <tag>  install the release <tag>, as v0.1.0, in place of the newest
  --dir <dir>      install into <dir>/bin, in place of ~/.zaka/bin
  --no-path        print the PATH line, and leave the startup files as is
  -h, --help       print this help

Variables:
  ZAKA_VERSION     as --version
  ZAKA_HOME        as --dir
  ZAKA_NO_PATH     as --no-path, when set
  ZAKA_RELEASES    where the releases are, for tests of this script
HELP
}

fail()
{
  echo "zaka install: $1" >&2
  exit 1
}

# The value of an option, which must not be blank.
value()
{
  [ -n "$2" ] || fail "$1 needs a value; see --help"
  echo "$2"
}

releases="${ZAKA_RELEASES:-https://github.com/heinthanth/zaka/releases}"
version="${ZAKA_VERSION:-}"
zaka_home="${ZAKA_HOME:-$HOME/.zaka}"
no_path="${ZAKA_NO_PATH:-}"

while [ $# -gt 0 ]; do
  case "$1" in
    -h | --help)
      usage
      exit 0
      ;;
    --version=*) version="$(value --version "${1#*=}")" ;;
    --version)
      version="$(value --version "${2:-}")"
      shift
      ;;
    --dir=*) zaka_home="$(value --dir "${1#*=}")" ;;
    --dir)
      zaka_home="$(value --dir "${2:-}")"
      shift
      ;;
    --no-path) no_path=1 ;;
    *) fail "unknown option $1; see --help" ;;
  esac
  shift
done
bin="$zaka_home/bin"

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
if [ -n "$version" ]; then
  url="$releases/download/$version/$asset"
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
if [ -n "$no_path" ]; then
  echo "Add $bin to the PATH: $line"
  exit 0
fi
mkdir -p "$(dirname "$profile")"
if ! grep -qsF "$line" "$profile"; then
  printf '\n# Zaka\n%s\n' "$line" >>"$profile"
fi
echo "Added $bin to the PATH in $profile; open a new terminal, or run: $line"
