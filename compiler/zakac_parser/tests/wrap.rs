use zakac_parser::wrap::in_start_block;
use zakac_parser::wrap::in_start_block_lines;
use zakac_parser::wrap::wrap_in_start;

#[test]
fn statements_move_into_a_start_block_at_the_end()
{
    let text = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
// show it
က ကို ပြသပါ။
လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ကိန်းရှင် ခ သည် ၂ ဖြစ်ပါစေ။
ခ ကို ပြသပါ။   // and this
";
    let want = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။

// show it
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ပြသပါ။
ကိန်းရှင် ခ သည် ၂ ဖြစ်ပါစေ။
ခ ကို ပြသပါ။   // and this
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";
    assert_eq!(in_start_block(text), want);
}

#[test]
fn a_program_of_declarations_gets_a_blank_start_block()
{
    let want = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";
    assert_eq!(in_start_block("ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။"), want);
}

#[test]
fn one_line_of_statements_moves_whole()
{
    let want = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြသပါ။ ၂ ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";
    assert_eq!(in_start_block("၁ ကို ပြသပါ။ ၂ ကို ပြသပါ။"), want);
}

#[test]
fn a_program_with_a_start_block_is_left_alone()
{
    let text = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ ပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    assert_eq!(wrap_in_start(text, |_, _, _| true), None);
    assert_eq!(in_start_block(text), text);
}

#[test]
fn keep_decides_for_each_declaration()
{
    let text = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။\nက ကို ပြသပါ။";
    let all_in = wrap_in_start(text, |_, _, _| false).unwrap().text;
    assert!(all_in.starts_with("ပရိုဂရမ်သည်"), "{all_in}");
}

#[test]
fn a_variable_a_function_uses_remains_at_the_top_level()
{
    let text = "၁ ကို ပြသပါ။
ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၂ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ က ကို ပြသပါ။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    let want = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ က ကို ပြသပါ။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ ကို ပြသပါ။
ကိန်းရှင် ခ သည် ၂ ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";
    assert_eq!(in_start_block(text), want);
}

#[test]
fn each_new_line_knows_its_old_line()
{
    let text = "၁ ကို ပြသပါ။
လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
၂ ကို ပြသပါ။";
    let wrapped = in_start_block_lines(text);
    let want = [Some(1), Some(2), None, None, Some(0), Some(3), None];
    assert_eq!(wrapped.lines, want, "{}", wrapped.text);
}
