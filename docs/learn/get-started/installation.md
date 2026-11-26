---
title: Installation
description: Install zakac on macOS, Linux or Windows with one command, from a release download, in Docker, or from source.
---

# Installation

The compiler, `zakac`, checks, runs and builds Zaka programs, and runs the language server
for editors, and its `fmt` command formats source files. Each release of Zaka holds it for
these platforms:

| OS | CPU |
| --- | --- |
| macOS 11 or later | Apple silicon (arm64) |
| Linux (glibc 2.35 or later) | x86_64 and arm64 |
| Windows 10 or later | x86_64 and ARM64 |

On another platform, such as an Intel Mac, we [build from source](#build-from-source).

## Install with one command

On macOS and Linux, run this in a terminal:

```sh
curl -fsSL https://zaka-lang.org/install.sh | sh
```

On Windows, run this in PowerShell:

```powershell
irm https://zaka-lang.org/install.ps1 | iex
```

The script downloads the newest release for the computer, puts `zakac` in
`~/.zaka/bin` (`%USERPROFILE%\.zaka\bin` on Windows), and adds that folder to the `PATH`:
through the startup file of the shell on macOS and Linux, and in the `PATH` of the user on
Windows. Open a new terminal afterwards, so it reads the new `PATH`.

The options of the script change what it does, and `--help` (`-Help` on Windows) lists
them. Each option has a variable as well, for a script that sets them in advance:

| macOS and Linux | Windows | Variable | What it does |
| --- | --- | --- | --- |
| `--version <tag>` | `-Version <tag>` | `ZAKA_VERSION` | installs that release in place of the newest, as `v0.1.0` |
| `--prerelease` | `-Prerelease` | `ZAKA_PRERELEASE` | installs the newest release, prereleases included |
| `--dir <dir>` | `-Dir <dir>` | `ZAKA_HOME` | installs into that folder in place of `~/.zaka` |
| `--no-path` | `-NoPath` | `ZAKA_NO_PATH` | prints the `PATH` line in place of adding it |

The options follow `sh -s --` when the script comes through a pipe, and a PowerShell
script block takes them on Windows:

```sh
curl -fsSL https://zaka-lang.org/install.sh | sh -s -- --version v0.1.0
```

```powershell
& ([scriptblock]::Create((irm https://zaka-lang.org/install.ps1))) -Version v0.1.0
```

To update Zaka later, run the same command again. With no option, the script installs the
newest stable release and passes over prereleases, such as `v0.2.0-beta.1`: `--prerelease`
installs the newest release of either kind, and `--version` with the tag of a prerelease
installs that one.

## Download a release

The [releases](https://github.com/zaka-lang/zaka/releases) page holds an archive for each
platform: `zakac-<target>.tar.gz`, or `.zip` on Windows, where the target names the CPU and
the OS, as `aarch64-apple-darwin` or `x86_64-pc-windows-msvc`. Unpack it and put its
folder on the `PATH`.

On macOS, a file downloaded in a browser carries a mark that stops it from running until
macOS trusts it. The script above downloads with `curl`, which sets no such mark; for a
browser download, remove the mark once:

```sh
xattr -d com.apple.quarantine zakac
```

On Windows, SmartScreen can warn about an executable it has not seen before. Choose
**More info**, then **Run anyway**.

## Run in Docker

Each release is also a Docker image, `ghcr.io/zaka-lang/zaka`, for Linux on x86_64 and
arm64, with `zakac` as its entry point. It needs no install: mount the folder of the
program, and the words after the image name are the `zakac` command to run.

```sh
docker run --rm -v "$PWD:/src" -w /src ghcr.io/zaka-lang/zaka run hello.zaka
```

The REPL needs `-it`, which keeps the terminal open for input:

```sh
docker run --rm -it ghcr.io/zaka-lang/zaka repl
```

The tag `latest` names the newest stable release, and a version names that release, as
`ghcr.io/zaka-lang/zaka:0.1.0`. The image holds no C compiler, so for
`zakac build --backend native` or `--backend c`, build an image on it that adds one:

```dockerfile
FROM ghcr.io/zaka-lang/zaka
RUN apt-get update && apt-get install -y --no-install-recommends gcc libc6-dev
```

To build the image from source, run `docker build -t zaka .` in the
[Zaka repo](https://github.com/zaka-lang/zaka), then use `zaka` in place of
`ghcr.io/zaka-lang/zaka`.

## Check the install

```console
$ zakac --version
Zaka Compiler (zakac v0.1.0+…)
```

When the shell reports that it cannot find `zakac`, the folder is not on the `PATH` of
that shell: open a new terminal, or add the folder to the `PATH` in the startup file of the
shell.

## Build from source

With [Rust](https://www.rust-lang.org/tools/install) installed, we build `zakac` from
the [Zaka repo](https://github.com/zaka-lang/zaka):

```sh
git clone https://github.com/zaka-lang/zaka
cd zaka
cargo install --path compiler/zakac
```

`cargo install` puts `zakac` in `~/.cargo/bin`, which the Rust installer has
added to the `PATH`. To build programs for WebAssembly as well
([WASI](../../manual/compiler/native.md#wasi)), add the targets of WASI to Rust first, with
`rustup target add wasm32-wasip1 wasm32-wasip2`; without them, `zakac` builds with a
warning and leaves WASI out.

## What else to install

`zakac run` and the VM need nothing else. To build a program as a native executable, as
[Quick start](quick-start.md#build-an-executable) shows, `zakac` uses a C compiler to
link it: `cc`, `gcc`, `clang` or `zig cc` on the `PATH`. macOS gets one from the Xcode
Command Line Tools (`xcode-select --install`), Linux from its `gcc` or `clang` package,
and Windows from Visual Studio or MinGW; [Native programs](../../manual/compiler/native.md#what-the-build-needs)
lists each choice.

Next: [Editor setup](editors.md).
