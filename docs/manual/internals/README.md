---
title: Inside the compiler
description: A tour of the crates, how to build, test and format the workspace, and where to start on a change, for contributors.
---

# Inside the compiler

This chapter describes how the compiler is built and where contributors can find the
code for each stage. For commands, compiler stages and how programs run, see [The
compiler](../compiler/README.md) and [The VM](../vm/README.md).

## The pipeline in crates

Each stage is a crate, and each crate has a `run_phases` function that runs all the
stages before it, so we can run each stage alone on a source file.

```text
source text
   │  zakac_span            read, normalize, give each octet a position
   ▼
zakac_burmese_segmenter     vannas, digits, other characters
   ▼
zakac_lexer                 tokens             ZK01       lexer.md
   ▼
zakac_parser (+ zakac_ast)  the tree           ZK02       parser.md
   ▼
zakac_naturalness           wording warnings   ZK05       semcheck.md
zakac_semcheck              names, data kinds  ZK03 ZK04  semcheck.md
   ▼
zakac_ir_builder            IR, folding        ZK06       codegen.md
   ▼
zakac_borrow_checker        flow and lint      ZK07       borrow-checker.md
   ▼
zakac_codegen_target_vm     operations                    vm-target.md
   ▼                        (build --backend vm)
zaka_vm                     runs the operations (traps)   vm-internals.md

zakac_codegen_target_c      C for each module, then an executable
                            (build --backend c; after the flow check)

zakac_codegen_target_native CLIF, run at once or linked into an executable
                            (run --jit, build --backend native)
```

In words: the lexer (tokenizer) makes tokens, the parser builds the tree (AST), the
naturalness check and the stage of names and data kinds (name resolution and type
checking) check the tree, the IR builder lowers it and folds constants, the flow checker
(borrow checker) proves the program safe, and one of three code generators (backends)
writes the result.

## The workspace

The workspace has three folders: `compiler/` for the language tools, `website/` for the
web pages and `editors/` for the editor extensions. One `Cargo.toml` at the root lists
them with `members = ["compiler/*", "website/*", "editors/vscode"]`, and
`default-members` is just `compiler/zakac`. The Zed extension, `editors/zed`, is outside
the workspace (`exclude`): Zed builds it alone, with the target folder inside it.

