---
title: Borrowing and storage
description: Part II of Learn - read text and lists with no duplicate, see where their cells live, keep a list's cells in place, and see what a duplicate costs.
---

# Borrowing and storage

This page starts Part II of Learn, *control*. Part I taught Zaka with values alone: each
assignment and each call gives a value of its own, and the program never asks where the
cells of text or a list live. A program written so keeps its meaning here. Part II
adds the words that control the cost of a value, for when the cost matters:

- a *view* reads text, or part of it, where it is, with no duplicate;
- a *slice* does the same for the cells of a list, and can write them as well;
- *static data* holds literals and constants with no heap block, and a *counted list*
  keeps its count and its cells in place;
- `--warn-duplicates` names each statement that still makes a duplicate.

```zaka
လုပ်ဆောင်ချက် ပြ သည်
စာသား တစ်စိတ်တစ်ပိုင်း စာ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "Zaka စံကား" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို အမည် ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်ပါ။       // the whole text, on loan
လုပ်ဆောင်ချက် ပြ ကို အမည် ၏ အကွက် ၀ မှစ၍ ၄ ကွက် တို့ဖြင့် လုပ်ဆောင်ပါ။  // its first four cells
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
Zaka စံကား
Zaka
```

| Zaka | Reads as |
| --- | --- |
| `စာသား တစ်စိတ်တစ်ပိုင်း` | "a part of text": the kind of a view |
| `အမည် ၏ တန်ဖိုးအငှား` | "the value of the name, on loan" |
| `အမည် ၏ အကွက် ၀ မှစ၍ ၄ ကွက်` | "four cells of the name, starting at cell 0" |

The pages of this part:

- [Borrowing text](views.md): lend text, and read part of it.
- [Slices](../lists/slices.md): lend a list, and read or write a run of its cells.
- [Static data and counted lists](counted-lists.md): where cells live, and lists that keep them in place.
- [What a duplicate costs](costs.md): when a duplicate is made, and how to see it.

Part III, *storage*, follows: addresses, the heap and ownership, in [Pointers and the
heap](../pointers/README.md).

Next: [Borrowing text](views.md).
