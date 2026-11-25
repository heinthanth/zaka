---
title: The stdlib
description: The stdlib modules for the core, text, paths, lists, I/O, math and time.
---

# The stdlib

The stdlib (standard library) is a set of modules that the compiler carries, and a
program imports the names it needs from each one, as it does from its own files
([Modules](../language/modules.md)). The Learn book teaches it in [The
stdlib](../../learn/stdlib/README.md). The stdlib is a package: `stdlib/zakac.toml` gives
each module its name, and each program has it as the dep `စံကား`
([Packages](../language/packages.md#the-stdlib-as-a-package)).

| Module | File | What it has |
| --- | --- | --- |
| [`"စံကား/အခြေခံအုတ်မြစ်"`](core.md) | `stdlib/core/` | the core module: built-in data kinds, traits and verbs, seen with no import |
| [`"စံကား/စာသား"`](text.md) | `stdlib/text.zaka` | finding, splitting, joining and changing text, and text as an integer |
| [`"စံကား/လမ်းကြောင်း"`](path.md) | `stdlib/path.zaka` | the path kind `ဖိုင်လမ်းကြောင်း` and the parts of a path |
| [`"စံကား/စာရင်း"`](list.md) | `stdlib/list.zaka` | sorting, removing, finding and reversing the cells of a list |
| [`"စံကား/အဝင်အထွက်"`](io.md) | `stdlib/io.zaka` | streams, files and folders |
| [`"စံကား/သင်္ချာ"`](math.md) | `stdlib/math.zaka` | math constants and functions on 64-bit floats |
| [`"စံကား/အချိန်"`](time.md) | `stdlib/time.zaka` | clocks and UTC formatting |

Each module page starts with what the module is for and an example, then lists each name
that the module exports, with its declaration and its doc comments, taken from the source.
The source of each module is in the stdlib folder of the compiler, and an editor opens it
through go to definition. The examples on the module pages use sentence forms, including
calls with [argument particles](../language/functions-sentences.md#prepositions) and
[conversions through traits](../language/conversions.md#conversions-through-traits). The
[sign form](../language/functions-calls.md#the-sign-form) also works.
