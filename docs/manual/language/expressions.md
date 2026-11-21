---
title: Expressions
description: Literals, names and the operators in each group - arithmetic, comparison, logic and bitwise - with precedence, division, overflow and grouping.
---

# Expressions

An expression produces a value. It can be a literal, a name, or an operator applied to
one or more expressions. Most operators have both a sign form, such as `+`, and an
equivalent Burmese word form, such as `နှင့် … ၏ ပေါင်းလဒ်`. We can mix the two forms in
one expression. The compiler determines the data kind (type) of each expression before
the program runs. See [Data kinds and values](kinds.md) for the data kinds and
[Operators](../../learn/operators/README.md) in Learn for examples.

## Three tiers

```text
notation   က + ခ * ၂       -ဂ        (က - ခ)         signs, prefix signs, parentheses
clause     က နှင့် ခ ၏ ပေါင်းလဒ်     က သည် ခ ထက် ကြီး     operands with particles, closed with a verb
logical    A ဖြစ်၍ B     A သို့မဟုတ် B     A မဟုတ်        joins true/false values
```

The grammar of an expression has three tiers, from the tightest to the loosest:

| Tier | What it contains | Operands |
| --- | --- | --- |
| notation | the signs, with the precedence of school maths | atoms: literals, names, calls and expressions in parentheses |
| clause | Burmese phrases, closed with a verb | whole notation expressions, each with the particle (postposition) that its role needs after it |
| logical | `ဖြစ်၍`, `သို့မဟုတ်` and `မဟုတ်` | clauses |

In a clause, a verb uses the operands just before it whose particles it wants, and its
result gets the next particle. [Particles](../../learn/sentences/particles.md) in the
Learn book shows the common ones. A clause can be part of a notation expression, and a
logical expression part of a clause, but each one needs parentheses there.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က + ခ * ၂ ကို ပြသမည်။
က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသမည်။
က + ခ နှင့် ၁၀ ၏ မြှောက်လဒ် ကို ပြသမည်။
(က နှင့် ခ ၏ ပေါင်းလဒ်) * ၁၀ ကို ပြသမည်။
(က သည် ခ အောက် ငယ်သည် ဖြစ်၍ ခ သည် ၅ အောက် ငယ်သည်) ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၈
၅
၅၀
၅၀
မှန်
```

## Atoms

| Atom | Example | Rules |
| --- | --- | --- |
| a number literal or a character literal | `၄၂`, `၃.၁၄`, `'က'` | [Source text and words](words.md#numbers) |
| `မှန်` and `မမှန်`, the two true/false values | `မှန်` | [True/false values](kinds.md#truefalse-values) |
| the name of a variable, a constant, a parameter or a loop counter | `အသက်` | [Names and scope](scope.md) |
| an expression in parentheses | `(က + ခ)` | [Grouping](#grouping) |
| a call | `f ဟူသော လုပ်ဆောင်ချက်ကို … လုပ်ဆောင်သော ရလဒ်` | [Function calls](functions-calls.md) |
| a string literal of the data kind `စာသား`, with template slots that contain expressions | `"{အမည်}"` | [Text](text.md) |
| a list literal, a cell `x ၏ အကွက် i` or a range | `[၁၊ ၂]` | [Lists](lists.md) |
| a field | `ကျော် ၏ အသက်` | [Structs](structs.md) |
| in unsafe code, the value at a pointer and the address of a variable | `p ၏ တန်ဖိုး`, `x ၏ လိပ်စာ` | [Unsafe Zaka](unsafe.md) |
| a function value | `f ဟူသော လုပ်ဆောင်ချက်` | [Function values](function-values.md) |
| a duplicate (copy), shallow or deep | `x ၏ မိတ္တူ`, `x တစ်ခုလုံး ၏ မိတ္တူ` | [Values and objects](objects.md) |

A number literal without a data kind after it is *flexible*: it gets the data kind that
its place wants, which can be a declaration, an assignment, a parameter or the other
operand of an operator, and the integer `၃` is an `isize` when nothing wants another data
kind. A literal that does not fit is ZK0406. A `-` right before a literal counts in the
range check, so `-၁၂၈` fits an `၈ ဘစ် ကိန်းပြည့်`.

A function has no variable for its result, so a name never refers to the result of the
function around it, and the function returns its value with `ကို ပြန်ပေးမည်`. A name that
is not declared where it is used is ZK0301.

## Arithmetic

| Sign form | Word form | Operation |
| --- | --- | --- |
| `a + b` | `a နှင့် b ၏ ပေါင်းလဒ်` | addition |
| `a - b` | `a နှင့် b ၏ နုတ်လဒ်` | subtraction |
| `a * b` | `a နှင့် b ၏ မြှောက်လဒ်` | multiplication |
| `a / b` | `a ကို b ဖြင့် စားသော စားလဒ်` | division (quotient) |
| `a % b` | `a ကို b ဖြင့် စားသော စားကြွင်း` | remainder |
| `-a` | `အနုတ် a` | negation |

The operands are numbers, integers or decimal numbers, of one common data kind, which is
also the data kind of the result, and [Conversions](conversions.md) states how two data
kinds are joined. Negation needs a signed integer or a decimal number, so `-က` for an
unsigned `က` is ZK0403, and a true/false value or a character in arithmetic is ZK0403 too.
`+`, in either form, also joins two pieces of text into new text
([Joining](text.md#joining)), and no other operator works on text.

### Integer division and remainder

Integer division drops the fraction toward zero, and the remainder has the sign of the
dividend, so each pair satisfies `a == (a / b) * b + a % b`:

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁၇ / ၅ ကို ပြသမည်။
၁၇ % ၅ ကို ပြသမည်။
-၁၇ / ၅ ကို ပြသမည်။
-၁၇ % ၅ ကို ပြသမည်။
၁၇ / -၅ ကို ပြသမည်။
၁၇ % -၅ ကို ပြသမည်။
၁၇.၅ % ၅ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၃
၂
-၃
-၂
-၃
၂
၂.၅
```

