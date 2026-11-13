---
title: Expressions
description: Literals, names and the operators in each group - arithmetic, comparison, logic and bitwise - with precedence, division, overflow and grouping.
---

# Expressions

An *expression* is a piece of program that has a value. It can be a literal, a name, or
operators applied to other expressions. Most operators have two spellings that mean the
same, a sign (`+`) and a Burmese phrase (`နှင့် ... ကို ပေါင်းသည့်ရလဒ်`), and the two can be mixed in
one expression. The compiler works out the kind of each expression before the program
runs ([Kinds and values](kinds.md)).

## Three tiers

The grammar of an expression has three tiers, from the tightest to the loosest:

```text
notation   က + ခ * ၂       -ဂ        (က - ခ)         signs, prefix signs, parentheses
clause     က နှင့် ခ ကို ပေါင်းသည့်ရလဒ်     က သည် ခ ထက် ကြီး     operands with particles, closed with a verb
logical    A ဖြစ်၍ B     A သို့မဟုတ် B     A မဟုတ်        joins truth values
```

- The **notation** tier holds the signs with the precedence of school maths. Its
  operands are *atoms*: literals, names, calls and parenthesized expressions.
- The **clause** tier holds Burmese phrases. Each operand is a whole notation
  expression, with the particle that its role needs after it, and a verb closes the
  clause. A verb takes the operands just before it whose particles it wants, and its
  result takes the next particle. [Particles in Learn](../../learn/sentences/particles.md)
  shows the common ones.
- The **logical** tier joins clauses with `ဖြစ်၍`, `သို့မဟုတ်` and `မဟုတ်`.

A clause can stand in a notation expression, and a logical expression in a clause, just
inside parentheses.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က + ခ * ၂ ကို ပြသမည်။
က နှင့် ခ ကို ပေါင်းသည့်ရလဒ် ကို ပြသမည်။
က + ခ နှင့် ၁၀ ကို မြှောက်သည့်ရလဒ် ကို ပြသမည်။
(က နှင့် ခ ကို ပေါင်းသည့်ရလဒ်) * ၁၀ ကို ပြသမည်။
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

- **Number literals** and **character literals**, written as in [Source text and
  words](words.md). A number literal without a kind after it is *flexible*: it takes the kind
  that its place wants (a declaration, an assignment, a parameter, the other operand of
  an operator), and the integer `၃` is an `isize` when nothing wants another kind.
  A literal that does not fit is ZK0406. A `-` right before a literal counts in the
  range check, so `-၁၂၈` fits an `၈ ဘစ် ကိန်းပြည့်`.
- **`မှန်` and `မမှန်`**, the two true/false values.
- **Names** of variables, constants, parameters and loop counters. A name that is not
  declared where it is used is ZK0301 ([Names and scope](scope.md)). A function has no
  variable of its own for its result: it gives the value back with `ကို ပြန်ပေးမည်`.
- **Parentheses**, `( ... )`, around an expression of each kind.
- **Calls**, `f ဟူသော လုပ်ဆောင်ချက်ကို ... လုပ်ဆောင်သော ရလဒ်` ([Functions](functions.md)).
- **String literals**, of the kind `စာသား`, with template slots that hold expressions
  ([Text](text.md)).
- **Places and ranges**: list literals, `x ၏ အကွက် i` and ranges ([Lists](lists.md)),
  fields ([Structs](structs.md)), and in unsafe code `p ၏ တန်ဖိုး` and `x ၏ လိပ်စာ`
  ([Unsafe Zaka](unsafe.md)).
- **Function values**, `f ဟူသော လုပ်ဆောင်ချက်` ([Function values](function-values.md)).
- **Copies**, `x ၏ မိတ္တူ` and the deep `x တစ်ခုလုံး ၏ မိတ္တူ` ([Values and
  objects](objects.md)).

## Arithmetic

| Sign | Words | Operation |
| --- | --- | --- |
| `a + b` | `a နှင့် b ကို ပေါင်းသည့်ရလဒ်` | addition |
| `a - b` | `a နှင့် b ကို နုတ်သည့်ရလဒ်` | subtraction |
| `a * b` | `a နှင့် b ကို မြှောက်သည့်ရလဒ်` | multiplication |
| `a / b` | `a ကို b ဖြင့် စားသည့်ရလဒ်` | division (quotient) |
| `a % b` | `a ကို b ဖြင့် အကြွင်းရှာသည့်ရလဒ်` | remainder |
| `-a` | `အနုတ် a` | negation |

