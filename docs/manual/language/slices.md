---
title: Slices
description: A borrowed run of cells with its count - the slice kind, the range that makes one, the lend of a whole list, cells and counts through a slice, its ownership rules, and the errors ZK0233, ZK0428 and ZK0605.
---

# Slices

A *slice* views a run of cells of one kind that sit somewhere else: in a list, in a heap
block, or in another slice. A slice is two words, the address of its first cell and the
count of its cells, and the count is known when the program runs. A function that takes a
slice takes lists of each length, and parts of them. [Slices](../../learn/lists/slices.md)
in the Learn book shows the same rules in small steps.

```zaka
လုပ်ဆောင်ချက် စုစုပေါင်း သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် န သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ န သည် ကိန်းများ ၏ အကွက်အရေအတွက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ရလဒ် ကို ရလဒ် + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။
ပြီးလျှင် န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်းရှင် အမှတ်များ သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၇၀၊ ၈၅၊ ၉၀၊ ၆၀] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် စုစုပေါင်း ကို အမှတ်များ ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် စုစုပေါင်း ကို အမှတ်များ ၏ အကွက် ၁ မှစ၍ ၂ ကွက် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
၃၀၅
၁၇၅
```

## The kind

```text
<kind> စာရင်း တစ်စိတ်တစ်ပိုင်း
```

| Kind | In messages |
| --- | --- |
| `ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း` | `&[]isize` |
| `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း` | `&[]u8` |
| `အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်စိတ်တစ်ပိုင်း` | `&[][3]isize`, a slice of rows of three |