An integer division or remainder with a zero divisor has no answer. When the compiler can
see the zero, it refuses the program (ZK0602), and when the zero shows up while the
program runs, the program stops at that operation ([When a program
stops](../vm/traps.md)).

```zaka-error ZK0602
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀ / ၀ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
၁၀ % ခ ကို ပြသမည်။
"ဤနေရာ မရောက်ပါ" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: integer division or remainder with zero as divisor
```

Decimal numbers never stop the program here: `၁.၀ / ၀.၀` is `inf`, and `၀.၀ / ၀.၀` is
`NaN`. The remainder of decimal numbers is the C function `fmod`, so `၁၇.၅ % ၅` is `၂.၅`.

### Overflow

An integer result that does not fit its data kind has one of two outcomes:

| Operands | What happens |
| --- | --- |
| all constants | the compiler works the operation out, and a result that does not fit is an error (ZK0601) |
| at least one variable | the program stops at run time, with a message that names the operator and the data kind |

The run-time stop covers `+`, `-`, `*`, a negation, and the smallest signed value divided
with -1. A conversion still wraps around, and so does the counter of a counted loop, which
never goes past its last value.

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဂ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
မှန် ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဂ ကို ဂ * ၂ ဟု ထားမည်။
ဂ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၂
၄
၈
၁၆
၃၂
၆၄
၁၂၈
the program stopped: the result of * does not fit u8
```

The compiler also follows the values that a variable can have, as the flow checks do
([Compiler checks](../compiler/checks.md)), and it warns when an operation overflows for
each of those values (ZK0719), because the program then stops there on each run.

```zaka-warning ZK0719
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် n သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
n - ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0719]: "-" on usize gives a value outside usize for each value of its operands, so the program stops here
```

With `--overflow wrap`, or with **Stop on overflow** turned off in the explorer, the
operation wraps around instead: the result is the true result cut to the width of the
data kind, like an odometer rolling over. Nothing stops the program, and the compiler
does not give ZK0719.

```zaka overflow=wrap
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅၀ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၇ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
က + ၁၀ ကို ပြသမည်။
က * ၂ ကို ပြသမည်။
ဂ - ၁ ကို ပြသမည်။
ခ + ၁ ကို ပြသမည်။
-ခ - ၂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၄
၂၄၄
၂၅၅
-၁၂၈
၁၂၇
```

The data kind sets the edge, so the same sum in a wider data kind does not wrap. We pick
a data kind that is big enough for the numbers at hand, and plain `ကိန်းပြည့်` is a safe
choice.

```zaka-error ZK0601
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၇ + ၁ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0601]: + on these constants does not fit i8
```

## Comparison

| Sign form | Word form | True when |
| --- | --- | --- |
| `a == b` | `a သည် b နှင့် ညီ` | equal |
| `a != b` | `a သည် b နှင့် မညီ` | not equal |
| `a < b` | `a သည် b အောက် ငယ်` | less |
| `a > b` | `a သည် b ထက် ကြီး` | greater |
| `a <= b` | `a သည် b ထက် မကြီး` | not greater |
| `a >= b` | `a သည် b အောက် မငယ်` | not less |

A comparison returns a `မှန်မှားတန်ဖိုး`. The two operands have one common data kind, and
the narrower one is widened first, as in arithmetic.

| Operator | Works on | Compares |
| --- | --- | --- |
| `==`, `!=` | each data kind: numbers, true/false values, characters, text, lists of one data kind, structs of one data kind, shared objects of one data kind, function values and pointers | the contents of text, lists, structs and objects, in depth, and safe on objects that reach each other in a ring; function values and pointers as the same function or address ([Values and objects](objects.md)) |
| `<`, `>`, `<=`, `>=` | numbers and characters | the values in order |

A decimal `NaN` is equal to nothing, not even itself, and each ordering with it is false.
An ordering on a true/false value is ZK0403, and a condition that is not a true/false value
is ZK0402.

To ask whether two objects of one data kind, or two function values, are one and the same,
we write `X နှင့် Y သည် တစ်ခုတည်း` ("are one and the same"). This comparison is ZK0404 on
other data kinds ([Values and objects](objects.md)).

A comparison cannot be the direct operand of another comparison, in either form
(ZK0210). To join two comparisons, we use `ဖြစ်၍`, or we put the inner one in parentheses.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၁၆ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅၀၀ ဖြစ်ပါစေ။
က < ခ ကို ပြသမည်။
က သည် ခ နှင့် ညီသည် ဟု ပြသမည်။
'က' < 'ခ' ကို ပြသမည်။
မှန် == မမှန် ကို ပြသမည်။
(က >= ၁ ဖြစ်၍ က <= ၁၀) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မှန်
မမှန်
မှန်
မမှန်
မှန်
```

