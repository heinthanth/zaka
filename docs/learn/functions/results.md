---
title: Results
description: Functions that work out a value and return it to the caller.
---

# Results

The functions so far *do* something, such as printing. A lot of functions instead *work
out* a value and return it to the caller, such as the area of a rectangle or the larger of
two numbers. The value that a function returns is its *result* (return value).

## You will learn

- how to declare a function with a result, and how to return the result
- how to use the result of a call as a value
- the short sign form of a call, `<name>(<arguments>)`
- the mistakes a function with a result can have

```zaka
// "Function square, accepting an integer k, doing the following,
//  will return an integer result."
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။ // return k × k as the result
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// Use the result like other values
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
```

```text
၄၉
```

In the function signature, we can use `<kind> တစ်ခု ကို ပြန်ပေးမည်` to define the return
value, and inside the function we write `<value> ကို ပြန်ပေးမည်။` to return it:

```text
<name> ဟူသော လုပ်ဆောင်ချက် သည်
<parameters> လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။
<value> ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

| Zaka | Reads as |
| --- | --- |
| `အောက်ပါတို့ကို လုပ်ဆောင်ကာ` | "doing the following, and" |
| `ကိန်းပြည့် တစ်ခု ကို` | "one integer" |
| `ပြန်ပေးမည်` | "will return" |
| `<value> ကို ပြန်ပေးမည်။` | "will return `<value>`" |
| `… လုပ်ဆောင်သော ရလဒ်` | "the result of running …", a value |

So three things change compared with a function that has no result:

| Where | No result | With a result |
| --- | --- | --- |
| function signature | `အောက်ပါတို့ကို လုပ်ဆောင်မည်။` | `အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။` |
| inside | no return of a value | `<value> ကို ပြန်ပေးမည်။` |
| call | `… လုပ်ဆောင်မည်။`, a statement | `… လုပ်ဆောင်သော ရလဒ်`, a value |

The word after `တစ်` is the measure word (classifier) of the result's data kind (type),
which is `ခု` for most data kinds and `လုံး` for a character.

## Returning the result

`<value> ကို ပြန်ပေးမည်။` ends the function at once and hands the value to the caller. A
function with a result must return a value on each path to its end, so it ends with a
return, or each branch before the end returns:

```zaka
// area: return the product, which ends the function
ဧရိယာ ဟူသော လုပ်ဆောင်ချက် သည်
အလျား ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
အနံ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အလျား * အနံ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// larger: return as soon as the answer is known
ပိုကြီး ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က သည် ခ ထက် ကြီးလျှင် က ကို ပြန်ပေးမည်။
ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဧရိယာ ဟူသော လုပ်ဆောင်ချက်ကို ၃ နှင့် ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပိုကြီး ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ နှင့် ၃ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
```

```text
၁၂
၁၀
```

## Using a result

```text
<name> ဟူသော လုပ်ဆောင်ချက်ကို <arguments> ဖြင့် လုပ်ဆောင်သော ရလဒ်
```

A call that ends in `လုပ်ဆောင်သော ရလဒ်` is a value, so we can use it wherever a value fits:
we can print it, store it, pass it to another function, or use it in a sum:

```zaka
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// store it
ကိန်းရှင် ကိုး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိုး ကို ပြသမည်။

// in a sum: 3² + 4²
(နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်သော ရလဒ်) + နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၄ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။

