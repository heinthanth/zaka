---
title: Tables
description: A ဇယား of rows found through an index, as a phone book finds a number through a name: making one, reading a row, setting and removing rows, and walking the rows.
---

# Tables

A list finds a value through a number: cell 0, cell 1, cell 2. A phone book finds a number
through a name. Zaka writes such a book as a ဇယား ("table"): it holds rows, each row has an
index (`အညွှန်း`) that finds it and a value (`တန်ဖိုး`), and no two rows have one index.

## Making a ဇယား

The kind names the index kind first, then the value kind:
`စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား` is a ဇယား of integers indexed with text. A ဇယား is written
out as `[<index>: <value>၊ …]`, and `[:]` is one with no rows:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖုန်း သည် ["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂] ဖြစ်ပါစေ။
ကိန်းရှင် အသစ် သည် စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
ဖုန်း ကို ပြသမည်။
အသစ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂]
[:]
```

The sentence form names each row as `<index> အနေဖြင့် <value>`:
`"မမ" အနေဖြင့် ၉၅၁၁၁ နှင့် "ကို" အနေဖြင့် ၉၅၂၂၂ တို့ပါသော ဇယား` is the same ဇယား.

## Reading a row

`<table> ထဲမှ အညွှန်း <index> ၏ တန်ဖိုး` reads the row of an index. A name the book does not
hold has no row, so the read is an [optional](../optionals/README.md): the value, or `အလွတ်`.
`၊ မရှိလျှင် <value>` after `ထဲမှ ဒေတာ` gives a value to use when the row is missing:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖုန်း သည် ["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂] ဖြစ်ပါစေ။
ဖုန်း ထဲမှ အညွှန်း "မမ" ၏ တန်ဖိုး ကို ပြသမည်။
ဖုန်း ထဲမှ အညွှန်း "ဇော်" ၏ တန်ဖိုး ကို ပြသမည်။
ဖုန်း ထဲမှ အညွှန်း "ဇော်" ၏ တန်ဖိုး ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ ကို ပြသမည်။
အကယ်၍ ဖုန်း ထဲမှ အညွှန်း "ကို" ၏ တန်ဖိုး ထဲတွင် ဒေတာ ရှိလျှင် "ကို ၏ ဖုန်း ရှိသည်" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဒေတာ(၉၅၁၁၁)
အလွတ်
၀
ကို ၏ ဖုန်း ရှိသည်
```

## Setting and removing rows

A set gives a row its value, or adds the row when no row holds the index; a remove takes a
row out. A new row goes last, and a row that takes a new value keeps its place:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖုန်း သည် ["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂] ဖြစ်ပါစေ။
ဖုန်း ထဲမှ အညွှန်း "ဇော်" ၏ တန်ဖိုး ကို ၉၅၃၃၃ ဟု ထားမည်။
ဖုန်း ထဲမှ အညွှန်း "မမ" ၏ တန်ဖိုး ကို ၉၅၁၀၀ ဟု ထားမည်။
ဖုန်း ထဲမှ အညွှန်း "ကို" ကို ဖျက်မည်။
ဖုန်း ကို ပြသမည်။
ဖုန်း ၏ အညွှန်းအရေအတွက် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["မမ": ၉၅၁၀၀၊ "ဇော်": ၉၅၃၃၃]
၂
```

`အညွှန်းအရေအတွက်` ("index count") is the number of rows.

## Walking the rows

`အညွှန်းတိုင်း` ("each index") walks the indexes and `အတန်းတိုင်း` ("each row") the values, in
the order the rows were added. Two names take the index and then the value:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖုန်း သည် ["မမ": ၉၅၁၁၁၊ "ကို": ၉၅၂၂၂] ဖြစ်ပါစေ။
ဖုန်း ၏ အညွှန်းတိုင်းကို အမည် အနေဖြင့် ယူ၍ အမည် ကို ပြသမည်။
ဖုန်း ၏ အညွှန်းတိုင်းကို အမည် နှင့် နံပါတ် အနေဖြင့် ယူ၍ "{အမည်}: {နံပါတ်}" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မမ
ကို
မမ: ၉၅၁၁၁
ကို: ၉၅၂၂၂
```

## Counting words

A ဇယား counts things well: each word reads its count so far, with 0 for a word not met, and
sets it one higher.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စကားလုံးများ သည် ["နေ"၊ "လ"၊ "နေ"၊ "ကြယ်"၊ "နေ"] ဖြစ်ပါစေ။
ကိန်းရှင် ရေ သည် စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
စကားလုံးများ ၏ အကွက်တိုင်း ကို စ အနေဖြင့် ယူ၍ ရေ ထဲမှ အညွှန်း စ ၏ တန်ဖိုး ကို ရေ ထဲမှ အညွှန်း စ ၏ တန်ဖိုး ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ + ၁ ဟု ထားမည်။
ရေ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
["နေ": ၃၊ "လ": ၁၊ "ကြယ်": ၁]
```

A ဇယား is a value, as a list is: a duplicate of one is a ဇယား of its own. The full rules,
from the kinds that can index rows to writing inside a row, are on
[Tables](../../manual/language/tables.md) in the Docs.
