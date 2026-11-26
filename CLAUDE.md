# Zaka

Zaka (စံကား) is a programming language based on a subset of Burmese. `zakac` compiles a
program for the Zaka VM. The website teaches the language (Learn), documents it (Docs),
and hosts a compiler explorer that runs in the browser.

## Repo map

| Path | What it holds |
| --- | --- |
| `compiler/zakac` | the `zakac` command; each stage adds a subcommand through its own `cli` module |
| `compiler/zakac_*` | one crate per stage (`burmese_segmenter`, `lexer`, `parser`, `naturalness`, `semcheck`, `ir_builder`, `borrow_checker`, `codegen_target_vm`, `codegen_target_c`, `codegen_target_native`), plus shared crates (`span`, `ast`, `diagnostics`, `reporter`, `session`, `i18n`, `cli_common`, `rust_sugars`, ...) and `diagnostics_fixes` (the checked fixes of a message) |
| `compiler/zaka_vm` | the VM and its checked mem |
| `compiler/zaka_host` | the host functions a program can call: their table, values and worlds |
| `compiler/zaka_runtime` | the runtime of native programs, in Rust: a static lib that `zakac` carries and the C and native targets link |
| `compiler/zakac_formatter` | the formatter: `zakac fmt` and `zakafmt` (feature `cli`); the language server and the explorer use the same code |
| `compiler/zakac_lsp` | the language server (`zakac lsp`, feature `server`): errors, colors, hover, completion, definitions, uses, renames and the outline; the explorer runs the same code in the page |
| `editors/vscode` | the VS Code extension: a Rust client built with wasm-pack; its manifest, comment rules and grammar are TOML that `build.rs` turns into JSON |
| `editors/zed` | the Zed extension; outside the workspace, as Zed builds it on its own |
| `website/zaka_website` | the pre-rendered Leptos site: home page, Learn, Docs |
| `website/zaka_explorer` | the compiler explorer: its own Trunk app (WASM, feature `csr`), and an island of the site at `/explorer/` |
| `website/zaka_web_ui_common` | UI parts, theme tokens and site URLs that the site and the explorer share |
| `docs/learn/` | Learn: the language taught through runnable examples (`/learn/`) |
| `docs/manual/` | Docs: the full rules, the compiler, the VM, concepts, internals, reference (`/docs/`) |
| `docs/*.internal.md` | design notes; gitignored, never published or cited |
| `samples/` | example programs with their expected output |
| `stdlib/` | the stdlib modules, in Zaka; the compiler carries each one (`STD_MODULES` in `zakac_parser/src/load.rs`), imported as `"စံကား/<name>"` |

A program passes through: segmenter → lexer → parser → naturalness → names and kinds
(semcheck) → lower to IR → fold constants → checker (flow facts, and the lint of marked
code) → codegen → VM. `zakac build
--backend vm|c|native` picks the codegen: ops for the VM (`.zkc` / `.zkct`), C and a native
executable, or Cranelift and a native executable; `zakac run` runs a source file as
Cranelift code at once when it can and on the VM otherwise; `--vm` and `--jit` pick one.

## Commands

