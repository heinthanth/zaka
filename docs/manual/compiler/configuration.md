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
mem-check = "error"

[build]
target = "c"
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

A value from a file is the default of its option, so `zakac build --target vm` builds for
the VM even when a file sets `target = "c"`.

## Names and sections

Each name is the long form of an option, without `--`: `max-depth` sets `--max-depth`,
`cc-flag` sets `--cc-flag`. [Commands](commands.md) lists each option.

A name at the top of the file sets the option of that name in each command that takes
it: `mem-check = "error"` reaches `borrowck`, `build` and `run`. A section named after a
command sets an option in that command alone, and wins over the top:

```toml
max-depth = 2048        # build --target c and run

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
| `--target c` | `target = "c"` |
| `--max-depth 8192` | `max-depth = 8192` |
| `--release` | `release = true` |
| `--cc-flag -march=native --cc-flag -g` | `cc-flag = ["-march=native", "-g"]` |
| `--error leaks --error ZK0712` | `error = ["leaks", "ZK0712"]` |
| `--checks mem,-sanitizers` | `checks = ["mem", "-sanitizers"]` or `checks = "mem,-sanitizers"` |

A list gives an option that takes more than one value each of them, in order. An option
on the command line that takes more than one value replaces the whole list of the file: with
`cc-flag = ["-g"]` in a file, `--cc-flag -O3` gives `-O3` alone.

A switch, such as `--release` or `--mem-strict`, takes `true` or `false`. A file can turn
a switch on, but the command line cannot turn it off again: leave the name out of the
file for the commands that need it off.

## Settings that variables hold

Some settings have no option, and a variable of the environment holds each of them. A
config file names them too. Each sets its variable when the variable is not set, so a
variable set in the shell wins:

| Name | Variable | Example | Sets |
| --- | --- | --- | --- |
| `carets` | `ZAKAC_CARETS` | `carets = "cluster"` | the rows under a marked span; see [Diagnostics](diagnostics.md) |
| `backtrace` | `ZAKA_BACKTRACE` | `backtrace = "full"` | how much of the stack a stop of `run` lists; see [Running a program](../vm/running.md) |
| `cflags` | `CFLAGS` | `cflags = "-O3"` | flags for the C compiler of `build --target c` |
| `ldflags` | `LDFLAGS` | `ldflags = ["-L/opt/lib", "-lfoo"]` | flags for its linker |
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
A value the option does not take, such as `mem-check = "loud"`, gives the error of the
option, with exit code 2, as the command line does:

```console
$ zakac run age.zaka
run: Invalid value "loud" for argument "--mem-check <MODE>". Should be one of: error, warn, off
```

`zakac` checks each name in the file. The executable of one stage checks its own section
alone, since a file is written for all of `zakac`: `zakac_lexer` reads a file that sets
`target = "c"`, an option it lacks, and passes over it.

## A full example

```toml
# ~/.config/zakac/config.toml
carets = "codepoint"    # the terminal draws one cell for each code point
mem-check = "error"     # each finding of the pointer lint is an error
allow = ["ZK0712"]      # but pointers into two blocks can be subtracted

[build]
target = "c"
checks = ["mem", "-sanitizers", "-depth"]
cc = "clang"
cc-flag = ["-march=native"]
cflags = "-O3"

[run]
max-depth = 8192
backtrace = "full"
leaks = true
```
