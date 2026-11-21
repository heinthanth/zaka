---
title: Shared objects
description: Declare a struct that is shared between names instead of duplicated, test whether two names reach the same object, and duplicate an object when needed.
---

# Shared objects

Text, lists and structs are values, so each assignment and each argument makes a duplicate
(copy) of them, the same as for a number. When we write `မျှဝေနိုင်သော` ("that can be
shared") before `ဒေတာအစုအဝေး`, each value of the struct becomes a *shared object*
(reference type) instead, so an assignment or an argument shares the same object, and a
change through one name shows through all the others.

## You will learn

- how to declare a shared object, and how it differs from a plain struct
- how to test whether two names reach the same object
- how to duplicate an object, and the difference between a shallow and a deep duplicate
- how objects can point at each other

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

// deposit: add to the balance of the account the caller passes in
အပ်ငွေ ဟူသော လုပ်ဆောင်ချက် သည် အ ဟူသော အကောင့် တစ်ခု ကို လည်းကောင်း ပမာဏ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အ ၏ လက်ကျန် ကို (အ ၏ လက်ကျန်) + ပမာဏ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။                        // one account, two names
အပ်ငွေ ဟူသော လုပ်ဆောင်ချက်ကို ခ နှင့် ၅၀ တို့ဖြင့် လုပ်ဆောင်မည်။
က ကို ပြသမည်။
```

```text
အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၅၀)
```

The program declares `အကောင့်` (account) as a shared object, and `ခ` and the parameter `အ`
reach the same account as `က`, so the deposit through `အ` shows when we print `က`. We can
use this pattern to declare a shared object:

```text
<name> ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် <field> ဟူသော <kind> တစ်ခု နှင့် <field> ဟူသော <kind> တစ်ခု တို့ပါဝင်သည်။
```

| Zaka | Reads as |
| --- | --- |
| `အကောင့် ဟူသော` | "called `အကောင့်`" |
| `မျှဝေနိုင်သော` | "that can be shared" |
| `ဒေတာအစုအဝေးတွင်` | "in the struct" |
| `… တို့ပါဝင်သည်` | "these are included" |

Apart from the declaration, a shared object works the same as a [struct](README.md): it
has the same literals, fields, measure word and printed form. If `အကောင့်` were a plain
struct, `ခ` and `အ` would each get a duplicate, and `က` would still print `၁၀၀`.

## Values and shared objects

This table shows what an assignment, an argument and a result give for each data kind
(type):

| Data kind | Assignment, argument or result |
| --- | --- |
| number, character, true/false value | a duplicate |
| text, list, table, struct | a duplicate, which copies the cells at its first write |
| shared object | the same object |

## Equal or the same

```text
<object> == <object>
အကယ်၍ <object> နှင့် <object> သည် တစ်ခုတည်း ဖြစ်လျှင် <sentence>
```

`==` compares the contents for each data kind, objects included, so two accounts with equal
fields are equal. To test whether two names reach one and the same object, we write
`က နှင့် ခ သည် တစ်ခုတည်း ဖြစ်လျှင်` ("if က and ခ are one and the same"):

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် က ဖြစ်ပါစေ။
က == ခ ကို ပြသမည်။                                       // the same contents
အကယ်၍ က နှင့် ခ သည် တစ်ခုတည်း ဖြစ်လျှင် "ခ" ဟု ပြသမည်။      // two accounts: not printed
အကယ်၍ က နှင့် ဂ သည် တစ်ခုတည်း ဖြစ်လျှင် "ဂ" ဟု ပြသမည်။      // one account
```

```text
မှန်
ဂ
```

## Duplicating an object

```text
<new name> ထဲသို့ <object> ကို ကူးထည့်မည်။
<object> ၏ မိတ္တူ
```

