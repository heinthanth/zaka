---
title: Configuration
description: Config files for zakac, where the commands look for them, which setting wins, and how each option and each setting of the environment is named in them.
---

# Configuration

Each option of each `zakac` command can be set once, in a config file, in place of on
each command line. A config file is TOML: a project keeps one beside its sources, and a
user keeps one for all of them.

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
with the mem checks, and `zakac run age.zaka` runs with room for 8192 nested calls and a
full backtrace on a stop.

## Where the files are

Each command reads these files, the lowest first:

| Order | File | On |
| --- | --- | --- |
| 1 | `/etc/zakac/config.toml` | Linux and macOS |
| 2 | `~/Library/Application Support/zakac/config.toml` | macOS |
| 3 | `zakac/config.toml` in the config folder of the user: `XDG_CONFIG_HOME`, or `~/.config` when it is not set | Linux and macOS |
| 3 | `zakac\config.toml` in `%APPDATA%` | Windows |
| 4 | `zakac.toml` in the working folder, or in the nearest folder above it that holds one | each |
| 5 | the file `--config <FILE>` names | each |

A file that is not there is passed over, but the file `--config` names must be there.
When the variable `ZAKAC_NO_CONFIG` is set, the commands read the file `--config` names
alone, and none of the search: the tests of `zakac` set it, so the files of the one who
runs them change nothing.
Each command takes `--config`, the stage commands and `run` as well as `build`:

```console
$ zakac run --config team.toml age.zaka
```

## Which setting wins