| Task | Command |
| --- | --- |
| Run a program | `cargo run -q -p zakac -- run program.zaka` |
| Test it all | `mise run test` (`cargo nextest run --workspace`) |
| Test the compiler and the VM, not the web crates | `mise run test:compiler` |
| Test the Learn and Docs pages | `mise run test:books` (`cargo nextest run -p zaka_website books::`) |
| Test the web crates: the site, the books and the explorer | `mise run test:web` |
| Headers for the editor of the C runtime | `mise run editor:mock-header` (a build of hello and of a lib in `target/c-headers`, which `runtime/.clangd` reads; again after a change to `zaka_runtime`) |
| Format | `mise run format` (rustfmt, clang-format on the C runtime headers, then `zakac fmt` on the samples and the stdlib; run after each edit) |
| Serve the site | `mise run website:dev` (port 8000, live reload) |
| Serve the explorer | `mise run explorer:dev` |
| Build the VS Code extension | `mise run editor:build-vscode` (wasm-pack, bun, vsce); `editor:install-vscode` and `editor:install-cursor` install it |
| Build and publish the site | `mise run pages:publish` (the release snapshots, `dist`, then Cloudflare Pages) |
| Make a release | Publish it on GitHub with the tag `v<version>` of `zakac` and the extension; `.github/workflows/release.yml` attaches the executables, the `.vsix` and the site snapshot, and publishes the extension when `VSCE_PAT` / `OVSX_PAT` are set |
| Snapshot the site of a release | `mise run pages:snapshot -- [ref] [version]` (the working tree when no ref; to `target/site-snapshots`), then `pages:snapshot-upload <tag>` when a release ships |
| Serve `dist` as the host does | `mise run pages:serve` (wrangler, 404 pages included) |

## Learn and Docs pages

Both books are Markdown under `docs/`; each book's `CONTENTS.md` lists its pages in
order. The build script of `zaka_website` reads them into `src/book/`, and one renderer
and one set of tests serve both books.

- Each page starts with front matter (`title`, `description`), then a `# Title`. `##`
  heads make the "On this page" list; no other heads get anchors.
- A stdlib page, `manual/stdlib/<module>.md`, names its module in the front matter
  (`module: စံကား/<name>`). The site adds the reference of the module after the page: each
  exported name with its declaration and its `///` lines, read from the stdlib source.
- Link with relative `.md` paths. A link from one book to the other goes through the
  `docs/` folder: `../../manual/reference/errors.md` from a Learn page.
- Code fences run in the tests:
  - `zaka` must run, and the `text` fence right after it holds its exact output.
  - `zaka-error ZK0701` must fail to compile with that code.
  - `zaka-warning ZK0501` must compile with that warning.
  - `zaka-stop` must stop at run time, with its output in the `text` fence after it.
  - `stdin` right after a `zaka` fence, before its `text` fence, holds what the program
    reads from standard input; the page shows it under the program.
  - `zaka-file` shows one file of a program in several files: painted, never run, and
    it must lex with no error. Start it with a `// <file>.zaka` line.
  - Options can follow the fence word, as the flags of the same names:
    `lint-error=mem`, `lint-warn=mem`, `lint-allow=mem`, `lint-warn=doubts`,
    `lint-allow=`, `lint-warn=` or `lint-error=` with each other lint group or code
    (`lint-allow=spoken`), `no-unsafe=on`, `overflow=stop`.
  - Each fence compiles in REPL mode, so it can leave out the program start block; a
    fence with a start block compiles as it is. `repl=off` turns the mode off, for a
    fence that teaches ZK0327.
- The tests also check that each link reaches a page, each `#mark` reaches a `##` head,
  and each code in `zakac_diagnostics` has a row on `manual/reference/errors.md`.
- A Zaka term carries its standard term in parentheses, as in "data kind (type)" or "table
  (map)": at its first use on a page, and again at the first use under a `##` head
  whose topic it is. The Zaka term stands first and remains the main word; later
  mentions stand alone. The pairs come from `manual/reference/terms.md`; a new Zaka
  term gets a row there.
- Terms the books use: "data kind" (not a bare "kind" as the term), "thread" (not
  "job"), "function signature" (not "header"), "mutable parameter" and "writable
  parameter" (not "changed" and "filled"), "operation" (not "op"), "core module" (not
  "the core"), "program start block" (not "start block"), "block closer" (not "closer").
- A Learn page has one shape: a short intro, `## You will learn`, the smallest example
  that runs (with no program start block, except in the Hello, Programs and Modules
  chapters, the first example of the tour, and an example whose point needs the block,
  such as a function or a declaration inside it) with its pattern line (`<name> ဟူသော <kind> တစ်ခု`) and a word-for-word gloss,
  one idea in each `##` section, `## Details` for the edge rules (with links to Docs),
  `## Recap`, `## Hands on`, then `Next:`. `learn/tour.md` is the fast path for a reader
  who knows another language.
