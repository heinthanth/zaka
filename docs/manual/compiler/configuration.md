---
title: Configuration
description: Config files for zakac, where the commands look for them, which setting wins, and how each flag and each environment setting is named in them.
---

# Configuration

We can set each flag of each `zakac` command once, in a config file, instead of on each
command line. A config file is TOML: a project keeps one beside its sources, and a user
keeps one for all their projects. [Commands](commands.md) lists the flags themselves.

```toml
# zakac.toml, beside the sources of a project
overflow = "wrap"

[lint]
error = ["mem"]

[build]
backend = "c"
checks = ["mem", "-sanitizers"]

[run]
max-depth = 8192
backtrace = "full"
```

With this file in the working folder, `zakac build age.zaka` builds a native executable
with the mem checks, and `zakac run age.zaka` runs with room for 8192 nested calls and
prints a full backtrace when the program stops.

## Where the files are

Each command reads these files, the lowest first:

| Order | File | On |
| --- | --- | --- |
| 1 | `/etc/zakac/config.toml` | Linux and macOS |
| 2 | `~/Library/Application Support/zakac/config.toml` | macOS |
| 3 | `zakac/config.toml` in the config folder of the user: `XDG_CONFIG_HOME`, or `~/.config` when it is not set | Linux and macOS |
| 3 | `zakac\config.toml` in `%APPDATA%` | Windows |
| 4 | `zakac.toml` in the working folder, or in the nearest folder above it that has one | all |
| 5 | the file that `--config <FILE>` names | all |

The commands skip a file that is not there, but the file that `--config` names must
exist. When the variable `ZAKAC_NO_CONFIG` is set, the commands read the file that
`--config` names and skip the search. The tests of `zakac` set it, so that the files of the
person who runs them change nothing.

Each command accepts `--config`: the stage commands, `build`, `run` and `lsp`.

```console
$ zakac run --config team.toml age.zaka
```

## Which setting wins

