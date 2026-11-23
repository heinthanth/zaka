---
title: Modules
description: How a program spans several files - the export mark, the import sentence, whole imports, shares, what an imported file can contain, how names meet across files - and the errors ZK0229 to ZK0232, ZK0261, ZK0262, ZK0322 to ZK0324, ZK0483 and ZK0604.
---

# Modules

A module is one source file. A top-level name is local to that file unless the
declaration marks it as exported with `အများသုံးနိုင်သော`. Another file can import the exported
name. For examples, see [Modules](../../learn/modules/README.md) in Learn.

```zaka-file
// math.zaka
နှစ်ထပ် ဟူသော အများသုံးနိုင်သော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
```

```zaka-file
// main.zaka
"math.zaka" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နှစ်ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
```

`zakac run main.zaka` prints `၄၉`.

## The files of a program

- A program is the file that we run, together with each file that it imports, straight
  or through other files.
- Each file is read once, however often other files import it. Two spellings of one
  path, such as `lib/a.zaka` and `lib/../lib/a.zaka`, refer to one file.
- The compiler reads the imports of a file before the file itself, in the order that the
  imports are written, so each imported file comes ahead of the files that import it, and
  the file that we run comes last.

## The export mark

```text
<name> ဟူသော အများသုံးနိုင်သော လုပ်ဆောင်ချက် သည် ...
<name> ဟူသော အများသုံးနိုင်သော ဒေတာအစုအဝေးတွင် ...
<name> ဟူသော အများသုံးနိုင်သော ဒေတာအမျိုးအစားကွဲတွင် ...
အများသုံးနိုင်သော ဒေတာအမျိုးအစား <alias> ကို <kind> ...
အများသုံးနိုင်သော ကိန်းရှင် <name> သည် ...
အများသုံးနိုင်သော ကိန်းသေ <name> သည် ...
```

We write `အများသုံးနိုင်သော` to export a function, a struct, an enumeration, a [kind
alias](aliases.md), a variable or a constant at the top level of a file, so that other
files can import it. A declaration without the mark is private to its file.

| Declaration | Where `အများသုံးနိုင်သော` goes |
| --- | --- |
| a function | after `ဟူသော`, together with the other marks of the function |
| a struct or an enumeration | after `ဟူသော`, before `မျှဝေနိုင်သော` or `ပြင်ပမှ လာသော` |
| a kind alias, a variable, a constant | first |

- The mark in another place is ZK0229: before a statement, before a generic loop that
  starts with `ကိန်းရှင်`, or before a declaration inside a block, a loop, a conditional or
  a function.
