mod common;

use common::compile_text;
use strum::IntoEnumIterator;
use zaka_vm::machine::FrameMode;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;
use zakac_codegen::bc::Op;
use zakac_codegen::options::EmitOptions;
use zakac_codegen::options::ResultMode;
use zakac_codegen::options::ScratchMode;
use zakac_ir_builder::ir::FoldOptions;
use zakac_semcheck::kind::DataKind;

/// Text built while the program runs, copied into a variable, a struct, a
/// list, a parameter the function changes, and a result: each duplicate is
/// changed while its source keeps its text.
const COPIES: &str = "ဒေတာအမျိုးအစား လူ တွင် စာသား အမည်၊ ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
လုပ်ဆောင်ချက် ပြောင်း သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို စ + \"?\" ဟု ထားပါ။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြန် သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။
စ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ကိန်းရှင် i သည် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် \"{i}\" ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ခ ကို ခ + \"!\" ဟု ထားပါ။
\"{က} {ခ}\" ဟု ပြသပါ။
ကိန်းရှင် ပ သည် လူ(အမည်: က၊ အသက်: ၃) ဖြစ်ပါစေ။
ကိန်းရှင် ဖ သည် ပ ဖြစ်ပါစေ။
ဖ ၏ အမည် ကို \"x\" ဟု ထားပါ။
ပ ကို ပြသပါ။
ဖ ကို ပြသပါ။
ကိန်းရှင် တန်း သည် [က၊ ခ] ဖြစ်ပါစေ။
ကိန်းရှင် တန်း၂ သည် တန်း ဖြစ်ပါစေ။
တန်း၂ ၏ အကွက် ၀ ကို \"y\" ဟု ထားပါ။
တန်း ကို ပြသပါ။
တန်း၂ ကို ပြသပါ။
လုပ်ဆောင်ချက် ပြောင်း ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။
က ကို ပြသပါ။
ပြန်(က) ကို ပြသပါ။
က ကို ပြသပါ။
";

/// What [`COPIES`] prints.
const PRINTED: &str = "၇ ၇!\nလူ(အမည်: \"၇\"၊ အသက်: ၃)\nလူ(အမည်: \"x\"၊ အသက်: ၃)\n[\"၇\"၊ \
                       \"၇!\"]\n[\"y\"၊ \"၇!\"]\n၇?\n၇\n၇\n၇\n";

/// Text lent to functions that read it, return it, change their parameter,
/// change the global lent to them, or call themselves with it; and a call
/// whose other argument changes the global first.
const LENDS: &str = "ကိန်းရှင် i သည် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် \"{i}\" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြန် သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။
စ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြင် သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို စ + \"?\" ဟု ထားပါ။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ဖျက် သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို \"အသစ်\" ဟု ထားပါ။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ဖျက်ပြီးပေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က ကို \"နောက်\" ဟု ထားပါ။
၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် နှစ်ခု သည်
စာသား စ၊ ကိန်းပြည့် န တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"{စ} {န}\" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြန်ကြိမ် သည်
စာသား စ၊ ကိန်းပြည့် န တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ န သည် ၀ ထက် ကြီးလျှင် လုပ်ဆောင်ချက် ပြန်ကြိမ် ကို စ၊ န - ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြ ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းရှင် ထွက် သည် ပြန်(က) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြင် ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ဖျက် ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။
က ကို \"{i}{i}\" ဟု ထားပါ။
လုပ်ဆောင်ချက် နှစ်ခု ကို က၊ ဖျက်ပြီးပေး() တို့ဖြင့် လုပ်ဆောင်ပါ။
က ကို \"{i}!\" ဟု ထားပါ။
လုပ်ဆောင်ချက် ပြန်ကြိမ် ကို က၊ ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။
ထွက် ကို ပြသပါ။
က ကို ပြသပါ။
";

/// What [`LENDS`] prints.
const LENT: &str = "၇\n၇?\n၇\n၇၇ ၁\n၇!\n၇!\n၇!\n၇\n၇!\n";

