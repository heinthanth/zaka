---
title: Tests
description: Check that functions do what is expected of them with test blocks, run them with zakac test, and read what a failed test shows.
---

# Tests

An assert checks a condition while the program runs. To check a function before we count
on it, we can write a test block (unit test): a block of asserts with a name, which the
command `zakac test` runs and which the program itself leaves out. A test passes when it
runs to its end, and it fails when one of its asserts does not hold.

## You will learn

- how to write a test block with `စမ်းသပ်ချက်`
- how to run the tests of a file with `zakac test`
- what the output shows when a test fails
- how to keep tests in files of their own
- how to run some of the tests and not the others

```zaka
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// a test of the function above
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

The file has a function, a test of it and a program start block. When we run the file,
the program start block runs and the test does not, so the program prints `၃၀`. We write
a test block with this pattern:

```text
<name> ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
<statements>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as |
| --- | --- |
| `ပေါင်း` | the name we choose |
| `ဟူသော` | "called" |
| `စမ်းသပ်ချက်` | "test" |
| `သည်` | (the subject particle) |
| `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | "will do the following" |
| `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | "the tasks are done" |

A test is written as a [function](../functions/README.md) with no parameters and no
result, with `စမ်းသပ်ချက်` in place of `လုပ်ဆောင်ချက်`. Nothing calls a test, so it can
have the name of the function it tests, as `ပေါင်း` does here.

## Running the tests

```console
$ zakac test add.zaka
test add.zaka::ပေါင်း ... ok

test result: ok. 1 passed, 0 failed
```

We save the file as `add.zaka` and give it to `zakac test`, which runs each test of the
file in place of the program start block. It writes one line for each test, with the file
and the name of the test, then a line that counts the tests that passed and the tests that
failed. A test that passes shows its line alone, and what it printed is not shown.

## When a test fails

A file of tests needs no program start block. Here `add_test.zaka` holds the function
and two tests, and the second test expects the wrong sum:

```zaka-file
// add_test.zaka
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပေါင်း ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း(၂၊ ၃) == ၅ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

သုည ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"သုည ပေါင်းမည်" ဟု ပြသမည်။
ပေါင်း(၇၊ ၀) == ၀ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```console
$ zakac test add_test.zaka
test add_test.zaka::ပေါင်း ... ok
test add_test.zaka::သုည ... FAILED
သုည ပေါင်းမည်
  an assert did not hold: ၇ == ၀ (in သုည, at add_test.zaka:14:1)

test result: FAILED. 1 passed, 1 failed
```

A failed test shows `FAILED`, then what it printed, then the stop of its assert, with the
test and the line of the assert. Each test runs on its own, so the first test still
passes. When one or more tests fail, `zakac test` ends with the exit code 1, and a script
or a CI job can stop there.

## Tests in their own files

A project can keep its code under `src` and its tests under `tests`. A test file imports
the functions it tests, as each [module](../modules/README.md) does:

```zaka-file
// src/math.zaka
နှစ်ဆ ဟူသော အများသုံးနိုင်သော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ဆ ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ဆ(၃) == ၆ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka-file
// tests/math_test.zaka
"../src/math.zaka" မှ နှစ်ဆ ကို ထည့်သွင်းမည်။

သုည ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ဆ(၀) == ၀ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဆယ် ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ဆ(၁၀) == ၂၀ ဖြစ်ရမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```console
$ zakac test
test src/math.zaka::နှစ်ဆ ... ok
test tests/math_test.zaka::သုည ... ok
test tests/math_test.zaka::ဆယ် ... ok

test result: ok. 3 passed, 0 failed
```

With no file named, `zakac test` reads each `.zaka` file under the folder `tests`. It runs
the tests of each file and also the tests of the files that file imports, so the test
written next to `နှစ်ဆ` in `src/math.zaka` runs as well. We can also name files and
folders: `zakac test tests src` reads both folders, and each test still runs once.

## Running some of the tests

```console
$ zakac test --filter သုည
test tests/math_test.zaka::သုည ... ok

test result: ok. 1 passed, 0 failed
```

`--filter` keeps the tests whose full name, `<file>::<test>`, holds the word we give, so
we can run one test while we work on it, or the tests of one file.

## Details

- A test stands at the top level of a file. A test inside a function or a block, or one
  that gives a result, is the mistake ZK0267, and a call of a test is ZK0303, as nothing
  can call one ([Tests](../../manual/language/tests.md#mistakes)). Two tests of one file
  need names of their own: a second test of the same name is ZK0268.
- `စမ်းသပ်မှု` is another word for `စမ်းသပ်ချက်`, with the same meaning.
- Each test runs on the VM, in a VM of its own: the variables at the top level are set
  first, then the test runs, so a test never sees what another test changed ([Running the
  tests](../../manual/language/tests.md#running-the-tests)). With `--jit`, each test runs
  as native code made with Cranelift instead, in a process of its own.
- A test that never ends stops after `--max-steps` operations on the VM, or after
  `--time-limit` seconds with `--jit`, and fails ([Endless
  tests](../../manual/language/tests.md#endless-tests)).
- A test can do all a program does: declare variables and functions, print, start
  threads, and read files when we give `zakac test` the right, as `zakac run` takes it
  with `--allow-read`.
- `zakac run` and `zakac build` leave the tests out: the program that comes out carries no
  code of a test ([Builds leave tests
  out](../../manual/language/tests.md#builds-leave-tests-out)).
- The `[test]` section of `zakac.toml` sets the folders and the filter, as
  `paths = ["tests", "src"]` ([Config](../../manual/language/tests.md#config)).
- The full rules, the flags and the output are on [Tests](../../manual/language/tests.md)
  and [Commands](../../manual/compiler/commands.md#test).

## Recap

- `<name> ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။` starts a test block, which
  stands at the top level and holds asserts.
- `zakac test <file>` runs each test of the file and of the files it imports, and with no
  file it reads the folder `tests`.
- A failed test shows what it printed and the assert that did not hold, and the run ends
  with the exit code 1.
- `--filter <word>` runs the tests whose names hold the word.
- `zakac run` and `zakac build` leave the tests out.

## Hands on

- Write a function that returns the larger of two integers, and a test with three asserts
  for it.
- Make one assert wrong on purpose, run `zakac test`, and read the line of the stop.
- Move the test into a file under `tests` that imports the function, then run
  `zakac test` with no file named.

Next: [Optional values](../optionals/README.md).
