---
title: Copies and storage
description: Part II of Learn - take ranges of lists and text, see where their cells live, keep a list's cells in place, and see what a duplicate costs.
---

# Copies and storage

This page starts Part II of Learn. Part I taught Zaka with values and shared objects: each
assignment and each call gives a value of its own, and the program never asks where the
cells of text or a list live. A program written so keeps its meaning here. Part II shows
what that costs, and the forms that keep the cost down:

- a *range* takes a run of cells from a list or from text, as a list or text of its own;
- *static data* holds literals and constants with no heap block, and a *counted list*
  keeps its count and its cells in place;
- a duplicate shares its cells until one side writes, and the compiler skips the duplicate where
  nothing can tell.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် "Zaka စံကား" ဖြစ်ပါစေ။
ကိန်းရှင် တို သည် အမည် ၏ အကွက် ၀ မှစ၍ ၄ ကွက် ဖြစ်ပါစေ။   // its first four cells
တို ကို ပြသမည်။
အမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Zaka
Zaka စံကား
```

| Zaka | Reads as |
| --- | --- |
| `အမည် ၏ အကွက် ၀ မှစ၍ ၄ ကွက်` | "four cells of the name, starting at cell 0" |

The pages of this part:

- [Ranges](ranges.md): a run of cells as a list or text of its own.
- [Static data and counted lists](counted-lists.md): where cells live, and lists that keep them in place.
- [What a duplicate costs](costs.md): when a duplicate is made, and when it is not.

Part III, [Unsafe Zaka](../pointers/README.md), follows: pointers, the heap and C, in
code marked as unchecked.

Next: [Ranges](ranges.md).
