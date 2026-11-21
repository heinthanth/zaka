---
title: The case for sentences
description: The reason Zaka uses the order of a Burmese sentence, how that order maps onto a grammar, and what it costs.
---

# The case for sentences

Most programming languages take their word order from English, with the verb first and its
objects after it, as in `print(age)`. Burmese puts the verb at the end. Zaka uses the
Burmese order as the shape of its grammar, so each statement is a Burmese sentence, and a
Burmese speaker can read a program aloud.

This page explains how that order works, how a compiler can parse it, and what it costs.
The rules are in [Expressions](../language/expressions.md) and
[Statements](../language/statements.md), and Learn teaches the sentences in [Zaka
sentences](../../learn/sentences/README.md).

## Subject, object, verb

A Burmese sentence names the things it is about first, and ends with the verb. After each
of these things comes a small word, a *particle* (postposition), which shows its role:
`သည်` for the subject, `ကို` for the object, `နှင့်` for "and" or "with", `ဖြင့်` for
"using" and `၏` for "of".

```text
 အသက်  ကို  ပြသမည် ။
 age   OBJ  show    .
  │     │     │
  │     │     └── the verb comes last and names the action
  │     └── a particle shows the role of the word before it
  └── the thing the sentence is about
```

One pattern covers both a statement that gives a command, such as `အသက် ကို ပြသမည်`, and a
verb that makes a value, such as `၃ နှင့် ၄ ၏ ပေါင်းလဒ်` ("the sum of 3 and 4"). A verb that
makes a value gives us an expression, and a verb that gives a command gives us a statement,
which ends with `။`.

## Compared with C

| Zaka | C-like code | The Zaka sentence, word for word |
| --- | --- | --- |
| `အသက် ကို ပြသမည်။` | `print(age);` | age, as the object, show |
| `ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။` | `int age = 20;` | the variable age is an integer, and its value is 20 |
| `အသက် ကို အသက် + ၁ ဟု ထားမည်။` | `age = age + 1;` | age, as the object, set to "age plus 1" |
| `၃ နှင့် ၄ ၏ ပေါင်းလဒ်` | `3 + 4` | the sum of 3 and 4 |
| `၁၀ ကို ၃ ဖြင့် စားသော စားကြွင်း` | `10 % 3` | 10, divided with 3, the remainder |
| `အသက် သည် ၁၈ အောက် ငယ်သည်` | `age < 18` | age is smaller than 18 |
| `အကယ်၍ အသက် သည် ၁၈ အောက် ငယ်လျှင် ...` | `if (age < 18) ...` | if age is smaller than 18, then |
| `နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို က ဖြင့် လုပ်ဆောင်သော ရလဒ်` | `twice(k)` | the result of doing the function with k |
| `ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။` | `*p = 7;` | the value of p, as the object, set to 7 |
| `မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။` | `p = malloc(2 * sizeof(int));` | taking room for 2 integers, keep the address in p |
| `လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။` | `free(p);` | the block at the address in p, release |

Where C-like code uses punctuation, a Zaka sentence uses a word with the same meaning.
The signs are still there for arithmetic, so `၃ + ၄` and `၃ နှင့် ၄ ၏ ပေါင်းလဒ်` mean the same
thing, and we can mix the two forms in one line.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက် ကို အသက် + ၁ ဟု ထားမည်။
အသက် ကို ပြသမည်။
အသက် သည် ၁၈ အောက် ငယ်သည် ဟု ပြသမည်။
၂ + ၃ * ၄ ကို ပြသမည်။
၂ နှင့် ၃ နှင့် ၄ ၏ မြှောက်လဒ် ၏ ပေါင်းလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၁
မမှန်
၁၄
၁၄
```

## How a sentence maps onto a grammar

A compiler has to parse a sentence without knowing Burmese, so it needs two things: a
closed list of particles and verbs, and a rule for combining them. Zaka has both.

A clause is a run of operands, each followed with the particle that its role needs, and
a verb at the end:

```text
 clause    :=  ( operand  particle )*  verb
 operand   :=  a notation expression: 2 + 3 * 4, a name, a number, (...)
