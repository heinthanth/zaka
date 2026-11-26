---
title: The random module
description: A generator of random numbers - integers in a range, decimal numbers, a pick from a list and a shuffle.
module: စံကား/ကျပန်း
---

# The random module

The random module, `"စံကား/ကျပန်း"`, draws random numbers. We make a generator, a
`ကျပန်းထုတ်စက်`, then pass it with the preposition `ဖြင့်` to each function that draws a
number. A draw changes the state of the generator, so the generator goes to the call as a
[mutable parameter](../language/functions-parameters.md#mutable-parameters), and the
variable that holds it must be a `ကိန်းရှင်`.

```zaka
"စံကား/ကျပန်း" မှ ကျပန်းထုတ်စက်ပြုလုပ်၊ ကျပန်းကိန်းပြည့်ထုတ်၊ ကျပန်းဒဿမကိန်းထုတ်၊
ကျပန်းရွေးထုတ် နှင့် ရောမွှေ တို့ကို ထည့်သွင်းမည်။
ကိန်းရှင် စက် သည် ၇ ဖြင့် ကျပန်းထုတ်စက်ပြုလုပ် သည့် ရလဒ် ဖြစ်ပါစေ။
စက် ဖြင့် ၁ မှ ၆ အထိ ကျပန်းကိန်းပြည့်ထုတ် သည့် ရလဒ် ကို ပြသမည်။
စက် ဖြင့် ၁ မှ ၆ အထိ ကျပန်းကိန်းပြည့်ထုတ် သည့် ရလဒ် ကို ပြသမည်။
စက် ဖြင့် ကျပန်းဒဿမကိန်းထုတ် သည့် ရလဒ် ကို ပြသမည်။
စက် ဖြင့် ["ကြက်"၊ "ငှက်"၊ "ငါး"] မှ ကျပန်းရွေးထုတ် သည့် ရလဒ် ကို ပြသမည်။
ကိန်းရှင် ကတ်များ သည် [၁၊ ၂၊ ၃၊ ၄၊ ၅] ဖြစ်ပါစေ။
စက် ဖြင့် ကတ်များ ကို ရောမွှေမည်။
ကတ်များ ကို ပြသမည်။
```

```text
၃
၅
၀.၉၅၈၇၁၄၆၅၇၁၁၃၀၂၃၄
ဒေတာ("ကြက်")
[၄၊ ၅၊ ၃၊ ၁၊ ၂]
```

| Function | Gives |
| --- | --- |
| `ကျပန်းထုတ်စက်ပြုလုပ်` | a generator: `<seed> ဖြင့်` for a fixed seed, `ကျပန်းထုတ်စက်ပြုလုပ်()` for a seed from the OS |
| `ကျပန်းကိန်းထုတ်` | the next unsigned 64-bit number, never zero |
| `ကျပန်းကိန်းပြည့်ထုတ်` | a `ကိန်းပြည့်` from `<a> မှ` to `<b> အထိ`, both ends included, each with the same chance; the ends can come in either order and can span the whole range |
| `ကျပန်းဒဿမကိန်းထုတ်` | a `ဒဿမကိန်း` from 0 up to 1, with 1 left out |
| `ကျပန်းရွေးထုတ်` | one cell of a list (`<list> မှ`), each cell with the same chance, as an optional that is `အလွတ်` for a list with no cells |
| `ရောမွှေ` | nothing: it shuffles the list (`<list> ကို`) in place |

## Seeds

A generator from a fixed seed gives the same numbers on each run and on each backend, which
suits a test or a rerun of the same game. `ကျပန်းထုတ်စက်ပြုလုပ်()` takes its seed from the OS, so each run
of `zakac run` or of a built program draws other numbers. The books and the explorer give
that call a fixed seed, so a program there prints the same thing each time, and a call at
[compile time](../language/compile-time.md) reads a seed of zero, as the clocks do, so a
build repeats.

The generator is xorshift64, written in Zaka: shifts and exclusive ors on 64 bits, with no
`+` or `*`, so it runs the same whether an overflow stops the program or wraps. It suits
games, samples and tests. It is not safe for secrets such as passwords, since its
next numbers follow from the ones it gave.
