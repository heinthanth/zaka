---
title: Results
description: Functions that work out a value and give it back.
---

# Results

The functions so far *do* something, such as printing. Many functions instead *work out*
a value and hand it back to the caller: the area of a rectangle, the larger of two
numbers. The value handed back is the function's *result*.

```zaka
// "Function square, accepting an integer k, doing the following,
//  will return an integer result."
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။      // return k × k as the result
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// Use the result like any other value
လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
၄၉
```

Two things change compared with a function that has no result:

| Where | No result | With a result |
| --- | --- | --- |
| header | `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` ("will do the following") | `အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> ရလဒ် ကို ပြန်ပေးမည်။` ("doing the following, will return a *kind* result") |
| call | `... လုပ်ဆောင်ပါ။` ("run it") | `... လုပ်ဆောင်သော ရလဒ်` ("the result of running it"), a value |

## Returning the result

There are two ways to give back the result.

**Return it.** `<value> ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။` ("give back *value* as the result") ends the
function at once.

**Set `ရလဒ်`.** A function with a result has a ready-made variable named `ရလဒ်`
("result") of the result's kind. It starts at the kind's default. Whatever it holds when
the function reaches its end is returned.

```zaka
// area: set ရလဒ် and let the function end
လုပ်ဆောင်ချက် ဧရိယာ သည်
ကိန်းပြည့် အလျား၊ ကိန်းပြည့် အနံ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို အလျား * အနံ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// larger: return as soon as the answer is known
လုပ်ဆောင်ချက် ပိုကြီး သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ က သည် ခ ထက် ကြီးလျှင် က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

လုပ်ဆောင်ချက် ဧရိယာ ကို ၃၊ ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် ပိုကြီး ကို ၁၀၊ ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
၁၂
၁၀
```

## Using a result

A call with `လုပ်ဆောင်သော ရလဒ်` is a value. Use it anywhere a value fits: print it, store
it, pass it to another function, or use it in a sum:

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

// store it
ကိန်းရှင် ကိုး သည် ကိန်းပြည့် လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိုး ကို ပြသပါ။

// in a sum: 3² + 4²
(လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်) + လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။

// pass a result into another call: (2²)²
လုပ်ဆောင်ချက် နှစ်ထပ် ကို လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
၉
၂၅
၁၆
```

Notice the parentheses in the sum. A statement that begins with `လုပ်ဆောင်ချက်` is read
as a call statement, so a sum that *starts* with a call needs the call in parentheses.
A call later in the sum, or a call on its own, needs none.

To throw a result away, call the function as a statement with `လုပ်ဆောင်ပါ။`.

## Results of any kind

A result can be any kind. This one answers yes or no:

```zaka
// divides: does n divide by d with nothing left over?
လုပ်ဆောင်ချက် ပြတ်သလား သည်
ကိန်းပြည့် န၊ ကိန်းပြည့် ဆခွဲ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
န % ဆခွဲ သည် ၀ နှင့် ညီသည် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

အကယ်၍ လုပ်ဆောင်ချက် ပြတ်သလား ကို ၁၅၊ ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် "ပြတ်သည်" ဟု ပြသပါ။
```

```text
ပြတ်သည်
```

## Mistakes

A function without a result has nothing to use as a value:

```zaka-error ZK0315
လုပ်ဆောင်ချက် ပြ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
error[ZK0315]: function "ပြ" returns nothing and cannot be used as a value
```

A function with a result must give a value back whenever it returns:

```zaka-error ZK0313
လုပ်ဆောင်ချက် တစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
```

```text
error[ZK0313]: function "တစ်" returns isize, but no value is returned
```

## Try it

- Write a function that returns the smallest of three numbers.
- Write a function that returns whether a year is a leap year. A year is a leap year if
  it divides by 4 but not by 100, or if it divides by 400.

Next: [Recursion](recursion.md).