The operands are numbers (integers or decimal numbers) of one common kind, which is the
kind of the result; [Conversions](conversions.md) tells how two kinds are joined. Negation
needs a signed integer or a decimal number: `-က` for an unsigned `က` is ZK0403. A
boolean or a character takes no arithmetic (ZK0403). `+`, in either spelling, also joins
two pieces of text into new text ([Joining](text.md#joining)); text takes no other
operator.

### Integer division and remainder

Integer division drops the fraction **toward zero**. The remainder has the sign of the
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

A zero divisor in an integer division or remainder has no answer. When the compiler can
see the zero it refuses the program (ZK0602). When the zero shows up while the program
runs, the program stops at that operation ([When a program stops](../vm/traps.md)).

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
`NaN`. The remainder of decimal numbers is the C function `fmod`: `၁၇.၅ % ၅` is `၂.၅`.

### Overflow

An integer result that does not fit its kind meets one of two fates.

- When all operands are constants, the compiler works the operation out and a result
  that does not fit is an error (ZK0601).
- When an operand is a variable, the program stops at run time, and the message names the
  operator and the kind. This covers `+`, `-`, `*`, a negation, and the smallest signed
  value divided with -1. A conversion still wraps, and so does the counter of a counted
  loop, which never goes past its last value.

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

The compiler also follows the values a variable can hold, as the flow checks do ([Compiler checks](../compiler/checks.md)), and
warns when an operation overflows for each of them (ZK0719): the program stops there each
time it runs.

```zaka-warning ZK0719
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် n သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
n - ၁ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
warning[ZK0719]: "-" on usize gives a value outside usize for each value of its operands, so the program stops here
```

With `--overflow wrap` (**Stop on overflow** turned off in the explorer), the operation
wraps around instead: the result is the true result cut to the width of the kind, like an
odometer rolling over. Nothing stops the program, and ZK0719 is not given.

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

The kind decides the edge: the same sum in a wider kind does not wrap. Pick a kind that
is big enough for the numbers at hand; plain `ကိန်းပြည့်` is a safe choice.

```zaka-error ZK0601
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၇ + ၁ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

```text
error[ZK0601]: + on these constants does not fit i8
```

## Comparison

| Sign | Words | True when |
| --- | --- | --- |
| `a == b` | `a သည် b နှင့် ညီ` | equal |
| `a != b` | `a သည် b နှင့် မညီ` | not equal |
| `a < b` | `a သည် b အောက် ငယ်` | less |
| `a > b` | `a သည် b ထက် ကြီး` | greater |
| `a <= b` | `a သည် b ထက် မကြီး` | not greater |
| `a >= b` | `a သည် b အောက် မငယ်` | not less |

A comparison gives a `မှန်မှားတန်ဖိုး`. The operands have one common kind (the narrower
one is widened first, as in arithmetic). `==` and `!=` take each kind, including true/false
values, characters, text, lists of one kind, structs of one kind, shared objects of one
kind, function values and pointers. Text, lists, structs and objects compare their
contents in depth, safe on objects that reach each other in a ring; function values and
pointers compare as the same function or address ([Values and objects](objects.md)). The ordering operators take numbers and
characters. A decimal `NaN` is equal to nothing, not even itself, and each ordering
with it is false.

`X နှင့် Y သည် တစ်ခုတည်း` ("are one and the same") is a comparison of its own: it asks
whether two objects of one kind, or two function values, are one, and is ZK0404 on other
kinds ([Values and objects](objects.md)).

A comparison cannot be the direct operand of another comparison, in either spelling
(ZK0210). Join two comparisons with `ဖြစ်၍`, or put the inner one in parentheses.

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

An ordering on a true/false value is ZK0403, and so is a condition that is no
true/false value (ZK0402).

## Logic

| Sign | Words | Value |
| --- | --- | --- |
| `a && b` | `a ဖြစ်၍ b` | true when both are true |
| `a \|\| b` | `a သို့မဟုတ် b` | true when either is true |
| `!a` | `a မဟုတ်` | true when `a` is false |

The operands are true/false values (ZK0403). `ဖြစ်၍` and `သို့မဟုတ်` are *short-circuit*:
when the left side settles the answer, the right side is not worked out, so a call on
the right side is not made. `မဟုတ်` goes **after** the value as in Burmese, and it
negates a whole clause: `က == ခ မဟုတ်` is `!(က == ခ)`, while `!က == ခ` is `(!က) == ခ`.

A logical expression used as a value to print needs parentheses, or the compiler cannot
tell where it ends (ZK0204).

```zaka
လုပ်ဆောင်ချက် စစ် သည်
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

| Sign | Words | Operation |
| --- | --- | --- |
| `a & b` | `a နှင့် b ကို ဘစ်ဆုံသည့်ရလဒ်` | and |
| `a \| b` | `a နှင့် b ကို ဘစ်ပေါင်းစုသည့်ရလဒ်` | or |
| `a ^ b` | `a နှင့် b ကို ဘစ်ခြားနားသည့်ရလဒ်` | exclusive or |
| `~a` | `a ကို ဘစ်ပြောင်းပြန်သည့်ရလဒ်` | not (each bit flips) |
| `a << n` | `a ကို n ဖြင့် ဘစ်ဘယ်ရွေ့သည့်ရလဒ်` | shift left |
| `a >> n` | `a ကို n ဖြင့် ဘစ်ညာရွေ့သည့်ရလဒ်` | shift right |

The operands of `&`, `|`, `^` and `~` are integers of one common kind. A shift keeps the
kind of its left operand. The distance is an unsigned integer (a flexible literal becomes
a `usize`) and it must be less than the bit width of the left operand: a constant
distance that is too big is ZK0407, and one that is too big while the program runs stops
it. A left shift drops the bits that leave the width. A right shift of a signed integer
repeats the sign bit, so `-၈ >> ၁` is `-၄`.

```zaka
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခြေ ၂ ဖြင့် ၁၁၀၀၁၀၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် -၈ ဖြစ်ပါစေ။
က & အခြေ ၂ ဖြင့် ၁၁၁၁၀၀၀၀ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမည်။
~က ကို ပြသမည်။
က << ၂ ကို ပြသမည်။
ခ >> ၁ ကို ပြသမည်။
က ကို ၄ ဖြင့် ဘစ်ညာရွေ့သည့်ရလဒ် ကို ပြသမည်။
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

When an expression holds several operators, the tighter ones bind first, and the
operators of one level group from left to right. From the tightest to the loosest:

1. prefix `-`, `အနုတ်`, `~`, `!`;
2. `*`, `/`, `%`;
3. `+`, `-`;
4. `<<`, `>>`;
5. `&`, then `^`, then `|`;
6. `<`, `>`, `<=`, `>=`, and then `==`, `!=` (none of these chain);
7. a word clause (`... ကို ပေါင်းသည့်ရလဒ်`, `... ထက် ကြီး`, ...);
8. `မဟုတ်` after a value;
9. `ဖြစ်၍`, `&&`;
10. `သို့မဟုတ်`, `||`.

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

Parentheses make an expression an atom, override precedence and move an
expression between tiers: a clause into a notation expression, and a logical
expression into a clause. A conversion is a clause, so
`(က ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) / ၂` needs its parentheses; a call is an atom and
needs none. A value read with a verb, such as `xs ၏ အကွက်အရေအတွက်`, takes no operator after
it unless it stands in parentheses: `xs ၏ အကွက်အရေအတွက် == ၀` is ZK0201, and the help
advises the clause go before `==` in parentheses.

Parentheses, operands, calls, lists and blocks nest up to 128 levels deep; a program that goes
further gets ZK0244 ([The nesting limit](../internals/parser.md#the-nesting-limit)).

Parentheses around a whole printed, assigned or returned value change nothing unless
the value is a logical expression. The compiler warns about them (ZK0507), as an editor
flags a double space; see [Natural wording](naturalness.md).

```zaka-error ZK0203
ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
က နှင့် ခ ကို ပေါင်းသည့်ရလဒ် * ၂ ကို ပြသမည်။
```

```text
error[ZK0203]: this statement does not end in a statement verb, found "*"
```

## Constants are worked out first

An operation whose operands are all constants is worked out in the compiler, so
`၂ + ၃ * ၄` reaches the VM as `၁၄`. This is the reason overflow and a zero divisor in a
constant operation are compile errors. `zakac run --fold none` turns the folding off,
and errors in constant operations are still reported.
