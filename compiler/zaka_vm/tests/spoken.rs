use zakac_diagnostics::Code;
use zakac_diagnostics::lint::LintLevel;
use zakac_diagnostics::lint::LintLevels;
use zakac_semcheck::semcheck::SemcheckOptions;

use crate::common::output;
use crate::common::output_with;

/// A program in written Burmese: a function, a preposition `ဖြင့်`, a
/// condition, a call in notation and a literal in base 16.
const WRITTEN: &str = "\
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ချဲ့ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် \"ဖြင့်\" ကို သုံး၍ \
                       လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ခ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၄၀ အောက် မငယ်လျှင် \"အောင်\" ဟု ပြသမည်။
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို အမှတ် ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
အခြေ ၁၆ ဖြင့် FF ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမည်။
၃ ကို ၄ ဖြင့် ချဲ့သည့်ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// The same program with its endings spoken, and `နဲ့` for each `ဖြင့်`.
const SPOKEN: &str = "\
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမယ်။
က * ၂ ကို ပြန်ပေးမယ်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ချဲ့ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် \"ဖြင့်\" ကို သုံး၍ \
                      လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမယ်။
က * ခ ကို ပြန်ပေးမယ်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မယ်။
ကိန်းရှင် အမှတ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၇၃ ဖြစ်ပါစေ။
အကယ်၍ အမှတ် သည် ၄၀ အောက် မငယ်ရင် \"အောင်\" ဟု ပြသမယ်။
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို အမှတ် နဲ့ လုပ်ဆောင်တဲ့ ရလဒ် ကို ပြသမယ်။
အခြေ ၁၆ နဲ့ FF ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြသမယ်။
၃ ကို ၄ နဲ့ ချဲ့တဲ့ရလဒ် ကို ပြသမယ်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

#[test]
fn a_spoken_program_runs_as_its_written_twin_once_allowed()
{
    let allowed = LintLevels::default().with("spoken", LintLevel::Off).unwrap();
    let options = SemcheckOptions::builder().lint(allowed).build();
    let (spoken, diags) = output_with(SPOKEN, &options);
    assert_eq!(spoken, output(WRITTEN));
    let codes: Vec<Code> = diags.iter().map(|d| d.code).collect();
    assert!(!codes.contains(&Code::SpokenForm), "{codes:?}");
    assert!(codes.contains(&Code::MixedForms), "{codes:?}");
}

#[test]
fn a_spelling_of_two_words_reads_as_either_where_the_grammar_asks()
{
    let allowed = LintLevels::default().with("spoken", LintLevel::Off).unwrap();
    let options = SemcheckOptions::builder().lint(allowed).build();
    let samples = [
        include_str!("../../../samples/basics/enums.zaka"),
        include_str!("../../../samples/generics/traits.zaka"),
    ];
    for written in samples
    {
        // `ရှိသည့်` of an enumeration value, `ဟူသည့်` of a trait.
        let spoken = written.replace(
            "\u{101b}\u{103e}\u{102d}\u{101e}\u{100a}\u{103a}\u{1037}",
            "\u{101b}\u{103e}\u{102d}\u{1010}\u{1032}\u{1037}",
        );
        let spoken = spoken.replace(
            "\u{101f}\u{1030}\u{101e}\u{100a}\u{103a}\u{1037}",
            "\u{101f}\u{1030}\u{1010}\u{1032}\u{1037}",
        );
        assert_ne!(spoken, written);
        assert_eq!(output_with(&spoken, &options).0, output(written));
    }
}
