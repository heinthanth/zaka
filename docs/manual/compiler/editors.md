---
title: Editors
description: The zakac language server, what it shows while a program is edited, and how to set it up in VS Code, Zed and other editors.
---

# Editors

`zakac lsp` is a language server: it speaks the Language Server Protocol over standard
input and output, so an editor can show what the compiler finds while a program is
edited. It is part of `zakac` itself, so the server and the compiler that builds the
program never differ in version.

## What it shows

- **Errors and warnings** of each stage up to the checker, with their codes, as
  the text changes. A message in a file that the open file imports shows at the start of
  the open file, named after that file, with a link to its place.
- **Colors** for each word, from the lexer: control words, verbs, operators, particles,
  kind nouns, truth values, names, numbers, characters, strings and comments. A pattern
  cannot find where a Burmese word ends, so the colors come from the server and not from
  a grammar.
- **A hover** on a name shows its declaration as Zaka writes it, in a code block, such
  as `ကိန်းရှင် ကျော်: လူ` or `လုပ်ဆောင်ချက် နှစ်ဆ(က: isize): isize`, with notes under a
  rule: what the name is (a parameter, a field and its offset, a member and its value, a
  kind alias, a loop variable) and the size and alignment of its kind. A struct or an
  enumeration lists its fields or members. The `///` lines right above a declaration
  show under its code block, at each use of the name. On another expression the hover
  shows its kind, in the short form the messages use, such as `i32`, `usize` or `text`.
- **Go to the declaration** of a variable, a parameter, a constant (also one used in a
  list count above its declaration), a function, a struct, a field, an enumeration, a
  member or a kind alias, in the open file or in a file it imports.
- **Find the uses** of a name, in the checks of all open files, and **rename** it at
  each of them. A new name must lex as one name: a reserved word, several words or a
  number is refused.
- **The outline** of the file: its functions with their parameters, the start block,
  its structs with their fields, its enumerations with their members, its kind aliases,
  and its global variables and constants.
- **Completion** of reserved words, joined phrases such as
  `အောက်ပါတို့ကို လုပ်ဆောင်မည်`, and the names in scope at the cursor: the top-level
  names of the file and the names it imports, and inside a function its parameters,
  its result and the variables declared above the cursor in the blocks around it. A
  name another file keeps private is never offered. After `<name> ၏` it offers the
  fields of the struct the name holds, with their kinds, following a chain of fields
  and list cells such as `က ၏ အိမ် ၏` or `l ၏ အကွက် n + ၁ ၏`: each cell of a list has
  the cell kind, whatever its number. After `<enumeration>များထဲမှ` it offers the
  members, through a kind alias too.
- **A compiler bug** on half-written text shows as one message at the start of the file,
  and the server goes on.

The server checks the files the editor holds open from their unsaved text, and reads
each other file from the disk. A file that exports a name (`အများသုံး`) and has no
start block is a module that other files import, so it is checked as one: it needs no
start block, and ZK0327 is not reported for it. A change to one open file checks the
other open files again, as one can import another. While a line does not parse, the
compiler stops before the check; the server checks the rest of the file all the same,
so hover, going to a declaration and completion keep working, and it shows the parse
error alone.

## Which zakac

Each extension starts `zakac` from the `PATH` unless a setting names another one. The
setting takes a bare name, which is looked up on the `PATH`; an absolute path; or a
relative path such as `target/debug/zakac`, which starts at the folder open in the
editor. A build of the repo works without an install:

```console
$ cargo build -p zakac
```

and the setting names `target/debug/zakac`. After a rebuild, restart the server: in VS
Code, **Developer: Reload Window**; in Zed, **editor: restart language server**.

To put `zakac` on the `PATH` instead, run `cargo install --path compiler/zakac` in the
Zaka repo.

Both extensions take files that end in `.zaka` or `.zk`.

## VS Code

1. Put `zakac` on the `PATH`, or set `zaka.server.path` (see
   [Which zakac](#which-zakac)).
2. Build and install the extension:

   ```console
   $ mise run vscode:build
   $ code --install-extension editors/vscode/zaka-0.1.0.vsix
   ```

3. Open a `.zaka` file. It shows the Zaka logo as its file icon, unless the icon theme
   has its own icon for Zaka.

The setting goes in the user settings or in the `.vscode/settings.json` of a folder:

```json
{ "zaka.server.path": "target/debug/zakac" }
```

The extension is written in Rust (`editors/vscode`) and built for Node.js with
wasm-pack; its manifest, comment rules and fallback grammar are TOML files that its
build script turns into the JSON files VS Code reads.

## Zed

1. Put `zakac` on the `PATH`, or set its path (see [Which zakac](#which-zakac)).
2. In Zed, run **zed: install dev extension** and pick the `editors/zed` folder.
3. Turn on the colors of the server for Zaka in the Zed settings; Zed leaves them off
   until a setting asks for them:

   ```json
   {
     "languages": {
       "Zaka": { "semantic_tokens": "full" }
     }
   }
   ```

Errors and hovers work without that setting. Another `zakac` goes in the standard
setting of a Zed language server. Name the `lsp` argument with the path, as Zed can
start the path from the setting itself:

```json
{
  "lsp": {
    "zakac": {
      "binary": { "path": "target/debug/zakac", "arguments": ["lsp"] }
    }
  }
}
```

## The explorer

The [compiler explorer](/explorer/) runs the same code in the page, with no server: its
editor shows the kind of an expression on hover, completes words, names, fields and
members, goes to a declaration (F12), finds the uses of a name (Shift+F12), renames
it (F2), and lists the outline (Ctrl+Shift+O, or Cmd+Shift+O on a Mac). The explorer
holds one file, so these reach that file and the stdlib modules it imports.

## Other editors

An editor with an LSP client runs `zakac lsp` with no arguments for files that end in
`.zaka` or `.zk`. The server takes the whole text on each change, reports positions in
UTF-16 columns, and answers `textDocument/hover`, `textDocument/semanticTokens/full` (a
`range` request gets the tokens of the whole file), `textDocument/definition`,
`textDocument/references`, `textDocument/prepareRename`, `textDocument/rename`,
`textDocument/documentSymbol` and `textDocument/completion`. It names its token kinds
with the standard LSP names (`keyword`, `function`, `operator`, `modifier`, `type`,
`enumMember`, `variable`, `number`, `string`, `comment`), so a theme colors them with no
extra rules.

## Not done

- Find uses and rename reach the open files and the files those import; a file that
  imports the name but is neither open nor imported is left as it is.
- The server does not run the program or the code generator, so the messages of a
  target, such as a construct the C target cannot build, show at build time.
