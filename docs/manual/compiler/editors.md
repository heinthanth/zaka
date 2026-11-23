---
title: Editors
description: The zakac language server, what it shows while we edit a program, and how to set it up in VS Code, Zed and other editors.
---

# Editors

`zakac lsp` runs a language server so the editor can show compiler findings as we edit a
program. The server uses the Language Server Protocol over standard input and output. It
ships with `zakac`, so it matches the version of the compiler. See
[Commands](commands.md#lsp) for the `--config` flag.

## What it shows

- **Errors and warnings** of each stage (compiler pass) up to the checker, with their
  codes, as the text changes. Each code links to the page that teaches it, as in
  [learner mode](diagnostics.md#learner-mode). A message in a file that the open file
  imports shows at the start of the open file, named after that file, with a link to its
  place.
- **Colors** for each word, from the lexer: control words, verbs, operators, particles,
  data kind nouns, truth values, names, numbers, characters, strings and comments. A
  pattern cannot find where a Burmese word ends, so the colors come from the server and
  not from a grammar file.
- **A hover** on a name shows its declaration as Zaka writes it, in a code block, such as
  `ကိန်းရှင် ကျော်: လူ` or `လုပ်ဆောင်ချက် နှစ်ဆ(က: isize): isize`, with notes under a rule:
  what the name is (a parameter, a field and its offset, a member and its value, a kind
  alias, a loop variable) and the size and alignment of its data kind (type). A struct or
  an enumeration lists its fields or members. The `///` lines right above a declaration
  show under its code block, at each use of the name.
  - On a data kind phrase, such as `၃၂ ဘစ် ကိန်းပြည့်` or `ကိန်းပြည့် စာရင်း`, the hover shows
    the data kind, its size and the doc of the built-in data kind of its last word.
  - On the preposition of a call argument, it shows the function and the parameter that
    the preposition belongs to, and on a phrase of a call, the function and which phrase
    it is. The function signature shows its phrases after its name.
  - On another expression, the hover shows its data kind in the short form that messages
    use, such as `i32`, `usize` or `text`.
- **Call hints** before each argument whose parameter the call itself does not show:
  `ဒေတာ ပြောင်းလဲရန်:` before an argument the function can change (a mutable parameter,
  `inout`), `ဒေတာ ထည့်ပေးရန်:` before one it fills (a writable parameter, `out`), and
  `အပိုင်အနေဖြင့်:` before one it takes over.
- **Go to the declaration** of a variable, a parameter, a constant (also one used in a
  list count above its declaration), a function, a struct, a field, an enumeration, a
  member or a kind alias, in the open file or in a file it imports. A declaration of the
  stdlib opens in a replica of its module that the server writes to
  `zakac/stdlib/<version>/` in the cache folder of the user (`~/.cache` on Linux and
  macOS), from the source that `zakac` carries. A change to the replica changes nothing,
  and the next replica writes over it. The server checks the replica, and a file of the
  `stdlib` folder of the Zaka repo, as its module, so its compiler marks give no ZK0349.
- **Find the uses** of a name, in the checks of all open files, and **rename** it at
  each of them. A new name must lex as one name, so the server refuses a reserved word,
  several words or a number.
- **Quick fixes**: the server offers each fix a message has on that message. It applies
  none on its own, so we pick one. The first fix is the preferred one, which the quick
  fix shortcut applies.
- **The outline** of the file: its functions with their parameters, the program start
  block, its structs with their fields, its enumerations with their members, its kind
  aliases, and its global variables and constants.
- **Completion** of reserved words and joined phrases such as
  `အောက်ပါတို့ကို လုပ်ဆောင်မည်`, and of the names in scope at the cursor:
  - For each function in scope whose function signature has phrases or prepositions, it
    offers the call written as a sentence, as a snippet with a stop on each argument
    (`ဖိုင် … ထဲမှ စာကို ဖတ် သည့် ရလဒ်`).
  - The names in scope are the top-level names of the file and the names it imports, and
    inside a function its parameters, its result and the variables declared above the
    cursor in the blocks around it. A name that another file keeps private is never
    offered.
  - After `<name> ၏` it offers the fields of the struct in that name, with their data
    kinds, and it follows a chain of fields and list cells such as `က ၏ အိမ် ၏` or
    `l ၏ အကွက် n + ၁ ၏`: each cell of a list has the cell's data kind, whatever its number.
  - After `<enumeration>များထဲမှ` it offers the members, through a kind alias too.
- **Formatting** of the whole file, as `zakac fmt` formats it (see
  [Formatting](#formatting)).
- **A compiler bug** on half-written text shows as one message at the start of the file,
  and the server goes on.

The server checks the files that the editor has open from their unsaved text, and reads
each other file from the disk. A file that exports a name (`အများသုံးနိုင်သော`) and has no
program start block is a module that other files import, so the server checks it as one:
it needs no program start block, and ZK0327 is not reported for it. A change to one open
file checks the other open files again, since one can import another. While a line does
not parse, the compiler stops before the check, but the server checks the rest of the
file all the same, so hover, going to a declaration and completion keep working, and it
shows the parse error alone.

The server reads the `[lint]` section of the nearest `zakac.toml`, so a project that
allows the [spoken spellings](../language/spoken.md) shows no ZK0510 in the editor either
([Lint levels](configuration.md#lint-levels)).

## Formatting

The server formats the open file when the editor asks, as `zakac fmt` does, with the
settings of the `[fmt]` section of the nearest `zakac.toml`
([fmt](commands.md#fmt)). It answers with one edit that replaces the whole text, and with
no edit for a file that does not parse, so the editor keeps the text as it is. VS Code
formats on save with `"editor.formatOnSave": true`, and Zed formats on save unless its settings turn that off.

## Which zakac

Each extension starts `zakac` from the `PATH` unless a setting names another one. The
setting accepts a bare name, which is looked up on the `PATH`, an absolute path, or a
relative path such as `target/debug/zakac`, which starts at the folder open in the editor.
A build of the repo works without an install:

```console
$ cargo build -p zakac
```

and the setting names `target/debug/zakac`. After a rebuild, restart the server: in VS
Code, **Zaka: Restart the Language Server**, and in Zed, **editor: restart language
server**. A new value of the setting needs a reload of the VS Code window
(**Developer: Reload Window**).

To put `zakac` on the `PATH` instead, run `cargo install --path compiler/zakac` in the
Zaka repo.

Both extensions open files that end in `.zaka` or `.zk`.

## VS Code

1. Put `zakac` on the `PATH`, or set `zaka.server.path` (see
   [Which zakac](#which-zakac)).
2. Build and install the extension:

   ```console
   $ mise run editor:install-vscode
   ```

   The task builds the `.vsix` first (`mise run editor:build-vscode`), and
   `mise run editor:install-cursor` installs it into Cursor.

3. Open a `.zaka` file. It shows the Zaka logo as its file icon, unless the icon theme
   has an icon for Zaka.

The setting goes in the user settings or in the `.vscode/settings.json` of a folder:

```json
{ "zaka.server.path": "target/debug/zakac" }
```

The extension is written in Rust (`editors/vscode`) and built for Node.js with wasm-pack.
Its manifest, comment rules and fallback grammar are TOML files that its build script
turns into the JSON files VS Code reads.

## Zed

1. Put `zakac` on the `PATH`, or set its path (see [Which zakac](#which-zakac)).
2. In Zed, run **zed: install dev extension** and pick the `editors/zed` folder.
3. Turn on the colors of the server for Zaka in the Zed settings, because Zed leaves them
   off until a setting asks for them:

   ```json
   {
     "languages": {
       "Zaka": { "semantic_tokens": "full" }
     }
   }
   ```

Errors and hovers work without that setting. Another `zakac` goes in the standard setting
of a Zed language server. Name the `lsp` argument together with the path, since Zed can
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

The [compiler explorer](/explorer/) runs the same code in the page, with no server. Its
editor shows the data kind of an expression on hover, completes words, names, fields and
members, goes to a declaration (F12), finds the uses of a name (Shift+F12), renames it
(F2), and lists the outline (Ctrl+Shift+O, or Cmd+Shift+O on a Mac). The explorer has one
file, so these reach that file and the stdlib modules it imports. The format button of its
run bar formats the code with the default settings of `zakac fmt`.

## Other editors

An editor with an LSP client runs `zakac lsp` with no arguments for files that end in
`.zaka` or `.zk`. The server:

- accepts each change as a range of the text and its new words (incremental changes),
  and a whole new text works too;
- reports positions in UTF-16 columns;
- answers `textDocument/hover`, `textDocument/semanticTokens/full` (a `range` request gets
  the tokens of the whole file), `textDocument/inlayHint`, `textDocument/definition`,
  `textDocument/references`, `textDocument/prepareRename`, `textDocument/rename`,
  `textDocument/documentSymbol`, `textDocument/completion`, `textDocument/codeAction`
  (quick fixes alone) and `textDocument/formatting`;
- names its token kinds with the standard LSP names (`keyword`, `function`, `operator`,
  `modifier`, `type`, `enumMember`, `variable`, `number`, `string`, `comment`), so a theme
  colors them with no extra rules.

## Not done

- Find uses and rename reach the open files and the files those import. A file that
  imports the name but is neither open nor imported is left as it is.
- The server does not run the program or the code generator, so the messages of a
  target, such as a construct the C target cannot build, show at build time.