| Crate | What it does |
| --- | --- |
| `zakac` | the `zakac` command: one program with the subcommands `segment`, `lex`, `parse`, `semcheck`, `irgen`, `borrowck`, `build`, `run`, `repl`, `lsp` and `fmt`; `build` hands the program to the code generator that `--backend` names |
| `zakac_span` | source files, the source map, positions and spans, the text normalizer (NFC plus a fixed order of Burmese signs), and `text_range`: spans back to ranges of the text as written, and UTF-8 to UTF-16 offsets |
| `zakac_burmese_segmenter` | cuts text into vannas, digits and other characters; the `segment` subcommand |
| `zakac_lexer` | tokens, particles, comments and text tables, the reserved words and their spellings; `TokenRole`, the role of a token that decides its color |
| `zakac_ast` | the tree: statements, expressions, data kinds, the verb table, visitors, the S-expression form |
| `zakac_parser` | tokens to tree; it goes on after a bad sentence; the loader of imports and of the stdlib |
| `zakac_naturalness` | warnings about wording that is allowed but not natural (ZK05) |
| `zakac_semcheck` | name resolution, scopes, data kinds, literal fitting, captures, unsafe code and the pointer kind |
| `zakac_ir_builder` | the IR, lowering from the tree, constant folding |
| `zakac_borrow_checker` | the flow check over the IR, and the lint on unsafe code (ZK07); the crate keeps its old name; `fixes::Recompile` runs the stages again on a patched text for the check of fixes |
| `zakac_codegen_target_vm` | the operations of the VM, and the asm listing |
| `zakac_codegen_target_c` | the C of each module, the runtime headers (`runtime/`) and the runtime lib built into it, and the build of an executable with a C compiler |
| `zakac_codegen_target_native` | the IR lowered to Cranelift IR: run at once in `zakac` (`run --jit`), or written as an object file and linked (`build --backend native`) |
| `zaka_runtime` | the runtime of native programs, in Rust: a static lib that the C and native targets link, and that `zakac` links for the JIT |
| `zakac_lsp` | the language server: it checks the files an editor has open and returns errors, colors, hovers, completions, definitions, uses, renames, the outline and quick fixes; the `lsp` subcommand, behind the feature `server`, which `zakac` turns on; without it the crate builds for the browser, and the explorer uses it |
| `zakac_formatter` | the formatter: the tokens of a file printed again with the space between them set, the rewrites of words, and the check of its output; the `fmt` subcommand and the program `zakafmt`, behind the feature `cli` ([The formatter](formatter.md)) |
| `zaka_vm` | the VM: the stack of values, storage blocks, traps; the `run` subcommand |
| `zaka_host` | the functions the host serves to a program (`HOST_FNS`), the values that cross over, and the worlds those functions reach: the terminal and one in storage |
| `zakac_diagnostics` | `Diagnostic`, the `Code` catalogue, one constructor for each message, the texts in `locales/codes.toml`; its build script reads the page of each code from the See column of the page of error messages |
| `zakac_diagnostics_fixes` | the fixes of a message: the rules that read the source text, and the check that compiles each fix again before it is shown ([Fixes](fixes.md)) |
| `zakac_reporter` | draws a diagnostic with its source line, the span marked in it, and the caret rows when asked for; in learner mode, the explanation of its code and the link to its page |
| `zakac_burmese_alignment` | finds the terminal and how it draws Burmese text; places a caret row where a terminal that flows Burmese text draws the line |
| `zakac_session` | the artifact envelope (text and raw forms), command input and output |
| `zakac_host_errors` | errors of the driver (a missing file, a bad artifact); these get no code |
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

An arrow means "uses". The foundation crates at the bottom are used in almost all the
rest, so the picture leaves out the arrows to them.

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

The two other code generators sit beside `zakac_codegen_target_vm`: the C target uses the
same stages up to the flow checker, and the native target uses the C target, the VM and
`zaka_runtime` as well.

Two facts about the picture help when a build fails to link. `zakac_ast` sits above the
lexer because the tree contains lexer words (`Kw`, `Particle`), and the parser, the
naturalness check and each later stage read the tree. And `zaka_vm` sits above
`zakac_codegen_target_vm`, so the code generator never needs the VM, and another backend
can read the same IR.

## Build, test and format

```sh
cargo build -p zakac                       # the compiler
cargo run -q -p zakac -- run hello.zaka    # compile and run a file
cargo run -q -p zakac -- lex hello.zaka    # stop after one stage and dump it
cargo nextest run --workspace              # all tests (default-members is zakac alone)
cargo nextest run -p zakac_parser          # tests of one crate
cargo +nightly fmt                         # format
```

`mise run test` runs the workspace with `cargo nextest`, and `test:compiler`,
`test:books` and `test:web` run a part of it. `mise run format` runs the formatters. The
tasks that serve the explorer and the site with live reload (`explorer:dev`,
`website:dev`) are Bun Shell scripts in `.mise-tasks/`.

CI (`.github/workflows/test.yml`) runs the workspace on Linux (x86-64), macOS and
Windows, with a C compiler on the PATH of each (gcc, clang, MinGW gcc). It sets
`ZAKA_REQUIRE_CC=1`, so the tests that build native executables fail when no C compiler
is found; on a host with `ZAKA_REQUIRE_CC` unset, these tests note it and pass.

The repo has these sorts of test:

