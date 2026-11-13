---
title: Grammar
description: The shape of each form of the language, rule after rule, in one place.
---

# Grammar

This page gives the shape of each form the parser reads. It states shapes alone: what a
form means, and the checks the compiler adds after the parser, are on the language page
that each section links to. The reserved words are listed on
[Reserved words](reserved-words.md), and the order of the operators on
[Operator precedence](precedence.md).

## The notation

The rules are productions in a small form of EBNF (Extended Backus–Naur Form). A
nonterminal is a lowercase English name; a terminal is a token, written in quotes:

| Written | Means |
| --- | --- |
| `name = ... ;` | a production: the nonterminal `name` derives what is on the right |
| `"ကို"` | a terminal: a reserved word, a phrase token or punctuation, as it stands in a program |
| `a b` | concatenation: `a`, then `b` |
| `a \| b` | alternation: `a` or `b` |
| `[ a ]` | option: `a`, or nothing |
| `{ a }` | repetition: `a`, zero or more times |
| `( a )` | a group |
| `items(x)` | a list in words of `x`: `x`, or `x { "၊" x } "နှင့်" x`; one item takes the closing word with no `တို့`, two or more take it with `တို့` ([Lists in words](../language/words.md#lists-in-words)) |
| `(* ... *)` | a comment, not part of the production |

The lexer does some of the work before the parser sees the token stream, so the
productions leave it out:

- **Spaces.** A word made of reserved words alone is split, and the words of a reserved
  phrase join into one token whether spaces stand between them or not. So
  `ပရိုဂရမ်သည်` reads as `ပရိုဂရမ် သည်`, `ဒေတာအစုအဝေးတွင်` as `ဒေတာအစုအဝေး တွင်`, and
  `အောက်ပါတို့ကို လုပ်ဆောင်မည်` is one token. A rule writes each phrase with the spaces
  the docs teach. A name still needs a space before the particle after it
  ([Words and spaces](../language/words.md#words-and-spaces)).
- **`ဒေတာ` and `အချက်အလက်`** are one word to the lexer: `အချက်အလက်အစုအဝေး` is
  `ဒေတာအစုအဝေး`, and `အချက်အလက်ဆဲလ်` is `ဒေတာဆဲလ်`. The rules write `ဒေတာ`. Alone,
  `ဒေတာ` is a name, save after `ထဲမှ`, `ထဲတွင်` and `အမှားအယွင်း`, and before
  `ပြောင်းလဲရန်` and `ထည့်ပေးရန်`.
- **Endings.** A comparison word, `မဟုတ်`, `မှန်` and `မမှန်` can take an attached
  ending, `ပါ`, `သည်` or `ပါသည်` (`ကြီးသည်`, `မဟုတ်ပါ`), which the lexer folds into the
  word. The rules leave the ending out; [Natural wording](../language/naturalness.md)
  tells which ending reads well where.
- **Spellings.** Where a word has several spellings, the rule lists each one.

A few rules also depend on what the file declares and on the form around them;
[Where the rules depend on context](#where-the-rules-depend-on-context) lists them.

## Words and tokens

```text
name         = word | "`" { character } "`" ;     (* a word that is not reserved, or a raw name *)
digits       = digit { digit } ;                  (* ၀ to ၉ or 0 to 9, one script in a number *)
number       = digits [ "." digits ] [ exponent ] ;
exponent     = ( "e" | "E" ) [ "+" | "-" ] digits ;   (* Western digits *)
char-lit     = "'" ( character | escape ) "'" ;
string-lit   = '"' { character | escape | slot } '"' ;
slot         = "{" expr "}" ;                     (* a template slot *)
escape       = "\" character | "{{" | "}}"
             | "{နောက်တစ်ကြောင်း}" | "{ယူနီကုဒ် " hex "}" ;
comment      = ( "//" | "///" | "မှတ်ချက်" ) { character } ;   (* to the end of the line *)
count-word   = "တစ်" classifier ;                 (* one word: တစ်ခု, တစ်လုံး, တစ်ယောက် *)
classifier   = name | "ကွက်" ;
punctuation  = "၊" | "။" | "၏" | "(" | ")" | "[" | "]" | ":" | "?" ;
```

See [Source text and words](../language/words.md) and
[Text](../language/text.md#string-literals).

## Program and files

```text
file         = { import } { top-item } ;
top-item     = declaration | start-block ;        (* another statement here is ZK0326 *)
import       = string-lit "မှ" items(name) ( "ကို" | "တို့ကို" ) "ထည့်သွင်းမည်" "။" ;
start-block  = "ပရိုဂရမ်" "သည်" [ params "လက်ခံပြီး" ] fn-open "။"
               { stmt }
               "ပရိုဂရမ်" done "။" ;
done         = "ပြီးဆုံးပါပြီး" | "ပြီးဆုံးပါပြီ" | "ပြီးပါပြီး" | "ပြီးပါပြီ" ;
```

An import after another statement, or inside a block, is parsed and reported. See
[Modules](../language/modules.md) and
[The start block](../language/functions.md#the-start-block).

## Declarations

```text
declaration  = var-decl | fn-decl | external-fn | struct-decl | handle-decl
             | enum-decl | alias-decl | trait-decl | trait-block ;

var-decl     = [ "အများသုံး" ] ( "ကိန်းရှင်" | "ကိန်းသေ" ) name
               ( "သည်" decl-head "ဖြစ်ပါစေ"
               | "ထဲသို့" clause "ကို" ( "ကူးထည့်မည်" | "အကုန်ကူးထည့်မည်" ) ) "။" ;
decl-head    = slot-kind [ "ဖြစ်၍" "တန်ဖိုး" "သည်" clause ]
             | clause ;                           (* the kind comes from the value *)

fn-decl      = [ "အများသုံး" ] [ "စိတ်မချရသော" ] "လုပ်ဆောင်ချက်" name [ sign-params ] "သည်"
               [ word-params "သည်" ]
               [ params "လက်ခံပြီး" ] fn-open "။"
               [ notes ]
               { stmt }
               block-close "။" ;

sign-params  = "<" kind-param { "၊" kind-param } ">" ;     (* no space before "<" *)
kind-param   = name [ ":" trait-ref { "နှင့်" trait-ref } ] ;
word-params  = items("ယေဘုယျဒေတာအမျိုးအစား" name) ( "ကို" | "တို့ကို" )
               "အစားထိုးသုံးနိုင်၍" "၎င်း" ;
trait-ref    = [ items(kind) ( "ကို" | "တို့ကို" ) "သုံး၍" ] name [ kind-args ] ;
notes        = "ဤနေရာတွင်" "သတိပြုရန်မှာ"
               { name "သည်" slot-kind "ဖြစ်သည်" "။"                       (* a local kind *)
               | name "တွင်" items(trait-ref "ဂုဏ်သတ္တိ") [ "တို့" ] "ရှိသည်" "။" }
               "သတိပြုစရာများ" done "။" ;

trait-decl   = name [ sign-params ] "ဟူသည့်" [ "အများသုံး" ] [ compiler-mark ]
               ( "ဂုဏ်သတ္တိ" (* with sign-params *) | "ဂုဏ်သတ္တိ" "သည်" word-params )
               "တွင်" "အောက်ပါ" "လုပ်ဆောင်ချက်များ" "ပါဝင်ရမည်" "။"
               [ notes ]
               { fn-head }
               "ဂုဏ်သတ္တိ" "တွင်" "ပါသော" "လုပ်ဆောင်ချက်များ" done "။" ;
fn-head      = "လုပ်ဆောင်ချက်" name "သည်"
               ( params ( "လက်ခံမည်" | "လက်ခံပြီး" result ) | result ) "။" ;
compiler-mark = "ကွန်ပိုင်လာထဲတွင်" "တစ်ပါတည်းပါဝင်သော" ;   (* the prelude alone, ZK0349 *)
trait-block  = kind "တွင်" trait-ref "ဂုဏ်သတ္တိ" "ရှိစေရန်" "အောက်ပါ" "လုပ်ဆောင်ချက်များ"
               "ကို" "ထည့်သွင်းမည်" "။"
               { fn-decl }
               "ထည့်သွင်းမည့်" "လုပ်ဆောင်ချက်များ" done "။" ;
fn-open      = "အောက်ပါတို့ကို လုပ်ဆောင်မည်"
             | "အောက်ပါတို့ကို လုပ်ဆောင်ကာ" result ;
result       = slot-kind "ကို" [ "အပိုင်အနေဖြင့်" ] "ပြန်ပေးမည်" ;
block-close  = "လုပ်ဆောင်စရာများ" done ;

params       = param [ "လည်းကောင်း" ]
             | param "လည်းကောင်း" param "လည်းကောင်း" { param "လည်းကောင်း" } ;
param        = name "ဟူသော" listed-kind "ကို" markers ;
markers      = [ "ဒေတာ ပြောင်းလဲရန်" | "ဒေတာ ထည့်ပေးရန်" ] [ "အပိုင်အနေဖြင့်" ] [ preposition ] ;
preposition  = "ဝိဘတ်" string-lit "ကို" "သုံး၍" ;   (* the string holds names and particles *)

external-fn  = [ "အများသုံး" ] [ "စိတ်မချရသော" ] "လုပ်ဆောင်ချက်" name "သည်"
               origin "လုပ်ဆောင်ချက်" [ "တစ်ခု" ]
               ( "ဖြစ်သည်" | "ဖြစ်၍" ext-tail ) "။" ;
origin       = "ပြင်ပမှလာသော"                     (* a host function *)
             | string-lit "မှလာသော" ;             (* a C function of that shared lib *)
ext-tail     = c-params "လက်ခံမည်"                (* no result *)
             | c-params "လက်ခံပြီး" result
             | result ;
c-params     = params
             | param "လည်းကောင်း" { param "လည်းကောင်း" } "အခြားတန်ဖိုးများနှင့်အတူ" ;

struct-decl  = name [ sign-params ] "ဟူသော" [ "အများသုံး" ] [ "မျှဝေနိုင်သော" ] "ဒေတာအစုအဝေး"
               ( "တွင်" | "သည်" word-params "တွင်" )
               fields closer "။" ;
handle-decl  = name "ဟူသော" [ "အများသုံး" ] "ပြင်ပမှ လာသော" "ဒေတာအစုအဝေး"
               ( "ဖြစ်သည်" | "တွင်" fields ( "ပါဝင်သည်" | "တို့ပါဝင်သည်" ) ) "။" ;
fields       = items(field) ;
field        = name "ဟူသော" listed-kind ;

enum-decl    = name "ဟူသော" [ "အများသုံး" ] "ဒေတာအမျိုးအစားကွဲ" "တွင်"
               items(variant) closer "။" ;
variant      = [ "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်" ] [ "တန်ဖိုးအနေဖြင့်" expr "ရှိသည့်" ] name ;

closer       = ( "ပါဝင်သည်" | "ပါဝင်ပြီး" word-clause )          (* after one field or variant *)
             | ( "တို့ပါဝင်သည်" | "တို့ပါဝင်ပြီး" word-clause ) ;  (* after two or more *)
word-clause  = "မျိုးပြပစ္စည်းမှာ" classifier "ဖြစ်သည်" ;

alias-decl   = [ "အများသုံး" ] "ဒေတာအမျိုးအစား" name
               ( [ sign-params ] "ကို" | "သည်" word-params "ကို" ) kind
               ( "ဟုလည်း မှတ်ယူနိုင်ပါသည်" | "ဟုလည်း မှတ်ယူနိုင်ပြီး" word-clause ) "။" ;
```

- With two or more parameters, each ends in `လည်းကောင်း`; either each parameter has a
  preposition or none has. The two words before `လုပ်ဆောင်ချက်`, `အများသုံး` and `စိတ်မချရသော`, can come in either order; the
  start block takes neither.
- The kind of an alias takes no measure word. A function, a struct, an enumeration or an
  alias inside a block is parsed and reported; a function with a block can stand inside
  another function.
- A declaration takes its kind parameters in one form: `sign-params` after the name or
  `word-params` after `သည်`. A trait takes one kind parameter or more. The functions of
  a trait block are functions of its file. A trait or a trait block inside a block is
  ZK0247.

See [Statements](../language/statements.md#declarations),
[Functions](../language/functions.md), [Structs](../language/structs.md),
[Enumerations](../language/enums.md), [Kind aliases](../language/aliases.md),
[Generics and traits](../language/generics.md) and
[C functions](../language/functions.md#c-functions).

## Statements

```text
stmt         = var-decl | fn-decl | block | if-stmt | switch-stmt
             | while-loop | counted-loop | each-loop | generic-loop
             | break | continue | return | call-stmt | verb-stmt | heap-stmt ;

block        = block-open "။" { stmt } block-close "။" ;
block-open   = "အောက်ပါတို့ကို လုပ်ဆောင်မည်"
             | "အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်" ;   (* unsafe code *)
branch       = block | stmt ;                     (* one statement in place of a block *)

if-stmt      = "အကယ်၍" expr then branch
               { "ထိုသို့မဟုတ်ဘဲ" expr then branch }
               [ "သို့မဟုတ်ပါက" branch ] ;
then         = "လျှင်" | "ဖြစ်လျှင်" ;

switch-stmt  = value "သည်" cases "ဖြစ်လျှင်" branch
               { cases "ဖြစ်လျှင်" branch }
               [ "တခြား" slot-kind "ဖြစ်လျှင်" branch ] ;
cases        = expr | expr { "၊" expr } "နှင့်" expr "တို့" ;

while-loop   = expr while branch ;
while        = "နေသရွေ့" | "ဖြစ်နေသရွေ့" ;
counted-loop = expr "မှ" expr "အထိ" [ expr "စီ" ] name "အနေဖြင့်"
               ( "ရေတွက်၍" | "တိုး၍" | "လျှော့၍" ) branch ;
each-loop    = expr "၏" each-part "ကို" [ name "နှင့်" ] name "အနေဖြင့်" "ယူ၍" branch ;
each-part    = "အကွက်တိုင်း" | "စာလုံးတိုင်း" | "အညွှန်းတိုင်း" | "အတန်းတိုင်း" ;
generic-loop = "ကိန်းရှင်" name "သည်" decl-head ( "ဖြစ်ကာ" | "ဖြစ်ပြီး" ) expr while
               block-open "။" { stmt } [ "ပြီးလျှင်" stmt ] block-close "။" ;
break        = "ကွင်းဆက်" "ကို" "ရပ်မည်" "။" ;
continue     = "နောက်တစ်ကြိမ်ဆီသို့" "ကျော်မည်" "။" ;

return       = value "ကို" "ပြန်ပေးမည်" "။"
             | "ပြန်ပေးမည်" "။"
             | "လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်" "။"
             | "လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်" "။" ;

verb-stmt    = value ( "ကို" | "ဟု" ) "ပြသမည်" "။"
             | value "ကို" value "ဟု" "ထားမည်" "။"
             | value "ထဲသို့" value "ကို" ( "ထည့်မည်" | "ကူးထည့်မည်" | "အကုန်ကူးထည့်မည်" ) "။"
             | value "၏" "အရှေ့တွင်" value "ကို" "ထည့်မည်" "။"
             | value "၏" ( "နောက်ဆုံးအကွက်" | "နောက်ဆုံးစာလုံး" ) "ကို" "ဖျက်မည်" "။"
             | value "ထဲမှ" "အညွှန်း" notation "ကို" "ဖျက်မည်" "။"
             | text-range "ထိ" "ကို" value "ဖြင့်" "အစားထိုးမည်" "။"
             | value "ကို" "ပြင်ပမှ" "ခေါ်ဆိုခွင့်" ( "ပြုမည်" | "ပြန်လည်ရုပ်သိမ်းမည်" ) "။" ;
text-range   = value "၏" "အကွက်" notation "မှ" notation
             | value "၏" notation ordinal "စာလုံး" "မှ" notation ordinal "စာလုံး" ;
ordinal      = "ခုမြောက်" | "လုံးမြောက်" ;

heap-stmt    = "မင်မိုရီတွင်" size "နေရာယူ၍ လိပ်စာကို" clause "တွင်" "သိမ်းထားမည်" "။"
             | "လိပ်စာ" clause "ရှိ မင်မိုရီကို" "ပြန်လည်စွန့်လွှတ်မည်" "။"
             | "လိပ်စာ" clause "ရှိ မင်မိုရီကို" size "ဖြစ်အောင်" "နေရာအချဲ့အချုံ့လုပ်မည်" "။"
             | "လိပ်စာ" clause "ရှိ မင်မိုရီကို" clause "ဖြင့်" size "ဖြည့်မည်" "။"
             | "လိပ်စာ" clause "ရှိ" size "မင်မိုရီ" "ကို" "လိပ်စာ" clause "ဆီသို့"
               ( "ကူးယူမည်" | "ပြောင်းထည့်မည်" ) "။" ;
size         = clause "ဘိုက်စာ"
             | kind clause ( "ခုစာ" | "လုံးစာ" | classifier "စာ" ) ;   (* the word of the kind *)
```

- `မင်မိုရီ` can be written `မှတ်ဉာဏ်`.
- A one-statement branch inside an `if` arm cannot be another `if`, and one inside a
  switch arm cannot be an `if` or a switch.
- The `ပြီးလျှင်` statement of a generic loop comes last in its block. A switch arm after
  the default arm is parsed and reported.
- `value` is an operand of a clause, or the value a verb of the clause gives
  ([Expressions](#expressions)). The operands of a statement verb come in the order the
  verb lists, and none can be left over before it.

See [Statements](../language/statements.md), [Lists](../language/lists.md), [Tables](../language/tables.md),
[Text](../language/text.md#changing-text), [Switch](../language/enums.md#switch) and
[Heap statements](../language/unsafe.md#heap-statements).

## Expressions

An expression has three tiers. The logical tier joins clauses; a clause holds operands,
each with the particle after it, and verbs; an operand is a notation expression with
signs.

```text
expr         = and-expr { ( "သို့မဟုတ်" | "||" ) and-expr } ;
and-expr     = not-expr { ( "ဖြစ်၍" | "&&" ) not-expr } ;
not-expr     = clause { "မဟုတ်" } ;

clause       = operand { particle operand | verb } ;   (* each verb takes the operands right before it *)
particle     = "၏" | "နှင့်" | "ကို" | "ဖြင့်" | "သည်" | "ထက်" | "အောက်" | "ဟု"
             | "ခုမြောက်" | "လုံးမြောက်" | "ထဲသို့" | "ထဲတွင်" ;
operand      = notation | word-list | word-table | join | word-struct
             | first-call | short-call | preposition-call ;   (* a call with a value end *)
join         = notation { "၊" notation } "နှင့်" notation "တို့ကို" "ဆက်ထားသည့်" "စာသား" ;

notation     = prefix-expr { infix-op prefix-expr } ;
prefix-expr  = { prefix-op } atom { data-read } ;
prefix-op    = "-" | "အနုတ်" | "~" | "!" | "*" | "&" ;   (* * and &: the value at an address, an address *)
infix-op     = "*" | "/" | "%" | "+" | "-" | "<<" | ">>" | "&" | "^" | "|"
             | "<" | ">" | "<=" | ">=" | "==" | "!=" ;
atom         = name | "(" expr ")" | literal | fn-value | size-of | long-call | sign-call ;
data-read    = "ထဲမှ" "ဒေတာ" [ fallback ] | "ထဲမှ" "အမှားအယွင်း" "ဒေတာ"
             | "ထဲမှ" "အညွှန်း" notation [ "၏" "တန်ဖိုး" ] ;   (* a row of a ဇယား, or its value *)
fallback     = "၊" "မရှိလျှင်" prefix-expr ;
fn-value     = name "ဟူသော" "လုပ်ဆောင်ချက်" ;
size-of      = slot-kind "၏" "အရွယ်အစား" ;
```

Each verb of a clause takes the operands right before it, each with the particle it
wants, and gives a value that can be an operand of a later verb. In these frames,
`value` is an operand or such a value:

```text
value "နှင့်" value "ကို" ( "ပေါင်း" | "နုတ်" | "မြှောက်" | "ဘစ်ဆုံ" | "ဘစ်ပေါင်းစု" | "ဘစ်ခြားနား" ) value-end
value "ကို" "ဘစ်ပြောင်းပြန်" value-end
value "ကို" value "ဖြင့်" ( "စား" | "အကြွင်းရှာ" | "ဘစ်ဘယ်ရွေ့" | "ဘစ်ညာရွေ့" ) value-end
value-end = ( "သည့်" | "မည့်" | "သော" ) ( "ရလဒ်" | "ရလဒ်ကောင်း" ), glued or spaced
value "သည်" value "နှင့်" ( "ညီ" | "မညီ" )
value "သည်" value "ထက်" ( "ကြီး" | "မကြီး" )
value "သည်" value "အောက်" ( "ငယ်" | "မငယ်" )
value "နှင့်" value "သည်" "တစ်ခုတည်း"
value "ကို" slot-kind "အဖြစ်" "ပြောင်းလဲသော ရလဒ်"
value "ထဲတွင်" ( "ဒေတာ" | "ရလဒ်ကောင်း" | "အောင်မြင်သည့်သတင်း"
                | "အမှားအယွင်း" | "မအောင်မြင်သည့်သတင်း" ) ( "ရှိ" | "ရှိနေ" )
value "ထဲတွင်" "ဒေတာ" ( "မရှိ" | "မရှိနေ" )
value "ဟူသော အမှားအယွင်း"                         (* an error value *)
value "၏" name                                     (* a field *)
value "၏" "အကွက်" notation                         (* a cell, counted from 0 *)
value "၏" notation ordinal "အကွက်"                 (* a cell, counted from 1 *)
value "၏" "အကွက်" notation "မှစ၍" notation "ကွက်"   (* a range: its start and its length *)
value "၏" "စာလုံး" notation                        (* a character, counted from 0 *)
value "၏" notation ordinal "စာလုံး"                (* a character, counted from 1 *)
value "၏" ( "အကွက်အရေအတွက်" | "စာလုံးအရေအတွက်" | "အညွှန်းအရေအတွက်" )
value "၏" "မိတ္တူ"                                 (* a shallow duplicate *)
value count-word "လုံး" "၏" "မိတ္တူ"               (* a deep duplicate: တစ်ခုလုံး ၏ မိတ္တူ *)
value "၏" ( "လိပ်စာ" | "တန်ဖိုး" )                  (* unsafe code: an address, the value at one *)
```

- A comparison cannot be an operand of another comparison without parentheses, in either
  tier.
- A clause used where a value goes, such as the value of a declaration, is one clause:
  a logical expression there goes in parentheses.
- `မိတ္တူ` is a name, read as the duplicate word after `၏` alone.

See [Expressions](../language/expressions.md), [Operator precedence](precedence.md),
[Conversions](../language/conversions.md), [Optional values](../language/optionals.md),
[Errors and results](../language/error-handling.md) and
[Values and objects](../language/objects.md#copies-of-an-object).

## Calls

```text
call         = long-call | first-call | short-call | preposition-call | sign-call ;

long-call    = long-head [ items(expr) ( "ဖြင့်" | "တို့ဖြင့်" ) ] long-end ;
long-head    = [ word-kinds "အစားထိုးအသုံးပြုသည့်" ] callee ( "ဟူသော" | "မည့်" ) "လုပ်ဆောင်ချက်" "ကို" ;
word-kinds   = items(kind) ( "ကို" | "တို့ကို" ) ;
callee       = name | "(" name "၏" ... ")" ;      (* a name, or a place that holds a function value *)
args         = expr { "၊" expr } ;
first-call   = [ word-kinds "အစားထိုးအသုံးပြု၍" ]
               operand [ { "၊" notation } "နှင့်" notation ] ( "ကို" | "တို့ကို" ) "သုံး၍"
               ( name short-end | long-head long-end ) ;
short-call   = name short-end ;                   (* no arguments *)
preposition-call = operand preposition-word { notation preposition-word }
               ( name short-end | name ( "ဟူသော" | "မည့်" ) "လုပ်ဆောင်ချက်" "ကို" long-end ) ;
sign-call    = name [ "::" kind-args ] "(" [ args ] ")" [ "?" ] { "(" [ args ] ")" [ "?" ] } ;
kind-args    = "<" kind { "၊" kind } ">" ;       (* ">>" closes two *)

long-end     = "လုပ်ဆောင်မည်" | "လုပ်ဆောင်သော ရလဒ်" | "လုပ်ဆောင်သော ရလဒ်ကောင်း" ;
short-end    = "မည်" | "သည့် ရလဒ်" | "သည့် ရလဒ်ကောင်း" ;
call-stmt    = call pass-on "။" ;                  (* a call that ends in လုပ်ဆောင်မည် or မည် *)
pass-on      = [ "၊" "မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်" ] ;
```

- `လုပ်ဆောင်မည်` and `မည်` end a call statement; the other ends give a value, which can
  start a clause (`... လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။`). `ရလဒ်ကောင်း` and `?` pass a
  failure on.
- `<name> ဟူသော လုပ်ဆောင်ချက်` is also the function as a value. After its `ကို`, a verb
  (`ပြန်ပေးမည်`, `ပြသမည်`, `ထည့်မည်`, `ပြင်ပမှ ခေါ်ဆိုခွင့် ...`) makes the value the object of
  that verb; each other word makes a long call.
- A preposition word is the word a parameter declares with `ဝိဘတ်`: a name, or a particle other
  than `သည်`, `ပါ`, `ဖြစ်ကာ`, `ဖြစ်ပြီး`, `ပြီးလျှင်` and `လက်ခံပြီး`; `တွင်` and `ဆီသို့`
  can be prepositions too. A preposition call is read when a function of that name declares
  prepositions and their count fits.
- In the sign form, `(` touches the name and `?` touches `)`; with a space, `f (က)` is a
  name, then a group. A `(` right after `)` calls the function value the call gives.
- Kinds for a generic function stand after `::` in the sign form and before the name in
  the word forms; `<name> "::" kind-args` alone is a function value of the generic, and
  `<struct> "::" kind-args "(" … ")"` a struct value with its kinds.

See [Calls](../language/functions.md#calls),
[Arguments first](../language/functions.md#arguments-first),
[Prepositions](../language/functions.md#prepositions),
[Calling a function value](../language/function-values.md#calling-a-function-value) and
[Passing a failure on](../language/error-handling.md#passing-a-failure-on).

## Literals

```text
literal      = number-lit | char-lit | string-lit | truth | list-lit | blank-list
             | table-lit | blank-table | struct-lit | member | null-word ;
number-lit   = [ "အခြေ" digits "ဖြင့်" ] ( number | letters ) [ "ဟူသော" slot-kind ] ;
truth        = "မှန်" | "မမှန်" ;
list-lit     = "[" notation { "၊" notation } "]" [ list-kind ] ;
word-list    = items(notation) ( "ပါသော" | "တို့ပါသော" ) "စာရင်း" [ list-kind ] ;
blank-list   = ( "[" "]" | "စာရင်းအလွတ်" ) [ list-kind ] ;
list-kind    = "ဟူသော" slot-kind ;                (* a list kind *)
table-lit    = "[" notation ":" notation { "၊" notation ":" notation } "]" [ list-kind ] ;
word-table   = items(table-row) ( "ပါသော" | "တို့ပါသော" ) "ဇယား" [ list-kind ] ;
table-row    = notation "အနေဖြင့်" notation ;
blank-table  = ( "[" ":" "]" | "ဇယားအလွတ်" ) [ list-kind ] ;   (* list-kind: a ဇယား kind here *)
struct-lit   = name "(" [ name ":" notation { "၊" name ":" notation } ] ")" ;
word-struct  = items(name "အနေဖြင့်" notation) ( "ရှိသော" | "ခုရှိသော" ) struct-count ;
struct-count = name count-word ;                  (* joined, လူတစ်ယောက်, or apart *)
member       = name "ထဲမှ" name ;                  (* အရောင်များထဲမှ နီ *)
null-word    = "အလွတ်"                             (* an optional with no data *)
             | "ကနဦးတန်ဖိုး"                       (* the initial value of the kind *)
             | "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ" ;     (* the null address, unsafe code *)
```

- `letters` are the digits above 9 (`A` to `Z`, either case), with a base above 10.
- A number takes a plain number kind after `ဟူသော`, and a list or a blank list takes a
  list kind. Characters, text and true/false values take none.
- There is no negative literal: `-` and `အနုတ်` are signs. A kind after the digits
  belongs to the literal, the sign included: `-၁၂၈ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု`.
- The member name before `ထဲမှ` can end in `များ`, and can be joined to `ထဲမှ`.

See [Numbers](../language/words.md#numbers),
[Characters and strings](../language/words.md#characters-and-strings),
[List literals](../language/lists.md#literals),
[ဇယား literals](../language/tables.md#literals),
[Struct literals](../language/structs.md#struct-literals),
[Members](../language/enums.md#members) and
[Initial values](../language/kinds.md#initial-values).

## Kinds

```text
kind         = { count } ( core | cell-kind | table-kind | fn-kind | "(" fn-kind ")" ) { level } ;
table-kind   = ( core { level } | cell-kind ) "ဖြင့်" "ညွှန်းသော" kind "ဇယား" ;
core         = [ sign ] [ digits "ဘစ်" ] base
             | name [ kind-args ]                 (* a struct, an enumeration or an alias *)
             | word-kinds "အစားထိုးအသုံးပြုသည့်" name   (* a generic with its kinds *)
             | "လိပ်စာ" ;                          (* an address of no kind *)
sign         = "လက္ခဏာရှိ" | "လက္ခဏာမဲ့" ;          (* integers alone *)
base         = "ကိန်းပြည့်" | "ဒဿမကိန်း" | "စာလုံး" | "စာသား"
             | "မှန်မှားတန်ဖိုး" | "ဘူလီယန်" ;
count        = "အကွက်" ( digits | notation ) ( "ခုရှိသော" | "ရှိသော" ) ;
level        = [ "၏" ] "စာရင်း"
             | [ "၏" ] "လိပ်စာ" ;

cell-kind    = good-side "ရှိကောင်းရှိနိုင်သော" "ဒေတာဆဲလ်"            (* an optional *)
             | ( good-side | "အောင်မြင်သည့်သတင်း" ) "သို့မဟုတ်"
               error-side "ရှိနိုင်သော" "ဒေတာဆဲလ်" ;               (* a result *)
good-side    = core { level } | "(" fn-kind ")" ;
error-side   = "မအောင်မြင်သည့်သတင်း" | "အမှားအယွင်းအနေဖြင့်" core { level } ;

fn-kind      = "လုပ်ဆောင်ချက် အမျိုးအစား"
             | sig-params "လက်ခံသော" "လုပ်ဆောင်ချက် အမျိုးအစား"
             | [ sig-params "လက်ခံပြီး" ] slot-kind "ကို" "ပြန်ပေးသော" "လုပ်ဆောင်ချက် အမျိုးအစား" ;
sig-params   = sig-param [ "လည်းကောင်း" ]
             | sig-param "လည်းကောင်း" sig-param "လည်းကောင်း" { sig-param "လည်းကောင်း" } ;
sig-param    = listed-kind "ကို" [ "ဒေတာ ပြောင်းလဲရန်" ] [ "အပိုင်အနေဖြင့်" ] [ preposition ] ;

slot-kind    = kind count-word
             | struct-count ;                     (* လူတစ်ယောက် *)
listed-kind  = slot-kind ;                        (* in a list: a function kind with parameters in parentheses *)
```

- **Measure words.** A kind in a slot takes `တစ်` and its word: declarations, loop heads,
  parameters, fields, results, the parts of a function kind, conversions, sizes, the
  default arm of a switch, and the kind after a literal. The word goes after the outermost
  kind alone. `စာလုံး` counts with `လုံး`, a struct, an enumeration or an alias with its
  own word, and each other kind with `ခု`; a word that does not fit is ZK0509.
- **Bits.** An integer kind takes 8, 16, 32 or 64 bits, a decimal kind 32 or 64.
- **Lists.** Each `စာရင်း` takes the count nearest the base that is left, and makes a
  list of that count of cells; with no count it is a plain list. A count with no list left
  for it is an error. Counts before an optional or a result go first to the lists of its
  data, before `ရှိကောင်းရှိနိုင်သော`, then to the levels after `ဒေတာဆဲလ်`.
- **Pointers.** A bare `လိပ်စာ` follows the base word or a list level; after a pointer
  level, or after the bare `လိပ်စာ` of no kind, the next level takes `၏`.
- **Function kinds.** A function kind with parameters stands bare where no list is open:
  a declaration, a result, a conversion, an alias. In a list of parameters or fields, and
  among the parameters of another function kind, it stands in parentheses (ZK0239). The
  result of a function kind is bare `လုပ်ဆောင်ချက် အမျိုးအစား` or a kind in parentheses.
  `ဒေတာ ထည့်ပေးရန်` on a parameter of a function kind is ZK0243.

See [Kinds and values](../language/kinds.md),
[Measure words](../language/kinds.md#measure-words), [Lists](../language/lists.md#list-kinds),
[Optional values](../language/optionals.md#the-optional-kind),
[Tables](../language/tables.md),
[Result kinds](../language/error-handling.md#result-kinds),
[Function values](../language/function-values.md#the-kind) and
[The pointer kind](../language/unsafe.md#the-pointer-kind).

## Where the rules depend on context

The rules above are written as a context-free grammar, but the language is not
context-free as a whole: a few productions are chosen with semantic predicates over
state that the rules do not show. The parser is a hand-written recursive-descent parser
with no backtracking; where one token does not decide the production, it scans ahead over
the tokens (unbounded lookahead, so the grammar is not LL(k) for a fixed k) and commits.
A tool that parses Zaka needs the same state; the EBNF alone is not enough.

**A name table from a pre-pass (the lexer hack).** As a C parser must know which names
are kind names, Zaka's parser must know which names are declared kinds. Before the main
pass, the lexer and the parser each walk the token stream of the file once and build three
tables; `parse_with` merges the kind names and the functions with prepositions an import brings in:

| Table | Holds | Decides |
| --- | --- | --- |
| kind names | each struct, enumeration and alias name, the stem of an enumeration name, in both spellings of `ဒေတာ` | whether a name starts a `kind` (`လူ တစ်ယောက်`) or an `expr`; whether `<stem>များထဲမှ` reads a member |
| classifiers | each word after `မျိုးပြပစ္စည်းမှာ` in the file | the lexer retags `<word>စာ` as the size token (`လူ ၂ ယောက်စာ`); else it stands as a name |
| functions with prepositions | each function whose header declares prepositions with `ဝိဘတ်`, and the prepositions of each overload | where a preposition call ends |

Since the tables come from a pre-pass, a kind can be used above its declaration. The
classifier table is per file: the retagging runs in the lexer, before imports resolve.

**Parser modes.** Some words are terminators in one context and plain tokens in another.
The parser keeps a flag per context and tests it before it takes the word:

| Mode (flag) | Set while parsing | Effect |
| --- | --- | --- |
| `in_count` | a count, `အကွက် <count> ခုရှိသော` | `ရှိသော` closes the count; no struct literal starts in it |
| `in_fill` | the value of a fill | `ဖြင့်` after a lone operand closes the value; the size follows |
| `in_list` | a list of parameters or fields | the bare function-kind production is turned off, so a function kind with parameters needs parentheses (`FnForm::NoParams`) |
| `in_word_list` | an item of a list in words | a `နှင့်` outside parentheses ends the item, and so does `ဖြင့်` after a lone operand before the end of a long call; parentheses turn it off |

**Agreement on counts.** A few productions agree with the number of items in a list: one
field or variant closes with `ပါဝင်သည်`, two or more with `တို့ပါဝင်သည်`; a list in words
takes its closing word without `တို့` after one item and with it after two or more
(`ကို` and `တို့ကို`, `ဖြင့်` and `တို့ဖြင့်`, `ပါသော` and `တို့ပါသော`), and joins its last
item with `နှင့်`; one parameter can leave out `လည်းကောင်း`, two or more each end in it;
prepositions go on each parameter or on none. These are finite, so a context-free grammar can
state them, and the rules above do, as separate alternatives.

**Count prefixes.** The count prefixes before a kind bind to its list levels from the
innermost out, a stack the parser pops as it reads each `စာရင်း`. Which level a count binds
to therefore depends on the levels that follow it, not on the prefix alone.

**Lexer lookahead.** The lexer joins multi-word reserved phrases longest first (maximal
munch), whether spaces stand between their words or not, and splits a word made of
reserved words alone. Two rules look at the neighbours as well: `ဒေတာ` is a reserved word after
`ထဲမှ`, `ထဲတွင်` and `အမှားအယွင်း` and before `ပြောင်းလဲရန်` and `ထည့်ပေးရန်`, and an
identifier elsewhere; `ဟူသော လုပ်ဆောင်ချက်` and `မည့် လုပ်ဆောင်ချက်` join into the
function-value phrase save when `အမျိုးအစား` follows, where `လုပ်ဆောင်ချက် အမျိုးအစား` opens a
function kind.

**The long call.** A long call opens with the function value phrase,
`<name> ဟူသော လုပ်ဆောင်ချက်ကို`, or with a place in parentheses before it. The parser looks
past the `ကို` at one token: a verb there makes the function value an object, and each
other token makes a call. For a place, it scans to the `)` that closes the group first.

**Order.** Imports come first in a file; the parser keeps a flag, `imports_closed`, and
reports an import after another top-level statement or inside a block.

**Static semantics, not the grammar.** A measure word that does not fit its kind
(ZK0509), a statement at the top level (ZK0326), names, scope and the kinds of values are
checked after parsing; the grammar accepts them. See [Stages](../compiler/stages.md) and
[The parser](../internals/parser.md).
