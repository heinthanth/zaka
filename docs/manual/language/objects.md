---
title: Values and objects
description: Values that are copied and objects that are shared, equal contents and one object, shallow and deep copies, counting, and the ring collector.
---

# Values and objects

Each kind of safe Zaka is a **value** or a **shared object**. A value is copied when it is
assigned, passed or returned, as a number is. A shared object is shared: each name for it
reaches the one object, and a change through one name is seen through the others. Which
of the two a kind is depends on its declaration alone, never on what sits inside it.

The VM frees each block when its last use goes, so safe code never frees a block itself
and holds no address. Pointers and the heap statements live in marked code, in [Unsafe
Zaka](unsafe.md). The reasons for this design are in [Storage,
explained](../concepts/storage.md).

## The rule in one table

| Kind | Assignment, argument, result | Freed |
| --- | --- | --- |
| numbers, characters, true/false | copied | nothing to free |
| text, lists, structs, enumerations, results | copied, and the cells are shared until one side writes | when the last duplicate goes |
| a shared object (`မျှဝေနိုင်သော ဒေတာအစုအဝေး`) | shared: the same object | when the last name for it goes, or when the ring collector runs |
| a function value, a closure | shared | when the last name for it goes, or when the ring collector runs |
| a pointer (`<kind> လိပ်စာ`) | the address is copied | in marked code, with a heap statement |

## Values are copied

Text, lists, structs, enumerations and results are values. Assigning one, passing it to a
function or returning it gives the other side a value of its own: a later write on one
side never reaches the other.

The duplicate is cheap. The new value shares the cells of the old one until one side writes;
the write then copies the cells first (*copied on write*). A value that is never written
after the duplicate is never copied at all.

```zaka
လုပ်ဆောင်ချက် ပြင် သည် စ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ၏ အကွက် ၀ ကို ၇ ဟု ထားမည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။               // shares the cells; nothing is copied
ခ ၏ အကွက် ၀ ကို ၉ ဟု ထားမည်။               // this write copies the cells first
က ကို ပြသမည်။
ခ ကို ပြသမည်။
လုပ်ဆောင်ချက် ပြင် ကို က တို့ဖြင့် လုပ်ဆောင်မည်။   // ပြင် changes its own duplicate
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၉၊ ၂၊ ၃]
[၇၊ ၂၊ ၃]
[၁၊ ၂၊ ၃]
```

A function that must change the caller's variable marks the parameter with
`ဒေတာ ပြောင်းလဲရန်` ([Functions](functions.md)). A range of a list or of text is a value too: it
gives a list or text of its own ([Lists](lists.md), [Text](text.md)).

A struct is copied in each case, whatever its fields hold. A field that holds a shared object
holds a reference to it, so duplicating the struct copies the reference, and both structs reach
the one object:

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု၊ y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ တန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
အထုပ် ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ ဘ ဟူသော ဘူး တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မ သည် အမှတ်(x: ၁၊ y: ၂) ဖြစ်ပါစေ။
ကိန်းရှင် န သည် မ ဖြစ်ပါစေ။
န ၏ x ကို ၉ ဟု ထားမည်။
မ ကို ပြသမည်။
ကိန်းရှင် ဘူး၁ သည် ဘူး(အမည်: "ဘူး"၊ တန်: ၁) ဖြစ်ပါစေ။
ကိန်းရှင် ထ သည် အထုပ်(အမည်: "a"၊ ဘ: ဘူး၁) ဖြစ်ပါစေ။
ကိန်းရှင် ဒ သည် ထ ဖြစ်ပါစေ။
ဒ ၏ အမည် ကို "b" ဟု ထားမည်။         // the duplicate's own text
ဒ ၏ ဘ ၏ တန် ကို ၅ ဟု ထားမည်။         // the one shared ဘူး
ထ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှတ်(x: ၁၊ y: ၂)
အထုပ်(အမည်: "a"၊ ဘ: ဘူး(အမည်: "ဘူး"၊ တန်: ၅))
```

## Shared objects

`မျှဝေနိုင်သော` ("that can be shared") before `ဒေတာအစုအဝေး` declares a shared object kind:
`<name> ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ... တို့ပါဝင်သည်။`. The literal, the fields,
the classifier and printing are those of a struct ([Structs](structs.md)); the declaration
alone differs. `အများသုံး` goes after `ဟူသော`, before `မျှဝေနိုင်သော`, to export the kind.

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် အပ်ငွေ သည် အ ဟူသော အကောင့် တစ်ခု ကို လည်းကောင်း ပမာဏ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အ ၏ လက်ကျန် ကို (အ ၏ လက်ကျန်) + ပမာဏ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။                               // one account, two names
လုပ်ဆောင်ချက် အပ်ငွေ ကို ခ၊ ၅၀ တို့ဖြင့် လုပ်ဆောင်မည်။
က ၏ လက်ကျန် ကို ပြသမည်။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅၀
အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၅၀)
```

