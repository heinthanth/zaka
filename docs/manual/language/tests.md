---
title: Tests
description: The test block, which zakac test runs and a build leaves out, its rules, how the command finds its files, its limits, what it prints and its exit code.
---

# Tests

A test block (unit test) is a piece of a file that `zakac test` runs and that `zakac run`
and `zakac build` leave out. It holds statements, as a function does, and its
[asserts](asserts.md) decide whether it passes: a test passes when it runs to its end and
fails when it stops. This page states the full rules, and the Learn book teaches tests in
[Tests](../../learn/error-handling/tests.md).

```zaka
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပေါင်း ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း(၂၊ ၃) == ၅ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း(၁၀၊ ၂၀) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃၀
```

Saved as `add.zaka`, the file holds a function, a test of it and a program start block.
`zakac run` runs the program start block and leaves the test out, while `zakac test` runs
the test in place of the program start block:

```console
$ zakac run add.zaka
၃၀
$ zakac test add.zaka
test add.zaka::ပေါင်း ... ok

test result: ok. 1 passed, 0 failed
```

## The form

```text
<name> ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<statements>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as | Standard term |
| --- | --- | --- |
| `<name> ဟူသော စမ်းသပ်ချက်` | "the test called `<name>`" | `#[test] fn name()`, `def test_name():` |
| `စမ်းသပ်မှု` | "a test", the same word | |

A test block is written as the head of a function with no parameters and no result, with
`စမ်းသပ်ချက်` (or `စမ်းသပ်မှု`, which means the same) where a function has `လုပ်ဆောင်ချက်`.
Both words are reserved. The statements end at the block closer, as those of a function
do.

A test block has three rules:

- It stands at the top level of a file, next to the functions and the program start
  block. A test inside a function, a block or a branch is ZK0267.
- It takes no parameters. A parameter after `သည်` is the parse error ZK0201, since the
  opener `အောက်ပါတို့ကို လုပ်ဆောင်မည်` must come right after it.
- It gives no result. A test that opens with `အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> ကို ပြန်ပေးမည်`
  is ZK0267.

No call can name a test, as no call can name the program start block: a call of the name
of a test finds no function and is ZK0303. A test can therefore share its name with a
function, and the test `ပေါင်း` above stands next to the function `ပေါင်း`. Two tests of
one file need names of their own, and a second test of a name the file has used is
ZK0268; tests of two files can share a name, since a report names each test with its
file.

## What a test can do

The statements of a test are those of a function. A test can declare variables, call the
functions of its file and of the files the file imports, declare a function inside it,
print, start [threads](threads.md), and read and write files and the environment with the
rights that `zakac test` grants. Its asserts are what make it fail, and so does each
other [stop](../vm/traps.md), such as a division with zero as divisor:

```zaka
ဆယ်ဆ ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဆယ်ဆ_လုပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၁၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ရ သည် ဆယ်ဆ_လုပ်(၄) ဖြစ်ပါစေ။
"ရ သည် {ရ}" ဟု ပြသမည်။
ရ == ၄၀ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

"ပြီး" ဟု ပြသမည်။
```

```text
ပြီး
```

This page runs a program with no program start block in REPL mode, as `zakac run --repl`
does, so the statement after the test runs and the test does not: a test runs under
`zakac test` alone. Under `zakac test`, the test passes, and its line `ရ သည် ၄၀` is not
shown, as for each test that passes.

## Builds leave tests out

`zakac run` and `zakac build` check each test as a function is checked, so a test with a
mistake in it stops the build, but the program that comes out carries no code of a test: each
test becomes a function with no statements, which nothing calls. A file can hold tests
and a program start block together, and `zakac run` runs the program start block.

A file of tests needs no program start block. `zakac test` checks each file as a module,
which asks for no program start block, while `zakac run` reports a file with neither a
program start block nor `--repl` as ZK0327. The [language
server](../compiler/editors.md) checks a file with a test block and no program start
block as a module too, so the editor shows no ZK0327 for a file of tests.

## Running the tests

```text
zakac test [flags] [PATH]...
```

Each `PATH` is a file or a folder, and a folder gives each `.zaka` file under it and
under its subfolders. With no `PATH`, the command reads the paths of `--paths`, a comma
list whose default is `tests`. The files of all the paths are sorted together, and each
file is read once. A path that does not exist is an error before a test runs, with the
exit code 3.

For each file, the command runs its tests and the tests of each file it imports, so tests
written next to the code in a module run when a test file imports that module. A test
runs once, though several test files import its file.

Each test runs on the VM, in a VM of its own, in place of the program start block: the
variables at the top level of its file and of the files it imports are set first, as
for a run, and then the test runs. A test therefore never sees what another test changed.
A stop fails that test alone, and the tests after it still run.

With `--jit`, each test runs as native code made with Cranelift, as `zakac run --jit` runs
a program. A stop of native code ends its process, so `zakac test` starts itself again
for each test and runs the test in that process of its own; the report is the same.

