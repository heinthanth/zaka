use std::sync::Arc;

use zakac_ast::target::Target;
use zakac_borrow_checker::artifact::run_loaded;
use zakac_borrow_checker::fixes::Recompile;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::open_chains::load_settled;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

/// The start block around one statement, after `က`, an integer of 2.
fn program(line: &str) -> String
{
    format!(
        "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ \
         ဖြစ်ပါစေ။\n{line}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n"
    )
}

/// Compile a text up to the checker and attach the fixes that hold.
fn fixed(text: &str) -> (Arc<SourceFile>, Vec<Diagnostic>)
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let options = SemcheckOptions::builder().build();
    let folding = FoldOptions::builder().build();
    let mem = BorrowCheckOptions::default();
    let lexer = LexerOptions::default();
    let loaded =
        load_settled(&file, &mut sm, &lexer, &Target::host(), &options).unwrap();
    let checked = run_loaded(loaded, &options, &folding, &mem);
    let recompile = Recompile::builder()
        .root(file.clone())
        .target(Target::host())
        .options(options)
        .folding(folding)
        .mem(mem)
        .build();
    (file.clone(), recompile.attach(checked.diagnostics))
}

/// The text with the first fix of the first diagnostic of `code` made.
fn first_fix(text: &str, code: Code) -> String
{
    let (file, diags) = fixed(text);
    let d = diags.iter().find(|d| d.code == code).expect("the diagnostic");
    let fix = d.fixes.first().unwrap_or_else(|| panic!("no fix for {code:?}"));
    let edit = &fix.edits[0];
    let lo = (edit.span.lo.as_u32() - file.start_pos.as_u32()) as usize;
    let hi = (edit.span.hi.as_u32() - file.start_pos.as_u32()) as usize;
    let mut out = file.src.clone();
    out.replace_range(lo..hi, &edit.text);
    out
}

#[test]
fn each_particle_rule_writes_the_idiomatic_line()
{
    let cases = [
        (
            Code::ParticleBeforeAttached,
            "အကယ်၍ က သည် ၁ အောက် မငယ်သည်လျှင် က ကို ပြသမည်။",
            "အကယ်၍ က သည် ၁ အောက် မငယ်လျှင် က ကို ပြသမည်။",
        ),
        (
            Code::NegativeBeforeOr,
            "(က သည် ၁ အောက် မငယ်သည် သို့မဟုတ် က သည် ၂ ထက် ကြီးသည်) ဟု ပြသမည်။",
            "(က သည် ၁ အောက် မငယ်ပါ သို့မဟုတ် က သည် ၂ ထက် ကြီးသည်) ဟု ပြသမည်။",
        ),
        (
            Code::PaBeforeAnd,
            "(က သည် ၁ အောက် မငယ်ပါ ဖြစ်၍ က သည် ၂ အောက် ငယ်သည်) ဟု ပြသမည်။",
            "(က သည် ၁ အောက် မငယ်သည် ဖြစ်၍ က သည် ၂ အောက် ငယ်သည်) ဟု ပြသမည်။",
        ),
        (
            Code::PrintSentenceWithKo,
            "က သည် ၁ နှင့် မညီပါ ကို ပြသမည်။",
            "က သည် ၁ နှင့် မညီပါ ဟု ပြသမည်။",
        ),
        (Code::PrintValueWithHu, "က ဟု ပြသမည်။", "က ကို ပြသမည်။"),
        (
            Code::VerbBeforeNot,
            "(က သည် ၁ နှင့် ညီ မဟုတ်) ကို ပြသမည်။",
            "(က သည် ၁ နှင့် ညီသည် မဟုတ်) ကို ပြသမည်။",
        ),
        (Code::RedundantGroup, "(က + ၁) ကို ပြသမည်။", "က + ၁ ကို ပြသမည်။"),
    ];
    for (code, line, want) in cases
    {
        assert_eq!(first_fix(&program(line), code), program(want), "{code:?}");
    }
}

#[test]
fn an_undeclared_name_gets_the_closest_name_of_the_file()
{
    let text =
        program("ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။\nအရေအတက် ကို ပြသမည်။");
    let want =
        program("ကိန်းရှင် အရေအတွက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။\nအရေအတွက် ကို ပြသမည်။");
    assert_eq!(first_fix(&text, Code::UndeclaredName), want);
}

#[test]
fn a_missing_word_is_inserted_and_a_classifier_rewritten()
{
    let text = program("ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုး ၂ ဖြစ်ပါစေ။");
    let want = program("ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။");
    assert_eq!(first_fix(&text, Code::ExpectedToken), want);
    let person =
        "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။\n\n";
    let text = format!("{person}{}", program("ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။"));
    let want = format!("{person}{}", program("ကိန်းရှင် ကျော် သည် လူ တစ်ယောက် ဖြစ်ပါစေ။"));
    assert_eq!(first_fix(&text, Code::ClassifierMismatch), want);
}

#[test]
fn a_fix_that_does_not_compile_is_left_out()
{
    // No name of the file is close to this one, so no fix holds.
    let (_, diags) = fixed(&program("ဇဇဇဇဇဇ ကို ပြသမည်။"));
    let d = diags.iter().find(|d| d.code == Code::UndeclaredName).unwrap();
    assert!(d.fixes.is_empty());
}
