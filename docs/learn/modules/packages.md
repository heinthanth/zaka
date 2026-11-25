---
title: Packages
description: Use the modules of another project through the deps of zakac.toml, give them Burmese names, and pin git deps with zakac fetch.
---

# Packages

A package is a folder of modules with a `zakac.toml` at its root. To use the modules of
another package, we list it as a dep (dependency) in our own `zakac.toml`, then import its
modules with a path that starts with the name that we gave it.

## You will learn

- How to list a package in a folder beside ours as a dep, and import its modules.
- How a package gives Burmese names to files with ASCII names.
- How to use a package from a git repo (repository), and how a build keeps off the
  network.
- How the stdlib fits in.

## A package beside ours

Our program sits in the folder `app`, and the package that we want to use sits beside it,
in `shapes`:

```text
shapes/
  zakac.toml
  square.zaka
app/
  zakac.toml
  main.zaka
```

The package gives its file a Burmese name in its `zakac.toml`, and shares a function from
that file:

```toml
# shapes/zakac.toml
[package.modules]
"စတုရန်း" = "square.zaka"
```

```zaka-file
// square.zaka
ဧရိယာ ဟူသော အများသုံးနိုင်သော လုပ်ဆောင်ချက် သည်
အနား ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အနား * အနား ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

Our program lists the package under `[dependencies]`, with the name that its imports will
use, and imports the function:

```toml
# app/zakac.toml
[dependencies]
"ပုံသဏ္ဌာန်" = { path = "../shapes" }
```

```zaka-file
// main.zaka
"ပုံသဏ္ဌာန်/စတုရန်း" မှ ဧရိယာ ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဧရိယာ ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```console
$ cd app
$ zakac run main.zaka
၄၉
```

An import of a package has the same form as an import of a file, with the name of the dep
and the name of the module in place of the file:

```
"<package>/<module>" မှ <name> ကို ထည့်သွင်းမည်။
```

| Zaka | Reads as |
| --- | --- |
| `"ပုံသဏ္ဌာန်/စတုရန်း" မှ` | "from the module `စတုရန်း` of `ပုံသဏ္ဌာန်`" |
| `ဧရိယာ ကို ထည့်သွင်းမည်` | "bring in `ဧရိယာ`" |

## Names for the modules of a package

File names on disk remain ASCII, since shells, git and each OS handle them well, while
our imports read as Burmese. The `[package.modules]` table of a package joins the two: each
row gives a file the name that imports use.

A name that the table does not list reads the file of that path under the root of the
package, with `.zaka` added, so both of these imports read `square.zaka`:

```zaka-file
"ပုံသဏ္ဌာန်/စတုရန်း" မှ ဧရိယာ ကို ထည့်သွင်းမည်။
"ပုံသဏ္ဌာန်/square" မှ ဧရိယာ ကို ထည့်သွင်းမည်။
```

We prefer the table names, which read as Burmese. A name that is in
neither place is the error ZK0270, and its help lists the names of the table.

## Packages from git

A dep can also come from a git repo, at a commit, a tag or a branch:

```toml
# app/zakac.toml
[dependencies]
json = { git = "https://example.org/zaka-json.git", rev = "v1.2.0" }
```

A build never reaches the network. We run `zakac fetch` once, which clones the repo into
the cache folder and writes `zakac.lock` beside our `zakac.toml`, with the commit that the
rev stood at:

```console
$ zakac fetch
fetched json: https://example.org/zaka-json.git at v1.2.0, commit 1a97de1ae314
git checkouts pinned in zakac.lock: 1
```

From then on, `zakac run`, `zakac build` and `zakac test` read the checkout in the cache.
Before the fetch, an import of `json` is the error ZK0271, whose help tells us to run `zakac
fetch`. We keep `zakac.lock` with our sources, so each build of the program reads the same
commits.

## Deps of deps

Each package lists its own deps in its own `zakac.toml`, and each of its files imports
through them, so a package can use other packages in turn. `zakac fetch` walks the deps of
the deps too, and our `zakac.lock` pins each git dep of the whole program.

## The stdlib

The stdlib (standard library) is a package too, one that the compiler carries. Each package
has it as a dep named `စံကား`, so `"စံကား/စာရင်း"` reads its module `စာရင်း` with no line in
`zakac.toml`. Its modules have the names of its table and no other, so `"စံကား/list"` is an
error though the file is `list.zaka`.

## Details

- **A dep hides a folder.** With a dep named `lib`, `"lib/a.zaka"` looks for a module of the
  dep. We write `"./lib/a.zaka"` to read the folder `lib` beside the file.
- **Tests.** `zakac test` leaves out the tests of the files of deps; those run when we
  test the package itself.
- **A newer commit.** `zakac fetch` keeps the commits that `zakac.lock` pins. To take the
  newest commit of a branch, we remove its rows from `zakac.lock` and fetch again.
- **Programs without a file.** A program on a Learn page or in the compiler explorer has no
  package, so it cannot import one ([Imports outside a
  file](mistakes.md#imports-outside-a-file)).
- **Mistakes.** An import that names no file and no dep is ZK0269, a module that the
  package lacks is ZK0270, and a git dep that is not fetched is ZK0271
  ([Packages](../../manual/language/packages.md#mistakes)).
- **The full rules.** [Packages](../../manual/language/packages.md) has the full rules,
  with the order in which the compiler reads an import path.

## Recap

- A package is a folder with a `zakac.toml`, and we list the packages that we use under
  `[dependencies]`.
- `"<package>/<module>" မှ <name> ကို ထည့်သွင်းမည်။` imports from a module of a dep.
- `[package.modules]` gives Burmese names to files with ASCII names.
- A git dep needs `zakac fetch` once, and `zakac.lock` pins its commit.
- The stdlib is the package `စံကား`, which each package has.

## Hands on

- Make the `shapes` and `app` folders above, and run `main.zaka`.
- Add a file `cube.zaka` to `shapes` with a function of its own, give it a Burmese name in
  `[package.modules]`, and import it from `main.zaka`.
- Import a module that `shapes` does not have, and read the error.

Next: [Threads](../threads/README.md).
