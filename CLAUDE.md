# Zaka

Zaka (စံကား) is a programming language based on a subset of Burmese. `zakac` compiles a
program for the Zaka VM. The website teaches the language (Learn), documents it (Docs),
and hosts a compiler explorer that runs in the browser.

## Repo map

| Path | What it holds |
| --- | --- |
| `compiler/zakac` | the `zakac` command; each stage adds a subcommand through its own `cli` module |
| `compiler/zakac_*` | one crate per stage (`burmese_segmenter`, `lexer`, `parser`, `naturalness`, `semcheck`, `ir_builder`, `borrow_checker`, `codegen`), plus shared crates (`span`, `ast`, `diagnostics`, `reporter`, `session`, `i18n`, `cli_common`, `rust_sugars`, ...) |
| `compiler/zaka_vm` | the VM and its checked mem |
| `website/zaka_website` | the pre-rendered Leptos site: home page, Learn, Docs |
| `website/zaka_explorer` | the compiler explorer: its own Trunk app (WASM, feature `csr`), and an island of the site at `/explorer/` |
| `website/zaka_web_ui_common` | UI parts, theme tokens and site URLs that the site and the explorer share |
| `docs/learn/` | Learn: the language taught through runnable examples (`/learn/`) |
| `docs/manual/` | Docs: the full rules, the compiler, the VM, concepts, internals, reference (`/docs/`) |
| `docs/*.internal.md` | design notes; gitignored, never published or cited |
| `samples/` | example programs with their expected output |

A program passes through: segmenter → lexer → parser → naturalness → names and kinds
(semcheck) → lower to IR → fold constants → borrow checker → codegen → VM.

## Commands

| Task | Command |
| --- | --- |
| Run a program | `cargo run -q -p zakac -- run program.zaka` |
| Test it all | `mise run test` (`cargo nextest run --workspace`) |
| Test the Learn and Docs pages | `cargo test -p zaka_website --test books` |
| Format | `cargo +nightly fmt` (run after each edit) |
| Serve the site | `mise run website:serve` (port 3000, live reload) |
| Serve the explorer | `mise run explorer:serve` |

## Learn and Docs pages

Both books are Markdown under `docs/`; each book's `CONTENTS.md` lists its pages in
order. The build script of `zaka_website` reads them into `src/book/`, and one renderer
and one set of tests serve both books.

- Each page starts with front matter (`title`, `description`), then a `# Title`. `##`
  heads make the "On this page" list; no other heads get anchors.
- Link with relative `.md` paths. A link from one book to the other goes through the
  `docs/` folder: `../../manual/reference/errors.md` from a Learn page.
- Code fences run in the tests:
  - `zaka` must run, and the `text` fence right after it holds its exact output.
  - `zaka-error ZK0701` must fail to compile with that code.
  - `zaka-warning ZK0501` must compile with that warning.
  - `zaka-stop` must stop at run time, with its output in the `text` fence after it.
  - `zaka-file` shows one file of a program in several files: painted, never run, and
    it must lex with no error. Start it with a `// <file>.zaka` line.
  - Options can follow the fence word: `mem-check=warn|off`, `mem-strict=on`, `no-raw=on`.
- The tests also check that each link reaches a page, each `#mark` reaches a `##` head,
  and each code in `zakac_diagnostics` has a row on `manual/reference/errors.md`.
- Learn shows; Docs states the rules in full. When the code and a note disagree, the
  code wins: check claims against the compiler before writing them down.

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
main path drift to the right.

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
