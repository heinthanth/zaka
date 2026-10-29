---
title: The case for sentences
description: The reason for Burmese sentence order in Zaka, how it maps onto a grammar, and what it costs.
---

# The case for sentences

Most programming languages borrow their order from English: a verb first, then its
objects, as in `print(age)`. Burmese puts the verb last. Zaka takes the Burmese order
as the shape of its grammar, so a statement reads as a Burmese sentence and a Burmese
reader can read it aloud.

This page tells how that order works, how it maps onto a grammar a compiler can read,
and what it costs.

## Subject, object, verb

A Burmese sentence names the things it talks about first, and ends with the verb. A small
word, a *particle*, follows each thing and tells its role: `သည်` marks the
subject, `ကို` the object, `နှင့်` "and" or "with", `ဖြင့်` "using", `၏` "of".

```text
 အသက်  ကို  ပြသပါ ။
 age   OBJ  show    .
  │     │     │
  │     │     └── the verb ends the sentence and tells what is done
  │     └── a particle marks the role of what came before it
  └── the thing the sentence is about
```

The same pattern covers a statement that commands, `အသက် ကို ပြသပါ`, and a verb that
makes a value, `၃ နှင့် ၄ ၏ ပေါင်းလဒ်` ("the sum of 3 and 4"). A verb that makes a value
gives an expression. A verb that commands gives a statement, which ends with `။`.

## Compared with C

| Zaka | C-like code | What the Zaka sentence tells |
| --- | --- | --- |
| `အသက် ကို ပြသပါ။` | `print(age);` | age, as the object, show |
| `ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။` | `int age = 20;` | the variable age is an integer, and its value is 20 |
| `အသက် ကို အသက် + ၁ ဟု ထားပါ။` | `age = age + 1;` | age, as the object, set to "age plus 1" |
| `၃ နှင့် ၄ ၏ ပေါင်းလဒ်` | `3 + 4` | the sum of 3 and 4 |
| `၁၀ ကို ၃ ဖြင့် စားသော စားကြွင်း` | `10 % 3` | 10, divided with 3, the remainder |
| `အသက် သည် ၁၈ အောက် ငယ်သည်` | `age < 18` | age is smaller than 18 |
| `အကယ်၍ အသက် သည် ၁၈ အောက် ငယ်လျှင် ...` | `if (age < 18) ...` | if age is smaller than 18, then |
| `လုပ်ဆောင်ချက် နှစ်ဆ ကို က တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်` | `twice(k)` | the result of doing the function with k |
| `ပ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။` | `*p = 7;` | the value of p, as the object, set to 7 |
| `မင်မိုရီ ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။` | `p = malloc(2 * sizeof(int));` | taking room worth 2 integers, will keep the address in p |
| `ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။` | `free(p);` | the block at the address in p, will give back |

Where C-like code uses punctuation, a Zaka sentence uses a word that tells what the
punctuation would mean. The signs are still there for arithmetic: `၃ + ၄` and
`၃ နှင့် ၄ ၏ ပေါင်းလဒ်` mean the same, and the two forms can be mixed in one line.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၀ ဖြစ်ပါစေ။
အသက် ကို အသက် + ၁ ဟု ထားပါ။
အသက် ကို ပြသပါ။
အသက် သည် ၁၈ အောက် ငယ်သည် ဟု ပြသပါ။
၂ + ၃ * ၄ ကို ပြသပါ။
၂ နှင့် ၃ နှင့် ၄ ၏ မြှောက်လဒ် ၏ ပေါင်းလဒ် ကို ပြသပါ။
```

```text
၂၁
မမှန်
၁၄
၁၄
```

## How a sentence maps onto a grammar

A compiler has to read a sentence without knowing Burmese. It needs two things: a
closed list of particles and verbs, and a rule for combining them. Zaka has both.

A clause is a run of operands, each with the particle its role needs after it, closed with a
verb:

```text
 clause    :=  ( operand  particle )*  verb
 operand   :=  a notation expression: 2 + 3 * 4, a name, a number, (...)
