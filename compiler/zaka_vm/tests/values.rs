mod common;

use common::compile_text;
use strum::IntoEnumIterator;
use zaka_vm::machine::FrameMode;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;
use zakac_codegen::options::EmitOptions;
use zakac_codegen::options::ResultMode;
use zakac_codegen::options::ScratchMode;
use zakac_ir_builder::ir::FoldOptions;

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

#[test]
fn duplicated_text_has_its_own_storage_in_each_mode()
{
    for (fold, emit, frames) in modes()
    {
        let module = compile_text(COPIES, &fold, emit);
        let options = RunOptions::builder().frames(frames).build();
        let mut out = Vec::new();
        let (machine, result) = run_and_keep(&module, &mut out, &options);
        let mode = format!("{fold:?} {emit:?} {frames}");
        assert_eq!(result, Ok(()), "{mode}");
        assert_eq!(String::from_utf8(out).unwrap(), PRINTED, "{mode}");
        assert!(machine.leaks().is_empty(), "{mode}");
    }
}