```zaka-error ZK0210
၁ < ၂ < ၃ ကို ပြသမည်။
```

```text
error[ZK0210]: a comparison cannot be the operand of another comparison
help: join the two comparisons with "ဖြစ်၍", or put the inner one in parentheses
```

## Logic

| Sign form | Word form | Value |
| --- | --- | --- |
| `a && b` | `a ဖြစ်၍ b` | true when both are true |
| `a \|\| b` | `a သို့မဟုတ် b` | true when at least one is true |
| `!a` | `a မဟုတ်` | true when `a` is false |

The operands are true/false values (ZK0403). `ဖြစ်၍` and `သို့မဟုတ်` short-circuit: when
the left side settles the answer, the right side is not worked out, so a call on the right
side is not made. We write `မဟုတ်` after the value, as Burmese does, and it negates a whole
clause, so `က == ခ မဟုတ်` is `!(က == ခ)`, while `!က == ခ` is `(!က) == ခ`.

To print a logical expression, we put it in parentheses, because otherwise the compiler
cannot tell where it ends (ZK0204).

```zaka
စစ် ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
"စစ်ပြီ" ဟု ပြသမည်။
မှန် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
(မမှန် ဖြစ်၍ စစ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ်) ကို ပြသမည်။
(မှန် သို့မဟုတ် စစ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ်) ကို ပြသမည်။
(မှန် ဖြစ်၍ စစ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ်) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
မမှန်
မှန်
စစ်ပြီ
မှန်
```

```zaka-error ZK0204
ကိန်းရှင် က သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မှန် ဖြစ်ပါစေ။
က ဖြစ်၍ မှန် ကို ပြသမည်။
```

## Bitwise operators

| Sign form | Word form | Operation |
| --- | --- | --- |
| `a & b` | `a နှင့် b ၏ ဘစ်အလိုက် ဆုံလဒ်` | and |
| `a \| b` | `a နှင့် b ၏ ဘစ်အလိုက် ပေါင်းစုလဒ်` | or |
| `a ^ b` | `a နှင့် b ၏ ဘစ်အလိုက် ခြားနားလဒ်` | exclusive or |
| `~a` | `a ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်` | not (each bit flips) |
| `a << n` | `a ကို ဘယ်သို့ n ဘစ် ရွေ့သောရလဒ်` | shift left |
| `a >> n` | `a ကို ညာသို့ n ဘစ် ရွေ့သောရလဒ်` | shift right |

The operands of `&`, `|`, `^` and `~` are integers of one common data kind. A shift keeps
the data kind of its left operand, and its distance is an unsigned integer, so a flexible
literal there becomes a `usize`.

- The distance must be less than the bit width of the left operand. A constant distance
  that is too big is ZK0407, and one that is too big while the program runs stops the
  program.
