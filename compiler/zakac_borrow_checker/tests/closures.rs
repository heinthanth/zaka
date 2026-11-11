mod common;

use common::codes;
use common::ok;
use zakac_diagnostics::Code;

/// `ခေါ်` calls the function it gets, and `နှစ်ခု` calls it while it holds a
/// block handed over to it.
const HEAD: &str = "\
လုပ်ဆောင်ချက် ခေါ် သည် ဖ ဟူသော (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ဖ ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် နှစ်ခု သည်
ငှား ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လည်းကောင်း ဖ ဟူသော (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို \
                    လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ဖ ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ငှား ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ငှား ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// The start of the start block: `ပ` holds a block, `ဖျက်` frees it and `ပြ`
/// reads it, both declared inside the start block, sharing `ပ`. Each does its
/// pointer work in a marked block, so neither is a marked function, and both
/// can be values.
const LOCALS: &str = "\
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ဖျက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ပြ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// The line that reads through the alias `ငှား`, in a marked block.
const READ: &str = "\
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
ငှား ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// The line that frees the block of `ပ`, in a marked block.
const FREE: &str = "\
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// The line that sets the alias `ငှား` to the pointer `ပ`.
const ALIAS: &str = "ကိန်းရှင် ငှား သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။\n";

/// The program: `HEAD`, then a start block of `LOCALS` and `stmts`.
fn program(stmts: &str) -> String
{
    format!("{HEAD}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{LOCALS}{stmts}ပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

#[test]
fn an_alias_used_after_a_call_that_gets_a_freeing_closure_is_zk0704()
{
    let call = "လုပ်ဆောင်ချက် ခေါ် ကို ဖျက် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။\n";
    let text = program(&format!("{ALIAS}{call}{READ}{FREE}"));
    assert_eq!(codes(&text), [Code::InvalidatedLoan]);
}

#[test]
fn a_block_a_closure_can_free_is_no_leak()
{
    let call = "လုပ်ဆောင်ချက် ခေါ် ကို ဖျက် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။\n";
    ok(&program(call));
    ok(&program(&format!("{call}{FREE}")));
}

#[test]
fn a_closure_beside_an_owned_argument_whose_block_it_can_free_is_zk0707()
{
    let call = "\
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် နှစ်ခု ကို ပ၊ ဖျက် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";
    let text = program(call);
    assert_eq!(codes(&text), [Code::AliasedArgument]);
}

#[test]
fn a_closure_kept_in_a_variable_runs_where_it_is_called()
{
    let call = "\
ကိန်းရှင် ဂ သည် ဖျက် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဂ ကို လုပ်ဆောင်မည်။
";
    let text = program(&format!("{ALIAS}{call}{READ}"));
    assert_eq!(codes(&text), [Code::InvalidatedLoan]);
}

#[test]
fn a_closure_that_reads_leaves_the_block_alone()
{
    let call = "လုပ်ဆောင်ချက် ခေါ် ကို ပြ ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။\n";
    ok(&program(&format!("{ALIAS}{call}{READ}{FREE}")));
}

#[test]
fn a_closure_a_nested_function_passes_on_is_checked()
{
    let call = "\
လုပ်ဆောင်ချက် ကြား သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ခေါ် ကို ဖျက် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ကြား ကို လုပ်ဆောင်မည်။
";
    let text = program(&format!("{ALIAS}{call}{READ}"));
    assert_eq!(codes(&text), [Code::InvalidatedLoan]);
}

#[test]
fn a_variable_holds_the_closure_of_its_last_set()
{
    // `ဂ` runs `ပြ` first, and holds `ဖျက်` from a later set alone.
    let first = "\
ကိန်းရှင် ဂ သည် ပြ ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဂ ကို လုပ်ဆောင်မည်။
";
    let then = "\
ဂ ကို ဖျက် ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။
လုပ်ဆောင်ချက် ဂ ကို လုပ်ဆောင်မည်။
";
    ok(&program(&format!("{ALIAS}{first}{READ}{then}")));
}

#[test]
fn a_closure_set_on_one_path_can_run_after_the_paths_meet()
{
    let call = "\
ကိန်းရှင် ဂ သည် ပြ ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ကိန်းရှင် လ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
လ ကို ပ ၏ တန်ဖိုး ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အကယ်၍ လ > ၀ ဖြစ်လျှင် ဂ ကို ဖျက် ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။
လုပ်ဆောင်ချက် ဂ ကို လုပ်ဆောင်မည်။
";
    let text = program(&format!("{ALIAS}{call}{READ}"));
    assert_eq!(codes(&text), [Code::InvalidatedLoan]);
}

#[test]
fn a_closure_two_levels_in_is_checked()
{
    let call = "\
လုပ်ဆောင်ချက် ကြား သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ခေါ် ကို ဖျက် ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ထိပ် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ကြား ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ထိပ် ကို လုပ်ဆောင်မည်။
";
    let text = program(&format!("{ALIAS}{call}{READ}"));
    assert_eq!(codes(&text), [Code::InvalidatedLoan]);
}
