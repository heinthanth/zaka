mod common;

use common::codes_with;
use common::repo_root;
use common::run;
use zakac_ast::s_expr::ToSexpr;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_diagnostics::Code;

/// A text built while the program runs, in `g1`.
const HEAP: &str = "ကိန်းရှင် i သည် ၇ ဖြစ်ပါစေ။\nကိန်းရှင် က သည် \"{i}\" ဖြစ်ပါစေ။\n";

/// The checked module of a program, as an s-expression.
///
/// # Arguments
///
/// * `rest` - The lines after [`HEAP`].
///
/// # Returns
///
/// The module.
fn ir(rest: &str) -> String
{
    let out = run(&format!("{HEAP}{rest}"));
    assert!(out.out.diagnostics.is_empty(), "{:?}", out.out.diagnostics);
    out.out.lowered.folded.as_ref().map(ToSexpr::to_sexpr).unwrap_or_default()
}

#[test]
fn the_last_read_of_text_moves()
{
    let module = ir("ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။\nခ ကို ပြသပါ။\n");
    assert!(module.contains("(set g2 (take (addr g1)))"), "{module}");
}

#[test]
fn text_read_again_is_duplicated()
{
    let module = ir("ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။\nက ကို ပြသပါ။\n");
    assert!(module.contains("(set g2 (dup g1))"), "{module}");
}

#[test]
fn text_read_in_the_next_round_of_a_loop_is_duplicated()
{
    let module = ir("၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ခ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
");
    assert!(module.contains("(dup g1)"), "{module}");
    assert!(!module.contains("(take (addr g1))"), "{module}");
}

#[test]
fn lent_text_is_duplicated()
{
    let module = ir("ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
v ကို ပြသပါ။
");
    assert!(module.contains("(dup g1)"), "{module}");
}

#[test]
fn a_parameter_is_duplicated_and_a_local_moves()
{
    let module = ir("လုပ်ဆောင်ချက် ပြန် သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် ဒေသ သည် စ ဖြစ်ပါစေ။
ဒေသ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ပြန်(က) ကို ပြသပါ။
");
    assert!(module.contains("(dup l0)"), "{module}");
    assert!(module.contains("(take (addr l2))"), "{module}");
    assert!(module.contains("(take (addr g1))"), "{module}");
}

#[test]
fn a_global_a_later_call_reads_is_duplicated()
{
    let module = ir("လုပ်ဆောင်ချက် ပြ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်ပါ။
");
    assert!(module.contains("(set g2 (dup g1))"), "{module}");
}

#[test]
fn two_reads_in_one_statement_are_duplicated()
{
    let module = ir("ကိန်းရှင် တန်း သည် [က၊ က] ဖြစ်ပါစေ။\nတန်း ကို ပြသပါ။\n");
    assert_eq!(module.matches("(dup g1)").count(), 2, "{module}");
}

#[test]
fn warn_duplicates_names_each_duplicate_left()
{
    let warn = BorrowCheckOptions::builder().warn_duplicates(true).build();
    let kept = format!("{HEAP}ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။\nက ကို ပြသပါ။\n");
    assert_eq!(codes_with(&kept, &warn), [Code::DuplicateMade]);
    assert_eq!(codes_with(&kept, &Default::default()), []);
    let moved = format!("{HEAP}ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။\nခ ကို ပြသပါ။\n");
    assert_eq!(codes_with(&moved, &warn), []);
}

/// A function that reads its text parameter.
const SHOW: &str = "လုပ်ဆောင်ချက် ပြ သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

/// A function that sets the global `က`.
const SET: &str = "လုပ်ဆောင်ချက် ထား သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို \"အသစ်\" ဟု ထားပါ။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

#[test]
fn text_a_call_reads_is_lent_to_it()
{
    let module = ir(&format!("{SHOW}လုပ်ဆောင်ချက် ပြ ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။\nက ကို ပြသပါ။\n"));
    assert!(module.contains("(lend_text g1)"), "{module}");
}

#[test]
fn a_global_the_callee_sets_is_duplicated()
{
    let module = ir(&format!("{SET}လုပ်ဆောင်ချက် ထား ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။\nက ကို ပြသပါ။\n"));
    assert!(module.contains("(dup g1)"), "{module}");
    assert!(!module.contains("(lend_text g1)"), "{module}");
}

#[test]
fn a_global_the_callee_sets_is_not_moved_into_it()
{
    let text = format!(
        "{SET}လုပ်ဆောင်ချက် ထား ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။\nက ကို \"ပြီး\" ဟု ထားပါ။\nက ကို ပြသပါ။\n"
    );
    let module = ir(&text);
    assert!(!module.contains("(take (addr g1))"), "{module}");
}

#[test]
fn a_lent_parameter_is_lent_on()
{
    let module = ir(&format!(
        "{SHOW}လုပ်ဆောင်ချက် ဆင့် သည်
စာသား စ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ ကို စ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ဆင့် ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။
က ကို ပြသပါ။
"
    ));
    assert!(module.contains("(lend_text l0)"), "{module}");
}

#[test]
fn warn_duplicates_leaves_lent_text_alone()
{
    let warn = BorrowCheckOptions::builder().warn_duplicates(true).build();
    let lent = format!("{HEAP}{SHOW}လုပ်ဆောင်ချက် ပြ ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။\nက ကို ပြသပါ။\n");
    assert_eq!(codes_with(&lent, &warn), []);
}

/// A struct, and functions that read it, write it, view it, and set the
/// global `ပ`.
const PEOPLE: &str = "ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ကိန်းရှင် ပ သည် လူ(အသက်: ၃) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖတ် သည်
လူ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ရေး သည်
လူ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ၏ အသက် ကို ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် လိပ်စာယူ သည်
လူ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်း သည် ငှားထားသော လူ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ထား သည်
လူ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပ ကို လူ(အသက်: ၉) ဟု ထားပါ။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

/// The checked module of a program.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// The module.
fn checked(text: &str) -> zakac_ir_builder::ir::Module
{
    let out = run(text);
    assert!(out.out.diagnostics.is_empty(), "{:?}", out.out.diagnostics);
    out.out.lowered.folded.expect("a module")
}

#[test]
fn a_struct_parameter_the_function_never_writes_is_read_in_place()
{
    let module = checked(PEOPLE);
    let shared: Vec<&[u32]> =
        module.funcs.iter().map(|f| f.shared_params.as_slice()).collect();
    assert_eq!(shared, [&[0][..], &[], &[], &[0]]);
}

#[test]
fn a_global_the_callee_sets_is_duplicated_for_a_shared_parameter()
{
    let text = format!(
        "{PEOPLE}လုပ်ဆောင်ချက် ထား ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။\nလုပ်ဆောင်ချက် ဖတ် ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။\n"
    );
    let module = checked(&text).to_sexpr();
    assert_eq!(module.matches("(dup g0)").count(), 1, "{module}");
}

/// The program of `samples/programs/scores_views.zaka` reads its lists and
/// text through views alone, so `--warn-duplicates` names nothing in it; the
/// same program with plain parameters, `scores.zaka`, has duplicates left.
#[test]
fn the_scores_program_with_views_makes_no_duplicate()
{
    let warn = BorrowCheckOptions::builder().warn_duplicates(true).build();
    let sample = |name: &str| {
        let path = repo_root().join("samples").join("programs").join(name);
        std::fs::read_to_string(path).unwrap()
    };
    assert_eq!(codes_with(&sample("scores_views.zaka"), &warn), []);
    let plain = codes_with(&sample("scores.zaka"), &warn);
    assert!(plain.contains(&Code::DuplicateMade), "{plain:?}");
}
