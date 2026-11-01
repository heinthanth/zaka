---
title: The language
description: What a Zaka program is, and a map of the chapter that states the rules of the language.
---

# The language

A Zaka program is a **list of sentences**, and each sentence ends with the Burmese full
stop `။`. The VM runs the sentences from the first to the last. A sentence follows
Burmese word order: first the things it talks about, each with a small word
(a *particle*) after it that marks its role, and **last the verb**, which decides what the
sentence means.

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသပါ။
```

```text
၅
```

The last sentence of the program is built like this:

```text
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသပါ ။
statement
├── clause (makes a value)
│   ├── operand က, then the particle နှင့်   (first of two)
│   ├── operand ခ, then the particle ၏       (closes the operands)
│   └── verb ပေါင်းလဒ်                       (sum)
├── particle ကို                             (marks the value to show)
├── verb ပြသပါ                               (print: makes the statement)
└── ။                                        (ends the sentence)
```

A verb that makes a value, such as `ပေါင်းလဒ်`, forms an *expression*. A verb that gives
a command, such as `ပြသပါ`, forms a *statement*. The compiler reads both with one set of rules.

## The pages of this chapter

| Page | What it states |
| --- | --- |
| [Source text and words](words.md) | the encoding, `။` and `၊`, spaces, comments, names, number, character and string literals, reserved words |
| [Kinds and values](kinds.md) | the integer, decimal, character, true/false and text kinds with their ranges, sizes and defaults, and the short names that messages use |
| [Conversions](conversions.md) | the conversion expression, which pairs convert and what happens to the value, and the widening that needs no words |
| [Expressions](expressions.md) | literals, names and the operators: arithmetic, comparison, logic and bitwise, with precedence, division, overflow and grouping |
| [Statements](statements.md) | each form of statement with its shape, the statement verbs and the error of a missing verb |
| [Names and scope](scope.md) | blocks, shadowing, constants, where a name can be seen, and the errors ZK0301 to ZK0308 |
| [Functions](functions.md) | declaring, calling and returning |
| [Pointers](pointers.md) | addresses, the heap and pointer arithmetic |
| [Ownership and borrowing](ownership.md) | the rules that keep a pointer from outliving its block |
| [Lists](lists.md) | fixed lists, their cells and kinds |
| [Slices](slices.md) | borrowed runs of cells with their count |
| [Text](text.md) | owned text and views of text, template literals, joining, cells and drops |
| [Natural wording](naturalness.md) | which particle and which ending read well, and the warnings for the others |

The pages go from the smallest unit to the largest: words, then the kinds of value, then
expressions and statements, then names. Functions, pointers, owners and lists build on
those. Where a rule has a gentler introduction, the page links to the [Learn
book](../../learn/README.md). The error codes (`ZK0101` and the rest) are listed in
[Error messages](../reference/errors.md), and the words of the language in [Reserved
words](../reference/reserved-words.md).
