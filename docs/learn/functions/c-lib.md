---
title: A lib for C
description: Mark functions that C can call, build them into a C lib, and call them from C or Rust.
---

# A lib for C

In [Calling C](c.md), Zaka called a function of a C lib. This page turns that around. A
program marks the functions that code outside can call, and `zakac` builds them into a C
lib, which C, Rust and each language that calls C can link.

The mark stands after `သည်`, in place of the statements of a C function's `"<lib>" မှလာသော`:
the name C calls the function with, then `အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော` ("under the
name, that code outside can call"). The rest is a header with statements, as before. In a
program the function is a plain function, so the program below runs:

```zaka
// "Function add, under the name add_numbers, that code outside can call,
//  accepting two 32-bit integers a and b, will return their sum."
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

| Part | Reads as |
| --- | --- |
| `"add_numbers"` | the name in C: ASCII letters, digits and `_` |
| `အမည်ဖြင့်` | "under the name" |
| `ပြင်ပမှ ခေါ်ဆိုနိုင်သော` | "that code outside can call" |

The name in C cannot be `main`, nor start with `zk_` or `ZK_`, which the runtime keeps for
itself, and each function takes a name of its own.

## Building the lib

`--emit lib` builds the lib, with the C backend or the Cranelift one. A lib needs no start
block: its functions are where a caller enters. Here the function above stands alone in
`mathlib.zaka`:

```console
$ zakac build --backend c --emit lib mathlib.zaka
```

The build writes three files beside the source: `libmathlib.a`, the shared lib
(`libmathlib.so` on Linux, `libmathlib.dylib` on macOS, `mathlib.dll` on Windows), and the
header `mathlib.h`. `-o out/mathlib` puts them in `out` instead.

## The header

The header declares each function under its name in C:

```c
#define ZK_OK 0
#define ZK_STOPPED (-1)

const char *zk_last_trap(void);
void zk_lib_reset(void);

int32_t add_numbers(int32_t zk_p0, int32_t zk_p1, int32_t *zk_out);
```

Each function gives back a status, not its result. The result goes to the address in the
last parameter, which can be NULL when the caller has no use for it. The status is
`ZK_OK` when the call went through. When the call stopped, as a division with zero as
divisor stops a program, the status is 1 more than the number of the stop, `zk_last_trap()` gives its
text, and the program that called goes on. The lib stops with it: each later call gives
`ZK_STOPPED`, since the stop can leave the lib half done, until `zk_lib_reset()` starts it
again from the start.

The first call starts the lib: it sets the globals of the program. Text a function takes
comes in as a `const char *`, and what the function prints comes out at the end of each
call.

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

The comment at the top of the header lists the libs a program links with the static lib,
such as `-liconv` on macOS.

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

The static lib sits in a folder of its own, since beside `libmathlib.dylib` the linker of
macOS takes the shared lib for `-l mathlib`. In a Cargo project, a build script writes the
same with `cargo:rustc-link-search=static` and `cargo:rustc-link-lib=static=mathlib`.

## Limits

- A function C can call takes integers, decimal numbers, true and false, characters,
  addresses, text, handles, optionals of text or handles, and structs of numbers, text
  and lists of numbers; it gives back one of these, text, or a plain list of numbers,
  through its out pointer (ZK0465). Text it gives back is a `char *`, a list is a
  `zk_list_<cell>` (`ptr` and `count`), and C frees each with `zk_free`.
- A struct it takes or gives back needs a C name, which the header declares it under
  (ZK0492):
  `point ဟူသော ဒေတာအစုအဝေးသည် "point" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် ...`.
- It stands at the top level, takes no kind parameters, and neither takes nor gives back a
  block as its own (ZK0490).
- Threads can call a lib, but one call runs at a time; the others wait.

## Hands on

- Add a function `"twice"` that gives back twice a 64-bit integer, build the lib, and
  call it from C.
- Call a function of the lib that divides with zero as divisor, then print
  `zk_last_trap()`.

Next: [The heap](../pointers/heap.md).
