---
title: Inside the compiler
description: A tour of the crates, how to build, test and format the workspace, and the house coding rules, for contributors.
---

# Inside the compiler

This chapter is for contributors. It describes the code as it stands now and changes with
it: when a crate is reworked, its page is rewritten, so a page here is a map of the code
as it is now and not a promise. Users of the language find what is needed in
[The compiler](../compiler/README.md) and [The VM](../vm/README.md); the stages are
named there, and the pages of this chapter open them up.

## The pipeline in crates

Each stage is a crate, and each crate has a `run_phases` function that runs all the
stages before it, so each stage can be tried alone from a source file.

```text
source text
   │  zakac_span            read, normalize, give each octet a position
   ▼
zakac_burmese_segmenter     vannas, digits, other chars
   ▼
zakac_lexer                 tokens             ZK01     lexer.md
   ▼
zakac_parser (+ zakac_ast)  the tree           ZK02     parser.md
   ▼
zakac_naturalness           wording warnings   ZK05     semcheck.md
zakac_semcheck              names and kinds    ZK03 ZK04  semcheck.md
   ▼
zakac_ir_builder            IR, folding        ZK06     codegen.md
   ▼
zakac_borrow_checker        flow and lint      ZK07     borrow-checker.md
   ▼
zakac_codegen_target_vm     ops                         codegen.md
   ▼                        (build --target vm)
zaka_vm                     runs the ops       (traps)

zakac_codegen_target_c      C for each module, then an executable
                            (build --target c; after the flow check)
```

## The workspace

The workspace has three folders: `compiler/` for the language tools, `website/` for the
web pages and `editors/` for the editor extensions. One `Cargo.toml` at the root lists
them with `members = ["compiler/*", "website/*", "editors/vscode"]`, and
`default-members` is just `zakac`. The Zed extension, `editors/zed`, stands outside the
workspace: Zed builds it on its own, with the target folder inside it.

| Crate | Job |
| --- | --- |
| `zakac` | the `zakac` command: one program with the subcommands `segment`, `lex`, `parse`, `semcheck`, `irgen`, `borrowck`, `build`, `run` and `lsp`; `build` hands the program to the target `--target` names |
| `zakac_span` | source files, the source map, positions and spans, the text normalizer (NFC plus a fixed order of Burmese signs), and `text_range`: spans back to ranges of the text as written, and UTF-8 to UTF-16 offsets |
| `zakac_burmese_segmenter` | cuts text into vannas, digits and other chars; the `segment` subcommand |
| `zakac_lexer` | tokens, particles, comments and string tables; `TokenRole`, the role of a token that decides its color |
| `zakac_ast` | the tree: statements, expressions, kinds, the verb table, visitors, the S-expression form |
| `zakac_parser` | tokens to tree; carries on after a bad sentence |
| `zakac_naturalness` | warnings about wording that is allowed but not natural (ZK05) |
| `zakac_semcheck` | name resolution, scopes, kinds, literal fitting, captures, unsafe code and the pointer kind |
| `zakac_ir_builder` | the IR, lowering from the tree, constant folding |
| `zakac_borrow_checker` | the flow check over the IR, and the lint on unsafe code (ZK07); the crate keeps its old name |
| `zakac_codegen_target_vm` | the ops of the VM, and the asm listing |
| `zakac_codegen_target_c` | the C of each module and its runtime (`runtime/`), and the build of an executable with a C compiler |
| `zakac_lsp` | the language server: checks the files an editor holds open and sends back errors, colors, hovers, completions, definitions, uses, renames and the outline; the `lsp` subcommand, behind the feature `server`, which `zakac` turns on; without it the crate builds for the browser, and the explorer uses it |
| `zaka_vm` | the VM: stack of values, storage blocks, traps; the `run` subcommand |
| `zaka_host` | the functions the host serves to a program (`HOST_FNS`), the values that cross over, and the worlds those functions reach: the terminal and one in storage |
| `zakac_diagnostics` | `Diagnostic`, the `Code` catalogue, one constructor per message, texts in `locales/codes.toml` |
| `zakac_reporter` | draws a diagnostic with its source line, the span marked in it, and the caret rows when asked for |
| `zakac_burmese_alignment` | finds the terminal and how it draws Burmese text; places a caret row where a terminal that flows Burmese text draws the line |
| `zakac_session` | the artifact envelope (text and raw forms), command input and output |
| `zakac_host_errors` | errors of the driver (a missing file, a bad artifact); no code is attached |
| `zakac_cli_common` | the `Command` trait, the shared `main`, helpers for `clap`, and the config files (`config.rs`), which set the default of each flag before `clap` parses |
| `zakac_i18n` | locale set-up for the message texts |
| `zakac_consts` | build number, licence and author, used in all programs |
| `zakac_rust_sugars` | one-line control-flow macros used all through the code |
| `zaka_explorer` | the compiler explorer: the whole pipeline in the browser (Leptos, built with Trunk) |
| `zaka_website` | the site with its books, served with `cargo-leptos` |
| `zaka_web_ui_common` | pieces used in both the site and the explorer: fonts, themes, controls |
| `zaka_web_builder` | build-script helper that names asset files after their hash |
| `zaka_vscode` | the VS Code extension: a Rust client built for Node.js with wasm-pack, which starts `zakac lsp` |
| `zaka_html2png` | renders the cover cards of the site to PNG with a headless browser |

## How the crates link

An arrow reads "uses". The foundation crates at the bottom are used in almost all the
rest; the arrows to them are left out of the picture.

