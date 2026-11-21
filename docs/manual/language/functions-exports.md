---
title: Functions C can call
description: Functions that a Zaka lib exports to C under a C name, the data kinds these functions can pass, structs with C names, and how the lib reports traps and handles threads.
---

# Functions C can call

A Zaka program can export a function to C, so that C code calls it from a lib that
`zakac build --emit lib` builds. This page covers how we mark such a function, the data
kinds (types) that it can pass, the C names of structs, and how the lib reports a trap and
serves several threads. The Learn page [A lib for C](../../learn/functions/c-lib.md) teaches
the same ideas with an example, and the other direction, a program that calls C, is on
[External and C functions](functions-c.md).

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည် "<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍
<parameters> လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။
<statements>
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

To let code outside the program call a function, we write a text and
`အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍` right after `သည်`, then the
parameters, the result and the statements, as for a function with statements. The program
calls it as it calls other functions, and `zakac build --emit lib` exports it to C under
the C name (see [Commands](../compiler/commands.md#build)).

```zaka
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် "add_numbers" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍
က ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၂ နှင့် ၄၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၂
```

## Where and how it is declared

| Rule | Code |
| --- | --- |
| the text before `အမည်ဖြင့်` is required | ZK0201 |
| the C name is a C name: ASCII letters, digits and `_`, not starting with a digit | ZK0337 |
| the C name is not `main` and does not start with `zk_` or `ZK_`, which the runtime keeps, and no C function that the program calls and no other exported function has it | ZK0491 |
| the function is at the top level, has statements, has no kind parameters, and has no `အပိုင်အနေဖြင့်` on a parameter or on its result | ZK0490 |
| the parameters and the result have data kinds that C can pass, as below | ZK0465 |
| no parameter is marked `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်` | ZK0466 |

```zaka-error ZK0491
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် "zk_add" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Data kinds C can pass

The data kinds are those that C passes to a function value that it calls back
([Callbacks](functions-c.md#callbacks)):

- A parameter is a number, a character, a true/false value, an address, text (a
  `const char *`, where NULL stops the call), a handle of a data kind from outside, an
  optional of text or of a handle (where NULL is the optional with no data), a plain struct
  of such fields, passed whole, or a value in its C form.
- The result is a number, a character, a true/false value, an address, a plain struct, or a
  value in its C form, which the lib writes through the out pointer.
- Each other data kind is ZK0465.

A value in its C form:

| Data kind | C form |
| --- | --- |
| text | a `char *` |
| plain list | `zk_list_<cells>`, the C struct `{ <cell> *ptr; size_t count; }` with the C form of each cell, named after its cells (`zk_list_int32_t`, `zk_list_text`, `zk_list_person`, `zk_list_list_text`) |
| counted list | its cells in a row |
| struct | the C forms of its fields |

The cells of a plain list are numbers, text, structs of such fields, or plain lists of
these. A ဇယား (table, map) and a counted list among the cells of a plain list have no C form
(ZK0465). A parameter of such a data kind, a plain list among them, comes from C, and the
lib copies what it points at. The lib allocates a result, and C frees it: one `char *` or
`ptr` with `zk_free`, or one together with each block that its cells point at, at each
depth, with `zk_free_all`. For a NULL out pointer the lib makes no C form.

```zaka-error ZK0465
ရေတွက် ဟူသော လုပ်ဆောင်ချက် သည် "counts" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍
က ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား တစ်ခု ကို ပြန်ပေးမည်။
[:] ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Structs and their C names

```text
<name> ဟူသော ဒေတာအစုအဝေးသည် "<C name>" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် <fields> တို့ပါဝင်သည်။
```

Each struct that a parameter or the result passes needs a C name, and so does each struct
among its fields (ZK0492). To give a struct a C name, we write
`"<C name>" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော ဒေတာအစုအဝေး ဖြစ်၍` after `ဒေတာအစုအဝေးသည်`
([Marks after the head](structs.md#marks-after-the-head)), and the C header file of the lib
declares the struct under that name. The C name follows the rules of the C name of a
function (ZK0337, ZK0491), and no two structs or functions share one.

```zaka
point ဟူသော ဒေတာအစုအဝေးသည် "point" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် x ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

အကွာအဝေး ဟူသော လုပ်ဆောင်ချက် သည် "manhattan" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍
က ဟူသော point တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
(က ၏ x) + (က ၏ y) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ထောင့် သည် x အနေဖြင့် ၃ နှင့် y အနေဖြင့် ၄ ရှိသော point တစ်ခု ဖြစ်ပါစေ။
အကွာအဝေး ဟူသော လုပ်ဆောင်ချက်ကို ထောင့် ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၇
```

The C header file of the lib declares the struct with its fields in order, each one under its
Zaka name when that is a C name, and as `f<n>` otherwise:

```c
typedef struct
{
    int32_t x;
    int32_t y;
} point;

int32_t manhattan(point zk_p0, int32_t *zk_out);
```

A function that returns such a struct, `... point တစ်ခု ကို ပြန်ပေးမည်။`, writes it whole
through its out pointer: `int32_t mirror(point zk_p0, point *zk_out);`.

```zaka-error ZK0492
point ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

အကွာအဝေး ဟူသော လုပ်ဆောင်ချက် သည် "manhattan" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍
က ဟူသော point တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
(က ၏ x) + (က ၏ y) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## Calls from C

In the lib, each such function returns an `int32_t` status and gives its result through a
last parameter, an address that it writes unless the address is NULL, so the function
`ပေါင်း` above is `int32_t add_numbers(int32_t, int32_t, int32_t *)` in C. The status is
one of these:

| Status | Meaning |
| --- | --- |
| `ZK_OK` (0) | the call ended well |
| `1 +` the number of a trap | the call stopped on that trap, and `zk_last_trap_from_<lib>()` returns it as a `zk_trap` struct |
| `ZK_STOPPED` (-1) | the lib is stopped |

A trap in a function that reaches a mutable global, or in one of the functions that run one
call at a time (below), stops the lib: each later call returns `ZK_STOPPED`, since the trap
can leave the heap and the globals half written. A trap in other functions fails that call
alone, and the lib goes on. `zk_reset_<lib>()` starts the lib again: the next call sets the
globals again, and the blocks that the stopped lib held remain out of reach. The first call
starts the runtime and sets the globals, and the text that a call prints goes out before it
returns.

Threads can call the lib. A function that reaches no mutable global, calls no C and no
function value, and starts no thread, itself or through the functions it calls, runs on
several threads at once, with no lock. Other functions run one call at a time, and a call
from another thread waits until the running call ends or waits in C, which lets the other
threads in. C that a call calls can call the lib again on the same thread, and C can call a
function of the lib from each thread (see
[Closures and C](closures.md#closures-and-c)).