When we need a separate object with the same fields, we use `ကူးထည့်မည်` ("will duplicate
into") as a sentence, or `က ၏ မိတ္တူ` ("the duplicate of က") as a value:

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ ထဲသို့ က ကို ကူးထည့်မည်။          // a new account with the same fields
ကိန်းရှင် ဂ သည် က ၏ မိတ္တူ ဖြစ်ပါစေ။           // the same, as a value
ခ ၏ လက်ကျန် ကို ၀ ဟု ထားမည်။
က ၏ လက်ကျန် ကို ပြသမည်။                      // က has not changed
က == ခ ကို ပြသမည်။
က == ဂ ကို ပြသမည်။
```

```text
၁၀၀
မမှန်
မှန်
```

| Zaka | Reads as |
| --- | --- |
| `ကိန်းရှင် ခ ထဲသို့` | "into a new variable ခ" |
| `က ကို ကူးထည့်မည်` | "will duplicate က" |
| `က ၏ မိတ္တူ` | "the duplicate of က" |

## Shallow and deep duplicates

```text
<new name> ထဲသို့ <object> ကို အကုန်ကူးထည့်မည်။
<object> တစ်ခုလုံး ၏ မိတ္တူ
```

The duplicate above is *shallow* (shallow copy): when a field contains another object, the
new object still shares that same object. A *deep* duplicate (deep copy) also duplicates
each object that it reaches, through fields, cells and structs at each depth. We write
`က တစ်ခုလုံး ၏ မိတ္တူ` ("the duplicate of the whole of က") as a value, or
`အကုန်ကူးထည့်မည်` ("will duplicate all into") as a sentence.

In the program below, a node can have a next node, so the field `နောက်` (next) is an
[optional value](../optionals/README.md), and the last node has `အလွတ်` there:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် ဒုတိယ သည် အဆစ်(တန်: ၂၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် အဆစ်(တန်: ၁၊ နောက်: ဒုတိယ) ဖြစ်ပါစေ။
ကိန်းရှင် တိမ် သည် ပထမ ၏ မိတ္တူ ဖြစ်ပါစေ။             // a new first node, the same second
ကိန်းရှင် နက် သည် ပထမ တစ်ခုလုံး ၏ မိတ္တူ ဖြစ်ပါစေ။      // a new first and a new second
ကိန်းရှင် နက်၂ ထဲသို့ ပထမ ကို အကုန်ကူးထည့်မည်။        // the same, as a sentence
ဒုတိယ ၏ တန် ကို ၉ ဟု ထားမည်။
တိမ် ၏ နောက် ထဲမှ ဒေတာ ၏ တန် ကို ပြသမည်။           // shares the second node of ပထမ
နက် ၏ နောက် ထဲမှ ဒေတာ ၏ တန် ကို ပြသမည်။            // a separate second node
နက်၂ ၏ နောက် ထဲမှ ဒေတာ ၏ တန် ကို ပြသမည်။
```

```text
၉
၂
၂
```

| Zaka | Duplicates |
| --- | --- |
| `ခ ထဲသို့ က ကို ကူးထည့်မည်`, `က ၏ မိတ္တူ` | the object, but not the objects in its fields |
| `ခ ထဲသို့ က ကို အကုန်ကူးထည့်မည်`, `က တစ်ခုလုံး ၏ မိတ္တူ` | the object and each object it reaches |

## An object that can be missing

A variable of a shared object has an object each time we read it, because Zaka has no
NULL object (null reference). When we declare it without a value, we must give it an
object before the first read, the same as each other variable:

```zaka-error ZK0722
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် က သည် အကောင့် တစ်ခု ဖြစ်ပါစေ။
က ၏ လက်ကျန် ကို ပြသမည်။
```

```text
error[ZK0722]: "က" can be unset here: a path reaches this read with no value given to it
```

`ကနဦးတန်ဖိုး` for an `အကောင့်` is a new account with each field at its initial value
(default value). When an object can be missing, such as the node after the last one, we
use an optional value for it, and we test the optional before we read it.

A field whose data kind is the struct itself, with no optional, would never end,
because each node would need a next node, and that one a next node again. No program could
build such a value, so Zaka refuses the declaration itself:

```zaka-error ZK0481
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် နောက် ဟူသော အဆစ် တစ်ခု တို့ပါဝင်သည်။
```

```text
error[ZK0481]: no value of "အဆစ်" can be built: အဆစ် → အဆစ်
```

The fix is to make `နောက်` an `အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, or a list of nodes,
which can have no cells.

## Objects that point at each other

A field of an object can point at an object of the same data kind, so in a tree, each
child can also point back at its parent. The root has no parent, so `မိဘ` (parent) is an
optional value. When printing reaches an object that it is still in the middle of
printing, it writes `…` in its place:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ ကလေးများ ဟူသော အဆစ် စာရင်း တစ်ခု နှင့် မိဘ ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ကိန်းရှင် အမြစ် သည် အဆစ်(အမည်: "အမြစ်"၊ ကလေးများ: []၊ မိဘ: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ကလေး သည် အဆစ်(အမည်: "ကလေး"၊ ကလေးများ: []၊ မိဘ: အမြစ်) ဖြစ်ပါစေ။
အမြစ် ၏ ကလေးများ ထဲသို့ ကလေး ကို ထည့်မည်။
ကလေး ၏ မိဘ ထဲမှ ဒေတာ ၏ အမည် ကို ပြသမည်။
အမြစ် ကို ပြသမည်။
```

```text
အမြစ်
အဆစ်(အမည်: "အမြစ်"၊ ကလေးများ: [အဆစ်(အမည်: "ကလေး"၊ ကလေးများ: []၊ မိဘ: ဒေတာ(…))]၊ မိဘ: အလွတ်)
```

The program never frees an object itself. Each object keeps a count of the names that
reach it, and Zaka frees the object as soon as its count drops to zero. The root and the
child above point at each other, so their counts never drop to zero on their own, and a
*ring collector* finds such a group once nothing outside reaches it, and
frees it.

## Details

- The test with `တစ်ခုတည်း` works on two objects or two [function
  values](../function-values/README.md), and on no other data kind ([Values and
  objects](../../manual/language/objects.md)).
- In `က တစ်ခုလုံး ၏ မိတ္တူ`, the word after `တစ်` is the [measure
  word](README.md#measure-words) (classifier) of the data kind, so a data kind counted
  with `ယောက်` uses `က တစ်ယောက်လုံး ၏ မိတ္တူ`.
- A deep duplicate duplicates each object once, so two fields that pointed at one object
  point at one new object, and a ring of objects comes out as a ring. Function values are
  shared, not duplicated. On a plain value, both duplicates do the same as `ဟု ထားမည်`
  ([Duplicating an object](../../manual/language/objects.md#duplicating-an-object)).
- `==` and the duplicates follow rings of objects too, and each one ends.
- An object that the ring collector frees goes when the collector next runs, a little
  later than an object that its count frees. A program can also steer the collector
  ([Rings on demand](../forms/costs.md#rings-on-demand)).
- The full rules for data kinds that no program can build are in [Data kinds no program can
  build](../../manual/language/objects.md#data-kinds-no-program-can-build).
- For programmers who know other languages: a shared object behaves like an object in Java
  or JavaScript, a shallow duplicate is like a JavaScript spread or a Java `clone()`, and
  Zaka frees objects through reference counting, with a collector for rings.

## Recap

- `မျှဝေနိုင်သော` before `ဒေတာအစုအဝေး` declares a shared object, which an assignment or an
  argument shares instead of duplicating.
- `==` compares contents, while `တစ်ခုတည်း` tests whether two names reach the same object.
- `ကူးထည့်မည်` and `၏ မိတ္တူ` make a shallow duplicate, and `အကုန်ကူးထည့်မည်` and
  `တစ်ခုလုံး ၏ မိတ္တူ` make a deep one.
- An object that can be missing uses an optional value, because Zaka has no NULL object.
- Objects can point at each other, and Zaka frees a ring of them once nothing else reaches it.

## Hands on

- Make `အကောင့်` a plain struct: remove `မျှဝေနိုင်သော`, and run the first program again.
- Pass an object to a function that sets one of its fields, and print the object after
  the call.
- Build a ring of three `အဆစ်` nodes, make a deep duplicate, change a value in the
  duplicate, and print both rings.

Next: [Enumerations and switches](../enums/README.md).