- A left shift drops the bits that leave the width.
- A right shift of a signed integer repeats the sign bit, so `-၈ >> ၁` is `-၄`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခြေ ၂ ဖြင့် ၁၁၀၀၁၀၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် -၈ ဖြစ်ပါစေ။
က & အခြေ ၂ ဖြင့် ၁၁၁၁၀၀၀၀ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမည်။
~က ကို ပြသမည်။
က << ၂ ကို ပြသမည်။
ခ >> ၁ ကို ပြသမည်။
က ကို ညာသို့ ၄ ဘစ် ရွေ့သောရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၉၂
၅၃
၄၀
-၄
၁၂
```

```zaka-error ZK0407
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
က << ၈ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```zaka-stop
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဆ သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၆၄ ဖြစ်ပါစေ။
"အစ" ဟု ပြသမည်။
၁ << ဆ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
အစ
the program stopped: shift distance 64 is not below the bit width 64
```

## Precedence

When an expression contains several operators, the tighter ones bind first, and the
operators of one level group from left to right. From the tightest to the loosest:

| Level | Operators |
| --- | --- |
| 1 | prefix `-`, `အနုတ်`, `~`, `!` |
| 2 | `*`, `/`, `%` |
| 3 | `+`, `-` |
| 4 | `<<`, `>>` |
| 5 | `&`, then `^`, then `\|` |
| 6 | `<`, `>`, `<=`, `>=`, and then `==`, `!=`; none of these chain |
| 7 | a word clause (`… ၏ ပေါင်းလဒ်`, `… ထက် ကြီး`, …) |
| 8 | `မဟုတ်` after a value |
| 9 | `ဖြစ်၍`, `&&` |
| 10 | `သို့မဟုတ်`, `\|\|` |

[Operator precedence](../reference/precedence.md) gives the full table with examples.
Unlike C, the bitwise operators bind tighter than comparison, so `၆ & ၃ == ၂` is
`(၆ & ၃) == ၂`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၂ + ၃ * ၄ ကို ပြသမည်။
၁၀ - ၃ - ၂ ကို ပြသမည်။
၂ * ၃ + ၄ * ၅ ကို ပြသမည်။
၁ << ၃ + ၁ ကို ပြသမည်။
၆ & ၃ == ၂ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
၁၄
၅
၂၆
၁၆
မှန်
```

## Grouping

```text
(<expression>)
```

Parentheses make an expression an atom, override precedence, and move an expression from
one tier to another: a clause into a notation expression, and a logical expression into a
clause.

| Expression next to a sign | Parentheses | Example |
| --- | --- | --- |
| a call | not needed, because a call is an atom | `f(၁) + ၁` |
| a conversion | needed, because a conversion is a clause | `(က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) / ၂` |
| a word clause | needed | `(က နှင့် ခ ၏ ပေါင်းလဒ်) * ၂` |
| a value read with a verb | needed | `(xs ၏ အကွက်အရေအတွက်) == ၀` |

A value read with a verb, such as `xs ၏ အကွက်အရေအတွက်`, gets no operator after it unless it
is in parentheses. The error for missing parentheses depends on the sentence:

| Where | Example | Error |
| --- | --- | --- |
| a condition or a declaration | `အကယ်၍ xs ၏ အကွက်အရေအတွက် == ၀ ဖြစ်လျှင် …` | ZK0201, with a help line that advises parentheses around the clause before `==` |
| a print or another sentence that ends in a statement verb | `xs ၏ အကွက်အရေအတွက် == ၀ ကို ပြသမည်။` | ZK0203, which finds no statement verb at `==` and gives no help about parentheses |
| a word clause before a sign | `က နှင့် ခ ၏ ပေါင်းလဒ် * ၂ ကို ပြသမည်။` | ZK0203 |

```zaka-error ZK0201
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် xs သည် [၁] ဖြစ်ပါစေ။
အကယ်၍ xs ၏ အကွက်အရေအတွက် == ၀ ဖြစ်လျှင် ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0201]: expected "လျှင်" or "ဖြစ်လျှင်", found "=="
```

```zaka-error ZK0203
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် xs သည် [၁] ဖြစ်ပါစေ။
xs ၏ အကွက်အရေအတွက် == ၀ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "=="
```

```zaka-error ZK0203
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က နှင့် ခ ၏ ပေါင်းလဒ် * ၂ ကို ပြသမည်။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "*"
```

Parentheses, operands, calls, lists and blocks nest up to 128 levels deep, and a program
that goes further gets ZK0244 ([The nesting limit](../internals/parser.md#the-nesting-limit)).

Parentheses around a whole printed, assigned or returned value change nothing unless the
value is a logical expression. The compiler warns about them (ZK0507), as an editor flags
a double space, and [Natural wording](naturalness.md) has the rule.

## Constants are worked out first

An operation whose operands are all constants is worked out in the compiler, so
`၂ + ၃ * ၄` reaches the VM as `၁၄`. That is the reason overflow and a zero divisor in a
constant operation are compile errors. `zakac run --fold none` turns this folding off, and
the compiler still reports errors in constant operations.
