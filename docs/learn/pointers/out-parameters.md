---
title: Changing the caller's variables
description: Pass addresses to a function so it can change the caller's variables.
---

# Changing the caller's variables

A [parameter](../functions/parameters.md) is a duplicate, so a function cannot change
the variable passed in. Pass the variable's *address* instead. The function then writes
through the pointer, and the change reaches the caller's variable.

```zaka
// swap: exchange the values of two integers
လုပ်ဆောင်ချက် လဲလှယ် သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ က နှင့် ငှားထားသော ကိန်းပြည့် လိပ်စာ ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုး ဖြစ်ပါစေ။     // keep the first value
က ၏ တန်ဖိုး ကို ခ ၏ တန်ဖိုး ဟု ထားပါ။                  // first = second
ခ ၏ တန်ဖိုး ကို ယာယီ ဟု ထားပါ။                         // second = the kept value
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပထမ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ဒုတိယ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် လဲလှယ် ကို ပထမ ၏ လိပ်စာ၊ ဒုတိယ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပထမ ကို ပြသပါ။
ဒုတိယ ကို ပြသပါ။
```

```text
၂
၁
```

| Zaka | Reads as |
| --- | --- |
| `ငှားထားသော ကိန်းပြည့် လိပ်စာ က` | a parameter `က` that borrows an integer: the caller keeps the variable |
| `ပထမ ၏ လိပ်စာ` | "the address of the first", passed as an argument |
| `ယာယီ` | "short-lived" |

## Several results

A function returns at most one result. Pointers let it hand back more: the caller
passes the addresses of the variables that should receive them.

```zaka
// divide: write the quotient and the remainder into the caller's variables
လုပ်ဆောင်ချက် ခွဲဝေ သည်
ကိန်းပြည့် တည်ကိန်း၊ ကိန်းပြည့် စားကိန်း၊ ငှားထားသော ကိန်းပြည့် လိပ်စာ အဖြေ နှင့် ငှားထားသော ကိန်းပြည့် လိပ်စာ အကြွင်း တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အဖြေ ၏ တန်ဖိုး ကို တည်ကိန်း / စားကိန်း ဟု ထားပါ။
အကြွင်း ၏ တန်ဖိုး ကို တည်ကိန်း % စားကိန်း ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ရ သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ကျန် သည် ကိန်းပြည့် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ခွဲဝေ ကို ၁၇၊ ၅၊ ရ ၏ လိပ်စာ၊ ကျန် ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ရ ကို ပြသပါ။
ကျန် ကို ပြသပါ။
```

```text
၃
၂
```

5 goes into 17 three times, with 2 left over.

## How long an address lasts

The variables of a function end with the call: when it returns, their RAM is given back.
So a borrow of one of them must not leave the function, and the compiler refuses a
result that would take it out:

```zaka-error ZK0706
လုပ်ဆောင်ချက် မှား သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ငှားထားသော ကိန်းပြည့် လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် x သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
x ၏ လိပ်စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0706]: returns a borrow of the local "x", which ends when the function returns
```

Pointing *into* a function at the caller's variables, as `လဲလှယ်` does, is fine in all
cases: the caller's variables outlive the call. RAM that must outlive the function that
creates it is a block from the [heap](heap.md), and the function hands its caller the
owner of that block; see [owners and borrows](ownership.md#a-new-block-from-a-function).

## Hands on

- Write a function that takes the addresses of three integers and sorts their values
  from smallest to largest, using `လဲလှယ်`.
- Call `လဲလှယ်` with the same address twice: `ပထမ ၏ လိပ်စာ၊ ပထမ ၏ လိပ်စာ`. What
  happens, and what is the reason?

Next: [The heap](heap.md).
