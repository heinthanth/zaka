---
title: The time module
description: Clocks and UTC formatting.
module: စံကား/အချိန်
---

# The time module

The time module, `"စံကား/အချိန်"`, reads clocks and writes a time as UTC text.

A clock reading is a signed 64-bit integer in milliseconds.

UTC formatting uses Western digits and three fractional digits. Negative stamps refer
to instants before 1970. The explorer uses a virtual clock that starts at the Unix epoch
and advances during waits. Comptime calls read zero from both clocks.

```zaka
"စံကား/အချိန်" မှ အိုင်အက်စ်အိုစာသားအဖြစ်ပြောင်း ကို ထည့်သွင်းမည်။
အိုင်အက်စ်အိုစာသားအဖြစ်ပြောင်း(၀) ကို ပြသမည်။
အိုင်အက်စ်အိုစာသားအဖြစ်ပြောင်း(-၁) ကို ပြသမည်။
```

```text
ရလဒ်ကောင်း("1970-01-01T00:00:00.000Z")
ရလဒ်ကောင်း("1969-12-31T23:59:59.999Z")
```
