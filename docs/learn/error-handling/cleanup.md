---
title: Cleanup
description: Run code when its block ends with လုပ်ဆောင်စရာများပြီးသည့် အခါတွင်, at each exit of the block, or at a failure alone with မအောင်မြင်၍.
---

# Cleanup

A block that takes something, such as a block of storage or a handle from C, gives it back
at each place where the block ends: after its last statement, at each return, at each
`break` and at each failure that it passes on. Instead of writing the same lines at each
exit, we write a cleanup (defer) once, next to the line that takes the thing, and the
program runs it when the block ends, however it ends.

## You will learn

- how to mark a statement or a block to run when its block ends
- which exits run a cleanup, and in which order several cleanups run
- when a cleanup reads its variables
- how to run a cleanup at a failure alone
- where a cleanup cannot stand

```zaka
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "ပိတ်ပြီ" ဟု ပြသမည်။ // runs when the program ends
"ဖွင့်ပြီ" ဟု ပြသမည်။
"အလုပ်လုပ်နေသည်" ဟု ပြသမည်။
```

```text
ဖွင့်ပြီ
အလုပ်လုပ်နေသည်
ပိတ်ပြီ
```

A cleanup is a statement with the mark in front of it:

```
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် <statement>
```

| Zaka | Reads as |
| --- | --- |
| `လုပ်ဆောင်စရာများပြီးသည့်` | "that the tasks are done" |
| `အခါတွင်` | "at the time" |
| `"ပိတ်ပြီ" ဟု ပြသမည်` | "print `ပိတ်ပြီ`" |

The mark echoes the block closer, `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`: the statement after it
runs when the tasks of its block are done. Here the block is the whole program, so the
line prints last. The words of the mark work with or without spaces between them.

## A block after the mark

For several statements, we put a block after the mark, and the whole block runs at the end:

```zaka
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
"ပထမ" ဟု ပြသမည်။
"ဒုတိယ" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
"အလုပ်" ဟု ပြသမည်။
```

```text
အလုပ်
ပထမ
ဒုတိယ
```

## Each exit

A cleanup runs at each exit of its block. In a function, a return in the middle runs it,
and so does the end of the function:

```zaka
စစ် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "စစ်ပြီး" ဟု ပြသမည်။
အကယ်၍ က သည် ၀ နှင့် ညီလျှင် ပြန်ပေးမည်။ // the cleanup runs here
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။ // and here

စစ် ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် လုပ်ဆောင်မည်။
စစ် ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်မည်။
```

```text
စစ်ပြီး
၅
စစ်ပြီး
```

The body of a loop is a block too, so a cleanup there runs at the end of each round, a
round that `continue` or `break` ends among them:

```zaka
၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "အကြိမ် {န} ပြီး" ဟု ပြသမည်။
အကယ်၍ န သည် ၂ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။
"အကြိမ် {န}" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
အကြိမ် ၁
အကြိမ် ၁ ပြီး
အကြိမ် ၂ ပြီး
အကြိမ် ၃
အကြိမ် ၃ ပြီး
```

A cleanup runs once its line is reached: an exit above the line leaves it out.

## The last one first

Several cleanups of one block run in reverse order, so the thing that was taken last goes
back first:

```zaka
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "ပထမ ပိတ်" ဟု ပြသမည်။
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "ဒုတိယ ပိတ်" ဟု ပြသမည်။
"အလုပ်" ဟု ပြသမည်။
```

```text
အလုပ်
ဒုတိယ ပိတ်
ပထမ ပိတ်
```

## Values at the exit

A cleanup reads its variables when it runs, at the exit, not where the mark stands:

```zaka
ကိန်းရှင် အရေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "အရေ {အရေ}" ဟု ပြသမည်။
အရေ ကို ၃ ဟု ထားမည်။
```

```text
အရေ ၃
```

## Giving things back

The things that a cleanup gives back most often are a block of storage that a pointer holds
and a handle that a C function gave us. We give one back right after we take it, and each
exit of the block gives it back. The pointer check of the compiler counts a free in a
cleanup at each exit, so a return in the middle of the block leaks nothing
([Cleanup](../../manual/language/cleanup.md#storage-and-the-checks)).

## Cleanup on failure

In a function that can fail, `မအောင်မြင်၍` ("as it failed") in front of the mark makes a
cleanup that runs at an exit that gives back a failure, and at no other. It undoes the
work of a setup that cannot finish:

```zaka
စားခွဲ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ ခ သည် ၀ နှင့် ညီလျှင် "သုညဖြင့် စား၍ မရပါ" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
က / ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ထပ်စား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ
ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "ပြီး" ဟု ပြသမည်။
မအောင်မြင်၍ လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် "မအောင်မြင်ပါ" ဟု ပြသမည်။
စားခွဲ(က၊ ခ)? ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ထပ်စား(၁၀၀၊ ၅) ကို ပြသမည်။
ထပ်စား(၁၀၀၊ ၀) ကို ပြသမည်။
```

```text
ပြီး
ရလဒ်ကောင်း(၂၀)
မအောင်မြင်ပါ
ပြီး
အမှားအယွင်း("သုညဖြင့် စား၍ မရပါ")
```

```
မအောင်မြင်၍ လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် <statement>
```

An exit gives back a failure when it passes one on, as `?` does here, when it returns an
error value, and when it returns a whole result that holds one. The two kinds of cleanup
share one order, the last one first.

## Details

- **No exit out of a cleanup.** A cleanup runs to its end, so a return, a pass-on of a
  failure, or a `break` or `continue` of a loop around it is the error ZK0365. A loop inside
  the cleanup can break as each loop can.
- **Alone in a branch.** A cleanup as the one statement of an `if` arm or of a loop on one
  line would run at once, which is the error ZK0272.
- **A function that cannot fail.** A cleanup on failure in a function that cannot fail, the
  program start block among them, never runs, which is the error ZK0366.
- **A stop.** When the program stops (trap), no cleanup runs.
- **The REPL.** `zakac repl` refuses a cleanup at the top of a session, which has no end of
  its own; we write it inside a function or a block there.
- **Spoken Burmese.** `ပြီးတဲ့` and `အခါမှာ` stand for `ပြီးသည့်` and `အခါတွင်`
  ([Spoken Burmese](../../manual/language/spoken.md)).
- **The full rules.** [Cleanup](../../manual/language/cleanup.md) has the full rules, with
  each error.

```zaka-error ZK0365
စစ် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

## Recap

- `လုပ်ဆောင်စရာများပြီးသည့် အခါတွင်` in front of a statement or a block runs it when the
  block around it ends, at each exit of that block.
- Several cleanups run the last one first, and each reads its variables at the exit.
- `မအောင်မြင်၍` in front of the mark runs the cleanup at a failure alone.
- A cleanup cannot leave its own code, and it does not run when the program stops.

## Hands on

- Add a third cleanup to the example of the last one first, and predict the order.
- Move the cleanup of `စစ်` below its `အကယ်၍` line, run it with `၀` again, and see which
  calls print `စစ်ပြီး`.
- Make `ထပ်စား` return an error value of its own with `ဟူသော အမှားအယွင်း`, and check that the
  cleanup on failure runs.

Next: [Optional values](../optionals/README.md).
