---
title: Terms
description: Each Zaka term, with the standard term from other languages and the Burmese word used in code.
---

# Terms

The Learn and Docs pages use a small set of Zaka terms, and programmers who know another
language will know most of these ideas under other names. This page lists each Zaka term
next to the standard term and the Burmese word that we write for it in a program. On the
other pages, the standard term appears in parentheses after the Zaka term when the term
is first used, as in "data kind (type)".

## Data kinds and values

| Zaka term | Standard term | Burmese | Page |
| --- | --- | --- | --- |
| data kind | type | `အမျိုးအစား` | [Data kinds and values](../language/kinds.md) |
| integer | integer, `int` | `ကိန်းပြည့်` | [Data kinds and values](../language/kinds.md#integer-kinds) |
| decimal number | float, floating-point number | `ဒဿမကိန်း` | [Data kinds and values](../language/kinds.md#decimal-number-kinds) |
| character | `char`, Unicode scalar value | `စာလုံး` | [Data kinds and values](../language/kinds.md#characters) |
| true/false value | boolean, `bool` | `မှန်မှားတန်ဖိုး`, `ဘူလီယန်` | [Data kinds and values](../language/kinds.md#truefalse-values) |
| text | string | `စာသား` | [Text](../language/text.md) |
| octet | byte | `ဘိုက်` | [Unsafe Zaka](../language/unsafe.md) |
| variable | mutable variable, `let mut` | `ကိန်းရှင်` | [Declarations and assignment](../language/declarations.md#declarations) |
| constant | constant, `const` | `ကိန်းသေ` | [Declarations and assignment](../language/declarations.md#declarations) |
| initial value | default value, zero value | `ကနဦးတန်ဖိုး` | [Data kinds and values](../language/kinds.md#initial-values) |
| conversion | cast, type conversion | `အဖြစ် ပြောင်းလဲသော ရလဒ်` | [Conversions](../language/conversions.md) |
| kind alias | type alias | | [Kind aliases](../language/aliases.md) |
| name | identifier | | [Source text and words](../language/words.md) |
| reserved word | keyword | | [Reserved words](reserved-words.md) |
| widening | implicit conversion | | [Conversions](../language/conversions.md) |
| size word | no counterpart; the unit in `n * sizeof(T)` | `ခုစာ`, `လုံးစာ` | [Data kinds and values](../language/kinds.md) |
| template slot | string interpolation, placeholder | `"{name}"` | [Text](../language/text.md) |

## Data

| Zaka term | Standard term | Burmese | Page |
| --- | --- | --- | --- |
| list | dynamic array, vector | `စာရင်း` | [Lists](../language/lists.md) |
| counted list | fixed-size array | `အကွက် n ခုရှိသော … စာရင်း` | [Counted lists and storage](../language/counted-lists.md) |
| cell | element | `အကွက်` | [Lists](../language/lists.md) |
| cell number | index | `အကွက် i` | [Lists](../language/lists.md) |
| range | slice, substring | | [Lists](../language/lists.md#ranges) |
| each-cell loop | for-each loop | `အကွက်တိုင်း` | [Blocks, branches and loops](../language/control-flow.md) |
| static data | static data, constant data | | [Counted lists and storage](../language/counted-lists.md#static-data) |
| table | map, dictionary, hash table | `ဇယား` | [Tables](../language/tables.md) |
| index (of a table) | key | `အညွှန်း` | [Tables](../language/tables.md) |
| row (of a table) | entry, key-value pair | | [Tables](../language/tables.md) |
| struct | struct, record (a value type) | `ဒေတာအစုအဝေး` | [Structs](../language/structs.md) |
| field | field, member | | [Structs](../language/structs.md) |
| shared object | reference type, object of a class | `မျှဝေနိုင်သော ဒေတာအစုအဝေး` | [Values and objects](../language/objects.md) |
| enumeration | enum, tagged union | `ဒေတာအမျိုးအစားကွဲ` | [Enumerations](../language/enums.md) |
| member (of an enumeration) | variant, enumerator | | [Enumerations](../language/enums.md) |
| data (of a member) | payload, associated value | `ဒေတာအနေဖြင့်` | [Enumerations](../language/enums.md#members-that-hold-data) |
| switch, arm | `switch` and `case`, `match` and its arms | `… ဖြစ်လျှင်` | [Enumerations](../language/enums.md) |
| default arm | default case | | [Enumerations](../language/enums.md) |
| data cell | the box of an option or a result | `ဒေတာဆဲလ်` | [Optional values](../language/optionals.md) |
| optional kind | option type, nullable type | `ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်` | [Optional values](../language/optionals.md) |
| blank | none, null, nil | `အလွတ်` | [Optional values](../language/optionals.md) |
| result kind | result type | `… သို့မဟုတ် အမှားအယွင်းအနေဖြင့် … ရှိနိုင်သော ဒေတာဆဲလ်` | [Result kinds](../language/result-kinds.md) |
| good result | `Ok` value, success value | `ရလဒ်ကောင်း` | [Errors and results](../language/error-handling.md) |
| error value | error, `Err` value | `အမှားအယွင်း` | [Errors and results](../language/error-handling.md) |
| whole result | a `Result` value | | [Reading a result](../language/reading-results.md) |
| fallback | `unwrap_or`, default on failure | | [Optional values](../language/optionals.md#a-fallback) |
| duplicate | copy; deep copy for a duplicate of all | `ကူးထည့်`, `အကုန်ကူးထည့်` | [Values and objects](../language/objects.md) |
| measure word | classifier (grammar); no counterpart in other languages | `ခု`, `လုံး`, … | [Data kinds and values](../language/kinds.md#measure-words) |

## Sentences and functions

| Zaka term | Standard term | Burmese | Page |
| --- | --- | --- | --- |
| sentence | statement | ends with `။` | [Statements](../language/statements.md) |
| particle | postposition; the role marker after a word | `ကို`, `သည်`, `၏`, … | [Source text and words](../language/words.md) |
| preposition | argument label, named-argument marker | `ဝိဘတ်` | [Calls written as sentences](../language/functions-sentences.md#prepositions) |
| program start block | entry point, `main` | `ပရိုဂရမ်သည် … ပရိုဂရမ် ပြီးဆုံးပါပြီး။` | [The program start block](../language/functions-start-block.md) |
| block closer | end of a block, closing brace | `လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။` | [Blocks, branches and loops](../language/control-flow.md#blocks) |
| function signature | function signature, function header | | [Functions](../language/functions.md) |
| function value | first-class function, function pointer, closure | `… ဟူသော လုပ်ဆောင်ချက်` | [Function values](../language/function-values.md) |
| function kind | function type | `လုပ်ဆောင်ချက် အမျိုးအစား` | [Function values](../language/function-values.md) |
| nested function | local function | | [Closures](../language/closures.md) |
| capture | captured variable | | [Closures](../language/closures.md) |
| one name, several functions | overloading | | [One name, several functions](../language/functions-overloads.md) |
| starting phrase, ending phrase | no counterpart; fixed words of a call written as a sentence | `အစ စကားစု`, `အဆုံး စကားစု` | [Calls written as sentences](../language/functions-sentences.md) |
| mutable parameter | `inout`, mutable reference parameter | `ဒေတာ ပြောင်းလဲရန်` | [Function parameters](../language/functions-parameters.md#mutable-parameters) |
| writable parameter | `out` parameter | `ဒေတာ ထည့်ပေးရန်` | [Function parameters](../language/functions-parameters.md#writable-parameters) |
| passing a failure on | error propagation, `?` in Rust | `ရလဒ်ကောင်း` | [Errors and results](../language/error-handling.md#passing-a-failure-on) |
| comptime function | compile-time function, `constexpr`, `comptime` | `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` | [Compile time](../language/compile-time.md) |
| C function | foreign function, FFI | `"<lib>" မှလာသော` | [External and C functions](../language/functions-c.md) |
| other values | variadic arguments | `အခြားတန်ဖိုးများနှင့်အတူ` | [External and C functions](../language/functions-c.md) |
| function C can call | exported function | | [Functions C can call](../language/functions-exports.md) |
| callback | callback | | [External and C functions](../language/functions-c.md) |
| handle, data kind from outside | opaque pointer, opaque type | | [External and C functions](../language/functions-c.md) |

## Programs in the large

| Zaka term | Standard term | Burmese | Page |
| --- | --- | --- | --- |
| module | module, one source file | | [Modules](../language/modules.md) |
| core module | prelude, built-in functions | | [The core module](../language/core.md) |
| stdlib | standard library | `"စံကား/<name>"` | [The stdlib](../language/stdlib.md) |
| package | package, crate | `"<dep>/<module>"` | [Packages](../language/packages.md) |
| dep | dependency | | [Packages](../language/packages.md#deps) |
| repo | repository | | [Packages](../language/packages.md#git-deps-and-the-lock-file) |
| lib | library | | [Native programs](../compiler/native.md) |
| generic | generic, type parameter | | [Generics](../language/generics.md) |
| kind parameter | type parameter | | [Generics](../language/generics.md) |
| instance | monomorphized instance | | [Generics](../language/generics.md) |
| trait | trait, interface, type class | | [Traits](../language/traits.md) |
| trait value | interface value, trait object | `<trait> အရာ` | [Trait values](../language/trait-values.md) |
| struct based on a struct | struct embedding, embedded struct | `<struct> ကို အခြေခံသော` | [Structs based on structs](../language/based-structs.md) |
| part | embedded field | | [Structs based on structs](../language/based-structs.md) |
| trait block | `impl` block | | [Traits](../language/traits.md) |
| default function | default method | | [Traits](../language/traits.md) |
| written-as hint | no counterpart; a second spelling for a data kind or a trait | | [Generics](../language/generics.md#written-as-hints) |
| conversion trait | `From` and `TryFrom`, `Into` and `TryInto` | `ပြောင်းလဲနိုင်သော`, `စမ်းသပ်ပြောင်းလဲနိုင်သော` | [Conversions](../language/conversions.md#conversions-through-traits) |
| share | re-export | | [Modules](../language/modules.md) |
| thread | thread, task | `အပြိုင်လုပ်ငန်း` | [Threads](../language/threads.md) |
| channel | channel, message queue | | [Threads](../language/threads.md) |
| wait on several channels | select, `select` statement | `ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်` | [Threads](../language/threads.md#waiting-on-several-channels) |
| OS thread | operating-system thread | | [Threads](../language/threads.md#threads-and-os-threads) |
| lock block | mutex lock, critical section | | [Threads](../language/threads.md) |
| atomic number | atomic integer | | [Threads](../language/threads.md) |
| parallel loop, parallel map | parallel `for`, parallel map | | [Threads](../language/threads.md) |
| naturalness check | lint | | [Natural wording](../language/naturalness.md) |

## Storage and checks

| Zaka term | Standard term | Burmese | Page |
| --- | --- | --- | --- |
| storage, RAM | memory | `မှတ်ဉာဏ်`, `မင်မိုရီ` | [Storage, explained](../concepts/storage.md) |
| the heap | heap, dynamic memory | | [Storage, explained](../concepts/storage.md) |
| block | allocation, memory block | | [Addresses and blocks](../vm/blocks.md) |
| address, pointer | pointer | `လိပ်စာ` | [Pointers and addresses](../language/pointers.md) |
| null address | null pointer, `NULL` | `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` | [Pointers and addresses](../language/pointers.md#the-null-address) |
| owned pointer | owning pointer, unique pointer | `အပိုင်အနေဖြင့်` | [The heap and owned pointers](../language/heap.md#owned-pointers) |
| pointer kind | pointer type | `<kind> လိပ်စာ` | [Pointers and addresses](../language/pointers.md) |
| opaque address | `void *` | `လိပ်စာ` | [Pointers and addresses](../language/pointers.md) |
| ring collector | cycle collector, garbage collector for reference cycles | | [Values and objects](../language/objects.md) |
| unsafe code | unsafe code | `စိတ်မချရသော` | [Unsafe Zaka](../language/unsafe.md) |
| the program stops | trap, panic, runtime error | | [When a program stops](../vm/traps.md) |
| assert | assertion, `assert!`, `assert` | `<condition> ဖြစ်ရမည်` | [Asserts](../language/asserts.md) |
| cleanup | `defer` | `လုပ်ဆောင်စရာများပြီးသည့် အခါတွင်` | [Cleanup](../language/cleanup.md) |
| cleanup on failure | `errdefer` | `မအောင်မြင်၍ လုပ်ဆောင်စရာများပြီးသည့် အခါတွင်` | [Cleanup](../language/cleanup.md#cleanup-on-failure) |
| debug assert | `debug_assert!`, an assertion that a build can leave out | `ပရိုဂရမ်ရေးသားချိန်တွင် <condition> ဖြစ်ရမည်` | [Asserts](../language/asserts.md#debug-asserts) |
| test block | unit test, test function, `#[test]` | `<name> ဟူသော စမ်းသပ်ချက် သည် …` | [Tests](../language/tests.md) |
| flow checker | borrow checker, ownership and definite-assignment analysis | | [The flow checker](../internals/borrow-checker.md) |
| names and data kinds | name resolution and type checking (semantic analysis) | | [Names and data kinds](../internals/semcheck.md) |
| the VM | virtual machine, bytecode interpreter | | [The VM](../vm/README.md) |
| operation | instruction, bytecode operation | | [Instructions](../vm/instructions.md) |
| host function | native function, system call of the VM | | [Running a program](../vm/running.md) |
| artifact | compiled file of a stage, bytecode file | | [Stages](../compiler/stages.md) |
| listing | disassembly | | [Instructions](../vm/instructions.md) |
| the IR | intermediate representation | | [The IR](../internals/codegen.md) |
| tree | AST, abstract syntax tree | | [The parser](../internals/parser.md) |
| right | permission | `--allow-read`, `--allow-ffi`, … | [Running a program](../vm/running.md) |
