---
title: Shared objects
description: A struct declared မျှဝေနိုင်သော ဒေတာအစုအဝေး is one object that each name shares, as in Java or JavaScript; compare it, test whether two names hold one object, and duplicate it.
---

# Shared objects

Text, lists and structs are values: each assignment and each argument gives a duplicate of its
own, as for a number. The duplicate is cheap, since the two share their cells until one of them
writes. Write `မျှဝေနိုင်သော` ("that can be shared") before `ဒေတာအစုအဝေး`, and each value
of the kind is a *shared object* instead, as in Java or JavaScript: an assignment or an
argument shares the one object, and a change through one name shows through each name.

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

// deposit: add to the balance of the account the caller gives
လုပ်ဆောင်ချက် အပ်ငွေ သည် အ ဟူသော အကောင့် တစ်ခု ကို လည်းကောင်း ပမာဏ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အ ၏ လက်ကျန် ကို (အ ၏ လက်ကျန်) + ပမာဏ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။                        // one account, two names
လုပ်ဆောင်ချက် အပ်ငွေ ကို ခ၊ ၅၀ တို့ဖြင့် လုပ်ဆောင်မည်။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၅၀)
```

The literal, the fields and the printed form are those of a struct; the declaration alone
differs. Had `အကောင့်` been a plain struct, `ခ` and the parameter `အ` would each hold a duplicate,
and `က` would still print `၁၀၀`.

| Kind | `=`, an argument, a result |
| --- | --- |
| number, character, true/false | a duplicate |
| text, list, struct | a duplicate, made cheap: the cells are copied at the first write |
| shared object | the same object |

## Equal or the same

`==` compares contents for each kind, objects included: two accounts with equal fields
are equal. To ask whether two names hold one and the same object, write
`က နှင့် ခ သည် တစ်ခုတည်း ဖြစ်လျှင်` ("if က and ခ are one and the same"):

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် က ဖြစ်ပါစေ။
က == ခ ကို ပြသမည်။                                       // the same contents
အကယ်၍ က နှင့် ခ သည် တစ်ခုတည်း ဖြစ်လျှင် "ခ" ဟု ပြသမည်။      // two accounts: not printed
အကယ်၍ က နှင့် ဂ သည် တစ်ခုတည်း ဖြစ်လျှင် "ဂ" ဟု ပြသမည်။      // one account
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
ဂ
```

The test of one and the same takes two objects, or two
[function values](../function-values/README.md).

## A duplicate of its own

`ကူးထည့်မည်` ("will duplicate in") on an object makes a new object with the same fields. The
expression `က ၏ မိတ္တူ` ("the duplicate of က") does the same wherever a value goes:

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ ထဲသို့ က ကို ကူးထည့်မည်။          // a new account with the same fields
ကိန်းရှင် ဂ သည် က ၏ မိတ္တူ ဖြစ်ပါစေ။           // the same, as an expression
ခ ၏ လက်ကျန် ကို ၀ ဟု ထားမည်။
က ၏ လက်ကျန် ကို ပြသမည်။                      // က stands as it was
က == ခ ကို ပြသမည်။
က == ဂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၀
မမှန်
မှန်
```

This duplicate is *shallow*, like a JavaScript spread or a Java `clone()`: a field that holds
another object still holds that same object. A *deep* duplicate copies each object it reaches
as well, through fields, cells and structs at each depth. Its expression is
`က တစ်ခုလုံး ၏ မိတ္တူ` ("a duplicate of the whole of က"), and its statement is
`အကုန်ကူးထည့်မည်` ("will duplicate all in"). In the program below, each node can have a
next node, so `နောက်` is an [optional](../optionals/README.md), and the last node holds
`အလွတ်` there; [An object that can be missing](#an-object-that-can-be-missing) gives the reason:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု၊ နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဒုတိယ သည် အဆစ်(တန်: ၂၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် အဆစ်(တန်: ၁၊ နောက်: ဒုတိယ) ဖြစ်ပါစေ။
ကိန်းရှင် တိမ် သည် ပထမ ၏ မိတ္တူ ဖြစ်ပါစေ။             // a new first node, the same second
ကိန်းရှင် နက် သည် ပထမ တစ်ခုလုံး ၏ မိတ္တူ ဖြစ်ပါစေ။      // a new first and a new second
ကိန်းရှင် နက်၂ ထဲသို့ ပထမ ကို အကုန်ကူးထည့်မည်။        // the same, as a statement
ဒုတိယ ၏ တန် ကို ၉ ဟု ထားမည်။
တိမ် ၏ နောက် ထဲမှ ဒေတာ ၏ တန် ကို ပြသမည်။           // shares ပထမ's second node
နက် ၏ နောက် ထဲမှ ဒေတာ ၏ တန် ကို ပြသမည်။            // a node of its own
နက်၂ ၏ နောက် ထဲမှ ဒေတာ ၏ တန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉
၂
၂
```

