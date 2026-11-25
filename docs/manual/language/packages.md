---
title: Packages
description: How a program uses the modules of another project - the deps of zakac.toml, the module table of a package, git deps with zakac fetch and zakac.lock, the stdlib as a package - and the errors ZK0269 to ZK0271.
---

# Packages

A package is a folder with a `zakac.toml` at its root, together with the `.zaka` files
under it. A package lists the other packages it uses, its deps (dependencies), in its
`zakac.toml`, and its files import the modules of a dep with a path that starts with the
name of the dep. The Learn book teaches them in [Packages](../../learn/modules/packages.md).

```toml
# app/zakac.toml
[dependencies]
"ပုံသဏ္ဌာန်" = { path = "../shapes" }
json = { git = "https://example.org/zaka-json.git", rev = "v1.2.0" }
```

```toml
# shapes/zakac.toml
[package.modules]
"စတုရန်း" = "square.zaka"
```

```zaka-file
// main.zaka
"ပုံသဏ္ဌာန်/စတုရန်း" မှ ဧရိယာ ကို ထည့်သွင်းမည်။
```

## The root of a package

- The nearest `zakac.toml` from a file upward marks the root of the package that the file
  belongs to. Each file reads the deps of its own package, so a file of a dep sees the
  deps of that dep, not those of the program.
- The same file holds the flags of the commands ([Configuration](../compiler/configuration.md)).
  The `[dependencies]` and `[package]` sections belong to packages, and no command reads
  them as flags.
- A file with no `zakac.toml` above it has no deps.

## Deps

`[dependencies]` lists each dep under a name, and each import of the dep starts with that
name.

- We write the name as we want the imports to read: Burmese, as in `"ပုံသဏ္ဌာန်"`, or ASCII,
  as in `json`. TOML wants quotes around a name with letters outside ASCII. The names are
  compared in normalized spelling, as import paths are.
- `စံကား` cannot name a dep, since it names [the stdlib](#the-stdlib-as-a-package).
- A dep gives `path`, a folder relative to the `zakac.toml` that lists it, or `git`, the URL
  of a git repo (repository), with `rev`, a commit, a tag or a branch of that repo. A dep
  with neither, with both, or with `git` but no `rev` stops the command with exit code 3
  and names the dep, and so does a field of another name.

## Modules of a package

An import `"<dep>/<name>"` reads a module of the dep.

- The `[package.modules]` table in the `zakac.toml` of the dep gives Burmese names to its
  files: `"စတုရန်း" = "square.zaka"` makes `"ပုံသဏ္ဌာန်/စတုရန်း"` read `square.zaka` at the root
  of the dep. The files keep ASCII names, which suit shells, git and each OS, while the
  imports read as Burmese.
- A name that the table does not list reads `<name>.zaka` under the root of the dep, so
  `"ပုံသဏ္ဌာန်/square"` reads `square.zaka` too, and `"ပုံသဏ္ဌာန်/geo/rect"` reads
  `geo/rect.zaka`.
- When neither file is there, the import is ZK0270, and its help lists the names of the
  table.
- `[package]` holds what a `zakac.toml` tells of its own package. `modules` is its one part
  for now.

## How an import path is read

The compiler reads an import path in this order:

1. A path that starts with `စံကား/` names a module of the stdlib.
2. A path whose first part, up to the first `/`, is the name of a dep of the package names
   a module of that dep.
3. Each other path names a file relative to the folder of the importing file, as
   [Modules](modules.md#the-import-sentence) describes. When no such file is there and the
   path does not end in `.zaka`, the path reads as the name of a package that the
   `zakac.toml` does not list, which is ZK0269.

The name of a dep hides a folder of the same name beside the importing file: with a dep
`lib`, `"lib/a.zaka"` looks for a module of the dep, while `"./lib/a.zaka"` reads the file
in the folder.

Each file is read once, whichever path reaches it, through a dep or beside the importing
file. Files that import each other through deps are ZK0231, as other files are.

## Git deps and the lock file

A build reads a git dep from a checkout in the cache folder and never reaches the
network. `zakac fetch` ([Commands](../compiler/commands.md#fetch)) clones each git dep of
the package and of its deps, deps of deps among them, and writes `zakac.lock` beside the
`zakac.toml`:

```toml
# zakac.lock: the commit of each git dep, as `zakac fetch` pinned it.
# Keep it with the sources, so each build reads the same commits.

[[git]]
url = "https://example.org/zaka-json.git"
rev = "v1.2.0"
commit = "1a97de1ae314508a6a69901fae6029e77278ee3a"
```

- `build`, `run`, `test` and `lsp` find each git dep through the lock file of the package
  of the file that runs: the checkout of the commit that the lock pins its URL and rev to.
  A git dep that the lock does not pin, or whose checkout is not in the cache, is ZK0271.
- The checkouts are in `zakac/git/<repo>-<hash>/<commit>` under the cache folder of the
  user, or under the folder that `ZAKAC_CACHE` names.
- `zakac fetch` keeps each commit that the lock pins while the cache holds its checkout, so
  it reaches the network for new revs alone. To take a newer commit of a branch, we remove
  its rows from `zakac.lock`, or the whole file, and fetch again.
- A path dep needs no fetch, and the path deps of a git dep are relative to its checkout.

## The stdlib as a package

The stdlib (standard library) is a package that the compiler carries. `stdlib/zakac.toml`
gives each of its modules a name in `[package.modules]`, and each file that the table lists
is built into the compiler, so a program reads it from no disk, and the explorer has it
too. Each package has the stdlib as a dep named `စံကား`, which no `zakac.toml` lists.

A module of the stdlib has the name its table gives it and no other: a path after `စံကား/`
reads no file, so `"စံကား/io"` is ZK0270 though the file is `io.zaka`. [The
stdlib](stdlib.md) lists the modules.

## Tests of deps

`zakac test` runs the tests of the files that it is given and of the files that those
import, except the files of deps: the tests of a dep run when we test that package
itself.

## Programs not read from a file

A program on standard input, in the compiler explorer or on a Learn page has no package,
so an import of a dep in it is ZK0232, as each import of a file is
([Modules](modules.md#programs-not-read-from-a-file)).

## Mistakes

```console
$ zakac run main.zaka
error[ZK0270]: the package "ပုံသဏ္ဌာန်" has no module "စက်ဝိုင်း"
 --> main.zaka:1:1
  |
1 | "ပုံသဏ္ဌာန်/စက်ဝိုင်း" မှ ဧရိယာ ကို ထည့်သွင်းမည်။
  | ^^^^^^^^^^^^^^
  = help: its [package.modules] table lists စတုရန်း, and it holds no file ပုံသဏ္ဌာန်/စက်ဝိုင်း.zaka
```

| Code | When |
| --- | --- |
| ZK0269 | an import that names no file beside the importing file and no dep of its package |
| ZK0270 | an import of a module that the dep does not have, the stdlib among them |
| ZK0271 | an import of a git dep that `zakac fetch` has not fetched, or whose checkout is not in the cache |

A mistake in `zakac.toml` itself, such as a dep with neither `path` nor `git`, or a dep
named `စံကား`, stops the command before it reads its input, with exit code 3:

```console
$ zakac run main.zaka
zakac run: the config file zakac.toml lists the dep "json" with neither path = "<folder>" nor git = "<url>" with rev = "<commit>"
```
