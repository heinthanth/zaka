---
title: Results
description: Functions that work out a value and give it back.
---

# Results

The functions so far *do* something, such as printing. A lot of functions instead *work
out* a value and hand it back to the caller: the area of a rectangle, the larger of two
numbers. The value handed back is the function's *result*.

```zaka
// "Function square, accepting an integer k, doing the following,
//  will return an integer result."
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။      // return k × k as the result
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// Use the result like other values
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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

There are two options for giving back the result.

**Return it.** `<value> ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။` ("give back *value* as the result") ends the
function at once.

**Set `ရလဒ်`.** A function with a result has a built-in variable named `ရလဒ်`
("result") of the result's kind. It starts at the kind's default. Whatever it holds when
the function reaches its end is returned.

```zaka
// area: set ရလဒ် and let the function end
လုပ်ဆောင်ချက် ဧရိယာ သည်
ကိန်းပြည့် အလျား၊ ကိန်းပြည့် အနံ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို အလျား * အနံ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// larger: return as soon as the answer is known
လုပ်ဆောင်ချက် ပိုကြီး သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ က သည် ခ ထက် ကြီးလျှင် က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ဧရိယာ ကို ၃၊ ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် ပိုကြီး ကို ၁၀၊ ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၂
၁၀
```

## Using a result

A call with `လုပ်ဆောင်သော ရလဒ်` is a value. Use it wherever a value fits: print it, store
it, pass it to another function, or use it in a sum:

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// store it
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိုး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိုး ကို ပြသပါ။

// in a sum: 3² + 4²
(လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်) + လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။

// pass a result into another call: (2²)²
လုပ်ဆောင်ချက် နှစ်ထပ် ကို လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉
၂၅
၁၆
```

Notice the parentheses in the sum. A statement that begins with `လုပ်ဆောင်ချက်` is read
as a call statement, so a sum that *starts* with a call needs the call in parentheses.
A call later in the sum, or a call on its own, needs none.

To discard a result, call the function as a statement with `လုပ်ဆောင်ပါ။`.

## The sign form

A call inside a sum or inside another call takes a lot of words. The *sign form* writes
the same call in fewer: the name, then the arguments in parentheses, with `၊` between
them. It is the same call, as `+` is the same sum as the word form.

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိုး သည် နှစ်ထပ်(၃) ဖြစ်ပါစေ။     // store it
ကိုး ကို ပြသပါ။
နှစ်ထပ်(၃) + နှစ်ထပ်(၄) ကို ပြသပါ။         // in a sum, with no parentheses around it
နှစ်ထပ်(နှစ်ထပ်(၂)) ကို ပြသပါ။            // one call inside another
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉
၂၅
၁၆
```

Write the `(` right after the name, with no space. A call whose result is dropped still
takes the statement form, `လုပ်ဆောင်ပါ။`.

## Arguments first, with a result

The arguments-first call of [Parameters](parameters.md#arguments-first) gives back the
result when it ends with `သည့် ရလဒ်` ("the result that ...") in place of `မည်`:

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// "the result that square (gives), using 5"
၅ ကို သုံး၍ နှစ်ထပ် သည့် ရလဒ် ကို ပြသပါ။
ကိန်းရှင် ခြောက်ဆယ့်လေး သည် ၈ ကို သုံး၍ လုပ်ဆောင်ချက် နှစ်ထပ် ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ခြောက်ဆယ့်လေး ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂၅
၆၄
```

## Results of other kinds

A result can be of whatever kind the function needs, lists and structs included (see
[Lists as results](../lists/passing.md#lists-as-results)). This one gives back true or
false:

```zaka
// divides: does d divide n with nothing left over?
လုပ်ဆောင်ချက် ပြတ်သလား သည်
ကိန်းပြည့် န၊ ကိန်းပြည့် ဆခွဲ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
န % ဆခွဲ သည် ၀ နှင့် ညီသည် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ လုပ်ဆောင်ချက် ပြတ်သလား ကို ၁၅၊ ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် "ပြတ်သည်" ဟု ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
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
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0315]: function "ပြ" returns nothing and cannot be used as a value
```

A function with a result must give a value back whenever it returns:

```zaka-error ZK0313
လုပ်ဆောင်ချက် တစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0313]: function "တစ်" returns isize, but no value is returned
```

## Hands on

- Write a function that returns the smallest of three numbers.
- Write a function that returns true for multiples of 4 that are not multiples of 100,
  and also for multiples of 400. This is the leap rule of the Gregorian calendar.

Next: [Recursion](recursion.md).
