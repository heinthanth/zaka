---
title: Modules
description: How a program spans several files - the export mark, the import sentence, what an imported file holds, how names meet across files - and the errors ZK0229 to ZK0232, ZK0322 to ZK0324, ZK0483 and ZK0604.
---

# Modules

A program can span several files. Each file keeps its top-level names to itself unless
it marks them `အများသုံး`, and a file takes marked names from another file with an
import. [Modules](../../learn/modules/README.md) in the Learn book
shows the same rules with examples.

```zaka-file
// math.zaka
အများသုံး လုပ်ဆောင်ချက် နှစ်ထပ် သည်
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

- A program is the file that is run and each file it imports, at once or through other
  files.
- Each file is read once, however often other files import it. Two spellings of one path, as
  `lib/a.zaka` and `lib/../lib/a.zaka`, name one file.
- The compiler reads the imports of a file before the file itself, in the order the
  imports are written, so each imported file comes ahead of the files that import it and
  the file that is run comes last.

## The export mark

```text
အများသုံး လုပ်ဆောင်ချက် <name> သည် ...
<name> ဟူသော အများသုံး ဒေတာအစုအဝေးတွင် ...
<name> ဟူသော အများသုံး ဒေတာအမျိုးအစားကွဲတွင် ...
အများသုံး ဒေတာအမျိုးအစား <alias> ကို <kind> ...
အများသုံး ကိန်းရှင် <name> သည် ...
အများသုံး ကိန်းသေ <name> သည် ...
```

- `အများသုံး` exports a function, a struct, an enumeration, a [kind alias](aliases.md),
  a variable or a constant at the top level of a file. On a struct or an enumeration it
  stands after `ဟူသော`, before `မျှဝေနိုင်သော` or `ပြင်ပမှ လာသော`; on the others it comes first. Other files can then import it. A declaration without the mark is private to
  its file.
- The mark in another place is ZK0229: before a statement, before a generic loop
  that starts with `ကိန်းရှင်`, or before a declaration inside a block, a loop, a
  conditional or a function.
- The mark exports a declaration of the file itself. A name the file imports is not
  exported again.
- Each struct that the kinds of an exported declaration name must be usable from other
  files: the parameters and the result of a function, the kind of a variable or
  constant, and the fields of a struct, through each pointer and list level. A struct
  the file declares without the mark there is ZK0324. A struct the file imports is
  fine, since its own file exports it.
- The mark exports a generic function, a generic struct and a trait too; the rules of
  [Generics in modules](generics.md#generics-in-modules) tell what an instance sees.

## The import sentence

```text
"<file>" မှ <name> ကို ထည့်သွင်းမည်။
"<file>" မှ <name>၊ ... နှင့် <name> တို့ကို ထည့်သွင်းမည်။
```

- The file is a string. Its path is relative to the folder of the importing file, with
  `/` between folders; `..` names the folder above.
- A path that starts with `စံကား/` names a module of the stdlib, such as
  `"စံကား/အဝင်အထွက်"` ([Input and output](io.md)): the compiler carries its source, so
  it is never read from disk, and a program read from storage can import it too.
  Another name after `စံကား/` is ZK0238.
- The imports of a file come first. An import after a declaration or a statement, or
  inside a block, is ZK0230.
- Each name brings each exported item of that name from the file: a function (each
  exported function of the name, when the file declares several), a variable or
  constant, and a struct, an enumeration or a kind alias. A name the file does not
  export is ZK0322, with
  help when the file declares the name without the mark.
- The imported names join the top-level names of the importing file. A declaration of
  the same name in that file, or a second import of it, is ZK0302 for a variable or
  constant and ZK0304 for a function.
- An import brings the names it lists and nothing more. The names that the imported file
  itself imports remain there; the importing file imports them from their own file, or it
  gets ZK0322.
- Files that import each other, at once or through other files, are ZK0231. The message
  names the files along the circle.
- When the file cannot be read, the compiler stops with the error of a missing or
  unreadable file, as it does for the file that is run. That error has no code.

An imported variable is the same variable in each file that imports it: when one file
sets it, the others read the new value.

## What an imported file holds

- The top level of an imported file holds imports, functions, structs, enumerations,
  kind aliases, variables and constants. Each other statement there, such as a print, a
  loop, a call or an assignment, is ZK0323. The file that is run holds statements as
  before.
- A variable of an imported file needs its value at its declaration, as the next section
  states (ZK0483).
- The starting value of a variable or a constant of an imported file must be known at
  compile time. Known values are literals, `ကနဦးတန်ဖိုး`, `အလွတ်`, the null address,
  constants of known value, the size of a
  kind, a [function named as a value](function-values.md#a-function-as-a-value),
  operators on known values, and list and struct literals of known values. Each other
  starting value is ZK0604; a function call is the usual case, and so is the value of a
  `ကိန်းရှင်`, since a variable can change.
- The check works out the values with each folding on, so the `--fold` option of a build
  does not change which programs compile.
- The variables of the imported files get their starting values before the statements of
  the file that is run, one file after another in the order the program reads them.

## Globals of a module

A variable at the top level of an imported file, a module, needs its value where it is
declared: `ကိန်းရှင် ရေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။` there is ZK0483. The file that is
run can declare a global with no value and set it in its start block, since the check
follows the start block ([Globals and calls](statements.md#globals-and-calls)); a module has
no start block, so nothing would set the global before a function of another file reads
it. `ကနဦးတန်ဖိုး` gives the [initial value](kinds.md#initial-values) where no other value
fits.

```zaka-file
// counter.zaka
အများသုံး ကိန်းရှင် ရေတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။      // not "ကိန်းပြည့် တစ်ခု" alone
```

## Names across files

- Function, variable and constant names belong to their file. Two files can each declare
  a private function or variable of one name, and each file uses its own.
- Struct names belong to the whole program, as enumeration and kind alias names do: two
  files that each declare a struct `လူ` are ZK0316, exported or not. A variable, function or parameter named like a struct of
  another file is ZK0319 as well.
- In the IR and in the ops, a function of an imported file is named `<name>@<file>`, the
  file being its name without folder or extension, so two functions of one name remain apart.

## Programs not read from a file

A program on standard input, in the compiler explorer or on a Learn page has no folder.
Each import in it is ZK0232, and the program does not go on to the check.

## The compiler

- `parse`, `semcheck`, `irgen`, `borrowck`, `codegen` and `run` read the imports
  of their input. `segment` and `lex` work on the one file given to them.
- The artifacts of those stages hold each imported file, its name and its text, under
  `imports`, so a saved artifact can still show the line of a diagnostic in an imported
  file.

## The errors

| Code | Raised when |
| --- | --- |
| ZK0229 | `အများသုံး` stands before something other than a top-level declaration |
| ZK0230 | an import comes after a declaration or statement, or inside a block |
| ZK0231 | files import each other |
| ZK0232 | a program that was not read from a file has an import |
| ZK0322 | an import names something the file does not export |
| ZK0323 | an imported file holds a statement |
| ZK0324 | the kind of an exported declaration names a private struct of its file |
| ZK0483 | a variable of an imported file declared with no value |
| ZK0604 | a variable or constant of an imported file starts with a value not known at compile time |

[Module mistakes](../../learn/modules/mistakes.md) shows each one with the files that
cause it.
