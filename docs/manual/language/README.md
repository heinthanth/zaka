---
title: The language
description: What a Zaka program is, and a map of the chapter that states the rules of the language.
---

# The language

A Zaka program is a list of sentences (statements), and each sentence ends with the
Burmese full stop `။`. The VM runs the sentences from the first to the last. A sentence
uses Burmese word order: first the things it talks about, each with a small word after it
that shows its role, called a particle (postposition), and last the verb, which decides
what the sentence means.

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

This is how the last sentence of the program is built:

```text
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသမည် ။
statement
├── clause (makes a value)
│   ├── operand က, then the particle နှင့်   (first of two)
│   ├── operand ခ, then the particle ၏       (after the last operand)
│   └── verb ပေါင်းလဒ်                       (sum)
├── particle ကို                             (the object: the value to print)
├── verb ပြသမည်                               (print: makes the statement)
└── ။                                        (ends the sentence)
```

A verb that makes a value, such as `ပေါင်းလဒ်`, forms an expression, and a verb that gives
a command, such as `ပြသမည်`, forms a statement. The compiler reads both with one set of
rules.

## The pages of this chapter

The pages go from the smallest parts of a program to the largest, and then to unsafe code.
A newcomer can read them in this order, and each page links to the Learn page that teaches
the same feature with gentle examples.

### The basics

| Page | What it covers |
| --- | --- |
| [Source text and words](words.md) | the encoding, `။` and `၊`, spaces, comments, names, number, character and string literals, and reserved words |
| [Data kinds and values](kinds.md) | the integer, decimal number, character, true/false and text kinds with their ranges and sizes, measure words, initial values (`ကနဦးတန်ဖိုး`), and the short names that messages use |
| [Conversions](conversions.md) | the conversion expression (cast), which pairs convert, what happens to the value, and the widening that needs no words |
| [Expressions](expressions.md) | literals, names and the operators: arithmetic, comparison, logic and bitwise, with precedence, division, overflow and grouping |
| [Statements](statements.md) | each form of statement at a glance, the verb that ends a sentence, printing, calls and returns, and the heap statements |
| [Declarations and assignment](declarations.md) | variables and constants, the rule that each value is set before it is read, globals, assignment and duplicates |
| [Blocks, branches and loops](control-flow.md) | blocks, the if statement, while, counted, each-cell and generic loops, break and continue |
| [Names and scope](scope.md) | what opens a scope, shadowing, constants and counters, and the errors ZK0301 to ZK0308 |

### Functions

| Page | What it covers |
| --- | --- |
| [Functions](functions.md) | what a function is, how to declare one, and a map of the function pages |
| [Function parameters](functions-parameters.md) | parameters, their markers, mutable (`inout`) and writable (`out`) parameters, and how arguments are passed |
| [Results and returns](functions-results.md) | the result of a function, the return statement, and the rule that each path returns |
| [Function calls](functions-calls.md) | the forms of a call, how arguments are checked, calls on a value with `၏`, and recursion |
| [Calls written as sentences](functions-sentences.md) | prepositions after the arguments, starting and ending phrases, and the cost of this grammar |
| [One name, several functions](functions-overloads.md) | overloaded functions and how a call picks one |
| [External and C functions](functions-c.md) | functions that the VM serves or a C lib provides, and the data kinds that cross to C |
| [Functions C can call](functions-exports.md) | functions that a Zaka lib exports to C |
| [The program start block](functions-start-block.md) | the block where each program starts (entry point, `main`), its arguments, environment and exit code |
| [Function values](function-values.md) | function kinds, a function as a value, calls through a function value, and comparing and printing function values |
| [Closures](closures.md) | nested functions, the variables that these capture, closures as values, and closures given to C |

### Errors and missing values

| Page | What it covers |
| --- | --- |
| [Errors and results](error-handling.md) | a function that returns a good result or an error value, passing a failure on, and the order of work |
| [Reading a result](reading-results.md) | keeping a whole result, testing which part it contains, and reading each part |
| [Result kinds](result-kinds.md) | how to write a result kind, sides with no value, and lists and pointers of results |
| [Optional values](optionals.md) | the optional kind (option type), the blank `အလွတ်`, and the reason Zaka has no NULL |

### Data

| Page | What it covers |
| --- | --- |
| [Lists](lists.md) | list kinds (dynamic arrays), literals, cells, adding and removing cells, and ranges |
| [Counted lists and storage](counted-lists.md) | counted lists (fixed-size arrays), static data, and how a list is stored |
| [Tables](tables.md) | the table kind (map): rows, indexes (keys), loops over rows, and how a table is stored |
| [Text](text.md) | text (strings), template literals, joining, cells and characters, and changing text |
| [How text is stored](text-storage.md) | the three words of text, heap growth, duplicates and static text |
| [Structs](structs.md) | the struct declaration, its measure word, literals, fields, and how a struct is stored |
| [Values and objects](objects.md) | values that are duplicated, shared objects (reference types), deep duplicates, and reference counting |
| [Enumerations](enums.md) | enumerations (enums), their members and values, and the switch statement |
| [Kind aliases](aliases.md) | a second name for a data kind (type alias) |

### Programs in the large

| Page | What it covers |
| --- | --- |
| [Generics](generics.md) | functions, structs and aliases with kind parameters (type parameters), and instances |
| [Traits](traits.md) | traits (interfaces), trait blocks, default functions and the built-in traits |
| [Threads](threads.md) | running a function on a thread, waiting for its result, channels, locks, atomic numbers and parallel loops |
| [Compile time](compile-time.md) | the compile-time conditional on the target, and comptime functions |
| [The core module](core.md) | the module that each file sees with no import (prelude) |
| [Modules](modules.md) | a program in several files: exports, imports, and names across files |
| [Input and output](io.md) | reading lines and files, writing files and streams |
| [The stdlib](stdlib.md) | the modules of the stdlib (standard library) that are written in Zaka |

### Wording

| Page | What it covers |
| --- | --- |
| [Natural wording](naturalness.md) | which particle and which ending fit Burmese best, and the warnings for the others |
| [Spoken Burmese](spoken.md) | the spoken spellings of the reserved words and how to turn them on |

### Unsafe code

| Page | What it covers |
| --- | --- |
| [Unsafe Zaka](unsafe.md) | unsafe functions and blocks, safe wrappers, and the checks on unsafe code |
| [Pointers and addresses](pointers.md) | the pointer kind, the null address, reading and writing through a pointer, and pointer arithmetic |
| [The heap and owned pointers](heap.md) | the heap statements and owned pointers |

The pages before Unsafe Zaka state the rules of safe Zaka. In safe code each value is
duplicated, each shared object is shared, and the VM frees each block when its last use
goes, so none of those pages talks about addresses. The last three pages state the rules of
unsafe code: pointers, the heap, owned pointers and the checks that the compiler runs on
that code.

The error codes (`ZK0101` and the rest) are listed in [Error
messages](../reference/errors.md), and the words of the language in [Reserved
words](../reference/reserved-words.md).