```text
 zakac        zaka_explorer        zaka_website
   │                │                    │
   └────────────────┼────────────────────┘
                    ▼
                zaka_vm ─────────▶ zakac_codegen_target_vm
                                        │
                                        ▼
                               zakac_borrow_checker
                                        │
                                        ▼
                                zakac_ir_builder
                                        │
                                        ▼
                      zakac_naturalness ◀── zakac_semcheck ──▶ zakac_parser
                                │                  │                 │
                                └────────▶ zakac_ast ◀───────────────┘
                                                │
                                                ▼
                                          zakac_lexer ──▶ zakac_burmese_segmenter

 foundation:  zakac_span  zakac_diagnostics  zakac_reporter  zakac_session
              zakac_cli_common  zakac_host_errors  zakac_i18n  zakac_consts
              zakac_rust_sugars  zakac_burmese_alignment
```

Two facts about the picture help when a build fails to link. `zakac_ast` sits above the
lexer because the tree holds lexer words (`Kw`, `Particle`), and the parser, the
naturalness check and each later stage read the tree. And `zaka_vm` sits above
`zakac_codegen_target_vm`, so the code generator never needs the VM; another backend could read
the same IR.

## Build, test and format

```sh
cargo build -p zakac                       # the compiler
cargo run -q -p zakac -- run hello.zaka    # compile and run a file
cargo run -q -p zakac -- lex hello.zaka    # stop after one stage and dump it
cargo test --workspace                     # all tests (default-members is zakac alone)
cargo test -p zakac_parser                 # tests of one crate
cargo +nightly fmt                         # format
```

`mise run test` runs the workspace with `cargo nextest`, and `mise run format` is the
formatter. The tasks that serve the explorer and the site with live reload
(`explorer:dev`, `website:dev`) are Bun Shell scripts in `.mise-tasks/`.

CI (`.github/workflows/test.yml`) runs the workspace on Linux (x86-64), macOS and Windows,
with a C compiler on the PATH of each (gcc, clang, MinGW gcc). It sets `ZAKA_REQUIRE_CC=1`:
the tests that build native executables then fail when no C compiler is found, where on a
host with `ZAKA_REQUIRE_CC` unset the tests note it and pass.

Four kinds of test sit in the repo:

- **Unit and stage tests** in each crate's `tests/` folder, one file per module, with
  shared helpers in `tests/common/mod.rs`.
- **Golden files.** The `tests/inputs/*.zaka` of a stage crate are run through the stage
  and compared with `tests/expected/*` (`.zktokt`, `.zkastt`, `.zksemt`, `.zkirt`,
  `.zkbckt`, `.zkct`, `.zkasm`). After an intended change, run the test with
  `ZAKA_BLESS=1` to rewrite the expected files, and read the diff before committing.
- **Samples.** The programs in `samples/` (`basics`, `control`, `functions`, `heap`,
  `lists`, `programs`) run through the whole pipeline; `compiler/zaka_vm/tests/expected`
  holds the output of each. The groups `diagnostics` and `mistakes` hold programs that
  fail on purpose and are skipped there. The flow checker has an oracle test: each
  sample that passes the check without raw addresses must run with no use after free,
  double free or bad free.
- **The book tests.** `cargo test -p zaka_website --test tests books::` runs each fenced
  `zaka` block of the Learn pages and of this manual and compares the result with what
  the page promises (output, a code, a stop). A link to a page that is not in a
  `CONTENTS.md` fails the test too. One more test of `zaka_website` uses the same
  programs:
  - `stops_in_c.rs` builds each `zaka-stop` program of the books with the C target (when a
    C compiler is on the `PATH`) and checks that it prints the same output and the same
    stop reason as the VM. It leaves out unsafe code (`စိတ်မချရ`), programs that declare
    a C function (`မှလာသော`), and the pages under `docs/manual/vm/`, which tell the
    limits of the VM itself. Addresses (`0x...`) are compared as `0x?`.
- **The mutants.** `compiler/zakac/tests/mutants.rs` changes the programs of `samples/`
  and of the Zaka fences of `docs/` a little (a line cut, doubled or moved, a word or a
  character cut, a piece such as `(` or `။` put in, the end cut off) from a fixed seed,
  and runs each mutant through each phase, a short VM run (200,000 ops) and the C text of
  the C target. It fails on a panic or on a run past 20 s. A program that declares a C
  function is compiled, not run. `ZAKA_MUTANT_SEED` sets the seed, `ZAKA_MUTANT_COUNT`
  the number of mutants (1000 when left out), and `ZAKA_MUTANT_LAST` names a file that
  keeps the mutant that is running, to find one that crashed the whole process.
- **The backtrace.** `compiler/zakac/tests/backtrace.rs` runs `zakac` with
  `ZAKA_BACKTRACE` unset, `full` and `0`, and checks how much of the stack each stop lists.

A new feature wants a test in each stage it touches, and a sample when a user can see it.

## Where to start

| To change | Read |
| --- | --- |
| a reserved word, a phrase, a number or string form | [The lexer](lexer.md) |
| a sentence form, a verb, a statement | [The parser](parser.md) |
| a rule about names or kinds, a wording warning | [Names and kinds](semcheck.md) |
| a flow check or a lint rule | [The flow checker](borrow-checker.md) |
| the IR, folding, an op | [IR and code](codegen.md) |
| the C of a native program, or its runtime | [The C target](c-backend.md) |
| a message or a code | [Diagnostics](../compiler/diagnostics.md) |