- Write the prose in plain English, the words a developer would use when talking to a
  workmate: "Zaka uses Burmese sentences", not "Zaka reads as Burmese". Sentences of
  medium length that join related ideas, not a row of short ones. Say how a form is used,
  not what it "names": "In the function signature, we can use `<name> ဟူသော <kind>` to
  define a parameter and `ပြန်ပေးမည်` to define a return value." Speak as "we", or use the
  passive ("a variable is declared with `ကိန်းရှင်`") or the imperative in steps ("save
  the file, then run it"). When the no-`y` rule rules a word out, pick another common
  word or reshape the sentence; never an odd or bookish phrase.
- Learn shows; Docs states the rules in full. When the code and a note disagree, the
  code wins: check claims against the compiler before writing them down.

## Keeping the docs current

A change is not done until the documents that describe it match the code. Before
reporting a change as done, go through both sets and update each page the change
touches:

- **Learn and Docs** (`docs/learn/`, `docs/manual/`): the language page of the feature
  and its Learn page, both `CONTENTS.md` files, `reference/errors.md` (codes and short
  kind names), `reference/reserved-words.md`, `reference/grammar.md` (productions),
  `reference/terms.md` (a new Zaka term), the `///` lines and the `stdlib/` page of a
  stdlib module that changes, `compiler/commands.md` and `compiler/build.md`
  (flags), `vm/instructions.md`, `vm/traps.md` and `vm/traps-unsafe.md` (operations and
  traps), `learn/tour.md` when a feature changes what a newcomer sees, and the
  `internals/` page of each stage the change touches. Long topics are split into sibling
  pages (`language/functions-*.md`, `internals/parser-*.md`): find the page with
  `CONTENTS.md`.
- **Internal notes** (`docs/*.internal.md`): the design note of the feature (record what
  was built and where the build left the plan), `ROADMAP.internal.md` (the status of the
  milestone), the checklist in `LANGUAGE.internal.md` §9, and the note of each stage the
  change touches (`CODEGEN.internal.md`, `OWNERSHIP.internal.md`, ...).
- A "known cost", "open" or "not done" line that the change resolves is updated in the
  same pass, and so is a later change to a feature: a new flag, a mode or a fix.
- Grep the docs for the old names, codes and wording the change replaces.

## Writing rules

### No lowercase letter `y`

Do not use the lowercase letter `y` in what we author: identifiers (functions,
variables, fields, modules, tests), file names, doc comments, descriptions, locale
strings, and the Learn and Docs pages (front matter, headings, prose, tables, diagram
labels, and the English comments in examples). Pick a different word instead.

| Avoid | Use |
| --- | --- |
| `directory` | `dir` or `folder` |
| `bytes` | `raw` or `u8s` |
| `payload` | `data` or `content` |
| `memory` | `mem` or `storage` |
| `type` (as a word we choose) | `kind` |
| `only`, `any`, `may`, `by`, `every` in prose | reword the sentence |

Exceptions, because we cannot rename them:

- Reserved Rust words (`dyn`, `type`, `try`, ...).
- std and external crate APIs (`is_empty`, `Display`, `Copy`, `Sync`, `as_bytes`, ...).
- Tool names and config names (`rustfmt.toml`, `mise.toml`, `cargo +nightly fmt`).
- Names that macros require, such as `_rust_i18n_try_translate`.
- Proper nouns and format names imposed from outside (`Myanmar Sabae`, `TrueType`,
  `WebAssembly`, `@keyframes`, ...).
- Zaka code in examples and the output it prints.
- Standard computing terms in the prose of Learn and Docs (`type`, `array`, `memory`,
  `syntax`, `library`, `copy`, `key`, `byte`, `body`, `binary`, `entry point`, ...), so
  that a reader who knows another language finds the word known to them. Each such term
  has a row on `docs/manual/reference/terms.md`. Identifiers, file names, comments in
  code and locale strings keep the rule.

Never reuse an exception in names we choose ourselves. Do not rename `Cargo.lock`
entries or sample data. This file follows the rule too; other agent instruction files
are not covered. To check, grep for `y` and confirm that each hit left is an exception.

### Burmese mark order

Write the asat (`်`, U+103A) before the dot below (`့`, U+1037): `ည` + `်` + `့`. The
reverse order (U+1037 then U+103A) is wrong in code, tests, samples, docs and locale
strings alike. The one exception is the input of the normalizer tests in
`compiler/zakac_span/src/`. Hand-entered Burmese often comes out reversed, so check that
the sequence `့်` occurs nowhere else before saving.

### Comments

State the rule in the comment itself. Never cite the internal notes ("design §3.3",
"LANGUAGE.internal.md"): the notes are not published, and their section numbers drift.
Doc comments use `# Arguments` and `# Returns` sections, as the surrounding code does.

## Writing Rust

### Flat code, shallow nesting

Keep nesting to two levels where possible and never deeper than three. Do not let the
main path drift to the right. The rules of this section and the next hold for all Rust:
crates, build scripts, tests and macros.

- Use guard clauses that return at the start instead of wrapping the main logic in
  `if` / `else`.
- Use `let ... else`, `?`, `if let ... else`, `matches!`, and `Option` / `Result`
  combinators (`map`, `and_then`, `ok_or`, `unwrap_or_else`) instead of nested `match`.
- Split deep blocks into small helper functions with names that tell what each one does.
- Prefer iterator chains over nested `for` loops with `if` inside them.

### Sugar macros

`zakac_rust_sugars` has one-line macros for common control flow. Use them where a macro
fits, in place of multi-line `if` / `for` / `while` blocks:

- `ret_if!`, `ret_true!`, `ret_false!`, `ret_tern!` for guard returns.
- `do_if!`, `tern!`, `extract!` for conditionals and pattern extraction.
- `do_for!`, `do_while!` for loops.

Read their doc comments in `compiler/zakac_rust_sugars/src/` for the exact grammar. If a
macro makes the code harder to read, plain Rust is fine.

### Building values and naming variants

- Derive `bon::Builder` on data structs and build them with `X::builder()...build()`
  instead of struct literals. Leave `const fn` constructors, one-field wrappers, enum
  variants and destructuring patterns unchanged.
- For the name of an enum variant as a string, derive strum's `IntoStaticStr` and convert
  with `.into()`; do not write `match` arms that return string literals.
- Reach for a maintained crate before hand-rolling a basic helper (debounce, storage,
  event listeners, ...). In Leptos code that means the `leptos-use-*` micro crates.

### Formatting

`rustfmt.toml` sets the format: 4 spaces, max width 88, braces on the next line. Run
`cargo +nightly fmt` after editing.

## Writing C

The runtime is Rust (`compiler/zaka_runtime`). Its C header, `zaka_runtime.h`, is
written from the Rust through cbindgen at build time (`zaka_runtime/cbindgen.toml`): the
functions, the structs the generated code shares with the runtime, and the ids of traps,
rights and kinds. Change those in the Rust, never in C. The hand-written headers of the C
target (`compiler/zakac_codegen_target_c/runtime/`) hold the inline code of each
operation; these follow the rules above: flat code with guard clauses, no lowercase `y`
(reserved C words, libc names and proper nouns are the exceptions), and comments that
state the rule.

- The headers and the C the compiler writes are strict C11 (`-std=c11
  -pedantic-errors` builds them): no statement expressions or other GNU forms.
  GCC and Clang builtins stand behind `__GNUC__`, each with a C11 path beside
  it.
- Doc comments are Doxygen: `/** ... */` on each function and function-like
  macro, with `@param` and `@return`. `zaka_runtime.h` carries the first paragraph
  of each Rust doc; the Rust holds the full doc.
- A struct shared with the runtime has a row in `tests/forms.rs` of the C target,
  which checks its C form against the Rust one; the C the compiler writes names each of
  its fields.
- Guards use the macros in `runtime/zaka_c_sugars.h` (`RET_IF`, `DONE_IF`, `DO_IF`,
  `NEXT_IF`, `LEAVE_IF`, `DO_FOR`), as Rust uses `zakac_rust_sugars`.
- `.clang-format` matches `rustfmt.toml`. `mise run format` runs both formatters;
  mise pins clang-format.

### Paths on each host

The compiler and its tests run on Windows as well as macOS and Linux.

- A path that becomes text a reader or a test sees (diagnostics, artifacts, unit names,
  expected output) uses `/`. `Path::display()` writes `\` on Windows, so turn
  `std::path::MAIN_SEPARATOR` into `/` before a path becomes text.
- Build paths with `Path::join` and walk them with `components()`; never split or join
  path strings on `/` or `\`.
- Compare paths as `Path` or `PathBuf`, not as strings: Windows treats `/` and `\` as one
  separator, and its absolute paths start with a drive (`C:`).
- A test that spells a path as text, such as `"lib/c.zaka"`, must pass on Windows too:
  compare it with the `/` form, not the host form.

## Diagnostics

Each source diagnostic has a code in `compiler/zakac_diagnostics`: one `define_codes!` row
in `src/code.rs` (`ZK01` lexer, `ZK02` parser, `ZK03` resolve, `ZK04` kinds, `ZK05`
naturalness, `ZK06` folding, `ZK07` ownership), its entries in `locales/codes.toml`, and a
constructor function in the file of its stage. A new code also needs a row on
`docs/manual/reference/errors.md`; the book tests fail without it. Host and driver errors
(file access, artifact formats) live in `zakac_host_errors` and get no codes.

Stage artifacts in text form are YAML (a `# zakac-<stage>-artifact v1` line, then YAML via
`serde-saphyr`). Raw artifacts use the stage's own extension and text adds `t`
(`.zktok` / `.zktokt`); never `.yaml` or `.txt`. Expected test output lives in
`tests/expected/`, with inputs in `tests/inputs/`.

