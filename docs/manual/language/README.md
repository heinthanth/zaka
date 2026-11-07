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
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
| [Functions](functions.md) | declaring, calling and returning, edit references, and how values are passed |
| [Function values](function-values.md) | function kinds, the address of a function, and calls through a function value |
| [Errors and results](error-handling.md) | a result or an error value, passing a failure on, reading a whole result, and the start block |
| [Lists](lists.md) | plain lists, their literals and cells, adding and removing cells, then counted lists and static data |
| [Text](text.md) | owned text, template literals, joining, cells and characters, then views, copies and drops |
| [Structs](structs.md) | declaring a struct, its classifier, literals and fields, then duplicating, moving and storage |
| [Enumerations](enums.md) | declaring an enumeration, its members and their values, and the switch |
| [Modules](modules.md) | a program in several files: the export mark, the import sentence and names across files |
| [Natural wording](naturalness.md) | which particle and which ending read well, and the warnings for the others |
| [Slices](slices.md) | borrowed runs of cells with their count |
| [Pointers](pointers.md) | addresses, the heap and pointer arithmetic |
| [Ownership and borrowing](ownership.md) | the rules that keep a pointer from outliving its block |

The pages climb from values to control to storage. The first pages, from words to
natural wording, state the rules of values: numbers, text, plain lists such as `isize[]`,
structs, functions and loops, with no address, loan or heap block in sight. The later
sections of those pages, and the page on slices, turn to control: views of text and of
lists, static data, counted lists such as `isize[3]`, and what a duplicate costs, which `--warn-duplicates` reports. The last two pages state the rules of
storage: owner, borrow and raw pointers, allocating, resizing and freeing a block, and
the ownership check. Each page orders its sections alike: the plain value rules
first, then counted lists, static data and views, and addresses, loans and the heap last.

Where a rule has a gentler introduction, the page links to the [Learn
book](../../learn/README.md). The error codes (`ZK0101` and the rest) are listed in
[Error messages](../reference/errors.md), and the words of the language in [Reserved
words](../reference/reserved-words.md).
