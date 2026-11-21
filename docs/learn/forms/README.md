---
title: Copies and storage
description: Part II of Learn - take ranges of lists and text, see where their cells live, keep a list's cells in place, and see what a duplicate costs.
---

# Copies and storage

This page starts Part II of Learn. Part I taught Zaka with values and shared objects:
each assignment and each call makes a separate value, a duplicate (copy), and we never had
to ask where in storage (memory) the cells of text or a list live. Programs written like
that still mean the same thing here, while Part II shows what the duplicates cost and the
forms that keep the cost down.

## You will learn

- How to take a range, a run of cells from a list or from text.
- Where the cells of text and lists live: in the program itself or on the heap (dynamic
  memory).
- How a counted list (fixed-size array) keeps its cells in place.
- When a duplicate is made, and when the compiler can skip it.

## A first range

This program takes the first four cells of a text as a new text:

```zaka
ကိန်းရှင် အမည် သည် "Zaka စံကား" ဖြစ်ပါစေ။
ကိန်းရှင် တို သည် အမည် ၏ အကွက် ၀ မှစ၍ ၄ ကွက် ဖြစ်ပါစေ။   // its first four cells
တို ကို ပြသမည်။
အမည် ကို ပြသမည်။
```

```text
Zaka
Zaka စံကား
```

```
<list or text> ၏ အကွက် <start> မှစ၍ <count> ကွက်
```

| Zaka | Reads as |
| --- | --- |
| `အမည် ၏` | "of the name" |
| `အကွက် ၀ မှစ၍` | "starting at cell 0" |
| `၄ ကွက်` | "four cells" |

The range is a duplicate, so `တို` is a new text and `အမည်` does not change.

## The pages of this part

- [Ranges](ranges.md): a run of cells as a separate list or text.
- [Static data and counted lists](counted-lists.md): where the cells live, and lists that
  keep them in place.
- [What a duplicate costs](costs.md): when a duplicate is made, and when it is not.

After this part, Part III, [Unsafe Zaka](../pointers/README.md), covers pointers, the heap
and C, in unsafe code.

## Recap

- Part II keeps the meaning of Part I, and shows the cost of each duplicate.
- A range takes a run of cells from a list or from text as a new value.
- A counted list has a fixed count of cells, kept in the list itself.

Next: [Ranges](ranges.md).
