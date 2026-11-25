---
title: Cleanup
description: Code that runs when its block ends - the mark လုပ်ဆောင်စရာများပြီးသည့် အခါတွင်, where it stands, the exits that run it, its order and values, cleanup on failure with မအောင်မြင်၍ - and the errors ZK0272, ZK0365 and ZK0366.
---

# Cleanup

A cleanup (defer) is a statement or a block that runs when the block around it ends, at
each exit of that block, in place of where it stands. A cleanup on failure (errdefer) runs
at the exits that give back a failure of the function alone. The Learn book teaches both in
[Cleanup](../../learn/error-handling/cleanup.md).

```text
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် <statement>
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ … လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
မအောင်မြင်၍ လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် <statement or block>
```

- The mark is the phrase of three words `လုပ်ဆောင်စရာများ`, `ပြီးသည့်` and `အခါတွင်`, and the
  failure mark adds `မအောင်မြင်၍` in front. Each works with spaces between its words or
  with none, and takes the second spellings of its words: `အခါ၌`, and `မအောင်မြင်ပြီး` or
  `မအောင်မြင်ကာ`. In spoken Burmese, `ပြီးတဲ့` and `အခါမှာ` stand for `ပြီးသည့်` and
  `အခါတွင်` ([Spoken Burmese](spoken.md)).
- After the mark comes one statement, or a block. A single statement stands in a block of
  its own, so a declaration in it belongs to the cleanup.

## Where it stands

A cleanup stands in each block that runs: the statements of a function, a closure or a
comptime function, the program start block, a test block, the arms of an `if` and of a
switch, the body of a loop, a lock block, an unsafe block, the arms of a wait on several
channels, and a block of `အောက်ပါတို့ကို လုပ်ဆောင်မည်`. It belongs to the innermost of
these blocks.

- In REPL mode, the top-level statements form the program start block, so a cleanup there
  runs at the end of the program. `zakac repl` refuses one at the top of a session, which
  runs its earlier chunks again for each new one and has no end of its own.
- At the top of an imported file, a cleanup is a statement outside each block (ZK0323).
- As the one statement of a branch written without a block, such as `အကယ်၍ … လျှင်
  <statement>` or a loop on one line, a cleanup would end its branch as it ran: ZK0272.

## When it runs

A cleanup runs at each exit of its block that comes after its mark:

- the end of the block;
- a return from the function, and a pass-on of a failure (`f(a)?`, `ရလဒ်ကောင်း`, and the
  `မအောင်မြင်ပါက` form), which leave each block of the function;
- a `break` or a `continue` that leaves the block, so a cleanup in the body of a loop runs
  at the end of each round.

An exit above the mark leaves the cleanup out, since the program never reached it. A
cleanup in an inner block runs when that block ends, before the cleanups of the blocks
around it.

## Order and values

- Several cleanups of one block run in reverse order of their marks, the last one first.
- A cleanup reads its variables when it runs, at the exit: a change after the mark shows.
  Its names resolve where the mark stands, so it uses the names declared above it.
- At an exit, the cleanups of each block that the exit leaves run first, then each
  parameter marked `ဒေတာ ပြောင်းလဲရန်` goes back to the caller, then the block drops its text
  and gives back its locks. A cleanup can still change such a parameter and use the object
  of a lock block around it.

## Cleanup on failure

A cleanup after `မအောင်မြင်၍ လုပ်ဆောင်စရာများပြီးသည့် အခါတွင်` runs at an exit of its block
that gives back a failure of the function, and at no other:

- a pass-on of a failure;
- a return of an error value, `… ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်`, and `လုပ်ဆောင်ချက်
  မအောင်မြင်ပါဟု သတ်မှတ်မည်`;
- a return of a whole result that holds a failure, which the program tests when it leaves.

A good return, the end of a block, a `break` and a `continue` leave it out. The two kinds of
cleanup share one order, the last mark first. A cleanup on failure in a function that
cannot fail never runs: ZK0366, and so in the program start block, which gives back an exit
code.

## Code that cannot leave

A cleanup runs to its end. A return, a pass-on of a failure, and a `break` or `continue` of
a loop around the cleanup, inside it, are ZK0365. A loop inside the cleanup breaks and
continues as each loop does, and a function declared inside it returns as each function
does. A failure inside a cleanup is handled there.

## Storage and the checks

The compiler checks the code of a cleanup as the code at each exit, so a free of a heap
block in a cleanup gives the block back on each path, and the leak check (ZK0708) holds for
each return in the middle of the block. A use of the block after the mark and before the
exit remains legal.

## When the program stops

A stop (trap) ends the program, or under `--trap unwind` the thread, with no cleanup: the
code after the stop never runs. A host call that ends the process runs none either.

## The compiler

The lowering (`zakac_ir_builder`) keeps the code of each cleanup with its block, beside the
drops of its places, and writes it at each exit of the block, so the VM, the C target and
Cranelift see plain statements. [Parser: statements](../internals/parser-statements.md)
and [Codegen](../internals/codegen.md) describe the steps.

## Mistakes

```zaka-error ZK0272
အကယ်၍ မှန် လျှင် လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "ပိတ်" ဟု ပြသမည်။
```

```zaka-error ZK0366
မအောင်မြင်၍ လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "ပိတ်" ဟု ပြသမည်။
```

| Code | When |
| --- | --- |
| ZK0272 | a cleanup as the one statement of a branch written without a block |
| ZK0365 | a return, a pass-on of a failure, or a `break` or `continue` of a loop around it, inside a cleanup |
| ZK0366 | a cleanup on failure in a function that cannot fail |