/// Structs and lists given to functions that read them in place, change
/// their own parameter, change the global given to them, return it, pass it
/// on, or call themselves with it.
const SHARES: &str = "ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အမြင့် တို့ ပါဝင်သည်။
ကိန်းရှင် က သည် လူ(အသက်: ၃၀၊ အမြင့်: ၂) ဖြစ်ပါစေ။
ကိန်းရှင် တန်း သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပေါင်း သည်
လူ ပ၊ အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း l တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
(ပ ၏ အသက်) + (l ၏ အကွက် ၁) ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြင် သည်
လူ ပ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပ ၏ အသက် ကို ၉၉ ဟု ထားပါ။
ပ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ဖျက် သည်
လူ ပ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို လူ(အသက်: ၁၊ အမြင့်: ၁) ဟု ထားပါ။
ပ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြန် သည်
လူ ပ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လူ ရလဒ် ကို ပြန်ပေးမည်။
ပ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြ သည်
လူ ပ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပ ကို ပြသပါ။
လုပ်ဆောင်ချက် ပြင် ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် စုစု သည်
အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း l၊ ကိန်းပြည့် န တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၃ နှင့် ညီလျှင် ၀ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
(l ၏ အကွက် န) + စုစု(l၊ န + ၁) ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ပေါင်း(က၊ တန်း) ကို ပြသပါ။
လုပ်ဆောင်ချက် ပြင် ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။
က ကို ပြသပါ။
လုပ်ဆောင်ချက် ဖျက် ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။
ကိန်းရှင် ခ သည် ပြန်(က) ဖြစ်ပါစေ။
ခ ကို ပြသပါ။
လုပ်ဆောင်ချက် ပြ ကို ခ တို့ဖြင့် လုပ်ဆောင်ပါ။
စုစု(တန်း၊ ၀) ကို ပြသပါ။
";

/// What [`SHARES`] prints.
const SHARED: &str = "၃၂\nလူ(အသက်: ၉၉၊ အမြင့်: ၂)\nလူ(အသက်: ၃၀၊ အမြင့်: ၂)\nလူ(အသက်: ၃၀၊ အမြင့်: \
                      ၂)\nလူ(အသက်: ၁၊ အမြင့်: ၁)\nလူ(အသက်: ၁၊ အမြင့်: ၁)\nလူ(အသက်: ၉၉၊ အမြင့်: \
                      ၁)\nလူ(အသက်: ၁၊ အမြင့်: ၁)\n၆\n";

