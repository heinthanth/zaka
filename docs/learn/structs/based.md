---
title: Based on another struct
description: Base a struct on another, reach the fields and the functions of that part as its own, replace one of its functions, and read the part itself.
---

# Based on another struct

A student is a person with a school, and a teacher a person with a subject. We can base a
struct on another struct, with `<struct> ကို အခြေခံသော` in its head, and the struct then
holds the other as a part (embedded field) whose fields and functions it reaches as its own,
the same as struct embedding in Go. The struct does not become a kind of the other: a
function that takes a `လူ` still takes a `လူ` alone, and a function that takes each data kind
that can greet takes a [trait value](../generics/trait-values.md).

## You will learn

- How to base a struct on another and set the part in a literal.
- How a value reaches the fields and the functions of its part.
- How a struct replaces a function of its part with its own.
- How to read the part itself.

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
လ ဟူသော လူ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာပါ {လ ၏ နာမည်}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကျောင်းသား ဟူသော ဒေတာအစုအဝေး သည်
လူ ကို အခြေခံသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင်
ကျောင်း ဟူသော စာသား တစ်ခု
ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

ကိန်းရှင် က သည် ကျောင်းသား(လူ: လူ(နာမည်: "မောင်")၊ ကျောင်း: "YTU") ဖြစ်ပါစေ။
က ၏ နာမည် ကို ပြသမည်။
က ၏ နှုတ်ဆက်() ကို ပြသမည်။
က ၏ နာမည် ကို "မမ" ဟု ထားမည်။
က ၏ နှုတ်ဆက်() ကို ပြသမည်။
ကိန်းရှင် ပ သည် က ကို လူ အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ပ ၏ နာမည် ကို ပြသမည်။
က ၏ ကျောင်း ကို ပြသမည်။
```

```text
မောင်
မင်္ဂလာပါ မောင်
မင်္ဂလာပါ မမ
မမ
YTU
```

`ကျောင်းသား` (student) is based on `လူ` (person), so a student holds a person as a part. The
literal sets the part under the name of its struct, `လူ: လူ(…)`. Then `က ၏ နာမည်` reads the
name of the part, `က ၏ နှုတ်ဆက်()` runs the `နှုတ်ဆက်` of `လူ` on the part, and a write to
`က ၏ နာမည်` changes the name of the part. We can use this pattern to base a struct on another:

```text
<name> ဟူသော ဒေတာအစုအဝေး သည် <struct> ကို အခြေခံသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင် <fields> ပါဝင်သည်။
```

| Zaka | Reads as |
| --- | --- |
| `လူ ကို အခြေခံသော` | "that is based on `လူ`" |
| `ဒေတာအစုအဝေး ဖြစ်၍` | "is a struct, and" |
| `၎င်းတွင်` | "in it" |

## Replacing a function

A function of the struct itself wins over a function of its part. Here a teacher greets in
its own form, while a student keeps the greeting of `လူ`:

```zaka
လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
လ ဟူသော လူ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"မင်္ဂလာပါ {လ ၏ နာမည်}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဆရာ ဟူသော ဒေတာအစုအဝေး သည်
လူ ကို အခြေခံသော ဒေတာအစုအဝေး ဖြစ်၍ ၎င်းတွင်
ဘာသာ ဟူသော စာသား တစ်ခု
ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။

// the teacher's own greeting wins over the one of its part
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည်
ဆ ဟူသော ဆရာ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
"{ဆ ၏ ဘာသာ} ဆရာ {ဆ ၏ နာမည်}" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ကိန်းရှင် ဆ သည် ဆရာ(လူ: လူ(နာမည်: "ဒေါ်မြ")၊ ဘာသာ: "သင်္ချာ") ဖြစ်ပါစေ။
ဆ ၏ နှုတ်ဆက်() ကို ပြသမည်။
```

```text
သင်္ချာ ဆရာ ဒေါ်မြ
```

## Reading the part

`x ကို <struct> အဖြစ် ပြောင်းလဲသော ရလဒ်` reads the part itself, as a value of its struct, so we
can pass it to a function that takes that struct: `ပြ(က ကို လူ အဖြစ် ပြောင်းလဲသော ရလဒ်)`.

## Details

- **Several parts.** `လူ နှင့် ဝန်ထမ်း တို့ကို အခြေခံသော` holds two parts. When both have a field
  or a function of one name, `x ၏` that name is ZK0364, and we read the part first.
- **Parts of parts.** A part can be based on a struct too, and the nearest part with the name
  wins.
- **Traits.** A struct has the traits its parts give, so it goes into a trait value of them
  ([Structs based on structs](../../manual/language/based-structs.md#traits-through-parts)).
- **Shared objects.** A struct based on a [shared object](objects.md) holds a reference to it.

## Recap

- `<struct> ကို အခြေခံသော` in the head bases a struct on another, which it holds as a part.
- A literal sets the part under the name of its struct.
- A value reaches the fields and the functions of its parts, and its own win.
- `x ကို <struct> အဖြစ် ပြောင်းလဲသော ရလဒ်` reads the part.

## Hands on

- Base a struct `ဝန်ထမ်း` (worker) on `လူ` with a field for the office, and greet one.
- Give `ဝန်ထမ်း` its own `နှုတ်ဆက်`, and check which greeting runs.
- Put a student and a worker in a list of `နှုတ်ဆက်နိုင်သော အရာ`, and greet each.

Next: [Enumerations and switches](../enums/README.md).
