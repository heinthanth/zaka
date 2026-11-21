---
title: A lib for C
description: Mark the functions that C can call, build them into a C lib, and call them from C or Rust.
---

# A lib for C

In [Calling C](c.md), Zaka called a function of a C lib (library), and this page turns that
around. We mark the functions of a program that code outside can call, and `zakac` builds
them into a C lib, which C, Rust and each other language that can call C can link.

## You will learn

- how to mark a function that C can call, under a name in C
- how to build a lib and the C header file that declares its functions
- how to call the lib from C and from Rust, and how a call reports a stop

```zaka
// "Function add, under the name add_numbers, that code outside can call,
//  accepting two 32-bit integers a and b, will return their sum."
ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် "add_numbers" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍
က ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၂ နှင့် ၄၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
```

```text
၄၂
```

In the function signature, we write `"<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍`
after `သည်` to give the function a name in C. This is where a C function has
`"<lib>" မှလာသော`, but here the function keeps its statements, and inside the program it
is a plain function, so the program above runs as usual.

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည် "<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍
<parameters> လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။
```

| Zaka | Reads as |
| --- | --- |
| `"add_numbers"` | the name in C: ASCII letters, digits and `_` |
| `အမည်ဖြင့်` | "under the name" |
| `ပြင်ပမှ ခေါ်ဆိုနိုင်သော` | "that code outside can call" |

The name in C cannot be `main`, and it cannot start with `zk_` or `ZK_`, which the runtime
keeps for itself. Two functions cannot share a name in C.

## Building the lib

```text
zakac build --backend c --emit lib <file>.zaka
```

We use `--emit lib` to build a lib, with the C backend or the Cranelift one
(`--backend native`). A lib needs no program start block, because its functions are where
a caller enters. Here the function above is alone in `mathlib.zaka`:

```console
$ zakac build --backend c --emit lib mathlib.zaka
```

The build writes three files next to the source: the static lib `libmathlib.a`, the shared
lib (`libmathlib.so` on Linux, `libmathlib.dylib` on macOS, `mathlib.dll` on Windows), and
the C header file `mathlib.h`. With `-o out/mathlib`, the files go into `out` instead.

## The C header file

`mathlib.h` declares each function under its name in C:

```c
#define ZK_OK 0
#define ZK_STOPPED (-1)
#define ZK_TRAP_ZERO_DIVISOR 0
/* ... an id for each stop ... */

typedef struct zk_trap
{
    int32_t code;            /* ZK_TRAP_ZERO_DIVISOR, ... */
    const char *kind;        /* "zero_divisor", ... */
    const char *message;     /* the line the stop prints */
    const char *lib;         /* "mathlib" */
    zk_trap_frame where;     /* the function, file, line and column it stopped in */
    const zk_trap_frame *frames;
    size_t frame_count;
} zk_trap;

const zk_trap *zk_last_trap_from_mathlib(void);
void zk_reset_mathlib(void);
const zk_trap *zk_last_trap(void);
void zk_free(void *p);
void zk_free_all(void *p);

int32_t add_numbers(int32_t zk_p0, int32_t zk_p1, int32_t *zk_out);
```

Each function returns a status, not its result, and the result goes to the address in the
last parameter, which can be NULL when the caller does not need the result. The status is
`ZK_OK` when the call went through. When the call stopped, as a program stops on a division
with zero as divisor, the status is 1 more than the number of the stop, and the calling
program goes on. `zk_last_trap_from_mathlib()` returns the stop as a struct with its
number, its name, its message, and the function, file and line where it stopped.

A function that changes no global, such as `add_numbers`, fails that one call, and the lib
goes on. A stop in a function that reaches a mutable global, or that calls C, stops the
whole lib, because the stop can leave the lib half done: each later call returns
`ZK_STOPPED` until `zk_reset_mathlib()` starts the lib again from the start.

The first call starts the lib, which sets the globals of the program. Text that a function
accepts comes in as a `const char *`, and what a function prints comes out at the end of
each call.

## From C

```c
#include <stdio.h>

#include "mathlib.h"

int main(void)
{
    int32_t sum = 0;
    if (add_numbers(2, 40, &sum) == ZK_OK)
    {
        printf("%d\n", sum);
    }
    return 0;
}
```

```console
$ cc main.c libmathlib.a -liconv -lm -o main
$ ./main
42
```

The comment at the top of the C header file lists the libs that a program must link
together with the static lib, such as `-liconv` on macOS.

## From Rust

Rust declares the same functions in an `extern` block and links the static lib:

```rust
unsafe extern "C" {
    fn add_numbers(a: i32, b: i32, out: *mut i32) -> i32;
}

fn main() {
    let mut sum = 0;
    let status = unsafe { add_numbers(2, 40, &mut sum) };
    println!("{status} {sum}");
}
```

```console
$ mkdir static && cp libmathlib.a static/
$ rustc main.rs -L static -l static=mathlib -l iconv
```

We put the static lib in a separate folder, because when `libmathlib.dylib` is next to it,
the linker of macOS picks the shared lib for `-l mathlib`. In a Cargo project, a build
script does the same with `cargo:rustc-link-search=static` and
`cargo:rustc-link-lib=static=mathlib`.

## Details

- A function that C can call accepts and returns integers, decimal numbers, true and false,
  characters, addresses, text, handles, optionals of text or handles, plain lists of these
  or of structs, and structs of all these, with the result through its out pointer
  (ZK0465). Text that it returns is a `char *`, and a list is a `zk_list_<cells>` with
  `ptr` and `count`, such as `zk_list_text`. C frees one with `zk_free`, or one together
  with each text and list inside it with `zk_free_all`.
- A struct that it accepts or returns needs a C name, which the C header file declares it
  under (ZK0492):
  `point ဟူသော ဒေတာအစုအဝေးသည် "point" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် ...`.
- It must be at the top level, it cannot have kind parameters, and it cannot accept or
  return a block as its own (ZK0490).
- Several threads can call a lib. A function that reaches no mutable global and calls no
  C, itself or through the functions it calls, runs on several threads at once. Calls to
  the other functions run one at a time, and the others wait until the call ends or waits
  in a C function. C can call a function of the program from each thread, built with
  `--allow-ffi` for the C functions that it calls.
- Several Zaka libs can link into one program, from one build of `zakac` or from several.
  Each lib keeps its own names and its own `--max-heap`, and a stop affects just its own
  lib, while `zk_free` and `zk_last_trap()`, the last stop of each lib, serve them all.
  The name in `zk_reset_<name>` is the name of the files of the lib, or the value of
  `--lib-name`, which accepts ASCII letters, digits and `_`.
- On Windows, a lib built with GNU and a lib built with MSVC each have their own C runtime,
  so a program that loads both frees what each returned with `zk_free_from_<name>()` of
  that lib.
- The full rules are in [Functions C can
  call](../../manual/language/functions-exports.md), and the flags in
  [Commands](../../manual/compiler/commands.md).

## Recap

- `"<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော` after `သည်` lets code outside call a
  function under that name in C.
- `zakac build --backend c --emit lib` builds a static lib, a shared lib and a C header file.
- Each function returns a status and writes its result through its last parameter.
- A stop returns a status that is not `ZK_OK`, and `zk_last_trap_from_<name>()` describes
  it.

## Hands on

- Add a function `"twice"` that returns twice a 64-bit integer, build the lib, and call it
  from C.
- Call a function of the lib that divides with zero as divisor, then print
  `zk_last_trap_from_mathlib()->message` and the line in `where`.

Next: [The heap](../pointers/heap.md).