## Web code

- Keep the web pieces in Rust (Leptos, cargo-leptos, Trunk). Generated wasm-bindgen glue
  is fine; hand-written JS is not, except a small inline script where nothing else works.
- Write Leptos UI with the builder API (`div().class(..).child(..)`), not the `view!`
  macro. Components are plain snake_case functions; islands are the exception.
- Each component lives in `src/components/<name>/{mod.rs, mod.sass}`. Sheets are indented
  Sass (`.sass`), never SCSS. Sheets go through `turf::style_sheet!`, which hashes the
  class names; Rust uses the generated `ClassName::FOO` constants, never class strings.
  Theme tokens and page-wide rules remain global and unhashed.
- Web fonts come from `@fontsource` packages that Trunk fetches, never from Google Fonts
  links. Build scripts (`zaka_web_builder`) content-hash our own asset files.
- Text the reader sees goes in the locale tables (`locales/*.toml`), under the no-`y` rule.
- The latest release is served at the root; each old release has a snapshot under
  `/v<major>.<minor>/`, built once at its release with `ZAKA_SITE_BASE` and kept as a
  GitHub release asset. A path from the site root goes through `SITE_BASE` (the
  constants of `zaka_web_ui_common/src/site.rs`, or `rooted`), with two exceptions:
  `/fonts/`, which the snapshots share with the latest (each snapshot brings the font
  folders it names), and `/versions.toml`, the list of releases that `pages:build` writes
  and the version picker of each snapshot reads. Its fields can be added to, never renamed
  or removed: an old picker skips a field it does not know, but drops a list it cannot
  read.