A later file wins over an earlier one, so a project's `zakac.toml` wins over the file of
the user, and `--config` wins over both. The [environment
variables](commands.md#environment) rank above the files, and the command line ranks
above those:

| Rank | Source |
| --- | --- |
| 1, the highest | a flag on the command line |
| 2 | an environment variable, such as `CC` or `ZAKA_BACKTRACE` |
| 3 | the file that `--config` names |
| 4 | `zakac.toml` |
| 5 | the file of the user |
| 6 | `/etc/zakac/config.toml` |
| 7, the lowest | the default of the flag |

A value from a file becomes the default of its flag, so `zakac build --backend vm` builds
for the VM (virtual machine) even when a file sets `backend = "c"`.

## Names and sections

Each name is the long form of a flag without `--`: `max-depth` sets `--max-depth`, and
`cc-flag` sets `--cc-flag`.

A name at the top of the file sets the flag of that name in each command that has it, so
`overflow = "wrap"` reaches `borrowck`, `build` and `run`. A section named after a command
sets a flag in that command alone, and wins over the top of the file:

```toml
max-depth = 2048        # build --backend c or native, and run

[run]
max-depth = 8192        # run alone
```

The sections are the names of the commands: `segment`, `lex`, `parse`, `semcheck`,
`irgen`, `borrowck`, `build`, `run`, `lsp` and `fmt`. The program of one stage reads the
section of its command, as that command does: `zakac_lexer` reads `[lex]`, `zaka_vm` reads
`[run]`, the code generators read `[build]`, and `zakafmt` reads `[fmt]`. The editors read
the `[fmt]` section of the nearest `zakac.toml` to format a file:

```toml
[fmt]
indent = 2                      # two spaces for each level of block depth
speech-mode = "spoken"          # reserved words in their spoken spellings
import-order = "sorted"         # the stdlib imports first
```

## Values

A value is text, a number, `true` or `false`, or a list of these:

| Flag | In the file |
| --- | --- |
| `--backend c` | `backend = "c"` |
| `--max-depth 8192` | `max-depth = 8192` |
| `--release` | `release = true` |
| `--cc-flag -march=native --cc-flag -g` | `cc-flag = ["-march=native", "-g"]` |
| `--lint-error leaks --lint-error ZK0712` | `error = ["leaks", "ZK0712"]` in `[lint]` |
| `--checks mem,-sanitizers` | `checks = ["mem", "-sanitizers"]` or `checks = "mem,-sanitizers"` |

A list gives each of its items, in order, to a flag that accepts more than one value. A
flag on the command line that accepts more than one value replaces the whole list of the
file: with `cc-flag = ["-g"]` in a file, `--cc-flag -O3` gives `-O3` alone.

A switch, such as `--release` or `--leaks`, accepts `true` or `false`. A file can turn a
switch on, but the command line cannot turn it off again, so we leave the name out of the
file for the commands that need it off.

### Grouped sections

Three groups of flags go in a separate section, named after the first word of their
flags. The other flags are plain names.

| Section | Example | Sets |
| --- | --- | --- |
| `[lint]` | `allow = ["spoken"]` | `--lint-allow spoken` (see [Lint levels](#lint-levels)) |
| `[target]` | `os = "linux"`, `bits = 32` | `--target-os linux`, `--target-bits 32` |
| `[comptime]` | `allow-read = ["data"]` | `--comptime-allow-read=data` |

Each of these sections can also go inside the section of a command, as
`[build.comptime]`, and then reaches that command alone.

```toml
[target]
os = "linux"                   # compile-time conditionals see linux
bits = 32                      # the platform-sized integers are 32 bits

[comptime]
allow-read = ["data"]          # comptime calls read under data/
allow-env = ["HOME"]

[run]
allow-read = ["data"]          # and so does the program, when it runs
```

## Settings that variables hold

Some settings have no flag, and an environment variable carries each of them. A config
file can name them too. Each one sets its variable when the variable is not set, so a
variable set in the shell wins:

| Name | Variable | Example | Sets |
| --- | --- | --- | --- |
| `carets` | `ZAKAC_CARETS` | `carets = "cluster"` | the rows under a marked span; see [Carets on a terminal](diagnostics.md#carets-on-a-terminal) |
| `learn` | `ZAKAC_LEARN` | `learn = true` | learner mode: each message also explains its code and links to the page that teaches it; see [Learner mode](diagnostics.md#learner-mode) |
| `backtrace` | `ZAKA_BACKTRACE` | `backtrace = "full"` | how much of the stack a stop lists; see [How a run ends](../vm/running.md#how-a-run-ends) |
| `cflags` | `CFLAGS` | `cflags = "-O3"` | flags for the C compiler of `build --backend c` |
| `ldflags` | `LDFLAGS` | `ldflags = ["-L/opt/lib", "-lfoo"]` | flags for its linker, which `build --backend native` links with too |
| `color` | `NO_COLOR` | `color = false` | turns color off; `true` leaves it as the terminal has it |

These names can go at the top of the file, for each command, or in a section, for the
command that runs: `backtrace` in `[run]` reaches `zakac run` alone.

`CC` has no name of its own: `cc = "clang"` sets `--cc`, and `CC` wins over it. A program
that `run` starts sees the variables a file sets, the same as a variable set in the shell.

## Mistakes in a file

A name that no command accepts, a section that names no command, or a value of another
form, such as a table, stops the command before it reads its input, with exit code 3 and
the name of the file:

```console
$ zakac run --config team.toml age.zaka
zakac: the config file team.toml sets run.max-dept, which no command here takes
```

The same goes for a file that is not TOML, and for a `--config` file that does not exist.
A value that the flag does not accept, such as `overflow = "loud"`, names the file too,
with the reason from the flag:

```console
$ zakac run --config team.toml age.zaka
zakac: the config file team.toml gives overflow a value the flag refuses: invalid value 'loud' for '--overflow <MODE>' [possible values: wrap, stop]
```

`zakac` checks each name in the file. The program of one stage checks its own section
alone, since a file is written for all of `zakac`: `zakac_lexer` reads a file that sets
`backend = "c"`, a flag it lacks, and skips it.

## Lint levels

The flags `--lint-allow`, `--lint-warn` and `--lint-error` set the level of a lint group or
of one code. A config file writes them in a separate section, `[lint]`, as `allow`, `warn`
and `error`:

```toml
[lint]
allow = ["spoken"]              # spoken Burmese, such as ထားမယ် for ထားမည်
warn = ["ZK0505"]               # one rule of natural wording, kept a warning
error = ["leaks"]               # each leak an error

[run.lint]
allow = ["ZK0511"]              # for zakac run alone: no warning for mixed spellings
```

A `[lint]` at the top reaches each command that has the flags, and a `[lint]` inside the
section of a command, as `[run.lint]`, reaches that command alone and wins over the top.
`lint-allow = ["spoken"]` at the top of the file means the same as `allow` in `[lint]`.

| Flag | In the file | Level |
| --- | --- | --- |
| `--lint-allow <NAME>` | `allow` in `[lint]` | left out |
| `--lint-warn <NAME>` | `warn` in `[lint]` | a warning |
| `--lint-error <NAME>` | `error` in `[lint]` | an error |

A name is a group (`mem`, `leaks`, `frees`, `borrows`, `nulls`, `pointers`, `doubts`,
`overflow`, `naturalness`, `spoken`) or a code such as `ZK0708`, and a code wins over a
group. [Lint names](commands.md#lint-names) lists the codes of each group. A code that is
not a lint, such as ZK0301, keeps its level, and a name for it is an error of the file.

The editor reads the `[lint]` section of the nearest `zakac.toml` too, so a project that
allows the [spoken spellings](../language/spoken.md) shows no ZK0510 there either.

The lint flags are not rights: `--allow-read`, `--allow-write`, `--allow-env` and
`--allow-ffi` grant a run its [permissions](../vm/running.md#permissions), and these go with
the other flags of `run`.

## The example file

The root of the Zaka repo has `zakac.example.toml`: each setting a config file can have,
with its default value, the flag it sets and the commands that read it, in the sections
above. We save it as `zakac.toml` beside the sources of a project, or as
`~/.config/zakac/config.toml`, and take the `#` off each line we want to set:

```toml
# zakac.toml, from zakac.example.toml
learn = true                    # explain each message and link its page
# overflow = "stop"             # integer overflow stops the program, or wraps

[lint]
allow = ["spoken"]

[run]
max-depth = 8192                # the most nested calls before the program stops
```

A test of the compiler turns on each setting of the example and runs a program with it,
so the file names each flag as the commands of this release spell it.

## A full example

```toml
# ~/.config/zakac/config.toml
carets = "codepoint"    # the terminal draws one cell for each code point
learn = true            # each message explains its code and links to its page

[lint]
error = ["mem"]                # each finding of the pointer lint is an error
allow = ["ZK0712", "spoken"]   # but pointers into two blocks can be subtracted,
                               # and the spoken spellings are allowed

[build]
backend = "c"
checks = ["mem", "-sanitizers", "-depth"]
cc = "clang"
cc-flag = ["-march=native"]
cflags = "-O3"

[run]
max-depth = 8192
backtrace = "full"
leaks = true
```