A view of text, `စာသား တစ်စိတ်တစ်ပိုင်း` (`&text`), is a slice of u8 cells too, with no
`စာရင်း` in its kind; [Text](text.md#views) has its rules.

- `တစ်စိတ်တစ်ပိုင်း` ("a part") follows a `စာရင်း` with no count. The `&` in the short name
  marks a borrow: a slice borrows its cells from their owner.
- The `စာရင်း` before it takes no `အကွက် <n> ခုရှိသော` prefix, since the count of a slice
  comes when the program runs: a count there is ZK0233. A count prefix pairs with the
  inner `စာရင်း` of a list of lists, as the rows example shows.
- The cells can be of each kind but a borrow: a slice whose cells are slices or borrowed
  addresses is ZK0428.
- A slice declared with no value views no cells: its address is `ဗလာ` and its count 0.

```zaka-error ZK0233
ကိန်းရှင် s သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်ပါစေ။
```

```text
error[ZK0233]: a slice takes no cell count; its count comes when the program runs
```

## Making a slice

```text
<value> ၏ အကွက် <start> မှစ၍ <length> ကွက်
<list> ၏ တန်ဖိုးအငှား
```

- The range ("`length` cells from cell `start` on") counts its cells from 0. The value is a
  list, a pointer to a list, a slice, or a pointer into a heap block; another value is
  ZK0422.
- The start and the length are integers (ZK0423). Each one reads up to the word after it,
  so `x ၏ အကွက် i + ၁ မှစ၍ န - ၁ ကွက်` needs no parentheses. A length of 0 makes a slice
  of no cells.
- `<list> ၏ တန်ဖိုးအငှား` ("the value of the list, on loan") is the slice of each cell of
  the list.
- A list in either form must be a place, as for its address: a literal or a call result is
  ZK0411, and a constant is ZK0412.
- Nothing turns into a slice on its own. A list given where a slice is wanted is ZK0401,
  and a pointer has no count to give. The lend or the range writes the count out.
- The range must lie in its source. With a list as the source and the start and the
  length as constants, a range past the end is ZK0605. Each other range is checked when
  the program runs: past a list or a slice, the program stops with the range it asked
  for; past a heap block, it stops as a pointer step out of the block does.

```zaka
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် အပိုင်း သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် ဖြစ်ပါစေ။
အပိုင်း ၏ အကွက် ၂ ကို ၇ ဟု ထားပါ။
အပိုင်း ကို ပြသပါ။
အပိုင်း ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
```

```text
[၀၊ ၀၊ ၇]
[၀၊ ၇]
```

```zaka-error ZK0605
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက် ၂ မှစ၍ ၂ ကွက် ကို ပြသပါ။
```

```text
error[ZK0605]: 2 cells from cell 2 pass the end of a list of 3 cells
```

```zaka-stop
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် n သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၄ ဖြစ်ပါစေ။
l ၏ အကွက် ၀ မှစ၍ n ကွက် ကို ပြသပါ။
```

```text
the program stopped: 4 cells from cell 0 pass the end of 3 cells
```

## Using a slice

| Written | Means |
| --- | --- |
| `s ၏ အကွက် i` | cell `i`, counted from 0; a place |
| `s ၏ n ခုမြောက်အကွက်` | cell `n`, counted from 1 |
| `s ၏ အကွက်အရေအတွက်` | the count, a `လက္ခဏာမဲ့ ကိန်းပြည့်` known when the program runs |
| `s ၏ အကွက် i မှစ၍ n ကွက်` | a slice of the slice |

- A cell of a slice is a cell of its source: writing it changes the list or the block.
- A cell number is checked against the count of the slice. The compiler does the check
  when it can see the answer, and leaves it out of the program:
  - a slice made with a known length, such as `l ၏ အကွက် ၁ မှစ၍ ၂ ကွက်` or the lend of a
    list, keeps that count, so a cell number the compiler can follow is checked as for a
    list, and one past the end is ZK0710;
  - in a loop that tests `န < s ၏ အကွက်အရေအတွက်` before each round, the cell `s ၏ အကွက် န`
    is in range until `န` or `s` changes, as in the first example. `န` must be a
    variable of the function whose address is never taken, at 0 or above.

  Each other cell number is checked when the program runs, and one outside the slice
  stops the program.
- A slice prints its cells, as a list does.
- Setting one slice to another duplicates the view, its address and its count, and not
  the cells.
- No operator takes a slice: `==` on two slices is ZK0403. Compare the cells one at a time.
- A counted loop to the count minus 1 goes wrong on a slice of no cells, since the count
  is unsigned and 0 - 1 wraps. A loop that tests `န < count` before each round, as in
  the first example, runs zero rounds there.

```zaka-error ZK0710
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
s ၏ အကွက် ၂ ကို ပြသပါ။            // s holds cells 0 and 1
```

```text
error[ZK0710]: offset 2 is past the end of a block of 2 elements
```

```zaka
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
s ၏ အကွက် ၀ ကို ၂၀ ဟု ထားပါ။
s ၏ ၂ ခုမြောက်အကွက် ကို ၃၀ ဟု ထားပါ။
l ကို ပြသပါ။
s ၏ အကွက်အရေအတွက် ကို ပြသပါ။
```

```text
[၁၊ ၂၀၊ ၃၀၊ ၄]
၂
```

## Ownership

A slice is a borrow, so the rules of [borrows](ownership.md#lending-a-block) hold for it:

- Freeing or resizing the block a slice views, while the slice is still used afterwards,
  is ZK0704.
- A function can return a slice of a borrow parameter or of a top-level variable; a slice
  of its own list is ZK0706.
- A struct field cannot hold a slice (ZK0718), and a function cannot keep one in a
  top-level variable (ZK0703).
- The cells of a slice cannot hold borrows (ZK0428): those cells live outside the frame,
  where the checker cannot follow a borrow.
- Cells that hold owners can be read and written through a slice, as the cells of a list
  can.

```zaka-error ZK0704
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
s ကို ပြသပါ။
```

```zaka
// the result points into the cells the caller lent
လုပ်ဆောင်ချက် အလယ်ပိုင်း သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းများ ၏ အကွက် ၁ မှစ၍ (ကိန်းများ ၏ အကွက်အရေအတွက်) - ၂ ကွက် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် အလယ်ပိုင်း ကို l ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
```

```text
[၂၊ ၃]
```

## The errors

| Code | Raised when |
| --- | --- |
| ZK0233 | the `စာရင်း` of a slice kind has a count |
| ZK0428 | the cells of a slice kind hold borrows |
| ZK0605 | a range with a constant start and length passes the end of its list |
| ZK0710 | a cell number proven past the end of a slice of a known count |
| ZK0401 | a list is given where a slice is wanted |
| ZK0422 | a range is taken of a value that is no list, no slice and no pointer to a block |
| ZK0403 | an operator is given a slice |
