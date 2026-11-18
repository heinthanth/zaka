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
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၅
```

The last sentence of the program is built like this:

```text
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသမည် ။
statement
├── clause (makes a value)
│   ├── operand က, then the particle နှင့်   (first of two)
│   ├── operand ခ, then the particle ၏       (closes the operands)
│   └── verb ပေါင်းလဒ်                       (sum)
├── particle ကို                             (marks the value to show)
├── verb ပြသမည်                               (print: makes the statement)
└── ။                                        (ends the sentence)
```

A verb that makes a value, such as `ပေါင်းလဒ်`, forms an *expression*. A verb that gives
a command, such as `ပြသမည်`, forms a *statement*. The compiler reads both with one set of rules.

## The pages of this chapter

| Page | What it states |
| --- | --- |
| [Source text and words](words.md) | the encoding, `။` and `၊`, spaces, comments, names, number, character and string literals, reserved words |
| [Kinds and values](kinds.md) | the integer, decimal, character, true/false and text kinds with their ranges, sizes and initial values (`ကနဦးတန်ဖိုး`), and the short names that messages use |
| [Conversions](conversions.md) | the conversion expression, which pairs convert and what happens to the value, and the widening that needs no words |
| [Expressions](expressions.md) | literals, names and the operators: arithmetic, comparison, logic and bitwise, with precedence, division, overflow and grouping |
| [Statements](statements.md) | each form of statement with its shape, the statement verbs, the error of a missing verb, and the rule that each value is set before it is read |
| [Names and scope](scope.md) | blocks, shadowing, constants, where a name can be seen, and the errors ZK0301 to ZK0308 |
| [Functions](functions.md) | declaring, calling and returning, the parameter markers, C functions, and how values are passed |
| [Function values](function-values.md) | function kinds, naming a function as a value, calls through a value, and nested functions that capture |
| [Errors and results](error-handling.md) | a result or an error value, passing a failure on, reading a whole result, and the start block |
| [Optional values](optionals.md) | the optional kind, the blank value, tests and reads, giving one back, and the reason Zaka has no NULL |
| [Lists](lists.md) | plain lists, their literals and cells, adding and removing cells, ranges, then counted lists and static data |
| [Text](text.md) | text values, template literals, joining, cells and characters, ranges, and text from u8s |
| [Structs](structs.md) | declaring a struct, its classifier, literals and fields, copies and storage |
| [Values and objects](objects.md) | values that are copied, shared objects, an object that can be missing, kinds no program can build, equal contents and one object, shallow and deep copies, counting and the ring collector |
| [Enumerations](enums.md) | declaring an enumeration, its members and their values, and the switch |
| [Kind aliases](aliases.md) | a second name for a kind: the alias sentence and where the name can stand |
| [Modules](modules.md) | a program in several files: the export mark, the import sentence and names across files |
| [Natural wording](naturalness.md) | which particle and which ending read well, and the warnings for the others |
| [Unsafe Zaka](unsafe.md) | unsafe code, the pointer kind, the heap, owned markers and the checks on unsafe code |

The pages climb from values to storage. The pages from words to natural wording state
the rules of safe Zaka: numbers, text, lists, structs, shared objects, functions and
loops. Each value copies, each shared object is shared, and the VM frees each block when
its last use goes, so no page before the last speaks of an address. The last page,
[Unsafe Zaka](unsafe.md), states the rules of unsafe code: the pointer kind, the heap
statements, owned markers and the checks the compiler runs on that code.

Where a rule has a gentler introduction, the page links to the [Learn
book](../../learn/README.md). The error codes (`ZK0101` and the rest) are listed in
[Error messages](../reference/errors.md), and the words of the language in [Reserved
words](../reference/reserved-words.md).
