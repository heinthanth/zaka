---
title: Values and objects
description: Values that are duplicated and objects that are shared, equal contents and one object, shallow and deep duplicates, reference counting, and the ring collector.
---

# Values and objects

In safe Zaka, assignment, argument passing and returns duplicate (copy) values or share
objects. A shared object (reference type) can have several names that refer to the same
object, so a change through one name is visible through the others. The declaration of
the data kind (type) determines which rule applies.

The VM frees each block when its last use is gone, so safe code never frees a block itself
and never sees an address. Pointers and the heap statements belong to unsafe code, in
[Unsafe Zaka](unsafe.md), and [Storage, explained](../concepts/storage.md) gives the
reasons for this design. The Learn book teaches shared objects in small steps in [Shared
objects](../../learn/structs/objects.md).

## The rule in one table

| Data kind | Assignment, argument, result | Freed |
| --- | --- | --- |
| numbers, characters, true/false values | duplicated | nothing to free |
| text, lists, structs, enumerations, results | duplicated, and the cells are shared until one side writes | when the last duplicate goes |
| a shared object (`မျှဝေနိုင်သော ဒေတာအစုအဝေး`) | shared: the same object | when the last name for it goes, or when the ring collector runs |
| a function value, a closure | shared | when the last name for it goes, or when the ring collector runs |
| a pointer (`<kind> လိပ်စာ`) | the address is duplicated | in unsafe code, with a heap statement |

## Values are duplicated

Text, lists, structs, enumerations and results are values. When we assign one, pass it to a
function or return it, the other side gets a separate value, so a later write on one side
never reaches the other.

The duplicate is cheap, because the new value shares the cells of the old one until one
side writes, and that write then duplicates the cells first (copy-on-write). When neither
side writes, the cells are never duplicated at all.

```zaka
ပြင် ဟူသော လုပ်ဆောင်ချက် သည်
စ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ၏ အကွက် ၀ ကို ၇ ဟု ထားမည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။ // shares the cells; nothing is duplicated
ခ ၏ အကွက် ၀ ကို ၉ ဟု ထားမည်။ // this write duplicates the cells first
က ကို ပြသမည်။
ခ ကို ပြသမည်။
ပြင် ဟူသော လုပ်ဆောင်ချက်ကို က ဖြင့် လုပ်ဆောင်မည်။ // ပြင် changes its own duplicate
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
[၁၊ ၂၊ ၃]
[၉၊ ၂၊ ၃]
[၇၊ ၂၊ ၃]
[၁၊ ၂၊ ၃]
```