A later file wins over an earlier one, so a project's `zakac.toml` wins over the file of
the user, and `--config` wins over both. Above the files stand the variables of the
[environment](commands.md#environment), and above those, the command line:

| Rank | Source |
| --- | --- |
| 1, the highest | an option on the command line |
| 2 | a variable of the environment, such as `CC` or `ZAKA_BACKTRACE` |
| 3 | the file `--config` names |
| 4 | `zakac.toml` |
| 5 | the file of the user |
| 6 | `/etc/zakac/config.toml` |
| 7, the lowest | the default of the option |

A value from a file is the default of its option, so `zakac build --backend vm` builds for
the VM even when a file sets `backend = "c"`.

## Names and sections

Each name is the long form of an option, without `--`: `max-depth` sets `--max-depth`,
`cc-flag` sets `--cc-flag`. [Commands](commands.md) lists each option.

A name at the top of the file sets the option of that name in each command that takes
it: `overflow = "wrap"` reaches `borrowck`, `build` and `run`. A section named after a
command sets an option in that command alone, and wins over the top:

```toml
max-depth = 2048        # build --backend c or native, and run

[run]
max-depth = 8192        # run alone
```

The sections are the names of the commands: `segment`, `lex`, `parse`, `semcheck`,
`irgen`, `borrowck`, `build`, `run` and `lsp`. The executable of one stage reads the
section of its command, as that command does: `zakac_lexer` reads `[lex]`, `zaka_vm`
reads `[run]`, and the two code generators read `[build]`.

## Values

A value is text, a number, `true` or `false`, or a list of these:

| Option | In the file |
| --- | --- |
| `--backend c` | `backend = "c"` |
| `--max-depth 8192` | `max-depth = 8192` |
| `--release` | `release = true` |
| `--cc-flag -march=native --cc-flag -g` | `cc-flag = ["-march=native", "-g"]` |
| `--lint-error leaks --lint-error ZK0712` | `error = ["leaks", "ZK0712"]` in `[lint]` |
| `--checks mem,-sanitizers` | `checks = ["mem", "-sanitizers"]` or `checks = "mem,-sanitizers"` |

A list gives an option that takes more than one value each of them, in order. An option
on the command line that takes more than one value replaces the whole list of the file: with
`cc-flag = ["-g"]` in a file, `--cc-flag -O3` gives `-O3` alone.

Three families of options stand in a section of their own, named after the first word of
their flags: the lint options in `[lint]` (see [Lint levels](#lint-levels)); the platform
a program is built for in `[target]`, where `os = "linux"` sets `--target-os linux` and
`bits = 32` sets `--target-bits 32`; and the rights of the comptime calls in
`[comptime]`, where `allow-read = ["data"]` sets `--comptime-allow-read=data`. Each section
can also stand inside the section of a command, as `[build.comptime]`, and reach that
command alone. The other options are flat names.

```toml
[target]
os = "linux"                   # compile-time conditionals read linux
bits = 32                      # the platform-sized integer kinds are 32 bits

[comptime]
allow-read = ["data"]          # comptime calls read under data/
allow-env = ["HOME"]

[run]
allow-read = ["data"]          # and so does the program, when it runs
```

A switch, such as `--release` or `--leaks`, takes `true` or `false`. A file can turn
a switch on, but the command line cannot turn it off again: leave the name out of the
file for the commands that need it off.

## Settings that variables hold

Some settings have no option, and a variable of the environment holds each of them. A
config file names them too. Each sets its variable when the variable is not set, so a
variable set in the shell wins:

| Name | Variable | Example | Sets |
| --- | --- | --- | --- |
| `carets` | `ZAKAC_CARETS` | `carets = "cluster"` | the rows under a marked span; see [Diagnostics](diagnostics.md) |
| `learn` | `ZAKAC_LEARN` | `learn = true` | learner mode: each message also explains its code and links to the page that teaches it; see [Diagnostics](diagnostics.md#learner-mode) |
| `backtrace` | `ZAKA_BACKTRACE` | `backtrace = "full"` | how much of the stack a stop of `run` lists; see [Running a program](../vm/running.md) |
| `cflags` | `CFLAGS` | `cflags = "-O3"` | flags for the C compiler of `build --backend c` |
| `ldflags` | `LDFLAGS` | `ldflags = ["-L/opt/lib", "-lfoo"]` | flags for its linker, which `build --backend native` links with too |
| `color` | `NO_COLOR` | `color = false` | turns color off; `true` leaves it as the terminal has it |

These names can stand at the top of the file, for each command, or in a section, for the
command that runs: `backtrace` in `[run]` reaches `zakac run` alone.

`CC` has no name of its own: `cc = "clang"` sets `--cc`, and `CC` wins over it. A
program that `run` starts sees the variables a file sets, as it sees a variable set in the
shell.

## Mistakes in a file

A name that no command takes, a section that names no command, or a value of another
form, such as a table, stops the command before it reads its input, with exit code 3 and
the name of the file:

```console
$ zakac run --config team.toml age.zaka
zakac: the config file team.toml sets max-dept, which no command here takes
```

The same holds for a file that is not TOML, and for a `--config` file that is not there.
A value the option does not take, such as `overflow = "loud"`, names the file too, with
the reason the option gives:

```console
$ zakac run --config team.toml age.zaka
zakac: the config file team.toml gives overflow a value the flag refuses: invalid value 'loud' for '--overflow <MODE>' [possible values: wrap, stop]
```

`zakac` checks each name in the file. The executable of one stage checks its own section
alone, since a file is written for all of `zakac`: `zakac_lexer` reads a file that sets
`backend = "c"`, an option it lacks, and passes over it.

## Lint levels

The options `--lint-allow`, `--lint-warn` and `--lint-error` set the level of a lint group
or of one code. A config file writes them in a section of their own, `[lint]`, as `allow`,
`warn` and `error`:

```toml
[lint]
allow = ["spoken"]              # spoken Burmese, such as ထားမယ် for ထားမည်
warn = ["ZK0505"]               # one rule of natural wording, kept a warning
error = ["leaks"]               # each leak an error

[run.lint]
allow = ["ZK0511"]              # for zakac run alone: no warning for mixed spellings
```

A `[lint]` at the top reaches each command that takes the options; a `[lint]` inside the
section of a command, as `[run.lint]`, reaches that command alone and wins over the top.
`lint-allow = ["spoken"]` at the top of the file means the same as `allow` in `[lint]`.

| Option | In the file | Level |
| --- | --- | --- |
| `--lint-allow <NAME>` | `allow` in `[lint]` | left out |
| `--lint-warn <NAME>` | `warn` in `[lint]` | a warning |
| `--lint-error <NAME>` | `error` in `[lint]` | an error |

A name is a group (`mem`, `leaks`, `frees`, `borrows`, `nulls`, `pointers`, `doubts`,
`overflow`, `naturalness`, `spoken`) or a code such as `ZK0708`; a code wins over a group.
[Commands](commands.md#options-of-the-compile-stages) lists the codes of each group. A code that
is no lint, such as ZK0301, keeps its level: a name for it is an error of the file.

The editor reads the `[lint]` section of the nearest `zakac.toml` too, so a project that
allows the [spoken spellings](../language/spoken.md) shows no ZK0510 there.

These options are no rights: `--allow-read`, `--allow-write`, `--allow-env` and
`--allow-ffi` grant a run its [permissions](../vm/running.md#permissions), and stand with
the other options of `run`.

## The example file

The root of the Zaka repo holds `zakac.example.toml`: each setting a config file can hold,
with its default value, the flag it stands for and the commands that read it, in the
sections above. Save it as `zakac.toml` beside the
sources of a project, or as `~/.config/zakac/config.toml`, and take the `#` off each line
to set:

```toml
# zakac.toml, from zakac.example.toml
learn = true                    # explain each message and link its page
# overflow = "stop"             # integer overflow stops the program, or wraps

[lint]
allow = ["spoken"]

[run]
max-depth = 8192                # the most nested calls before the program stops
```

A test of the compiler turns each setting of the example on and runs a program with it,
so the file names each option as the commands of this release spell it.

## A full example

The editor reads the `[lint]` section of the nearest `zakac.toml` as well, so a project that allows the [spoken spellings](../language/spoken.md)
shows no ZK0510 there either.

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