`zakac test` takes the flags of the compile stages, such as `--debug-asserts off`,
`--no-unsafe`, `--overflow` and the lint flags, and the rights flags of `zakac run`
(`--allow-read`, `--allow-write`, `--allow-env`, `--allow-ffi`, `-A` and
`--no-permission-prompt`). [Commands](../compiler/commands.md#test) lists them.

```console
$ zakac test --debug-asserts off --allow-read tests
```

## Endless tests

A test that never ends would hold up the whole run, so each test has a limit:

| Flag | Value | Default | What it does |
| --- | --- | --- | --- |
| `--max-steps` | `<OPS>` | `100000000` | the most operations one test runs on the VM; a test that runs more stops and fails |
| `--time-limit` | `<SECONDS>` | `60` | the most seconds one test runs with `--jit`; a test that runs longer is killed and fails |

`--max-steps` counts the operations of the VM, and native code counts none, so it does
not hold under `--jit`, where `--time-limit` holds instead; `--time-limit` in turn does not
hold on the VM. A test past its limit fails, and the tests after it still run. Here
`loop.zaka` holds the test `ဆုံးမဲ့`, whose loop never ends, and the test `ပြီး`:

```console
$ zakac test --max-steps 1000 loop.zaka
test loop.zaka::ဆုံးမဲ့ ... FAILED
  stopped after 1000 ops (in ဆုံးမဲ့, at loop.zaka:3:24)
test loop.zaka::ပြီး ... ok

test result: FAILED. 1 passed, 1 failed
$ zakac test --jit --time-limit 2 loop.zaka
test loop.zaka::ဆုံးမဲ့ ... FAILED
  the test ran past 2 seconds and was killed
test loop.zaka::ပြီး ... ok

test result: FAILED. 1 passed, 1 failed
```

## Picking tests

```console
$ zakac test --filter သုည
test tests/math_test.zaka::သုည ... ok

test result: ok. 1 passed, 0 failed
```

`--filter <WORD>` runs the tests whose full name, `<file>::<test>`, holds the word, and
leaves the others out. Since the name holds the file, `--filter src/` keeps the tests of
the files under `src`.

## Config

The `[test]` section of [`zakac.toml`](../compiler/configuration.md) sets the flags of
`zakac test`, so that `zakac test` with no `PATH` reads the folders of the project:

```toml
[test]
paths = ["tests", "src"]
filter = "math"
jit = true
max-steps = 100000000
time-limit = 60
```

A `PATH` on the command line wins over `paths`, and a flag on the command line wins over
its setting.

## The output

`zakac test` writes one line for each test, then a blank line and a line for the whole
run:

```text
test src/math.zaka::နှစ်ဆ ... ok
test tests/math_test.zaka::သုည ... ok
test tests/math_test.zaka::ဆယ် ... ok

test result: ok. 3 passed, 0 failed
```

`<file>` is the path of a file as the command walked it, with `/` on each host, for a file
that the command line or a folder names, and the name of the file as diagnostics give it
for an imported file. A test that passes shows its line alone, and what it printed is not
shown.

A test that fails shows `FAILED`, then what it printed, then its stop, indented with two
spaces, in the long form that `zakac run` writes, with the test and the place of the stop:

```text
test add_test.zaka::ပေါင်း ... ok
test add_test.zaka::သုည ... FAILED
သုည ပေါင်းမည်
  an assert did not hold: ၇ == ၀ (in သုည, at add_test.zaka:14:1)

test result: FAILED. 1 passed, 1 failed
```

With `--jit`, the stop line is the one a native run writes, with the name of the file
first:

```text
  add_test.zaka: the program stopped: an assert did not hold: ၇ == ၀ (in သုည, at add_test.zaka:14:1)
```

A file that does not compile writes its diagnostics once, to standard error. When the
parser found its tests, each of them shows `FAILED` and counts as failed; when it found
none, such as after a parse error in the head of a test, the file itself shows one line,
`test <file> ... FAILED`, with no `::<test>`, and counts as one failure. The warnings of a
file that compiles go to standard error too.

```console
$ zakac test broken.zaka
error[ZK0203]: this statement does not end in a statement verb, found "ဟူသော"
 --> broken.zaka:1:6
  |
1 | စမ်း ဟူသော အများသုံးနိုင်သော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
  |     ^^^^
  = help: a statement ends in "ပြသမည်", "ထားမည်" or "ကို ပြန်ပေးမည်", or is a loop with "နေသရွေ့" or "မှ ... အထိ"
test broken.zaka ... FAILED

test result: FAILED. 0 passed, 1 failed
```

The exit code is 0 when no test failed, and 1 when one or more failed or a file did not
compile. A path that does not exist gives 3, as each command does for a file it cannot
read ([Exit codes](../compiler/commands.md#exit-codes)).

## Mistakes

| Mistake | Code |
| --- | --- |
| a test inside a function, a block or a branch | ZK0267 |
| a test that gives a result | ZK0267 |
| a parameter on a test | ZK0201 |
| a call of a test | ZK0303 |
| two tests of one name in one file | ZK0268 |
| a file of tests run with `zakac run`, with no program start block | ZK0327 |

```zaka-error ZK0267
စမ်း ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0267]: a test block stands at the top level of a file, takes no parameters and gives no result
```

```zaka-error ZK0267
ပြ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စမ်း ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ == ၁ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka-error ZK0268
စမ်း ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ == ၁ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စမ်း ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၂ == ၂ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0268]: a test named "စမ်း" stands in this file before
```

## How a test is built

The parser reads a test with `parse_test_decl` and marks the function it makes with
`FnDecl::test` ([The parser](../internals/parser-statements.md#test-blocks)). The check of
names and data kinds keeps tests out of the function table, as it keeps the program start
block out ([Names and data kinds](../internals/semcheck.md#the-passes-in-order)). The lowering takes
the test to run from `LowerOptions::test_at`, the place where its name starts: that test
takes the place of the program start block, and each other test lowers to a function with
no statements, so each function keeps its number ([The
IR](../internals/codegen.md#lowering)).