/// Numbers, text, a field and a cell changed through edit references, at
/// one call and through a function that passes its reference on.
const EDITS: &str = "ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
လုပ်ဆောင်ချက် တိုး သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် အော် သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို စ + \"!\" ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် နှစ်ခါတိုး သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် တိုး ကို န (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် တိုး ကို န (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
န ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး ကို က (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
က ကို ပြသပါ။
ကိန်းရှင် i သည် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် \"{i}\" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် အော် ကို ခ (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် အော် ကို ခ (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
ခ ကို ပြသပါ။
ကိန်းရှင် ပ သည် လူ(အသက်: ၃၀) ဖြစ်ပါစေ။
နှစ်ခါတိုး(ပ ၏ အသက် (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)) ကို ပြသပါ။
ပ ကို ပြသပါ။
ကိန်းရှင် တန်း သည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး ကို တန်း ၏ အကွက် ၁ (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။
တန်း ကို ပြသပါ။
";

/// What [`EDITS`] prints.
const EDITED: &str = "၆\n၇!!\n၃၂\nလူ(အသက်: ၃၂)\n[၁၊ ၃]\n";

/// Text compared with `==` and `!=`: heap, static, a view, and blank text.
const COMPARES: &str = "ကိန်းရှင် i သည် ၇ ဖြစ်ပါစေ။\nကိန်းရှင် က သည် \"{i}\" ဖြစ်ပါစေ။\nကိန်းရှင် v \
                        သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။\nက \
                        == \"၇\" ကို ပြသပါ။\nက != \"၈\" ကို ပြသပါ။\nv == \"၇\" ကို \
                        ပြသပါ။\nက == \"abc\" ကို ပြသပါ။\n\"\" == \"\" ကို ပြသပါ။\n";

/// Each-cell and each-character loops: a list, a slice, text cells with
/// `continue` and `break`, a variable set inside the round, characters of
/// one to four u8s, and blank text.
const WALKS: &str = "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
l ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသပါ။
l ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသပါ။
ကိန်းရှင် n သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [\"က\"၊ \"ခ\"၊ \"ဂ\"၊ \"ဃ\"] ဖြစ်ပါစေ။
n ၏ အကွက်တိုင်း ကို w အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ w သည် \"ခ\" နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။
အကယ်၍ w သည် \"ဃ\" နှင့် ညီလျှင် ကွင်းဆက် ကို ရပ်ပါ။
w ကို \"{w}!\" ဟု ထားပါ။
w ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
n ကို ပြသပါ။
\"aé၁😀\" ၏ စာလုံးတိုင်း ကို c အနေဖြင့် ယူ၍ c ကို ပြသပါ။
\"\" ၏ စာလုံးတိုင်း ကို c အနေဖြင့် ယူ၍ c ကို ပြသပါ။
";

/// What [`WALKS`] prints.
const WALKED: &str =
    "၁\n၂\n၃\n၂\n၃\nက!\nဂ!\n[\"က\"၊ \"ခ\"၊ \"ဂ\"၊ \"ဃ\"]\na\né\n၁\n😀\n";

/// Plain, fixed and heap text, set from each other, in a struct field, a
/// list and a parameter.
const FORMS: &str = "ဒေတာအမျိုးအစား လူ တွင် စာသားရှင် အမည်၊ ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
လုပ်ဆောင်ချက် ပြ သည်
စာသားရှင် စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ကိန်းရှင် p သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"a\" ဖြစ်ပါစေ။
ကိန်းရှင် f သည် စာသားသေ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"b\" ဖြစ်ပါစေ။
ကိန်းရှင် h သည် စာသားရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"c\" ဖြစ်ပါစေ။
h ကို p ဟု ထားပါ။
f ကို h ဟု ထားပါ။
p ကို \"{f}{h}\" ဟု ထားပါ။
\"{p} {f} {h}\" ဟု ပြသပါ။
ကိန်းရှင် ပ သည် လူ(အမည်: f၊ အသက်: ၃) ဖြစ်ပါစေ။
ပ ကို ပြသပါ။
ကိန်းရှင် l သည် စာသားရှင် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [\"x\"၊ p] ဖြစ်ပါစေ။
l ကို ပြသပါ။
လုပ်ဆောင်ချက် ပြ ကို \"q\" တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ပြ ကို h တို့ဖြင့် လုပ်ဆောင်ပါ။
h ကို ပြသပါ။
";

/// What [`FORMS`] prints.
const FORMED: &str = "aa a a\nလူ(အမည်: \"a\"၊ အသက်: ၃)\n[\"x\"၊ \"aa\"]\nq\na\na\n";

/// Each mode a program can be built and run in.
///
/// # Returns
///
/// The foldings, the code generation choices and the frame mode of each.
fn modes() -> Vec<(FoldOptions, EmitOptions, FrameMode)>
{
    let folds = [FoldOptions::default(), FoldOptions::none()];
    let emits = ResultMode::iter().flat_map(|results| {
        ScratchMode::iter().map(move |scratch| EmitOptions { results, scratch })
    });
    let emits: Vec<EmitOptions> = emits.collect();
    let built =
        folds.into_iter().flat_map(|f| emits.clone().into_iter().map(move |e| (f, e)));
    built.flat_map(|(f, e)| FrameMode::iter().map(move |m| (f, e, m))).collect()
}

/// Run a program in each mode, and check its output and that it leaks
/// nothing.
///
/// # Arguments
///
/// * `program` - The program.
/// * `printed` - What it prints.
fn runs_in_each_mode(program: &str, printed: &str)
{
    for (fold, emit, frames) in modes()
    {
        let module = compile_text(program, &fold, emit);
        let options = RunOptions::builder().frames(frames).build();
        let mut out = Vec::new();
        let (machine, result) = run_and_keep(&module, &mut out, &options);
        let mode = format!("{fold:?} {emit:?} {frames}");
        assert_eq!(result, Ok(()), "{mode}");
        assert_eq!(String::from_utf8(out).unwrap(), printed, "{mode}");
        assert!(machine.leaks().is_empty(), "{mode}");
    }
}

#[test]
fn duplicated_text_has_its_own_storage_in_each_mode()
{
    runs_in_each_mode(COPIES, PRINTED)
}

#[test]
fn lent_text_is_read_in_place_and_kept_in_each_mode() { runs_in_each_mode(LENDS, LENT) }

#[test]
fn shared_structs_and_lists_read_the_callers_value_in_each_mode()
{
    runs_in_each_mode(SHARES, SHARED)
}

#[test]
fn edit_references_change_the_callers_place_in_each_mode()
{
    runs_in_each_mode(EDITS, EDITED)
}

#[test]
fn text_compares_its_u8s_in_each_mode()
{
    runs_in_each_mode(COMPARES, "မှန်\nမှန်\nမှန်\nမမှန်\nမှန်\n")
}

#[test]
fn each_loops_walk_cells_and_characters_in_each_mode()
{
    runs_in_each_mode(WALKS, WALKED)
}

#[test]
fn text_forms_convert_to_each_other_in_each_mode() { runs_in_each_mode(FORMS, FORMED) }

/// Heap text gets a block of its own where a value of another form, such as
/// a literal in static storage, becomes heap text.
#[test]
fn heap_text_owns_its_block()
{
    let module = compile_text(FORMS, &FoldOptions::default(), EmitOptions::default());
    let heap = DataKind::parse("text~").expect("text~");
    let owns = module
        .funcs
        .iter()
        .flat_map(|f| &f.code)
        .filter(|op| **op == Op::OwnText(heap));
    assert!(owns.count() >= 3);
}