```

The verb tells which particles it needs and the count of operands. The compiler reads the
sentence from left to right, and keeps the operands waiting on a stack. A verb takes the
operands it needs from the top, checks their particles, and puts its result back. The
result carries the particle that follows the verb, so clauses nest:

```text
 ၂ နှင့် ၃ နှင့် ၄ ၏ မြှောက်လဒ် ၏ ပေါင်းလဒ်

  read ၂ [နှင့်]                   waiting: ၂[နှင့်]
  read ၃ [နှင့်]                   waiting: ၂[နှင့်] ၃[နှင့်]
  read ၄ [၏]                      waiting: ၂[နှင့်] ၃[နှင့်] ၄[၏]
  verb မြှောက်လဒ်: takes ၃ and ၄   waiting: ၂[နှင့်] (၃*၄)[၏]
  verb ပေါင်းလဒ်: takes both       result: ၂ + (၃*၄) = ၁၄
```

Moving one verb changes the meaning, because the verb takes what is waiting at that
moment:

```text
 ၂ နှင့် ၃ ၏ ပေါင်းလဒ် နှင့် ၄ ၏ မြှောက်လဒ်

  ၂[နှင့်] ၃[၏]   ပေါင်းလဒ်: (၂+၃)[နှင့်]
  ၄[၏]            မြှောက်လဒ်: (၂+၃) * ၄ = ၂၀
```

```zaka
၂ နှင့် ၃ ၏ ပေါင်းလဒ် နှင့် ၄ ၏ မြှောက်လဒ် ကို ပြသပါ။
```

```text
၂၀
```

Three levels share this one mechanism. The *notation* level is the signs (`+ - * /`,
comparisons) with the precedence of C; it makes the operands. The *clause* level is the
particle-and-verb pattern above. The *logical* level joins clauses with `ဖြစ်၍` (and),
`သို့မဟုတ်` (or) and a postfix `မဟုတ်` (not). The full rules are in
[Expressions](../language/expressions.md) and [Statements](../language/statements.md).

## What the pattern gives the compiler

**The particles check the sentence.** A verb names the particles it wants, so a wrong
particle is found where it is written, with a message that names it:

```zaka-error ZK0206
၃ ကို ၄ ၏ ပေါင်းလဒ် ကို ပြသပါ။
```

```text
error[ZK0206]: "ပေါင်းလဒ်" needs "နှင့်" after operand 1, found ကို
```

An operand with no verb to take it is found at the end of the sentence:

```zaka-error ZK0209
၃ နှင့် ၄ ကို ပြသပါ။
```

```text
error[ZK0209]: operands left over before "ပြသပါ": ၃ (နှင့်)
```

**The verb is the last word.** The compiler knows what a sentence does when it reaches
the verb, and `။` ends the statement. No guess is needed about where one statement stops.

**Expressions and statements are one machine.** `ပေါင်းလဒ်` makes a value and `ပြသပါ`
commands, but both take operands through the same stack. A statement is a clause whose
verb is a command.

**No word list.** The grammar is bounded: a fixed list of reserved words, a fixed list
of particles, and the rest are names. No word list of Burmese is needed. [Finding words
in Burmese](segmenting.md) tells how that works when the text has no spaces.

## What it costs

- **Longer lines.** `၃ နှင့် ၄ ၏ ပေါင်းလဒ်` is longer than `3 + 4`. The signs are kept for
  arithmetic, so a long sum need not be nested in words, and a line can mix both forms.
- **Spaces matter more.** Burmese is written without spaces between words, and the
  compiler has no word list, so a name must be separated from the particle after it:
  `အသက် ကို`, never `အသက်ကို`. The next page is about this.
- **A fixed set of shapes.** Zaka reads the sentence shapes of its grammar, not
  Burmese in general. A sentence that a speaker finds fine can still be a compile error
  if its words or order are not in the table.
- **Endings are a matter of taste.** A verb accepts `သည်`, `ပါ`, `ပါသည်`, or none, but a
  speaker prefers one from what follows. The compiler warns about the odd ones (ZK05xx)
  and never fails a program for them; see [Natural wording](../language/naturalness.md).
- **Input and showing.** Burmese needs an input method, and its signs stack and
  reorder. The compiler fixes the order of the marks before it reads the text, so that
  one word has one spelling in the table. [Source text and words](../language/words.md)
  has the rule.
- **Two languages in one place.** The program is Burmese, while the compiler's messages
  and this manual are English.

The bet is that a program a person can read aloud is a program that person can follow, and
that the order of the language that person speaks is a good guide to the order of the
program. [Learn](../../learn/sentences/README.md) teaches the sentences one after another.