To let a function change the caller's variable, we mark the parameter with
`ဒေတာ ပြောင်းလဲရန်`, which makes it a mutable parameter (`inout`)
([Mutable parameters](functions-parameters.md#mutable-parameters)). A range of a list or of text is a value too, and it gives a
new list or text ([Lists](lists.md), [Text](text.md)).

A struct is duplicated in all cases, whatever its fields contain. A field that keeps a
shared object keeps a reference to it, so duplicating the struct duplicates the reference,
and both structs reach the same object:

```zaka
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
အမည် ဟူသော စာသား တစ်ခု နှင့်
တန် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။
အထုပ် ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု နှင့် ဘ ဟူသော ဘူး တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် မ သည် အမှတ်(x: ၁၊ y: ၂) ဖြစ်ပါစေ။
ကိန်းရှင် န သည် မ ဖြစ်ပါစေ။
န ၏ x ကို ၉ ဟု ထားမည်။
မ ကို ပြသမည်။
ကိန်းရှင် ဘူး၁ သည် ဘူး(အမည်: "ဘူး"၊ တန်: ၁) ဖြစ်ပါစေ။
ကိန်းရှင် ထ သည် အထုပ်(အမည်: "a"၊ ဘ: ဘူး၁) ဖြစ်ပါစေ။
ကိန်းရှင် ဒ သည် ထ ဖြစ်ပါစေ။
ဒ ၏ အမည် ကို "b" ဟု ထားမည်။ // the duplicate's own text
ဒ ၏ ဘ ၏ တန် ကို ၅ ဟု ထားမည်။ // the one shared ဘူး
ထ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အမှတ်(x: ၁၊ y: ၂)
အထုပ်(အမည်: "a"၊ ဘ: ဘူး(အမည်: "ဘူး"၊ တန်: ၅))
```

## Shared objects

```text
<name> ဟူသော [အများသုံးနိုင်သော] မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် <fields> တို့ပါဝင်သည်။
```

To declare a shared object, we put `မျှဝေနိုင်သော` ("that can be shared") before
`ဒေတာအစုအဝေး`. The literal, the fields, the measure word and printing all work as for a
struct ([Structs](structs.md)), and the declaration is the one difference. To export the
data kind, we put `အများသုံးနိုင်သော` after `ဟူသော`, before `မျှဝေနိုင်သော`.

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
အမည် ဟူသော စာသား တစ်ခု နှင့်
လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။

အပ်ငွေ ဟူသော လုပ်ဆောင်ချက် သည်
အ ဟူသော အကောင့် တစ်ခု ကို လည်းကောင်း
ပမာဏ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အ ၏ လက်ကျန် ကို (အ ၏ လက်ကျန်) + ပမာဏ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။ // one account, two names
အပ်ငွေ ဟူသော လုပ်ဆောင်ချက်ကို ခ နှင့် ၅၀ တို့ဖြင့် လုပ်ဆောင်မည်။
က ၏ လက်ကျန် ကို ပြသမည်။
က ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၅၀
အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၅၀)
```

A variable, a field or a cell of a shared object keeps a reference to one object, which is
one word wide, so assigning, passing and returning share the object and nothing is
duplicated. A write to a field changes the object itself, never a duplicate, so each name
for the object sees the write.

A shared object can name its own data kind in its fields, at a depth of one or more, as
when a node names the next node or its parent. Each such loop needs a field that can end
the chain, which is an optional or a plain list ([Data kinds no program can
build](#data-kinds-no-program-can-build)). A tree with links to the parent keeps its
children in a plain list and its parent in an optional, because the root has no parent:

```zaka-file
// tree.zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု၊ ကလေးများ ဟူသော အဆစ် စာရင်း တစ်ခု နှင့် မိဘ ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။
```

## Objects threads can use

```text
<name> ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော၊ မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် <fields> တို့ပါဝင်သည်။
```

To make objects that threads can use, we put the mark
`အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` before `မျှဝေနိုင်သော`, and a lock then guards
each object. We can write the words apart or glued
(`အပြိုင်လုပ်ငန်းများတွင်အသုံးပြုနိုင်သော`), and none of them is a reserved word.

```zaka-file
// account.zaka
အကောင့် ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော၊ မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
```

- The same phrase names the built-in trait of the data kinds that threads can use
  ([Built-in traits](traits.md#built-in-traits)). An object of a struct with the mark has
  this trait when threads can use each of its fields, and a plain shared object does not
  have it.
- A value needs no mark, because a thread that gets a plain struct, a list or text gets a
  duplicate as soon as either side writes to it. The mark on a plain struct changes nothing
  and gives the warning ZK0493.
- We read and write each field of an object with the mark inside a lock block of that
  object, `<obj> ကို ခေတ္တယူ၍ <block>`, and a field read or written outside one is ZK0499
  ([Objects threads share](threads.md#objects-threads-share)).
- The mark goes with a shared object alone: on an enumeration, or together with a C name
  or `ပြင်ပမှ လာသော`, it is ZK0263.

```zaka-warning ZK0493
အမှတ် ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော ဒေတာအစုအဝေးတွင်
x ဟူသော ကိန်းပြည့် တစ်ခု
ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အမှတ်(x: ၁) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

## An object that can be missing

A variable, a field or a cell of a shared object has an object at each point, because no
NULL can take the place of one. A declaration with no value leaves the variable unset, and
reading it before each path gives it a value is ZK0722, at each level of the `mem` lint
([Set before read](declarations.md#set-before-read)). `ကနဦးတန်ဖိုး` creates a new object
with each field at its [initial value](kinds.md#initial-values) (default value).

```zaka
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ဘူး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // a new object
ကိန်းရှင် ဂ သည် ဘူး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // another new object
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
error[ZK0722]: this read can happen before "ခ" is set
```

For an object that is there at some times and missing at others, we use an
[optional](optionals.md) of the data kind, `ဘူး ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`. It is
`အလွတ်` when no object is there, and we test it before we read the object, as with each
optional.

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

## Data kinds no program can build

Since there is no NULL, a field of a shared object needs an object to point to. When the
fields of a data kind lead back to the data kind itself, straight or through other data
kinds, and no field along the loop can be blank, the data kind has no first value, because
each value of it needs another one to exist first. The compiler rejects such a data kind at
its declaration with ZK0481.

- A field leads to its data kind unless it is an optional or a plain list. Both can be
  blank (`အလွတ်`, `[]`), so neither needs a value of the data kind.
- To find these data kinds, the compiler first marks the ones that can be built: numbers,
  text, enumerations, pointers, function values and handles. It then marks each data kind
  whose fields that cannot be blank all have marked data kinds, and a data kind left
  unmarked at the end can never be built.
- One error names each loop, the shortest one, as `က → ခ → ဂ → က`. A data kind that
  contains a data kind of a broken loop gets no separate error, because fixing the loop
  fixes it too.
- To fix the loop, we make one of its fields optional or a plain list.
- A plain struct that contains itself is ZK0427 instead of ZK0481, because its size would
  also have no end ([Structs](structs.md#declaring-a-struct)).

```zaka-error ZK0481
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် နောက် ဟူသော အဆစ် တစ်ခု တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0481]: no value of "အဆစ်" can be built: အဆစ် → အဆစ်
```

When `နောက်` is optional, the last node has `အလွတ်` there, and the chain ends:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့်
နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဒုတိယ သည် အဆစ်(တန်: ၂၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် အဆစ်(တန်: ၁၊ နောက်: ဒုတိယ) ဖြစ်ပါစေ။
ပထမ ကို ပြသမည်။
ကိန်းရှင် ဗ သည် အဆစ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။ // the walk stops at the blank optional
ဗ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အဆစ်(တန်: ၁၊ နောက်: ဒေတာ(အဆစ်(တန်: ၂၊ နောက်: အလွတ်)))
အဆစ်(တန်: ၀၊ နောက်: အလွတ်)
```

## Equal contents: `==`

```text
<a> == <b>
```

`==` compares the contents for all data kinds, objects included. Two objects are equal
when their fields are equal, so two accounts with the same name and balance are equal, and
a duplicate equals its original.

- Fields are compared in depth, including the shared objects inside them.
- When both sides are the same object, the answer is "equal" at once.
- Objects that reach each other in a ring do not stop `==` from finishing, because a pair
  that is still being compared further up counts as equal.
- Function values inside an object compare as the same function or closure.

Text, lists and structs compare their contents in the same manner, so `"a" == "a"` and
`[၁] == [၁]` are true.

## One object: `တစ်ခုတည်း`

```text
<a> နှင့် <b> သည် တစ်ခုတည်း
```

To ask whether two names reach the same object, we write `X နှင့် Y သည် တစ်ခုတည်း` ("are one
and the same"). It is a comparison that returns a true/false value, and with `ဖြစ်လျှင်` or
`မဟုတ်လျှင်` after it we can use it as a condition. It works on two objects of the same
data kind, or on two function values.

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
အမည် ဟူသော စာသား တစ်ခု နှင့်
လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် က ဖြစ်ပါစေ။
က == ခ ကို ပြသမည်။ // the same contents
က နှင့် ခ သည် တစ်ခုတည်း ကို ပြသမည်။ // two accounts
အကယ်၍ က နှင့် ဂ သည် တစ်ခုတည်း ဖြစ်လျှင် "ဂ" ဟု ပြသမည်။ // one account
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
ဂ
```

Two values of other data kinds are not objects, so the compiler rejects the test with
ZK0404:

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

## Duplicating an object

```text
ကိန်းရှင် <new> ထဲသို့ <object> ကို ကူးထည့်မည်။
ကိန်းရှင် <new> ထဲသို့ <object> ကို အကုန်ကူးထည့်မည်။
<object> ၏ မိတ္တူ
<object> တစ်<word>လုံး ၏ မိတ္တူ
```

A shared object is duplicated when the program asks for it, and at no other time. There
are two depths, and each one has a statement form and an expression form:

| Depth | Statement | Expression |
| --- | --- | --- |
| shallow | `ကိန်းရှင် ခ ထဲသို့ က ကို ကူးထည့်မည်။` | `က ၏ မိတ္တူ` |
| deep | `ကိန်းရှင် ခ ထဲသို့ က ကို အကုန်ကူးထည့်မည်။` | `က တစ်<word>လုံး ၏ မိတ္တူ` |

We can use the expression forms wherever a value is expected: in a declaration, an
argument, a result, a field or a cell.

- A shallow duplicate (copy), with `ကူးထည့်မည်` or `၏ မိတ္တူ`, creates a new object whose
  fields are those of the old one. A value field is duplicated as usual, and a field that
  keeps another object still reaches that same object.
- A deep duplicate (deep copy), with `အကုန်ကူးထည့်မည်` or `တစ်<word>လုံး ၏ မိတ္တူ`, also
  duplicates each object that it reaches, through fields, list cells and struct fields, at
  all depths. Each object is duplicated once: two fields that reached one object reach one
  new object, and a ring is duplicated as a ring, so the duplicate has the shape of the
  original.
- Function values and closures are shared in both forms, never duplicated, because
  duplicating the variables of a closure would split state that the program shares on
  purpose.
- On a value with no object inside, a duplicate is the same as an assignment.

The measure word in the deep form is the measure word (classifier) of the data kind, as in
`လူ တစ်ယောက်`, so we write `က တစ်ကောင်လုံး ၏ မိတ္တူ` for a data kind counted with `ကောင်`,
and `ခု` for the rest. A different word gives the warning ZK0509. The spaces are optional,
so `တစ်ခုလုံး၏` and `အကုန် ကူးထည့်မည်` mean the same as the spaced forms.

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
အိတ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့်
အတွင်း ဟူသော ဘူး တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘ သည် ဘူး(တန်: ၂) ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် အိတ်(တန်: ၁၊ အတွင်း: ဘ) ဖြစ်ပါစေ။
ကိန်းရှင် တိမ် သည် ပထမ ၏ မိတ္တူ ဖြစ်ပါစေ။ // a new အိတ်, the same ဘူး
ကိန်းရှင် နက် သည် ပထမ တစ်ခုလုံး ၏ မိတ္တူ ဖြစ်ပါစေ။ // a new အိတ် and a new ဘူး
ကိန်းရှင် နက်၂ ထဲသို့ ပထမ ကို အကုန်ကူးထည့်မည်။ // the same, as a statement
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

A shallow duplicate is a separate object, so a write to it leaves the original as it was:

```zaka
အကောင့် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
အမည် ဟူသော စာသား တစ်ခု နှင့်
လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကောင့်(အမည်: "မမ"၊ လက်ကျန်: ၁၀၀) ဖြစ်ပါစေ။
ကိန်းရှင် ခ ထဲသို့ က ကို ကူးထည့်မည်။ // a new account with the same fields
ကိန်းရှင် ဂ သည် က ၏ မိတ္တူ ဖြစ်ပါစေ။ // the same, as an expression
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

Each block of a value, an object or a closure has a count of the references to it
(reference counting). A duplicate of a reference adds one, and a reference that ends
removes one: a variable at the end of its block, a field or a cell that is written over,
or a value made for one step, after that step. The block is freed as soon as its count
reaches zero, together with the references inside it. Freeing happens at a known point in
the program, on the VM and in native code alike.

The compiler leaves out most of the counting: a value moved at its last use needs no new
reference, and a value that is never shared needs no change to its count.

### Rings

Counting alone never frees objects that reach each other in a ring, since each one keeps a
reference to the next. For these, the VM and the native runtime also run a ring collector:

- A block whose count drops, but not to zero, waits in a list when it can be part of a
  ring, which means that the blocks in its fields and cells lead back to a block of its
  own data kind, or contain a closure. Text, lists of numbers and structs of numbers never
  wait, and neither does an object whose fields cannot lead back to its data kind, such as
  a server that keeps a logger of text alone.
- When the waiting blocks reach 16,384 octets (bytes), and again when the program ends,
  the collector checks whether the waiting blocks are kept alive through each other alone,
  and frees each such group.
- An object outside a ring is still freed as soon as its last reference goes, while an
  object inside a ring is freed the next time the collector runs.
- While [threads](threads.md) run, a collection halts each running thread at its next
  allocation, at the next round of a loop, or as it wakes from a wait, and it runs once
  all of them have halted. A thread that is inside a call of C for a long time lets that
  collection go, and the next one tries again once the waiting blocks have doubled.

So safe code cannot leave a block behind, and `zakac run --leaks` lists nothing for a
program with no unsafe code.

### Steering the collector

Four statements steer the collector, on the VM and in native code alike. Each one uses the
reserved word `မင်မိုရီ`, while its other words are not reserved, so we can still use each
of them as a variable name.

| Statement | What it does |
| --- | --- |
| `မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်။` | runs the collector now |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ခေတ္တရပ်မည်။` | pauses the runs that a full list starts; blocks still wait, and the end of the program still collects |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ပြန်လည်စတင်မည်။` | resumes the runs, and collects at once when the list filled up during the pause |
| `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို <n> ဘိုက် နှင့်အထက်မှ လုပ်ဆောင်မည်။` | sets the octets of waiting blocks that start a run to `n`, a `usize`, which is 16,384 at first; it collects at once when the list has reached that size and the runs are not paused |

- We can write each word glued or apart: `အရာများကို` or `အရာများ ကို`, `မင်မိုရီရှိ` or
  `မင်မိုရီ ရှိ`.
- A pause suits a part of the program that must not halt for a collection, such as one
  frame of a game, and a run now suits a point where the program has time to spare, such
  as the end of a level.
- A small threshold keeps the heap small and runs the collector often, while a large one
  runs it less often and lets more rings wait.

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့်
နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု
တို့ပါဝင်သည်။

ကွင်းဆောက် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အဆစ်(တန်: ၁၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အဆစ်(တန်: ၂၊ နောက်: က) ဖြစ်ပါစေ။
က ၏ နောက် ကို ခ ဟု ထားမည်။ // a ring of two nodes, left behind
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ခေတ္တရပ်မည်။
၁ မှ ၁၀၀ အထိ န အနေဖြင့် ရေတွက်၍ ကွင်းဆောက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်။ // the 100 rings go now
အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ပြန်လည်စတင်မည်။
အလိုအလျောက် မင်မိုရီ ရှင်းလင်းမှုကို ၁၀၂၄ ဘိုက် နှင့်အထက်မှ လုပ်ဆောင်မည်။
"ပြီး" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
ပြီး
```

One expression reads the state of the collector,
`အလိုအလျောက် မင်မိုရီ ရှင်းလင်းသည့် အခြေအနေ`. It returns a struct of the core module,
`အလိုအလျောက်မင်မိုရီရှင်းလင်းမှုအခြေအနေပြကိန်းဂဏန်းများ`, whose fields contain the numbers at
the time the expression runs:

| Field | Data kind | Contains |
| --- | --- | --- |
| `အကြိမ်အရေအတွက်` | `ကိန်းပြည့်` | the count of collections that have run |
| `ရှင်းလင်းခဲ့သောဘိုက်အရေအတွက်` | `ကိန်းပြည့်` | the octets of the blocks that the collections freed |
| `ရှင်းလင်းရန်စီစဥ်နေသောဘိုက်အရေအတွက်` | `ကိန်းပြည့်` | the octets of the blocks waiting in the list |
| `ရှင်းလင်းမှုလုပ်ဆောင်ရန်လိုအပ်သောဘိုက်အရေအတွက်` | `ကိန်းပြည့်` | the octets of waiting blocks that start a run |
| `ခေတ္တရပ်ထား` | `မှန်မှားတန်ဖိုး` | whether the runs that a full list starts are paused |

A block that is freed while it waits, because its last holder let go, counts in no
collection. The VM and native code count the same octets.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလိုအလျောက် မင်မိုရီ ရှင်းလင်းသည့် အခြေအနေ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အလိုအလျောက်မင်မိုရီရှင်းလင်းမှုအခြေအနေပြကိန်းဂဏန်းများ(အကြိမ်အရေအတွက်: ၀၊ ရှင်းလင်းခဲ့သောဘိုက်အရေအတွက်: ၀၊ ရှင်းလင်းရန်စီစဥ်နေသောဘိုက်အရေအတွက်: ၀၊ ရှင်းလင်းမှုလုပ်ဆောင်ရန်လိုအပ်သောဘိုက်အရေအတွက်: ၁၆၃၈၄၊ ခေတ္တရပ်ထား: မမှန်)
```

### Printing a ring

When we print an object, the output shows its fields and the objects inside it. An object
that is still being printed further up shows as `…`, so printing a ring ends:

```zaka
အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်
တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့်
နောက် ဟူသော အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု
တို့ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဒုတိယ သည် အဆစ်(တန်: ၂၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် အဆစ်(တန်: ၁၊ နောက်: ဒုတိယ) ဖြစ်ပါစေ။
ဒုတိယ ၏ နောက် ကို ပထမ ဟု ထားမည်။ // a ring of two nodes
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
| ZK0481 | a data kind whose fields lead back to it with no optional or plain list on the loop |
| ZK0493 | the mark `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော` on a struct that is not a shared object (warning) |
| ZK0722 | a read of an object variable that a path reaches with no value given |
