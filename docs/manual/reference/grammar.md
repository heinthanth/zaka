---
title: Grammar
description: The shape of each form of the language, rule after rule, in one place.
---

# Grammar

This page gives the grammar of Zaka: the shape of each form that the parser
reads, rule after rule. It covers shapes alone, so for what a form means, and for the
checks that the compiler runs after parsing, we follow the link at the end of each section
to the language page. [Reserved words](reserved-words.md) lists the reserved words, and
[Operator precedence](precedence.md) gives the order of the operators.

## The notation

The rules are productions in a small form of EBNF (Extended Backus–Naur Form). A
nonterminal is a lowercase English name, and a terminal is a token in quotes:

| Written | Means |
| --- | --- |
| `name = ... ;` | a production: the nonterminal `name` derives what is on the right |
| `"ကို"` | a terminal: a reserved word, a phrase token or punctuation, as we write it in a program |
| `a b` | concatenation: `a`, then `b` |
| `a \| b` | alternation: `a` or `b` |
| `[ a ]` | option: `a`, or nothing |
| `{ a }` | repetition: `a`, zero or more times |
| `( a )` | a group |
| `items(x)` | a list in words of `x`: `x`, or `x { "၊" x } "နှင့်" x`; one item takes the closing word with no `တို့`, two or more take it with `တို့` ([Lists in words](../language/words.md#lists-in-words)) |
| `(* ... *)` | a comment, not part of the production |

The lexer does some of the work before the parser sees the tokens, so the productions
leave that work out:

- **Spaces.** The lexer splits a word made of reserved words alone, and joins the words of
  a reserved phrase into one token, with or without spaces between them. So
  `ပရိုဂရမ်သည်` becomes `ပရိုဂရမ် သည်`, `ဒေတာအစုအဝေးတွင်` becomes `ဒေတာအစုအဝေး တွင်`, and
  `အောက်ပါတို့ကို လုပ်ဆောင်မည်` is one token. The rules write each phrase with the spaces
  that the other pages teach. A name still needs a space before the particle after it
  ([Words and spaces](../language/words.md#words-and-spaces)).
- **`ဒေတာ` and `အချက်အလက်`** are one word to the lexer, so `အချက်အလက်အစုအဝေး` is
  `ဒေတာအစုအဝေး` and `အချက်အလက်ဆဲလ်` is `ဒေတာဆဲလ်`. The rules write `ဒေတာ`. Alone,
  `ဒေတာ` is a name, except after `ထဲမှ`, `ထဲတွင်` and `အမှားအယွင်း` and before
  `ပြောင်းလဲရန်` and `ထည့်ပေးရန်`.
- **Endings.** A comparison word, `မဟုတ်`, `မှန်` and `မမှန်` can have an attached
  ending, `ပါ`, `သည်` or `ပါသည်` (`ကြီးသည်`, `မဟုတ်ပါ`), which the lexer folds into the
  word. The rules leave the ending out, and [Natural wording](../language/naturalness.md)
  explains which ending sounds natural where.
- **Spellings.** The rules write one spelling of most reserved words. The lexer has a
  table of second spellings for parts of reserved words, and a reserved word written with
  a second spelling of a part is the same token: the verb ending `၍` can be `ပြီး` or
  `ကာ` (`လက်ခံကာ`, `ဖြစ်ပြီး`), `သော` can be `သည့်`, `တွင်` can be `၌`, and the `သည်`
  that ends a verb can be `၏` (`ဖြစ်၏`, `ပါဝင်၏`), while `၏` on its own is still "of". The four
  spellings of `done` are one row of that table too. The spoken forms of the same table count
  once a program allows them ([Spoken Burmese](../language/spoken.md)), and
  [Second spellings](../language/words.md#second-spellings) lists the parts.

A few rules also depend on what the file declares and on the form around them, and
[Where the rules depend on context](#where-the-rules-depend-on-context) lists these.

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
import       = string-lit "မှ" items(name) ( "ကို" | "တို့ကို" ) "ထည့်သွင်းမည်" "။"
             | string-lit "ကို" "ထည့်သွင်းမည်" "။"
             | string-lit "ကို" "ပြန်လည်" "မျှဝေမည်" "။"
             | items(name) ( "ကို" | "တို့ကို" ) "အများသုံးနိုင်ရန်" "ပြန်လည်" "မျှဝေမည်" "။" ;
start-block  = "ပရိုဂရမ်" "သည်" [ params "လက်ခံပြီး" ] fn-open "။"
               { stmt }
               "ပရိုဂရမ်" done "။" ;
done         = "ပြီးဆုံးပါပြီး" | "ပြီးဆုံးပါပြီ" | "ပြီးပါပြီး" | "ပြီးပါပြီ" ;
```

The last two forms are shares. For an import or a share after another statement, or
inside a block, the parser reads it and reports an error. See
[Modules](../language/modules.md) and
[The program start block](../language/functions-start-block.md).

## Declarations

```text
declaration  = var-decl | fn-decl | external-fn | builtin-fn | struct-decl | handle-decl
             | enum-decl | alias-decl | builtin-kind | trait-decl | trait-block ;

var-decl     = [ "အများသုံး" ] ( "ကိန်းရှင်" | "ကိန်းသေ" ) name
               ( "သည်" decl-head "ဖြစ်ပါစေ"
               | "သည်" compiler-mark slot-kind "ဖြစ်သည်"     (* a constant of the core module *)
               | "ထဲသို့" clause "ကို" ( "ကူးထည့်မည်" | "အကုန်ကူးထည့်မည်" ) ) "။" ;
decl-head    = slot-kind [ "ဖြစ်၍" "တန်ဖိုး" "သည်" clause ]
             | clause ;                           (* the kind comes from the value *)

fn-decl      = name [ sign-params ] "ဟူသော" fn-marks "လုပ်ဆောင်ချက်" "သည်"
               [ c-callable ] [ word-params "သည်" ] [ phrases ]
               [ params "လက်ခံပြီး" ] fn-open "။"
               [ notes ]
               { stmt }
               block-close "။" ;
fn-marks     = { "အများသုံး" | "စိတ်မချရသော" | "ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော" } ;
c-callable   = string-lit "အမည်ဖြင့်" "ပြင်ပမှ ခေါ်ဆိုနိုင်သော" "လုပ်ဆောင်ချက်" [ "တစ်ခု" ]
               "ဖြစ်၍" ;                          (* a function that C can call, under that C name *)

sign-params  = "<" kind-param { "၊" kind-param } ">" ;     (* no space before "<" *)
kind-param   = name [ ":" trait-ref { "နှင့်" trait-ref } ] ;
word-params  = word-params-open "၎င်း" ;
word-params-open = items("ယေဘုယျဒေတာအမျိုးအစား" name) ( "ကို" | "တို့ကို" )
               "အစားထိုးသုံးနိုင်၍" ;
trait-ref    = hint-use                           (* a trait with a written-as hint *)
             | word-kinds "အစားထိုးအသုံးပြုသည့်" ( name | threads-mark )
             | ( name | threads-mark ) [ kind-args ] ;
notes        = "ဤနေရာတွင်" "သတိပြုရန်မှာ"
               { name "သည်" slot-kind "ဖြစ်သည်" "။"                       (* a local kind *)
               | name "တွင်" items(trait-ref "ဂုဏ်သတ္တိ") [ "တို့" ] "ရှိသည်" "။" }
               "သတိပြုစရာများ" done "။" ;

trait-decl   = name [ sign-params ] "ဟူသည့်" [ compiler-mark ] [ "အများသုံး" ] "ဂုဏ်သတ္တိ"
               ( [ "သည်" hint "ရေးသားနိုင်ပြီး" "၎င်း" ]     (* with sign-params *)
               | "သည်" word-params-open [ hint "ရေးသားနိုင်ပြီး" ] "၎င်း" )
               "တွင်" "အောက်ပါ" "လုပ်ဆောင်ချက်များ" "ပါဝင်ရမည်" "။"
               [ notes ]
               { fn-head }
               "ဂုဏ်သတ္တိ" "တွင်" "ပါသော" "လုပ်ဆောင်ချက်များ" done "။" ;
fn-head      = name "ဟူသော" "လုပ်ဆောင်ချက်" "သည်"
               ( params ( "လက်ခံမည်" | "လက်ခံပြီး" result ) | result ) "။"
             | name "ဟူသော" "လုပ်ဆောင်ချက်" "သည်" params "လက်ခံပြီး" fn-open "။"   (* a default *)
               { stmt } block-close "။" ;
compiler-mark = "ကွန်ပိုင်လာထဲတွင်" "တစ်ပါတည်းပါဝင်သော" ;   (* the core module alone, ZK0349 *)
hint         = "ရေးသားရာတွင်" hint-word { hint-word } "ဟု" ;   (* ဟု can be glued to the last word *)
hint-word    = name | reserved ;                  (* reserved: a reserved word; no signs *)
trait-block  = kind "တွင်" trait-ref "ဂုဏ်သတ္တိ" "ရှိစေရန်" "အောက်ပါ" "လုပ်ဆောင်ချက်များ"
               "ကို" "ထည့်သွင်းမည်" "။"
               { fn-decl }
               [ trait-ref "ဂုဏ်သတ္တိ" "၏" "အခြားလုပ်ဆောင်ချက်များကိုလည်း" "မူရင်းအတိုင်း"
                 "ဆက်ခံမည်" "။" ]
               "ထည့်သွင်းမည့်" "လုပ်ဆောင်ချက်များ" done "။" ;
fn-open      = "အောက်ပါတို့ကို လုပ်ဆောင်မည်"
             | "အောက်ပါတို့ကို လုပ်ဆောင်ကာ" result ;
result       = slot-kind "ကို" [ "အပိုင်အနေဖြင့်" ] "ပြန်ပေးမည်" ;
block-close  = "လုပ်ဆောင်စရာများ" done ;

params       = param [ "လည်းကောင်း" ]
             | param "လည်းကောင်း" param "လည်းကောင်း" { param "လည်းကောင်း" } ;
param        = name "ဟူသော" listed-kind "ကို" markers ;
markers      = [ "ဒေတာ ပြောင်းလဲရန်" | "ဒေတာ ထည့်ပေးရန်" ] [ "အပိုင်အနေဖြင့်" ] [ preposition ] ;
preposition  = "ဝိဘတ်" string-lit "ကို" "သုံး၍" ;   (* the string has names and particles *)
phrases      = ( start-phrase | end-phrase ) "ကို" "သုံး၍"
             | start-phrase "နှင့်" end-phrase "တို့ကို" "သုံး၍" ;
start-phrase = "အစ စကားစု" string-lit ;          (* the words before the first argument *)
end-phrase   = "အဆုံး စကားစု" string-lit ;        (* the words right before the function name *)

external-fn  = name "ဟူသော" fn-marks "လုပ်ဆောင်ချက်" "သည်"
               origin "လုပ်ဆောင်ချက်" [ "တစ်ခု" ]
               ( "ဖြစ်သည်" | "ဖြစ်၍" ext-tail ) "။" ;
origin       = "ပြင်ပမှလာသော"                     (* a host function *)
             | string-lit "မှလာသော" ;             (* a C function of that shared lib *)
ext-tail     = [ phrases ] ext-params ;
ext-params   = c-params "လက်ခံမည်"                (* no result *)
             | c-params "လက်ခံပြီး" result
             | result ;
c-params     = params
             | param "လည်းကောင်း" { param "လည်းကောင်း" } "အခြားတန်ဖိုးများနှင့်အတူ" ;

builtin-fn   = name [ sign-params ] "ဟူသော" fn-marks "လုပ်ဆောင်ချက်" "သည်"
               compiler-mark "လုပ်ဆောင်ချက်"
               ( "ဖြစ်သည်"
               | "ဖြစ်၍" [ word-params "သည်" ] [ phrases ]
                 params ( "လက်ခံမည်" | "လက်ခံပြီး" result ) ) "။" ;
builtin-kind = name "သည်" compiler-mark [ "အများသုံး" ] "ဒေတာအမျိုးအစား"
               ( "ဖြစ်သည်" | "ဖြစ်၍" [ word-params "ကို" ] hint "ရေးသားနိုင်သည်" ) "။" ;

struct-decl  = name [ sign-params ] "ဟူသော" [ "အများသုံး" ] [ threads-mark ] [ "မျှဝေနိုင်သော" ]
               "ဒေတာအစုအဝေး"
               ( "တွင်" | "သည်" [ struct-mark "ဒေတာအစုအဝေး" "ဖြစ်၍" ] [ word-params-open ] "၎င်း" "တွင်" )
               fields ( closer | hinted-closer ) "။" ;
hinted-closer = ( "ပါဝင်ပြီး" | "တို့ပါဝင်ပြီး" ) "မျိုးပြပစ္စည်းမှာ" classifier
               "ဖြစ်၍" hint "ရေးသားနိုင်သည်" ;
struct-mark  = string-lit "အမည်ဖြင့်" "ပြင်ပမှ ယူသုံးနိုင်သော"   (* a struct a lib gives to C *)
             | "ကွန်ပိုင်လာထဲတွင်" "တစ်ပါတည်းပါဝင်သော" ;         (* a struct the compiler gives *)
handle-decl  = name "ဟူသော" [ "အများသုံး" ] "ဒေတာအစုအဝေး" "သည်" "ပြင်ပမှ လာသော" "ဒေတာအစုအဝေး"
               ( "ဖြစ်သည်" | "ဖြစ်၍" "၎င်း" "တွင်" fields ( "ပါဝင်သည်" | "တို့ပါဝင်သည်" ) ) "။" ;
fields       = items(field) ;
field        = name "ဟူသော" listed-kind ;

enum-decl    = name [ sign-params ] "ဟူသော" [ "အများသုံး" ] [ threads-mark ] "ဒေတာအမျိုးအစားကွဲ"   (* threads-mark: ZK0263 *)
               ( "တွင်" | "သည်" word-params "တွင်" )   (* kind parameters: members with data *)
               items(variant) closer "။" ;
variant      = [ "ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်" ] [ "တန်ဖိုးအနေဖြင့်" expr "ရှိသည့်" ]
               [ "ဒေတာအနေဖြင့်" slot-kind "ရှိသည့်" ] name ;   (* a value or data, not both: ZK0353 *)

closer       = ( "ပါဝင်သည်" | "ပါဝင်ပြီး" word-clause )          (* after one field or variant *)
             | ( "တို့ပါဝင်သည်" | "တို့ပါဝင်ပြီး" word-clause ) ;  (* after two or more *)
word-clause  = "မျိုးပြပစ္စည်းမှာ" classifier "ဖြစ်သည်" ;

alias-decl   = [ "အများသုံး" ] "ဒေတာအမျိုးအစား" name
               ( [ sign-params ] "ကို" | "သည်" word-params "ကို" ) kind
               ( "ဟုလည်း မှတ်ယူနိုင်ပါသည်" | "ဟုလည်း မှတ်ယူနိုင်ပြီး" word-clause ) "။" ;
```

- With two or more parameters, we end each one with `လည်းကောင်း`, and each parameter has
  a preposition or none. A function signature starts with the function name, as a struct
  declaration does, and we can write its marks (`fn-marks`) between `ဟူသော` and
  `လုပ်ဆောင်ချက်` in a free order. The program start block has no marks, and the comptime
  mark goes on a function with statements alone.
- The data kind of an alias has no measure word. For a function, a struct, an enumeration
  or an alias inside a block, the parser reads it and reports an error, but a function
  with a block can be declared inside another function.
- A declaration has its kind parameters in one form: `sign-params` after the name or
  `word-params` after `သည်`. A trait has one kind parameter or more. The functions of a
  trait block are functions of its file. A trait or a trait block inside a block is
  ZK0247.
- We write the marks of a struct in the order of the rule: `အများသုံး`, then
  `threads-mark`, then `မျှဝေနိုင်သော`. In a trait, the compiler mark comes before
  `အများသုံး`. The compiler mark, `builtin-fn` and `builtin-kind` belong to the core module
  alone (ZK0349).
- A written-as hint (`hint`) gives the words that a use of a generic struct, a data kind
  of the core module or a trait can be written with. Each word that is a kind parameter of
  the declaration is a slot, where a use writes a data kind, and the last word is the
  declared name ([Data kinds](#data-kinds) shows the uses).

See [Declarations and assignment](../language/declarations.md#declarations),
[Functions](../language/functions.md), [Structs](../language/structs.md),
[Enumerations](../language/enums.md), [Kind aliases](../language/aliases.md),
[Generics](../language/generics.md) and
[C functions](../language/functions-c.md#c-functions).

## Statements

```text
stmt         = var-decl | fn-decl | block | if-stmt | switch-stmt
             | while-loop | counted-loop | each-loop | generic-loop | channel-loop
             | lock-block | select | break | continue | return | call-stmt | verb-stmt
             | collector-stmt ;

block        = block-open "။" { stmt } block-close "။" ;
block-open   = "အောက်ပါတို့ကို လုပ်ဆောင်မည်"
             | "အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်" ;   (* unsafe code *)
branch       = block | stmt ;                     (* one statement in place of a block *)

if-stmt      = "အကယ်၍" [ "ကွန်ပိုင်းလုပ်ချိန်တွင်" ] expr then branch
               { "ထိုသို့မဟုတ်ဘဲ" expr then branch }
               [ "သို့မဟုတ်ပါက" branch ] ;
then         = "လျှင်" | "ဖြစ်လျှင်" ;

switch-stmt  = value "သည်" ( cases "ဖြစ်လျှင်" | member-test ) branch
               { ( cases "ဖြစ်လျှင်" | member-test ) branch }
               [ "တခြား" slot-kind "ဖြစ်လျှင်" branch ] ;
member-test  = [ name "ဟူသည့်" ] name "အဖြစ်သို့" "ပြောင်းလဲနိုင်လျှင်" ;   (* a member, and a name for its data *)
cases        = expr | expr { "၊" expr } "နှင့်" expr "တို့" ;

while-loop   = expr while branch ;
while        = "နေသရွေ့" | "ဖြစ်နေသရွေ့" ;
counted-loop = expr "မှ" expr "အထိ" [ expr "စီ" ] name "အနေဖြင့်" [ parallel ]
               ( "ရေတွက်၍" | "တိုး၍" | "လျှော့၍" ) branch ;
each-loop    = expr "၏" each-part "ကို" [ name "နှင့်" ] name "အနေဖြင့်" [ parallel ]
               "ယူ၍" branch ;
each-part    = "အကွက်တိုင်း" | "စာလုံးတိုင်း" | "အညွှန်းတိုင်း" | "အတန်းတိုင်း" ;
generic-loop = "ကိန်းရှင်" name "သည်" decl-head ( "ဖြစ်၍" | "ဖြစ်ကာ" | "ဖြစ်ပြီး" ) expr while
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

collector-stmt = mem "ရှိ" "မလိုအပ်သော" "အရာများကို" "ရှင်းလင်းမည်" "။"   (* collect now *)
             | "အလိုအလျောက်" mem "ရှင်းလင်းမှုကို"
               ( "ခေတ္တရပ်မည်"                       (* pause the collections *)
               | "ပြန်လည်စတင်မည်"                     (* resume them *)
               | notation "ဘိုက်" "နှင့်အထက်မှ" "လုပ်ဆောင်မည်" ) "။" ;   (* the threshold *)
mem          = "မင်မိုရီ" | "မှတ်ဉာဏ်" ;
```

- The heap sentences are [sentence calls](#calls) of functions of the core module
  (`core/heap.zaka` in the stdlib), which the parser turns into their statements, and in
  these sentences we can write `မင်မိုရီ` as `မှတ်ဉာဏ်`.
- When a branch of an `if` arm is a single statement with no block, that statement cannot
  be another `if`, and in a switch arm it cannot be an `if` or a switch.
- The `ပြီးလျှင်` statement of a generic loop comes last in its block. For a switch arm
  after the default arm, the parser reads it and reports an error.
- The collector statements run and tune the collector of the VM. `mem` is a reserved
  word, but the parser reads the other words through their text, so each of these words
  can be glued to the next or written apart, and none of them is reserved.
  [Threads](#threads) has `parallel`, `channel-loop`, `lock-block` and `select`.
- `value` is an operand of a clause, or the value that a verb of the clause makes
  ([Expressions](#expressions)). We write the operands of a statement verb in the order
  that the verb lists them, and none can be left over before the verb.

See [Statements](../language/statements.md), [Lists](../language/lists.md), [Tables](../language/tables.md),
[Text](../language/text.md#changing-text), [Switch](../language/enums.md#switch) and
[Heap statements](../language/heap.md#heap-statements).

## Expressions

An expression has three tiers. The logical tier joins clauses with `ဖြစ်၍`, `သို့မဟုတ်`
and `မဟုတ်`. A clause has operands, each with the particle after it, and verbs. An
operand is a notation, which is an expression written with signs.

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

notation     = prefix-expr { infix-op prefix-expr } { ei-read } [ "ဘိုက်စာ" ] ;   (* with ဘိုက်စာ: a size in octets *)
ei-read      = "၏" ( "တန်ဖိုး" | "လိပ်စာ" | "အကွက်" notation [ "မှစ၍" notation "ကွက်" ] | name ) ;
prefix-expr  = { prefix-op } atom { data-read } ;
prefix-op    = "-" | "အနုတ်" | "~" | "!" | "*" | "&" ;   (* * and &: the value at an address, an address *)
infix-op     = "*" | "/" | "%" | "+" | "-" | "<<" | ">>" | "&" | "^" | "|"
             | "<" | ">" | "<=" | ">=" | "==" | "!=" ;
atom         = name | "(" expr ")" | literal | fn-value | size-of | size | long-call | sign-call
             | collector-stats ;
collector-stats = "အလိုအလျောက်" mem "ရှင်းလင်းသည့်" "အခြေအနေ" ;   (* the stats of the collector *)
data-read    = "ထဲမှ" "ဒေတာ" [ fallback ] | "ထဲမှ" "အမှားအယွင်း" "ဒေတာ"
             | "ထဲမှ" "အညွှန်း" notation [ "၏" "တန်ဖိုး" ]   (* a row of a ဇယား, or its value *)
             | receive | par-map ;               (* see Threads *)
fallback     = "၊" "မရှိလျှင်" prefix-expr ;
fn-value     = name "ဟူသော" "လုပ်ဆောင်ချက်" ;
size-of      = slot-kind "၏" "အရွယ်အစား" ;
size         = kind notation count-word ;   (* count-word: a measure word and စာ, as ခုစာ *)
```

Each verb of a clause takes the operands right before it, each with the particle it
wants, and gives a value that can be an operand of a later verb. In these frames,
`value` is an operand or such a value:

```text
value "နှင့်" value "၏" ( "ပေါင်းလဒ်" | "နုတ်လဒ်" | "မြှောက်လဒ်" )
value "နှင့်" value "၏" "ဘစ်အလိုက်" ( "ဆုံလဒ်" | "ပေါင်းစုလဒ်" | "ခြားနားလဒ်" )
value "၏" "ဘစ်အလိုက်" "ပြောင်းပြန်ပြောင်းလဒ်"
value "ကို" value "ဖြင့်" "စားသော" ( "စားလဒ်" | "စားကြွင်း" )
value "ကို" ( "ဘယ်သို့" | "ညာသို့" ) notation "ဘစ်" "ရွေ့သော" "ရလဒ်"
value "သည်" value "နှင့်" ( "ညီ" | "မညီ" )
value "သည်" value "ထက်" ( "ကြီး" | "မကြီး" )
value "သည်" value "အောက်" ( "ငယ်" | "မငယ်" )
value "နှင့်" value "သည်" "တစ်ခုတည်း"
value "ကို" slot-kind "အဖြစ်" "ပြောင်းလဲသော ရလဒ်"
value "ကို" name "အဖြစ်" "ပြောင်းလဲသော ရလဒ်"           (* the data of a member *)
value "သည်" [ name "ဟူသည့်" ] name "အဖြစ်သို့" "ပြောင်းလဲနိုင်"   (* whether a value is a member *)
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
- In a notation, `၏ တန်ဖိုး`, `၏ လိပ်စာ`, `၏ အကွက် …` and a field `၏ <name>` cover
  the whole notation before them, as `ဘိုက်စာ` does, so `p + ၁ ၏ တန်ဖိုး` is the value at
  `p + ၁`. A name after `၏` is a field unless a function or a preposition uses that name,
  and `x ၏ {f}` reads the field all the same. When an operator follows such a read, the
  parser leaves the read to the clause, which reports the operator (ZK0209). The other
  forms after `၏` (characters, counts, duplicates, a cell counted from 1) are clauses.
- Where a value goes, such as the value of a declaration, we can write one clause alone,
  so a logical expression there needs parentheses.
- `မိတ္တူ` is a name, and it means a duplicate after `၏` alone.

See [Expressions](../language/expressions.md), [Operator precedence](precedence.md),
[Conversions](../language/conversions.md), [Optional values](../language/optionals.md),
[Errors and results](../language/error-handling.md) and
[Values and objects](../language/objects.md#duplicating-an-object).

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
preposition-call = [ start-words ] clause [ preposition-word ] { [ "၊" ] clause [ preposition-word ] }
               [ end-words ] ( name short-end | name ( "ဟူသော" | "မည့်" ) "လုပ်ဆောင်ချက်" "ကို" long-end )
             | [ start-words ] [ end-words ] name short-end ;   (* phrases and no arguments *)
sign-call    = name [ "::" kind-args ] "(" [ args ] ")" [ "?" ] { "(" [ args ] ")" [ "?" ] } ;
kind-args    = "<" kind { "၊" kind } ">" ;       (* ">>" closes two *)

long-end     = "လုပ်ဆောင်မည်" | "လုပ်ဆောင်သော ရလဒ်" | "လုပ်ဆောင်သော ရလဒ်ကောင်း"
             | thread-end ;                       (* see Threads *)
short-end    = "မည်" | value-end | value-end-pass ;
value-end    = "သည့် ရလဒ်" | "မည့် ရလဒ်" | "သော ရလဒ်" ;
value-end-pass = "သည့် ရလဒ်ကောင်း" | "မည့် ရလဒ်ကောင်း" | "သော ရလဒ်ကောင်း" ;
call-stmt    = call pass-on "။" ;                  (* a call that ends in လုပ်ဆောင်မည်, မည် or a thread-end statement *)
pass-on      = [ "၊" "မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်" ] ;
```

- We use `လုပ်ဆောင်မည်` and `မည်` to end a call statement. The other ends make a value,
  which can start a clause (`... လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။`). With `ရလဒ်ကောင်း` and
  `?`, the call passes a failure on. A value end can be glued to the function name, as in
  `ဖတ်သည့်ရလဒ်` and `စစ်သော ရလဒ်ကောင်း`.
- `<name> ဟူသော လုပ်ဆောင်ချက်` is also the function as a value. When a verb follows its
  `ကို` (`ပြန်ပေးမည်`, `ပြသမည်`, `ထည့်မည်`, `ပြင်ပမှ ခေါ်ဆိုခွင့် ...`), the function value is
  the object of that verb, and with each other word after `ကို`, it starts a long call.
- A preposition word is the word a parameter declares with `ဝိဘတ်`: a name, or a particle other
  than `သည်`, `ပါ`, `ဖြစ်ကာ`, `ဖြစ်ပြီး`, `ပြီးလျှင်` and `လက်ခံပြီး`; `တွင်` and `ဆီသို့`
  can be prepositions too. The parser reads a call with prepositions when a function of
  that name declares prepositions or phrases, and the arguments have prepositions in the
  slots of its parameters: a preposition after each argument whose parameter has one, and
  none after the others. Two arguments with no preposition between them touch or have `၊`
  between them. Each argument is a clause: the parser cuts the words before the function
  at its prepositions and keeps each cut whose pieces are each one clause, and two such
  cuts are ZK0260. `start-words` and `end-words` are the words of the phrases that the
  function signature declares with `အစ စကားစု` and `အဆုံး စကားစု`.
- In the sign form, `(` touches the name and `?` touches `)`, so with a space, `f (က)` is
  a name and then a group. A `(` right after `)` calls the function value that the call
  returns.
- We write the data kinds for a generic function after `::` in the sign form and before
  the name in the word forms. `<name> "::" kind-args` alone is a function value of the
  generic, and `<struct> "::" kind-args "(" … ")"` is a struct value with its data kinds.

See [Calls](../language/functions-calls.md),
[Arguments first](../language/functions-calls.md#arguments-first),
[Prepositions](../language/functions-sentences.md#prepositions),
[Starting and ending phrases](../language/functions-sentences.md#starting-and-ending-phrases),
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
member       = [ word-kinds ] name [ sign-kinds "များ" ] "ထဲမှ" name [ "(" expr ")" ]   (* ပုံများထဲမှ စက်ဝိုင်း(၅) *)
             | "ဒေတာအနေဖြင့်" notation "ရှိသော" [ word-kinds ] name "ထဲမှ" name ;
             (* word-kinds: <kinds> (ကို|တို့ကို) အစားထိုးအသုံးပြုသည့်; sign-kinds: <kind၊ …> *)
null-word    = "အလွတ်"                             (* an optional with no data *)
             | "ကနဦးတန်ဖိုး"                       (* the initial value of the kind *)
             | "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ" ;     (* the null address, unsafe code *)
```

- `letters` are the digits above 9 (`A` to `Z`, either case), with a base above 10.
- After `ဟူသော`, a number can have a plain number kind, and a list or a blank list can
  have a list kind. Characters, text and true/false values cannot have one.
- There is no negative literal, because `-` and `အနုတ်` are signs. A data kind after the
  digits belongs to the whole literal, sign included: `-၁၂၈ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု`.
- The member name before `ထဲမှ` can end in `များ`, and can be joined to `ထဲမှ`.

See [Numbers](../language/words.md#numbers),
[Characters and strings](../language/words.md#characters-and-strings),
[List literals](../language/lists.md#literals),
[ဇယား literals](../language/tables.md#literals),
[Struct literals](../language/structs.md#struct-literals),
[Members](../language/enums.md#members) and
[Initial values](../language/kinds.md#initial-values).

## Data kinds

```text
kind         = { count } ( core | cell-kind | fn-kind | "(" fn-kind ")" ) { level }
             | atomic-kind ;                      (* see Threads *)
core         = [ sign ] [ digits "ဘစ်" ] base
             | name [ kind-args ]                 (* a struct, an enumeration, an alias, or စာရင်း and ဇယား *)
             | word-kinds "အစားထိုးအသုံးပြုသည့်" name   (* a generic with its data kinds *)
             | hint-use                           (* a hint that starts with a word *)
             | "လိပ်စာ" ;                          (* an address of no data kind *)
sign         = "လက္ခဏာရှိ" | "လက္ခဏာမဲ့" ;          (* integers alone *)
base         = "ကိန်းပြည့်" | "ဒဿမကိန်း" | "စာလုံး" | "စာသား"
             | "မှန်မှားတန်ဖိုး" | "ဘူလီယန်" ;
count        = "အကွက်" ( digits | notation ) ( "ခုရှိသော" | "ရှိသော" ) ;
level        = [ "၏" ] "စာရင်း"
             | [ "၏" ] "လိပ်စာ"
             | hint-rest ;                        (* the parts of a hint after its first slot *)
hint-use     = hint-part { hint-part } ;          (* the parts of one hint, in its order *)
hint-rest    = hint-part { hint-part } ;
hint-part    = hint-word | kind ;                 (* the hint's own word, or a data kind in a slot *)
table-kind   = kind "ဖြင့်" "ညွှန်းသော" kind "ဇယား"   (* the hint of ဇယား in the core module, a level *)
             | "ဇယား" kind-args ;                 (* the sign form, ဇယား<စာသား၊ ကိန်းပြည့်> *)

cell-kind    = good-side "ရှိကောင်းရှိနိုင်သော" "ဒေတာဆဲလ်"            (* an optional *)
             | ( good-side | "အောင်မြင်သည့်သတင်း" ) "သို့မဟုတ်"
               error-side "ရှိနိုင်သော" "ဒေတာဆဲလ်" ;               (* a result *)
good-side    = core { level } | "(" fn-kind ")" ;
error-side   = "မအောင်မြင်သည့်သတင်း" | "အမှားအယွင်းအနေဖြင့်" core { level } ;

fn-kind      = "လုပ်ဆောင်ချက် အမျိုးအစား"
             | [ kind-phrases "သုံး၍" ] sig-params "လက်ခံသော" "လုပ်ဆောင်ချက် အမျိုးအစား"
             | [ kind-phrases "သုံး၍" ] [ sig-params "လက်ခံပြီး" ] slot-kind "ကို" "ပြန်ပေးသော" "လုပ်ဆောင်ချက် အမျိုးအစား"
             | kind-phrases "သုံးသော" "လုပ်ဆောင်ချက် အမျိုးအစား" ;   (* phrases alone *)
kind-phrases = ( start-phrase | end-phrase ) "ကို"
             | start-phrase "နှင့်" end-phrase "တို့ကို" ;
sig-params   = sig-param [ "လည်းကောင်း" ]
             | sig-param "လည်းကောင်း" sig-param "လည်းကောင်း" { sig-param "လည်းကောင်း" } ;
sig-param    = listed-kind "ကို" [ "ဒေတာ ပြောင်းလဲရန်" | "ဒေတာ ထည့်ပေးရန်" ] [ "အပိုင်အနေဖြင့်" ] [ preposition ] ;

slot-kind    = kind count-word
             | struct-count ;                     (* လူတစ်ယောက် *)
listed-kind  = slot-kind ;                        (* in a list: a function kind with parameters in parentheses *)
```

- **Measure words.** A data kind in a slot needs `တစ်` and its measure word (classifier):
  in declarations, loop heads, parameters, fields, results, the parts of a function kind,
  conversions, sizes, the default arm of a switch, and after a literal. The measure word
  goes after the outermost data kind alone. We count `စာလုံး` with `လုံး`, a struct, an
  enumeration or an alias with the measure word it declares, and each other data kind with
  `ခု`, and a word that does not fit is ZK0509.
- **Bits.** An integer kind has 8, 16, 32 or 64 bits, and a decimal kind 32 or 64.
- **Written-as hints.** A use of a hint follows the hint that its declaration gives: the
  hint's words in order, with a data kind in each slot. A hint that starts with a slot is
  a level after the data kind of that slot, as `ဖြင့် ညွှန်းသော V ဇယား` is after `K`, with
  no `၏` before it. Where several hints fit, the parser reads the longest one. The list
  and the table of the core module have the hints `A စာရင်း` and
  `K ဖြင့် ညွှန်းသော V ဇယား`. Of the data kinds of the core module, these two alone have
  the sign form too, `စာရင်း<ကိန်းပြည့်>` and `ဇယား<စာသား၊ ကိန်းပြည့်>`.
- **Lists.** Each `စာရင်း` uses the count nearest the base that is still free, and makes a
  list of that count of cells, or a plain list when no count is left. A count with no list
  left for it is an error. Counts before an optional or a result go first to the lists of
  its data, before `ရှိကောင်းရှိနိုင်သော`, then to the levels after `ဒေတာဆဲလ်`.
- **Pointers.** A bare `လိပ်စာ` follows the base word or a list level. After a pointer
  level, or after the bare `လိပ်စာ` of no data kind, the next level needs `၏`.
- **Function kinds.** We write a function kind with parameters bare where no list is
  open: in a declaration, a result, a conversion or an alias. In a list of parameters or
  fields, and among the parameters of another function kind, it goes in parentheses
  (ZK0239). The result of a function kind is a bare `လုပ်ဆောင်ချက် အမျိုးအစား` or a data
  kind in parentheses. `လက်ခံသည့်`, `ပြန်ပေးသည့်` and `သုံးသည့်` are the same words as
  `လက်ခံသော`, `ပြန်ပေးသော` and `သုံးသော`.

See [Data kinds and values](../language/kinds.md),
[Measure words](../language/kinds.md#measure-words), [Lists](../language/lists.md#list-kinds),
[Optional values](../language/optionals.md#the-optional-kind),
[Tables](../language/tables.md),
[Result kinds](../language/result-kinds.md),
[Function kinds](../language/function-values.md#function-kinds) and
[The pointer kind](../language/pointers.md#the-pointer-kind).

## Threads

The forms of threads (`အပြိုင်လုပ်ငန်း`), channels and atomic numbers use words that are
not reserved, so the parser reads them through their text. The rules of the other sections
refer to these productions.

```text
thread-end   = [ "သီးသန့်" ] "အပြိုင်" "လုပ်ဆောင်မည်"              (* a statement that starts the thread *)
             | [ "သီးသန့်" ] "အပြိုင်" "လုပ်ဆောင်သော" "လုပ်ငန်း" ;   (* a value: the thread *)
parallel     = "အပြိုင်"
             | "အပြိုင်လုပ်ငန်း" notation "ခုခွဲ၍" ;     (* the count of threads *)
par-map      = "၏" "အကွက်တိုင်း" "ကို" name "ဟူသော" "လုပ်ဆောင်ချက်" "ဖြင့်" parallel
               "လုပ်ဆောင်သော" "ရလဒ်များ" ;             (* after the list it maps *)
receive      = "မှ" received "ဒေတာ" ;                 (* after the channel *)
received     = "ရရှိသော" | "လက်ခံရရှိသော" ;
channel-loop = expr "မှ" received ( "ဒေတာတိုင်း" "ကို" | "ဒေတာတိုင်းကို" )
               name "အနေဖြင့်" "ယူ၍" branch ;
lock-block   = name "ကို" ( "ခေတ္တ" | "ခဏ" ) "ယူ၍" branch ;
select       = "အောက်ပါ" "ဒေတာလမ်းကြောင်းများ" "ထဲမှ" "ဦးစွာရောက်လာသော" "ဒေတာ" "ကို"
               "စောင့်" "မည်" "။" channel-arm { channel-arm } [ time-arm | idle-arm ] ;
channel-arm  = name "ထဲတွင်" "ဒေတာ" "ရှိ" "လျှင်" branch ;
time-arm     = value ( "မီလီစက္ကန့်" | "စက္ကန့်" ) "အတွင်း" "ဒေတာ" "မရောက်လာလျှင်" branch ;
idle-arm     = "မည်သည့်လမ်းကြောင်းတွင်မှ" "ဒေတာ" "မရှိ" "လျှင်" branch ;
threads-mark = "အပြိုင်လုပ်ငန်းများတွင်" "အသုံးပြုနိုင်သော" ;

atomic-kind  = "အက်တမ်မစ်" "ဖြစ်သော" kind ;
thread-kind  = kind "ပြန်ပေးမည့်" "အပြိုင်လုပ်ငန်း"      (* a level *)
             | "အပြိုင်လုပ်ငန်း" ;                      (* a thread whose function returns nothing *)
channel-kind = kind [ digits name ] ( "ပို့နိုင်သော" | "ပို့ဆောင်နိုင်သော" ) "ဒေတာလမ်းကြောင်း" ;
```

- A long call, and a call written arguments first with a long head, starts a thread when
  we end it with `thread-end`. Its words are written apart: a glued `အပြိုင်လုပ်ဆောင်မည်` is no
  call end (ZK0208).
- `parallel` goes right before the verb of a counted loop or an each loop (`ရေတွက်၍`,
  `တိုး၍`, `လျှော့၍` or `ယူ၍`), and before `လုပ်ဆောင်သော ရလဒ်များ` in a parallel map. We can
  write `ခုခွဲ၍` glued or as `ခု ခွဲ၍`.
- `receive` and `par-map` are reads, as the data reads of [Expressions](#expressions) are.
- The arms of a `select` end at the first statement that is no arm, as the arms of a
  switch do. A `select` with no `channel-arm` is ZK0264, and one with both a `time-arm`
  and an `idle-arm` is ZK0265 ([Threads](../language/threads.md#waiting-on-several-channels)).
- In a lock block, we can glue the last two words, as in `ခေတ္တယူ၍`, or write them apart.
  The glued form has `ယူ၍` alone, while the form written apart takes each spelling of
  `ယူ၍`.
- We can write the two words of `threads-mark` glued or apart. The mark is for a shared
  object kind, and on another struct it is the warning ZK0493. In `struct-decl` it goes
  between `အများသုံး` and `မျှဝေနိုင်သော`, and in `trait-ref` it is the name of a built-in
  trait.
- `thread-kind` and `channel-kind` are written-as hints of the core module
  ([Data kinds](#data-kinds)). In a channel kind, `digits name` is the room of a bounded
  channel, as in `၂ ခု`, and a room that is not a whole number above zero is ZK0227.
- Sending, closing, waiting, sleeping and adding to an atomic number are sentence calls of
  functions of the core module, so these forms need no rules here:
  `<pipe> ထဲသို့ <value> ကို ပို့မည်။`, `<pipe> ကို ပိတ်မည်။`, `<thread> ကို စောင့်မည်။`,
  `<thread> ကို စောင့်သော ရလဒ်`, `<count> မီလီစက္ကန့် စောင့်မည်။`, `<count> စက္ကန့် စောင့်မည်။`
  and `<atomic> ထဲသို့ <value> ကို ပေါင်းထည့်မည်။`.

See [Threads](../language/threads.md).

## Where the rules depend on context

The rules above are written as a context-free grammar, but the language as a whole is not
context-free, because the parser picks a few productions with semantic predicates over
state that the rules do not show. The parser is a hand-written recursive-descent parser
with no backtracking. Where one token does not decide the production, it scans ahead over
the tokens (unbounded lookahead, so the grammar is not LL(k) for a fixed k) and commits.
A tool that parses Zaka needs the same state, because the EBNF alone is not enough.

**A name table from a pre-pass (the lexer hack).** A C parser must know which names are
type names, and in the same manner Zaka's parser must know which names are declared data
kinds. Before the main pass, the lexer and the parser each walk the tokens of the file
once and build the tables below, and `parse_with` merges in the data kind names, the
functions with prepositions and the written-as hints that the imports bring:

| Table | Contains | Decides |
| --- | --- | --- |
| data kind names | each struct, enumeration and alias name, the stem of an enumeration name, in both spellings of `ဒေတာ` | whether a name starts a `kind` (`လူ တစ်ယောက်`) or an `expr`; whether `<stem>များထဲမှ` reads a member |
| measure words | each word after `မျိုးပြပစ္စည်းမှာ` in the file | the lexer retags `<word>စာ` as the size token (`လူ ၂ ယောက်စာ`); otherwise the word is a name |
| functions with prepositions | each function whose signature declares prepositions with `ဝိဘတ်` or phrases with `အစ စကားစု` and `အဆုံး စကားစု`, with the words of each overload | where a call written as a sentence starts and ends |
| written-as hints | the written-as hint of each generic struct, data kind and trait that the file declares | where a data kind or a trait written in words starts and ends |

Because the tables come from a pre-pass, we can use a data kind above its declaration. The
table of measure words is separate for each file, because the retagging runs in the lexer,
before imports resolve.

**Parser modes.** Some words end a form in one context and are plain tokens in another.
The parser keeps a flag for each context and tests it before it takes the word:

| Mode (flag) | Set while parsing | Effect |
| --- | --- | --- |
| `in_count` | a count, `အကွက် <count> ခုရှိသော` | `ရှိသော` ends the count, and no struct literal starts in it |
| `in_list` | a list of parameters or fields | the bare function-kind production is turned off, so a function kind with parameters needs parentheses (`FnForm::NoParams`) |
| `in_word_list` | an item of a list in words | a `နှင့်` outside parentheses ends the item, and so does `ဖြင့်` after a lone operand before the end of a long call; parentheses turn it off |

**Agreement on counts.** A few productions agree with the number of items in a list. One
field or variant ends with `ပါဝင်သည်`, and two or more with `တို့ပါဝင်သည်`. A list in
words ends with its closing word without `တို့` after one item and with it after two or
more (`ကို` and `တို့ကို`, `ဖြင့်` and `တို့ဖြင့်`, `ပါသော` and `တို့ပါသော`), and joins its
last item with `နှင့်`. One parameter can leave out `လည်းကောင်း`, while two or more each
end in it. These cases are finite, so a context-free grammar can state them, and the rules
above do, as separate alternatives.

**Count prefixes.** The count prefixes before a data kind bind to its list levels from the
innermost out, as a stack that the parser pops as it reads each `စာရင်း`. So the level
that a count binds to depends on the levels that follow it, not on the prefix alone.

**Lexer lookahead.** The lexer joins multi-word reserved phrases longest first (maximal
munch), with or without spaces between their words, and splits a word made of reserved
words alone. Two rules look at the neighbouring words as well. `ဒေတာ` is a reserved word
after `ထဲမှ`, `ထဲတွင်` and `အမှားအယွင်း` and before `ပြောင်းလဲရန်` and `ထည့်ပေးရန်`, and
an identifier elsewhere. `ဟူသော လုပ်ဆောင်ချက်` and `မည့် လုပ်ဆောင်ချက်` join into the
function-value phrase, but the lexer keeps `ဟူသော` apart when `အမျိုးအစား` follows, where
`လုပ်ဆောင်ချက် အမျိုးအစား` opens a function kind, and when `သည်` follows, where a
function declaration starts.

**The long call.** A long call starts with the function value phrase,
`<name> ဟူသော လုပ်ဆောင်ချက်ကို`, or with a place in parentheses before it. The parser looks
at the one token after the `ကို`: a verb there makes the function value an object, and
each other token makes a call. For a place, it first scans to the `)` that ends the group.

**Order.** Imports come first in a file. The parser keeps a flag, `imports_closed`, and
reports an import after another top-level statement or inside a block.

**Static semantics, not the grammar.** The compiler checks a measure word that does not
fit its data kind (ZK0509), a statement at the top level (ZK0326), names, scope and the
data kinds of values after parsing, and the grammar accepts all of these. See [Stages](../compiler/stages.md) and
[The parser](../internals/parser.md).
