---
title: Pointers and memory
description: Hold the address of a variable, then read and change the variable through it.
---

# Pointers and memory

Every variable lives somewhere in the computer's memory, at a numbered place called its
*address*. A *pointer* is a value that holds an address. With a pointer you can read or
change a variable without using its name: from inside another function, or in memory
that has no name at all.

```zaka
ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။                       // age = 18
ကိန်းရှင် ညွှန်း သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် အသက် ၏ လိပ်စာ ဖြစ်ပါစေ။    // a pointer to age
ညွှန်း ၏ တန်ဖိုး ကို ပြသပါ။                                 // read through it
ညွှန်း ၏ တန်ဖိုး ကို ၂၀ ဟု ထားပါ။                              // write through it
အသက် ကို ပြသပါ။                                            // age has changed
```

```text
၁၈
၂၀
```

The program never assigns to `အသက်` by name after the first line. It changes the
variable through the pointer `ညွှန်း` ("the pointer").

| Zaka | Reads as |
| --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | "integer address": the kind of a pointer to an integer |
| `အသက် ၏ လိပ်စာ` | "the address of age" |
| `ညွှန်း ၏ တန်ဖိုး` | "the value at the pointer" |
| `လိပ်စာ` | "address" |
| `တန်ဖိုး` | "value" |

## Pointer kinds

A pointer kind is the kind of the value it points to, followed by `လိပ်စာ`. A pointer to
a pointer adds `၏ လိပ်စာ` ("address of") for each further level:

| Kind | Holds the address of |
| --- | --- |
| `ကိန်းပြည့် လိပ်စာ` | an integer |
| `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ` | an unsigned 8-bit integer |
| `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ` | a pointer to an integer |
| `လိပ်စာ` | anything (an *opaque* address) |

The older spelling with `၏` at the first level, `ကိန်းပြည့် ၏ လိပ်စာ`, still works and
means the same as `ကိန်းပြည့် လိပ်စာ`.

A pointer holds addresses of its own kind only. The address of a `ကိန်းပြည့်` does not fit
a pointer to an 8-bit integer:

```zaka-error ZK0401
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
```

```text
error[ZK0401]: expected *i8, found *isize
```

A bare `လိပ်စာ` holds any address, but you cannot read through it until you say what is
there. The page on [pointer arithmetic](arithmetic.md) shows how.

## The empty address

A pointer that points nowhere holds `ဗလာ` ("empty"). A new pointer variable starts out
empty. A pointer is not true or false, so compare it with `ဗလာ` to test it:

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။     // no address yet
ပ ကို ပြသပါ။
အကယ်၍ ပ သည် ဗလာ နှင့် ညီလျှင် "ဘာမှ မညွှန်ပါ" ဟု ပြသပါ။
```

```text
ဗလာ
ဘာမှ မညွှန်ပါ
```

Reading or writing through an empty pointer stops the program. See [Memory
mistakes](mistakes.md).

## Short forms

Programmers who know C can write `&` for "the address of" and `*` for "the value at":

```zaka
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် &က ဖြစ်ပါစေ။     // the same as က ၏ လိပ်စာ
*ပ ကို *ပ + ၁ ဟု ထားပါ။                              // the same as ပ ၏ တန်ဖိုး
က ကို ပြသပါ။
```

```text
၆
```

Both spellings mean the same thing. They group differently, which the page on [pointer
arithmetic](arithmetic.md) explains.

## What has an address

Variables, parameters, `ရလဒ်` and anything reached through a pointer have addresses. A
number such as `၅` does not, and neither does the result of a sum. A constant has an
address in memory, but Zaka will not give it out, because writing through it would change
the constant. The same holds for the variable of a counted loop:

```zaka-error ZK0412
ကိန်းသေ ကြီးဆုံး သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။
ကြီးဆုံး ၏ လိပ်စာ ကို ပြသပါ။
```

```text
error[ZK0412]: "ကြီးဆုံး" cannot be changed, so its address cannot be taken
```

## Try it

- Declare two integers and one pointer. Point at the first, change it through the
  pointer, then point at the second and change that one.
- Take the address of `၅`. Read the message.
- Declare a pointer to a pointer, `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ`, and reach an integer
  through it with `၏ တန်ဖိုး ၏ တန်ဖိုး`.

Next: [Changing the caller's variables](out-parameters.md).