| Tests | Where | What these check |
| --- | --- | --- |
| unit and stage tests | the `tests/` folder of each crate, one file for each module, with shared helpers in `tests/common/mod.rs` | the code of one crate |
| golden files | `tests/inputs/*.zaka` of a stage crate against `tests/expected/*` (`.zktokt`, `.zkastt`, `.zksemt`, `.zkirt`, `.zkbckt`, `.zkct`, `.zkasm`) | the artifact of a stage, line for line |
| samples | the programs in `samples/`, with the output of each in `compiler/zaka_vm/tests/expected` | the whole pipeline |
| book tests | `mise run test:books` (`cargo nextest run -p zaka_website books::`) | each code fence of Learn and Docs, and each link |
| mutants | `compiler/zakac/tests/mutants.rs` | that no damaged program crashes a stage |
| backtrace | `compiler/zakac/tests/backtrace.rs` | how much of the stack a stop lists |

Some notes on each:

- **Golden files.** After an intended change, run the test with `ZAKA_BLESS=1` to rewrite
  the expected files, and read the diff before committing.
- **Samples.** Each group of `samples/` runs through the whole pipeline, except
  `diagnostics` and `mistakes`, which contain programs that fail on purpose. The flow
  checker has an oracle test: each sample that passes the check without raw addresses
  must run with no use after free, double free or bad free.
- **Book tests.** These run each fenced `zaka` block of the Learn pages and of this
  manual and compare the result with what the page promises (output, a code, a stop). A
  link to a page that is not in a `CONTENTS.md` fails the test too. One more test of
  `zaka_website` uses the same programs: `stops_in_c.rs` builds each `zaka-stop` program
  of the books with the C target (when a C compiler is on the `PATH`) and checks that it
  prints the same output and the same stop reason as the VM. It leaves out unsafe code
  (`စိတ်မချရ`), programs that declare a C function (`မှလာသော`), and the pages under
  `docs/manual/vm/`, which describe the limits of the VM itself. Addresses (`0x...`) are
  compared as `0x?`.
- **Mutants.** The test changes the programs of `samples/` and the Zaka fences of `docs/`
  a little (a line cut, doubled or moved, a word or a character cut, a piece such as `(`
  or `။` put in, the end cut off) from a fixed seed, and runs each mutant through each
  phase, a short VM run (200,000 operations) and the C text of the C target. It fails on
  a panic or on a run past 20 s. A program that declares a C function is compiled, not
  run. `ZAKA_MUTANT_SEED` sets the seed, `ZAKA_MUTANT_COUNT` the number of mutants (1000
  when left out), and `ZAKA_MUTANT_LAST` names a file that keeps the mutant that is
  running, to find one that crashed the whole process.
- **Backtrace.** The test runs `zakac` with `ZAKA_BACKTRACE` unset, `full` and `0`, and
  checks how much of the stack each stop lists.

A new feature needs a test in each stage it touches, and a sample when a user can see it.

## Where to start

| To change | Read |
| --- | --- |
| a reserved word, a phrase, a spelling, a number or text form | [The lexer](lexer.md) |
| the parser's tiers, the verb table, recovering from a bad sentence | [The parser](parser.md) |
| a statement or a declaration form | [Statements and declarations](parser-statements.md) |
| an expression, a call form, a verb of the core module | [Expressions and calls](parser-expressions.md) |
| a data kind phrase, generics, the loader of imports | [Data kinds, generics and imports](parser-kinds.md) |
| a rule about names or data kinds, a wording warning | [Names and data kinds](semcheck.md) |
| a flow check or a lint rule | [The flow checker](borrow-checker.md) |
| the IR, lowering, folding | [The IR](codegen.md) |
| an operation of the VM, or the code that writes it | [The VM target](vm-target.md) |
| the run loop of the VM, its traps or its host functions | [Inside the VM](vm-internals.md) |
| the C of a native program, or its runtime | [The C target](c-backend.md) |
| the Cranelift code of a native program, or the JIT | [The native target](native-backend.md) |
| a message or a code | [Diagnostics](../compiler/diagnostics.md) |

Next: [The lexer](lexer.md)