// pass a result into another call: (2²)²
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
```

```text
၉
၂၅
၁၆
```

Note the parentheses in the sum. Zaka treats a statement that starts with a call in the
word form as a call statement, so when a sum *starts* with a call, we put that call in
parentheses. A call later in the sum, or a call on its own, needs none. To drop a result,
we call the function as a statement with `လုပ်ဆောင်မည်။`.

## The sign form

```text
<name>(<argument>၊ <argument>)
```

A call inside a sum or inside another call needs a lot of words, so we can use the *sign
form* to write the same call in fewer: the name, then the arguments in parentheses, with
`၊` between them. It is the same call, just as `+` is the same sum as its word form, and
it looks like a call in most other languages.

```zaka
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ကိုး သည် နှစ်ထပ်(၃) ဖြစ်ပါစေ။ // store it
ကိုး ကို ပြသမည်။
နှစ်ထပ်(၃) + နှစ်ထပ်(၄) ကို ပြသမည်။ // in a sum, with no parentheses around it
နှစ်ထပ်(နှစ်ထပ်(၂)) ကို ပြသမည်။ // one call inside another
```

```text
၉
၂၅
၁၆
```

We write the `(` right after the name, with no space. The sign form is a value, so a call
whose result we drop still needs the statement form, `လုပ်ဆောင်မည်။`.

## Arguments first, with a result

```text
<arguments> ကို သုံး၍ <name> သည့် ရလဒ်
```

The arguments-first call of [Parameters](parameters.md#arguments-first) returns the result
when we end it with `သည့် ရလဒ်` ("the result that …") in place of `မည်`:

```zaka
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

// "the result that square (gives), using 5"
၅ ကို သုံး၍ နှစ်ထပ် သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် ခြောက်ဆယ့်လေး သည် ၈ ကို သုံး၍ နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ခြောက်ဆယ့်လေး ကို ပြသမည်။
```

```text
၂၅
၆၄
```

## Results of other data kinds

A result can have whatever data kind the function needs, including lists and structs (see
[Lists as results](../lists/passing.md#lists-as-results)). This function returns true or
false:

```zaka
// divides: does d divide n with nothing left over?
ပြတ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
ဆခွဲ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
န % ဆခွဲ သည် ၀ နှင့် ညီသည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အကယ်၍ ပြတ်သလား ဟူသော လုပ်ဆောင်ချက်ကို ၁၅ နှင့် ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင်
"ပြတ်သည်" ဟု ပြသမည်။
```

```text
ပြတ်သည်
```

## Mistakes

A function without a result has no value for a call to use:

```zaka-error ZK0315
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
```

```text
error[ZK0315]: function "ပြ" returns nothing and cannot be used as a value
```

A function with a result must return a value each time it returns, so a bare
`ပြန်ပေးမည်။` is an error there:

```zaka-error ZK0313
တစ် ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0313]: function "တစ်" returns isize, but no value is returned
```

When a function with a result can reach the end of its block without a return, the caller
would be left with no value, so that is an error too:

```zaka-error ZK0339
ဆယ်ကျော် ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က သည် ၁၀ ထက် ကြီးလျှင် က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```text
error[ZK0339]: function "ဆယ်ကျော်" gives back a result, but its block can reach its end without giving one back
```

## Details

- Returning a value from a function without a result is the error ZK0312, and a value that
  does not fit the result's data kind is ZK0401 ([Results and
  return](../../manual/language/functions-results.md#return-statements)).
- A function signature with no measure word after `တစ်` is ZK0234, and a measure word that
  does not fit the data kind gets the warning ZK0509 ([Results and
  return](../../manual/language/functions-results.md#declaring-a-result)).
- A function has no variable named `ရလဒ်`: we return the value with
  `<value> ကို ပြန်ပေးမည်။`.
- A loop at the end does not count as a return, because its test can be false at the start;
  a loop whose test is `မှန်` and that no break leaves does count, since it never ends
  ([Results and return](../../manual/language/functions-results.md#each-path-returns)).
- `သော ရလဒ်` and `မည့် ရလဒ်` mean the same as `သည့် ရလဒ်` at the end of an arguments-first
  call ([Arguments first](../../manual/language/functions-calls.md#arguments-first)).
- A function can also fail and return an error value instead of a result: [Errors and
  results](../error-handling/README.md) teaches it.

## Recap

- In the function signature, we write `အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> တစ်ခု ကို ပြန်ပေးမည်။`
  to give the function a result.
- Inside, `<value> ကို ပြန်ပေးမည်။` returns the value and ends the function, and each path
  must return one.
- A call that ends in `လုပ်ဆောင်သော ရလဒ်` or `သည့် ရလဒ်` is a value, and so is the sign form
  `<name>(<arguments>)`.
- When a sum starts with a word-form call, we put that call in parentheses.

## Hands on

- Write a function that returns the smallest of three numbers.
- Write a function that returns true for multiples of 4 that are not multiples of 100,
  and also for multiples of 400. This is the leap rule of the Gregorian calendar.
- Rewrite the sum `3² + 4²` with the arguments-first form.

Next: [Prepositions](prepositions.md).
