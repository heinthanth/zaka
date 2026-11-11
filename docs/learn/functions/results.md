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
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။      // return k × k as the result
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// Use the result like other values
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၇ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄၉
```

Two things change compared with a function that has no result:

| Where | No result | With a result |
| --- | --- | --- |
| header | `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` ("will do the following") | `အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။` ("doing the following, will return one value of the kind") |
| inside | no return of a value | `<value> ကို ပြန်ပေးမည်။` ("give back *value*") |
| call | `... လုပ်ဆောင်မည်။` ("run it") | `... လုပ်ဆောင်သော ရလဒ်` ("the result of running it"), a value |

The word after `တစ်` is the one the kind takes: `ခု` for most kinds, `လုံး` for a character.

## Returning the result

`<value> ကို ပြန်ပေးမည်။` ("give back *value*") ends the function at once and hands the
value to the caller. A function with a result gives a value back on each path that
reaches its end, so it ends with a return, or each branch before it returns.

```zaka
// area: give back the product, which ends the function
လုပ်ဆောင်ချက် ဧရိယာ သည်
အလျား ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း အနံ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အလျား * အနံ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// larger: give back as soon as the answer is known
လုပ်ဆောင်ချက် ပိုကြီး သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က သည် ခ ထက် ကြီးလျှင် က ကို ပြန်ပေးမည်။
ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ဧရိယာ ကို ၃၊ ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်ချက် ပိုကြီး ကို ၁၀၊ ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
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
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// store it
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိုး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိုး ကို ပြသမည်။

// in a sum: 3² + 4²
(လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်) + လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။

// pass a result into another call: (2²)²
လုပ်ဆောင်ချက် နှစ်ထပ် ကို လုပ်ဆောင်ချက် နှစ်ထပ် ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
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

To discard a result, call the function as a statement with `လုပ်ဆောင်မည်။`.

## The sign form

A call inside a sum or inside another call takes a lot of words. The *sign form* writes
the same call in fewer: the name, then the arguments in parentheses, with `၊` between
them. It is the same call, as `+` is the same sum as the word form.

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကိုး သည် နှစ်ထပ်(၃) ဖြစ်ပါစေ။     // store it
ကိုး ကို ပြသမည်။
နှစ်ထပ်(၃) + နှစ်ထပ်(၄) ကို ပြသမည်။         // in a sum, with no parentheses around it
နှစ်ထပ်(နှစ်ထပ်(၂)) ကို ပြသမည်။            // one call inside another
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉
၂၅
၁၆
```

Write the `(` right after the name, with no space. A call whose result is dropped still
takes the statement form, `လုပ်ဆောင်မည်။`.

## Arguments first, with a result

The arguments-first call of [Parameters](parameters.md#arguments-first) gives back the
result when it ends with `သည့် ရလဒ်` ("the result that ...") in place of `မည်`:

```zaka
လုပ်ဆောင်ချက် နှစ်ထပ် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
// "the result that square (gives), using 5"
၅ ကို သုံး၍ နှစ်ထပ် သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် ခြောက်ဆယ့်လေး သည် ၈ ကို သုံး၍ လုပ်ဆောင်ချက် နှစ်ထပ် ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ခြောက်ဆယ့်လေး ကို ပြသမည်။
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
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ဆခွဲ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
န % ဆခွဲ သည် ၀ နှင့် ညီသည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ လုပ်ဆောင်ချက် ပြတ်သလား ကို ၁၅၊ ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် "ပြတ်သည်" ဟု ပြသမည်။
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
၁ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0315]: function "ပြ" returns nothing and cannot be used as a value
```

A function with a result must give a value back whenever it returns:

```zaka-error ZK0313
လုပ်ဆောင်ချက် တစ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0313]: function "တစ်" returns isize, but no value is returned
```

A function with a result whose block can reach its end with no return is an error too,
since the caller would be left with no value:

```zaka-error ZK0339
လုပ်ဆောင်ချက် ဆယ်ကျော် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က သည် ၁၀ ထက် ကြီးလျှင် က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0339]: function "ဆယ်ကျော်" gives back a result, but its block can reach its end without giving one back
```

## Hands on

- Write a function that returns the smallest of three numbers.
- Write a function that returns true for multiples of 4 that are not multiples of 100,
  and also for multiples of 400. This is the leap rule of the Gregorian calendar.

Next: [Recursion](recursion.md).
