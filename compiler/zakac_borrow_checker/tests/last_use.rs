use zakac_ast::s_expr::ToSexpr;

use crate::common::repo_root;
use crate::common::run;

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
    let module = ir("ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။\nခ ကို ပြသမည်။\n");
    assert!(module.contains("(set g2 (take (addr g1)))"), "{module}");
}

#[test]
fn text_read_again_is_duplicated()
{
    let module = ir("ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။\nက ကို ပြသမည်။\n");
    assert!(module.contains("(set g2 (dup g1))"), "{module}");
}

#[test]
fn text_read_in_the_next_round_of_a_loop_is_duplicated()
{
    let module = ir("၁ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
");
    assert!(module.contains("(dup g1)"), "{module}");
    assert!(!module.contains("(take (addr g1))"), "{module}");
}

#[test]
fn a_parameter_is_duplicated_and_a_local_moves()
{
    let module = ir("လုပ်ဆောင်ချက် ပြန် သည်
စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ဒေသ သည် စ ဖြစ်ပါစေ။
ဒေသ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ပြန်(က) ကို ပြသမည်။
");
    assert!(module.contains("(dup l0)"), "{module}");
    assert!(module.contains("(take (addr l1))"), "{module}");
    // The start block is a function, which never moves out of a global.
    assert!(module.contains("(call ပြန် (dup_lent g1))"), "{module}");
}

#[test]
fn a_global_a_later_call_reads_is_duplicated()
{
    let module = ir("လုပ်ဆောင်ချက် ပြ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
");
    assert!(module.contains("(set g2 (dup g1))"), "{module}");
}

#[test]
fn two_reads_in_one_statement_are_duplicated()
{
    let module = ir("ကိန်းရှင် တန်း သည် [က၊ က] ဖြစ်ပါစေ။\nတန်း ကို ပြသမည်။\n");
    assert_eq!(module.matches("(dup g1)").count(), 2, "{module}");
}

#[test]
fn a_read_kept_for_later_leaves_a_duplicate()
{
    let kept = format!("{HEAP}ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။\nက ကို ပြသမည်။\n");
    assert_eq!(dups(&kept), 1);
    let moved = format!("{HEAP}ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။\nခ ကို ပြသမည်။\n");
    assert_eq!(dups(&moved), 0);
}

/// A function that reads its text parameter.
const SHOW: &str = "လုပ်ဆောင်ချက် ပြ သည်
စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

/// A function that sets the global `က`.
const SET: &str = "လုပ်ဆောင်ချက် ထား သည်
စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို \"အသစ်\" ဟု ထားမည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

#[test]
fn text_a_call_reads_is_lent_to_it()
{
    let module =
        ir(&format!("{SHOW}ပြ ဟူသော လုပ်ဆောင်ချက်ကို က တို့ဖြင့် လုပ်ဆောင်မည်။\nက ကို ပြသမည်။\n"));
    assert!(module.contains("(dup_lent g1)"), "{module}");
}

#[test]
fn a_global_the_callee_sets_is_duplicated()
{
    let module =
        ir(&format!("{SET}ထား ဟူသော လုပ်ဆောင်ချက်ကို က တို့ဖြင့် လုပ်ဆောင်မည်။\nက ကို ပြသမည်။\n"));
    assert!(module.contains("(dup g1)"), "{module}");
    assert!(!module.contains("(dup_lent g1)"), "{module}");
}

#[test]
fn a_global_the_callee_sets_is_not_moved_into_it()
{
    let text = format!(
        "{SET}ထား ဟူသော လုပ်ဆောင်ချက်ကို က တို့ဖြင့် လုပ်ဆောင်မည်။\nက ကို \"ပြီး\" ဟု ထားမည်။\nက ကို ပြသမည်။\n"
    );
    let module = ir(&text);
    assert!(!module.contains("(take (addr g1))"), "{module}");
}

#[test]
fn a_lent_parameter_is_lent_on()
{
    let module = ir(&format!(
        "{SHOW}လုပ်ဆောင်ချက် ဆင့် သည်
စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို စ တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ဆင့် ဟူသော လုပ်ဆောင်ချက်ကို က တို့ဖြင့် လုပ်ဆောင်မည်။
က ကို ပြသမည်။
"
    ));
    assert!(module.contains("(dup_lent l0)"), "{module}");
}

#[test]
fn lent_text_leaves_no_duplicate()
{
    let lent = format!("{HEAP}{SHOW}ပြ ဟူသော လုပ်ဆောင်ချက်ကို က တို့ဖြင့် လုပ်ဆောင်မည်။\nက ကို ပြသမည်။\n");
    assert_eq!(dups(&lent), 0);
}

/// A struct, and functions that read it, write it, view it, and set the
/// global `ပ`.
const PEOPLE: &str = "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ကိန်းရှင် ပ သည် လူ(အသက်: ၃) ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖတ် သည်
က ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ရေး သည်
က ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ၏ အသက် ကို ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် လိပ်စာယူ သည်
က ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ညွှန်း သည် လူ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ထား သည်
က ဟူသော လူ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပ ကို လူ(အသက်: ၉) ဟု ထားမည်။
က ကို ပြသမည်။
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
    assert_eq!(shared, [&[0][..], &[], &[], &[0], &[]]);
}

#[test]
fn a_global_the_callee_sets_is_duplicated_for_a_shared_parameter()
{
    let text = format!(
        "{PEOPLE}ထား ဟူသော လုပ်ဆောင်ချက်ကို ပ တို့ဖြင့် လုပ်ဆောင်မည်။\nဖတ် ဟူသော လုပ်ဆောင်ချက်ကို ပ တို့ဖြင့် \
         လုပ်ဆောင်မည်။\n"
    );
    let module = checked(&text).to_sexpr();
    assert_eq!(module.matches("(dup g0)").count(), 1, "{module}");
}

/// A list that two calls read is lent to both, though the second read of it is
/// its last, which moves it: the move ends the list, so it keeps no earlier
/// lend from being one.
#[test]
fn a_list_lent_twice_before_its_last_read_is_never_duplicated()
{
    let text = "လုပ်ဆောင်ချက် ပြ သည်\nက ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nက ၏ အကွက် ၀ ကို ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။\nကိန်းရှင် ခ သည် \
                ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။\nပြ ဟူသော လုပ်ဆောင်ချက်ကို ခ \
                တို့ဖြင့် လုပ်ဆောင်မည်။\nပြ ဟူသော လုပ်ဆောင်ချက်ကို ခ တို့ဖြင့် လုပ်ဆောင်မည်။\n";
    assert_eq!(dups(text), 0);
}

/// The program of `samples/programs/scores.zaka` keeps no duplicate in its
/// checked module: each call lends its list, and each each-cell loop, which
/// reads alone, lends its list too.
#[test]
fn the_scores_program_makes_no_duplicate()
{
    let sample = |name: &str| {
        let path = repo_root().join("samples").join("programs").join(name);
        std::fs::read_to_string(path).unwrap()
    };
    assert_eq!(dups(&sample("scores.zaka")), 0);
}

/// An each loop lends its list when it reads alone, and walks a duplicate
/// when its block writes the list or calls a function that writes it.
#[test]
fn an_each_loop_duplicates_a_list_it_changes_alone()
{
    let walk = |inner: &str| {
        format!(
            "ကိန်းရှင် ကများ သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ချဲ့ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကများ ထဲသို့ ၉ ကို ထည့်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကများ ၏ အကွက်တိုင်း ကို က အနေဖြင့် ယူ၍ {inner}
ကများ ကို ပြသမည်။
"
        )
    };
    let read = walk("က ကို ပြသမည်။");
    assert_eq!(dups(&read), 0);
    let counted = walk("ကများ ၏ အကွက်အရေအတွက် ကို ပြသမည်။");
    assert_eq!(dups(&counted), 0);
    let written = walk("ကများ ၏ အကွက် ၀ ကို က ဟု ထားမည်။");
    assert_eq!(dups(&written), 1);
    let grown = walk("ကများ ထဲသို့ က ကို ထည့်မည်။");
    assert_eq!(dups(&grown), 1);
    let called = walk("ချဲ့ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။");
    assert_eq!(dups(&called), 1);
}

/// The count of duplicates the checked module of a whole program keeps: each
/// `dup` that is not lent counts a shared block or copies lent cells.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// The count.
fn dups(text: &str) -> usize
{
    let out = run(text);
    assert!(out.out.diagnostics.is_empty(), "{:?}", out.out.diagnostics);
    let module = out.out.lowered.folded.as_ref().map(ToSexpr::to_sexpr);
    module.unwrap_or_default().matches("(dup ").count()
}