The rules:

- A variable, field or cell of an object kind holds a reference to one object, one word
  wide. Assigning, passing and returning share the object; nothing is copied.
- A write to a field of an object changes the object itself, never a duplicate, so each name
  for it sees the write.
- An object kind can name itself in its fields, at a depth of one or more, as a node names
  the next node or its parent. Each such loop needs a field that can end the chain, an
  optional or a plain list ([Kinds no program can build](#kinds-no-program-can-build)). A
  tree with parent links holds its children in a plain list, and its parent in an
  optional, since the root has none:

```zaka-file
// tree.zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု၊ ကလေးများ ဟူသော အဆစ် စာရင်း တစ်ခု၊ မိဘ ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။
```

## An object that can be missing

A variable, a field or a cell of an object kind holds an object at each point: no NULL stands in
for one. A declaration with no value leaves the variable unset, and a read of it before
each path gives it a value is ZK0722, at each `--mem-check` level ([Set before
read](statements.md#set-before-read)). `ကနဦးတန်ဖိုး` gives a new object with each field at
its own [initial value](kinds.md#initial-values).

```zaka
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ဘူး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။     // a new object
ကိန်းရှင် ဂ သည် ဘူး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။     // another new object
ခ ကို ပြသမည်။
ခ နှင့် ဂ သည် တစ်ခုတည်း ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ဘူး(တန်: ၀)
မမှန်
```

```zaka-error ZK0722
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ဘူး တစ်ခု ဖြစ်ပါစေ။
ခ ၏ တန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0722]: "ခ" can be unset here: a path reaches this read with no value given to it
```

An object that is there at times and not at others is an
[optional](optionals.md) of the kind, `ဘူး ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`: `အလွတ်` when no
object is there, and the program tests it before it reads the object, as for each optional.

```zaka
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ဘူး ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အလွတ် ဖြစ်ပါစေ။
က ကို ပြသမည်။
က ကို ဘူး(တန်: ၅) ဟု ထားမည်။
အကယ်၍ က ထဲတွင် ဒေတာ ရှိလျှင် က ထဲမှ ဒေတာ ၏ တန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အလွတ်
၅
```

## Kinds no program can build

With no NULL, a field of an object kind needs an object to hold. A kind whose fields lead
back to itself, at once or through other kinds, with no field along the loop that can be
blank, has no first value: each value of it needs one of it to exist first. The compiler refuses
such a kind at its declaration with ZK0481.

- A field leads to its kind, unless it is an optional or a plain list: both can be blank
  (`အလွတ်`, `[]`), so neither needs a value of the kind.
- The compiler marks the kinds that can be built: numbers, text, enumerations, pointers,
  function values and handles first, then each kind whose fields that cannot be blank
  all have marked kinds. A kind left unmarked can never be built.
- One error names each loop, the shortest one, as `က → ခ → ဂ → က`. A kind that holds a kind
  of a broken loop gets no error of its own: fixing the loop fixes it.
- The fix is to make one field of the loop optional, or a plain list.
- A plain struct that holds itself is ZK0427 in place of ZK0481: it would also have no end
  in size ([Structs](structs.md#declaring-a-struct)).

```zaka-error ZK0481
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု၊ နောက် ဟူသော အဆစ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0481]: no value of "အဆစ်" can be built: အဆစ် → အဆစ်
```

With `နောက်` optional, the last node holds `အလွတ်`, and the chain ends:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု၊ နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဒုတိယ သည် အဆစ်(တန်: ၂၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် အဆစ်(တန်: ၁၊ နောက်: ဒုတိယ) ဖြစ်ပါစေ။
ပထမ ကို ပြသမည်။
ကိန်းရှင် ဗ သည် အဆစ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။      // the walk stops at the blank optional
ဗ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အဆစ်(တန်: ၁၊ နောက်: ဒေတာ(အဆစ်(တန်: ၂၊ နောက်: အလွတ်)))
အဆစ်(တန်: ၀၊ နောက်: အလွတ်)
```

## Equal contents: `==`

`==` compares contents for each kind, objects included. Two objects are equal when their
fields are equal, so two accounts with the same name and balance are equal, and a duplicate
equals its original. The rules:

- Fields are compared in depth, shared objects inside included.
- When both sides are the same object, the answer is "equal" at once.
- Objects that reach each other in a ring do not stop `==` from ending: a pair still being
  compared further up counts as equal.
- Function values inside an object compare as the same function or closure.

Text, lists and structs compare their contents in the same manner: `"a" == "a"` and `[၁] == [၁]`
are true.

## One object: `တစ်ခုတည်း`

`X နှင့် Y သည် တစ်ခုတည်း` ("are one and the same") asks whether two names reach one object.
It is a comparison: it gives a true/false value, and `ဖြစ်လျှင်` or `မဟုတ်လျှင်` makes a
condition of it. It takes two objects of one kind, or two function values.

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် က ဖြစ်ပါစေ။
က == ခ ကို ပြသမည်။                                        // the same contents
က နှင့် ခ သည် တစ်ခုတည်း ကို ပြသမည်။                        // two accounts
အကယ်၍ က နှင့် ဂ သည် တစ်ခုတည်း ဖြစ်လျှင် "ဂ" ဟု ပြသမည်။       // one account
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
ဂ
```

Two values of another kind are no objects, and the test refuses them (ZK0404):

```zaka-error ZK0404
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် [၁] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် [၁] ဖြစ်ပါစေ။
အကယ်၍ က နှင့် ခ သည် တစ်ခုတည်း ဖြစ်လျှင် "ခ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0404]: cannot combine isize[] and isize[] without a conversion
```

## Copies of an object

A shared object is copied when the program asks for it, and at no other time. Two
depths exist, each as a statement and as an expression:

| | Statement | Expression |
| --- | --- | --- |
| shallow | `ကိန်းရှင် ခ ထဲသို့ က ကို ကူးထည့်မည်။` | `က ၏ မိတ္တူ` |
| deep | `ကိန်းရှင် ခ ထဲသို့ က ကို အကုန်ကူးထည့်မည်။` | `က တစ်<classifier>လုံး ၏ မိတ္တူ` |

The expression forms stand wherever a value goes: a declaration, an argument, a result, a
field or a cell.

- A **shallow** duplicate (`ကူးထည့်မည်`, `၏ မိတ္တူ`) makes a new object whose fields are those
  of the old one. A value field is copied as ever; a field that holds another object
  still reaches that same object.
- A **deep** duplicate (`အကုန်ကူးထည့်မည်`, `တစ်<classifier>လုံး ၏ မိတ္တူ`) also copies each
  object it reaches, through fields, list cells and struct fields, at each depth. Each
  object is copied once: two fields that reached one object reach one new object, and a
  ring is copied as a ring, so the duplicate has the shape of the original.
- Function values and closures are shared in both copies, never copied: duplicating a
  closure's variables would split state that the program shares on purpose.
- On a value with no object inside, a duplicate is the same as an assignment.

The measure word in the deep form is the classifier of the kind, as in `လူ တစ်ယောက်`:
`က တစ်ကောင်လုံး ၏ မိတ္တူ` for a kind counted with `ကောင်`, and `ခု` for the rest.
Another word is the warning ZK0509. The spaces are optional: `တစ်ခုလုံး၏`
and `အကုန် ကူးထည့်မည်` read the same as the spaced forms.

```zaka-warning ZK0509
လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လူ(အသက်: ၂၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က တစ်ခုလုံး ၏ မိတ္တူ ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0509]: လူ is counted with "ယောက်", not "ခု"
```

```zaka
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
အိတ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု၊ အတွင်း ဟူသော ဘူး တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘ သည် ဘူး(တန်: ၂) ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် အိတ်(တန်: ၁၊ အတွင်း: ဘ) ဖြစ်ပါစေ။
ကိန်းရှင် တိမ် သည် ပထမ ၏ မိတ္တူ ဖြစ်ပါစေ။          // a new အိတ်, the same ဘူး
ကိန်းရှင် နက် သည် ပထမ တစ်ခုလုံး ၏ မိတ္တူ ဖြစ်ပါစေ။   // a new အိတ် and a new ဘူး
ကိန်းရှင် နက်၂ ထဲသို့ ပထမ ကို အကုန်ကူးထည့်မည်။      // the same, as a statement
ဘ ၏ တန် ကို ၉ ဟု ထားမည်။
တိမ် ၏ အတွင်း ၏ တန် ကို ပြသမည်။
နက် ၏ အတွင်း ၏ တန် ကို ပြသမည်။
နက်၂ ၏ အတွင်း ၏ တန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၉
၂
၂
```

A shallow duplicate is an object of its own, so a write to it leaves the original as it was:

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ ထဲသို့ က ကို ကူးထည့်မည်။          // a new account with the same fields
ကိန်းရှင် ဂ သည် က ၏ မိတ္တူ ဖြစ်ပါစေ။           // the same, as an expression
ခ ၏ လက်ကျန် ကို ၀ ဟု ထားမည်။
က ၏ လက်ကျန် ကို ပြသမည်။
က == ခ ကို ပြသမည်။
က == ဂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၀၀
မမှန်
မှန်
```

## When a block is freed

Each block of a value, an object or a closure carries a count of the references to it. A
duplicate of a reference adds one; a reference that goes (a variable at the end of its block, a
field or cell written over, a value made for one step, after that step) removes one. The block is
freed the moment its count reaches zero, together with the references it holds. Freeing
happens at a known point in the program, on the VM and in native code alike.

The compiler leaves out most of the counting: a value moved at its last use needs no new
reference, and a value that is never shared needs no count change.

### Rings

Counting alone never frees objects that reach each other in a ring, since each holds a
reference to the next. The VM and the native runtime also run a **ring collector**:

- A block whose count drops but not to zero waits in a list, when its kind can hold an
  object or a closure. Text, lists of numbers and structs of numbers never wait.
- When 1,024 blocks wait, and again when the program ends, the collector checks whether
  the waiting blocks are kept alive through each other alone, and frees each such group.
- An object outside a ring is still freed the moment its last reference goes; an object
  inside a ring is freed when the collector next runs.

So safe code cannot leave a block behind: `zakac run --leaks` lists nothing for a program
with no marked code.

### Printing a ring

Printing shows the fields of an object, and the objects inside it. An object still being
printed further up shows as `…`, so printing a ring ends:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု၊ နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဒုတိယ သည် အဆစ်(တန်: ၂၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် အဆစ်(တန်: ၁၊ နောက်: ဒုတိယ) ဖြစ်ပါစေ။
ဒုတိယ ၏ နောက် ကို ပထမ ဟု ထားမည်။      // a ring of two nodes
ပထမ ကို ပြသမည်။
ကိန်းရှင် ပွား သည် ပထမ တစ်ခုလုံး ၏ မိတ္တူ ဖြစ်ပါစေ။
ပွား == ပထမ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အဆစ်(တန်: ၁၊ နောက်: ဒေတာ(အဆစ်(တန်: ၂၊ နောက်: ဒေတာ(…))))
မှန်
```

## Errors at a glance

| Code | When |
| --- | --- |
| ZK0404 | `တစ်ခုတည်း` on two values that are not objects or function values |
| ZK0481 | a kind whose fields lead back to it with no optional or plain list on the loop |
| ZK0722 | a read of an object variable that a path reaches with no value given |