- The mark exports a declaration of the file itself. A name that the file imports is not
  exported again, unless a [share](#sharing-imported-names) passes it on.
- Each struct that the data kinds (types) of an exported declaration name must be usable
  from other files. This covers the parameters and the result of a function, the data kind
  of a variable or a constant, and the fields of a struct, through each pointer and list
  level. A struct that the file declares without the mark is ZK0324 there. A struct that
  the file imports is fine, because its own file exports it.
- The mark exports a generic function, a generic struct and a trait too, and the rules of
  [Generics in modules](generics.md#generics-in-modules) tell what an instance sees.

## The import sentence

```text
"<file>" မှ <name> ကို ထည့်သွင်းမည်။
"<file>" မှ <name>၊ ... နှင့် <name> တို့ကို ထည့်သွင်းမည်။
"<file>" ကို ထည့်သွင်းမည်။
```

We use the first two forms, the named imports, to bring the names we list from a file,
and the third form, a [whole import](#whole-imports), to bring each name that the file
exports.

- The file is a string. Its path is relative to the folder of the importing file, with
  `/` between folders, and `..` is the folder above.
- A path that starts with `စံကား/` names a module of the stdlib (standard library), such
  as `"စံကား/အဝင်အထွက်"` ([Input and output](io.md)). The compiler carries its source, so
  it is never read from disk, and a program read from storage can import it too. Another
  name after `စံကား/` is ZK0238.
- The imports of a file come first, together with its
  [shares](#sharing-imported-names). An import or a share after a declaration or a
  statement, or inside a block, is ZK0230.
- Each name of a named import brings each exported item of that name from the file: a
  function (each exported function of the name, when the file declares several), a
  variable or a constant, and a struct, an enumeration or a kind alias. A name that the
  file does not export is ZK0322, with help when the file declares the name without the
  mark.
- The imported names join the top-level names of the importing file. A declaration of the
  same name in that file, or a second import of it, is ZK0302 for a variable or a constant
  and ZK0304 for a function.
- A named import brings the names it lists and nothing else. The names that the imported
  file imports itself remain there, unless it shares them, so the importing file imports
  them from their own file, or it gets ZK0322.
- Files that import each other, straight or through other files, are ZK0231, and the
  message names the files along the circle.
- When the file cannot be read, the compiler stops with the error of a missing or
  unreadable file, as it does for the file that we run. That error has no code.

An imported variable is the same variable in each file that imports it, so when one file
sets it, the other files read the new value.

## Whole imports

```text
"<file>" ကို ထည့်သွင်းမည်။
```

A whole import brings each name that the file exports: its own exported declarations and
each name that it [shares](#sharing-imported-names). The names join the top-level names of
the importing file, as the names of a named import do.

- A name that the importing file declares itself keeps its own meaning, so the whole import
  leaves that name out. A name that a named import of the file brings comes from that
  import.
- When two whole imports bring one name from two different declarations, the name clashes.
  It is then left out of both imports, and a use of it in the importing file is ZK0262,
  which names the two files. A clash that the file never uses is not an error, and a named
  import or a declaration of the name settles it.
- When two whole imports bring a name from the same declaration, as when one file shares
  what the other declares, the name is brought once and does not clash.
- Functions of one name from two whole imports clash as well, even with different
  parameters. To join them as overloads, we write a named import of the name from both
  files.

```zaka-file
// main.zaka
"စံကား/လမ်းကြောင်း" ကို ထည့်သွင်းမည်။
"lib/point.zaka" ကို ထည့်သွင်းမည်။
```

## Sharing imported names

```text
<name> ကို အများသုံးနိုင်ရန် ပြန်လည် မျှဝေမည်။
<name>၊ ... နှင့် <name> တို့ကို အများသုံးနိုင်ရန် ပြန်လည် မျှဝေမည်။
"<file>" ကို ပြန်လည် မျှဝေမည်။
```

A share (re-export) passes names that a file imports on to the files that import it, as if
the file exported them. A file that imports the sharing file, through a named or a whole
import, gets the declaration from the file where the name comes from.

- We use the first two forms to share names that the file imports, through a named or a
  whole import. A name that the file does not import is ZK0261, and a declaration of the
  file itself uses the mark `အများသုံးနိုင်သော` instead.
- We use the third form to share each name that another file exports, without bringing one
  of them in: the sharing file does not see them, and imports what it uses itself. The
  compiler reads that file, as it reads an imported one.
- Shares come first, among the imports of the file (ZK0230).

```zaka-file
// lib/shapes.zaka
"point.zaka" ကို ပြန်လည် မျှဝေမည်။
"circle.zaka" မှ စက်ဝိုင်း ကို ထည့်သွင်းမည်။
စက်ဝိုင်း ကို အများသုံးနိုင်ရန် ပြန်လည် မျှဝေမည်။
```

## What an imported file can contain

- The top level of an imported file can contain imports, shares, functions, structs,
  enumerations, kind aliases, variables and constants. Each other statement there, such
  as a print, a loop, a call or an assignment, is ZK0323. The file that we run can contain
  statements as before.
- A variable of an imported file needs its value where it is declared, as the next section
  explains (ZK0483).
- The starting value of a variable or a constant of an imported file must be known at
  compile time. Known values are literals, `ကနဦးတန်ဖိုး`, `အလွတ်`, the null address,
  constants of known value, the size of a data kind, a [function named as a
  value](function-values.md#a-function-as-a-value), operators on known values, list and
  struct literals of known values, and calls of comptime functions with known arguments.
  Each other starting value is ZK0604. The common cases are a call of a function that is
  not a [comptime function](compile-time.md#comptime-functions), and the value of a
  `ကိန်းရှင်`, because a variable can change.
- The check works out the values with all folding on, so the `--fold` option of a build
  does not change which programs compile.
- The variables of the imported files get their starting values before the statements of
  the file that we run, one file after another in the order that the program reads them.

## Globals of a module

A variable at the top level of an imported file (a global of a module) needs its value
where it is declared, so `ကိန်းရှင် ရေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။` there is ZK0483. The
file that we run can declare a global without a value and set it in its program start
block, because the check follows the program start block ([Globals and
calls](declarations.md#globals-and-calls)). A module has no program start block, so nothing
would set the global before a function of another file reads it. We can use
`ကနဦးတန်ဖိုး` to give the [initial value](kinds.md#initial-values) (default value) where no
other value fits.

```zaka-file
// counter.zaka
အများသုံးနိုင်သော ကိန်းရှင် ရေတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။      // not "ကိန်းပြည့် တစ်ခု" alone
```

## Names across files

- Function, variable and constant names belong to their file. Two files can each declare a
  private function or variable with the same name, and each file uses its own.
- Struct names belong to the whole program, and so do enumeration and kind alias names.
  Two files that each declare a struct `လူ` are ZK0316, exported or not. A variable,
  function or parameter with the name of a struct of another file is ZK0319.
- In the IR and in the operations, a function of an imported file is named
  `<name>@<file>`, where the file is its name without folder or extension, so two
  functions with the same name remain apart.

## Programs not read from a file

A program on standard input, in the compiler explorer or on a Learn page has no folder.
Each import in it is ZK0232, and the program does not go on to the check. An import from
the stdlib is the exception ([Input and output](io.md#the-rules)).

## The compiler

- `parse`, `semcheck`, `irgen`, `borrowck`, `codegen` and `run` read the imports of their
  input, while `segment` and `lex` work on the one file that is given to them.
- The artifacts of those stages contain each imported file, its name and its text, under
  `imports`, so a saved artifact can still show the line of a diagnostic in an imported
  file.

## The errors

| Code | When |
| --- | --- |
| ZK0229 | `အများသုံးနိုင်သော` before something other than a top-level declaration |
| ZK0230 | an import or a share after a declaration or a statement, or inside a block |
| ZK0231 | files that import each other |
| ZK0232 | an import in a program that was not read from a file |
| ZK0261 | a share of a name that the file does not import |
| ZK0262 | a use of a name that two whole imports bring from two files |
| ZK0322 | an import of something that the file does not export |
| ZK0323 | a statement in an imported file |
| ZK0324 | the data kind of an exported declaration names a private struct of its file |
| ZK0483 | a variable of an imported file declared without a value |
| ZK0604 | a variable or a constant of an imported file whose starting value is not known at compile time |

[Module mistakes](../../learn/modules/mistakes.md) shows each one with the files that
cause it.
