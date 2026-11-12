mod common;

use common::compile_text;
use strum::IntoEnumIterator;
use zaka_vm::machine::FrameMode;
use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;
use zaka_vm::trap::TrapKind;
use zakac_codegen_target_vm::options::EmitOptions;
use zakac_codegen_target_vm::options::ResultMode;
use zakac_codegen_target_vm::options::ScratchMode;
use zakac_ir_builder::ir::FoldOptions;

/// A text constant set into a variable, given to a function and read
/// again: each read duplicates the static text.
const PROGRAM: &str = "ကိန်းသေ အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"မမ\" ဖြစ်ပါစေ။
ကိန်းသေ နှုတ်ဆက် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"ဟေး {အမည်}\" + \"!\" ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ သည် စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ကိန်းရှင် x သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အမည် ဖြစ်ပါစေ။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို အမည် တို့ဖြင့် လုပ်ဆောင်မည်။
x ကို ပြသမည်။
အမည် ကို ပြသမည်။
နှုတ်ဆက် ကို ပြသမည်။
\"{နှုတ်ဆက်} {x}\" ဟု ပြသမည်။
";

#[test]
fn a_text_constant_is_duplicated_in_each_mode_and_leaks_nothing()
{
    let folds = [
        FoldOptions::default(),
        FoldOptions::none(),
        FoldOptions::builder().exprs(false).build(),
        FoldOptions::builder().consts(false).build(),
    ];
    for fold in folds
    {
        for results in ResultMode::iter()
        {
            for scratch in ScratchMode::iter()
            {
                let module =
                    compile_text(PROGRAM, &fold, EmitOptions { results, scratch });
                for frames in FrameMode::iter()
                {
                    let options = RunOptions::builder().frames(frames).build();
                    let mut out = Vec::new();
                    let (machine, result) =
                        run_and_keep(&module, &mut out, &options, &HostData::default());
                    let mode = format!("{fold:?} {results} {scratch} {frames}");
                    assert_eq!(result, Ok(()), "{mode}");
                    let out = String::from_utf8(out).unwrap();
                    assert_eq!(out, "မမ\nမမ\nမမ\nဟေး မမ!\nဟေး မမ! မမ\n", "{mode}");
                    assert!(machine.leaks().is_empty(), "{mode}");
                }
            }
        }
    }
}

#[test]
fn characters_are_read_and_text_keeps_whole_characters()
{
    let text = "ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"ကမ္ဘာ ok\" ဖြစ်ပါစေ။
t ၏ စာလုံးအရေအတွက် ကို ပြသမည်။
t ၏ စာလုံး ၁ ကို ပြသမည်။
t ၏ ၂ ခုမြောက်စာလုံး ကို ပြသမည်။
t ၏ အကွက် ၀ မှစ၍ ၆ ကွက် ကို ပြသမည်။
ကိန်းရှင် b သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၁၀၅] ဖြစ်ပါစေ။
b ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။
";
    assert_eq!(common::output(text), "၈\nမ\nမ\nကမ\nhi\n");
}

#[test]
fn a_cut_character_bad_utf8_and_a_number_past_the_end_stop_the_program()
{
    let trap = |text: &str| common::run_source(text).1.expect_err(text).kind;
    let cut = "ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"ကမ္ဘာ\" ဖြစ်ပါစေ။
ကိန်းရှင် n သည် ၂ ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
t ၏ အကွက် ၀ မှစ၍ n ကွက် ကို ပြသမည်။";
    assert!(matches!(trap(cut), TrapKind::TextCut { .. }));
    let bad = "ကိန်းရှင် b သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၄၊ ၂၅၅] ဖြစ်ပါစေ။
b ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။";
    assert!(matches!(trap(bad), TrapKind::BadUtf8 { .. }));
    let past = "ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"ab\" ဖြစ်ပါစေ။\nt ၏ စာလုံး ၂ ကို ပြသမည်။";
    assert!(matches!(trap(past), TrapKind::CharOutOfRange { .. }));
    let zero =
        "ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"ab\" ဖြစ်ပါစေ။\nt ၏ ၀ ခုမြောက်စာလုံး ကို ပြသမည်။";
    assert!(matches!(trap(zero), TrapKind::CharOutOfRange { first: 1, .. }));
}

#[test]
fn a_verbal_join_builds_new_text()
{
    let text = "ကိန်းရှင် အမည် သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"မမ\" ဖြစ်ပါစေ။
\"ဟေး \"၊ အမည်၊ \"!\" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသမည်။";
    assert_eq!(common::output(text), "ဟေး မမ!\n");
}