```

Each verb knows which particles it needs and the number of its operands. The compiler reads the
sentence from left to right and keeps the operands waiting on a stack. When it reaches a
verb, the verb pops the operands it needs from the top of the stack, checks their
particles, and pushes its result back. The result gets the particle that follows the
verb, so clauses can nest:

```text
 ၂ နှင့် ၃ နှင့် ၄ ၏ မြှောက်လဒ် ၏ ပေါင်းလဒ်

  read ၂ [နှင့်]                   waiting: ၂[နှင့်]
  read ၃ [နှင့်]                   waiting: ၂[နှင့်] ၃[နှင့်]
  read ၄ [ကို]                     waiting: ၂[နှင့်] ၃[နှင့်] ၄[ကို]
  verb မြှောက်: pops ၃ and ၄       waiting: ၂[နှင့်] (၃*၄)[ကို]
  verb ပေါင်း: pops both           result: ၂ + (၃*၄) = ၁၄
```

Moving one verb changes the meaning, because a verb uses whatever is waiting at that
moment:

```text
 ၂ နှင့် ၃ ၏ ပေါင်းလဒ် နှင့် ၄ ၏ မြှောက်လဒ်

  ၂[နှင့်] ၃[ကို]   ပေါင်း: (၂+၃)[နှင့်]
  ၄[ကို]            မြှောက်: (၂+၃) * ၄ = ၂၀
```

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၂ နှင့် ၃ ၏ ပေါင်းလဒ် နှင့် ၄ ၏ မြှောက်လဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၀
```

Three levels share this one mechanism. The *notation* level is the signs (`+ - * /` and
the comparisons) with the precedence of C, and it builds the operands. The *clause* level
is the pattern of particles and verbs above. The *logical* level joins clauses with
`ဖြစ်၍` (and), `သို့မဟုတ်` (or) and `မဟုတ်` (not) after a clause.

## What the pattern gives the compiler

**The particles check the sentence.** Each verb lists the particles it wants, so the
compiler finds a wrong particle where we wrote it, and the message names it:

```zaka-error ZK0206
၃ ကို ၄ ၏ ပေါင်းလဒ် ကို ပြသမည်။
```

```text
error[ZK0206]: "ပေါင်းလဒ်" needs "နှင့်" after operand 1, found ကို
```

When no verb uses an operand, the compiler finds it at the end of the sentence:

```zaka-error ZK0204
၃ နှင့် ၄ ကို ပြသမည်။
```

```text
error[ZK0204]: statement verb "ပြသမည်" cannot stand inside an expression
```

**The verb is the last word.** The compiler knows what a sentence does when it reaches the
verb, and `။` ends the statement, so it never has to guess where one statement stops.

**Expressions and statements work the same.** `ပေါင်းလဒ်` makes a value and `ပြသမည်` gives
a command, but both get their operands from the same stack, so a statement is a clause
whose verb is a command.

**No word list.** The grammar is closed: a fixed list of reserved words, a fixed list of
particles, and names for the rest. The compiler needs no word list of Burmese,
and [Finding words in Burmese](segmenting.md) explains how that works when the text has
no spaces.

## What it costs

- **Longer lines.** `၃ နှင့် ၄ ၏ ပေါင်းလဒ်` is longer than `3 + 4`. Zaka keeps the signs for
  arithmetic, so a long sum does not need to be nested in words, and a line can mix both
  forms.
- **Spaces matter more.** Burmese is written without spaces between words, and the
  compiler has no word list, so we must separate a name from the particle after it with a
  space: `အသက် ကို`, never `အသက်ကို`. The next page explains this.
- **A fixed set of shapes.** Zaka accepts the sentence shapes of its grammar, not Burmese
  in general, so a sentence that sounds fine to a speaker can still be a compile error
  when its words or its order are not in the grammar.
- **Endings are a matter of taste.** A verb accepts `သည်`, `ပါ`, `ပါသည်` or no ending at
  all, but a speaker prefers one of them from what follows. The compiler warns about the
  odd ones (ZK05xx) and never fails a program for them (see [Natural
  wording](../language/naturalness.md)).
- **Input and rendering.** Burmese needs an input method, and its marks stack and reorder.
  The compiler puts the marks in a fixed order before it reads the text, so that each word
  has one spelling in the list of reserved words, and [Source text and
  words](../language/words.md) has the rule.
- **Two languages in one place.** The program is in Burmese, while the messages of the
  compiler and these pages are in English.

The bet is that a program a person can read aloud is a program that person can follow,
and that the word order of the language a person speaks is a good guide to the order of a
program.