| Zaka | Copies |
| --- | --- |
| `ခ ထဲသို့ က ကို ကူးထည့်မည်`, `က ၏ မိတ္တူ` | the object, and not the objects in its fields |
| `ခ ထဲသို့ က ကို အကုန်ကူးထည့်မည်`, `က တစ်ခုလုံး ၏ မိတ္တူ` | the object and each object it reaches |

The word after `တစ်` is the [count word](README.md#count-words) of the kind, as in
`လူ တစ်ယောက်`: `က တစ်ယောက်လုံး ၏ မိတ္တူ` for a kind counted with `ယောက်`. A deep duplicate copies
each object once, so two fields that held one object hold one new object, and a ring of
objects comes out as a ring. Function values are shared, not copied. On a plain value,
both copies are the same as `ဟု ထားမည်`.

## An object that can be missing

An object variable holds an object on each read; Zaka has no NULL object. Declared with no
value, it gets an object before its first read, as each variable does:

```zaka-error ZK0722
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့် တစ်ခု ဖြစ်ပါစေ။
က ၏ လက်ကျန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0722]: "က" can be unset here: a path reaches this read with no value given to it
```

`ကနဦးတန်ဖိုး` for an `အကောင့်` is a new account with each field at its initial value. Where an
object can be missing, as the node after the last one, the kind states it with an
optional, and a read asks first. A field that holds its own kind with no optional
would have no end: each node would need a next node, and that one a next node in turn.
No program could build such a kind, so the declaration itself is refused:

```zaka-error ZK0481
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု၊ နောက် ဟူသော အဆစ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0481]: no value of "အဆစ်" can be built: အဆစ် → အဆစ်
```

Make `နောက်` an `အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, or a list of nodes, which can hold
no nodes. See [Kinds no program can
build](../../manual/language/objects.md#kinds-no-program-can-build).

## Objects that point at each other

A field of an object can hold an object of the same kind, so a tree can link each child
to its parent as well. The root has no parent, so `မိဘ` is an optional. Printing shows an object it is still in the middle of printing as `…`:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ ကလေးများ ဟူသော အဆစ် စာရင်း တစ်ခု၊ မိဘ ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမြစ် သည် အဆစ်(အမည်: "အမြစ်"၊ ကလေးများ: []၊ မိဘ: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ကလေး သည် အဆစ်(အမည်: "ကလေး"၊ ကလေးများ: []၊ မိဘ: အမြစ်) ဖြစ်ပါစေ။
အမြစ် ၏ ကလေးများ ထဲသို့ ကလေး ကို ထည့်မည်။
ကလေး ၏ မိဘ ထဲမှ ဒေတာ ၏ အမည် ကို ပြသမည်။
အမြစ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမြစ်
အဆစ်(အမည်: "အမြစ်"၊ ကလေးများ: [အဆစ်(အမည်: "ကလေး"၊ ကလေးများ: []၊ မိဘ: ဒေတာ(…))]၊ မိဘ: အလွတ်)
```

The program frees nothing itself. Each object keeps a count of the names that reach it, and goes
the moment its count drops to zero. The root and the child above reach each other, so
their counts never drop to zero alone; a *ring collector* finds such a group once
nothing outside reaches it, and frees it. `==` and the copies follow these rings too, and
each ends. An object that the collector frees goes when the collector next runs, a little
later than one that its count frees.

## Hands on

- Make `အကောင့်` a plain struct: remove `မျှဝေနိုင်သော`, and run the first program again.
- Give an object to a function that sets one of its fields, and print the object after
  the call.
- Build a ring of three `အဆစ်` nodes, make a deep duplicate, change a value in the duplicate, and
  print both rings.

Next: [Enumerations and switches](../enums/README.md).
