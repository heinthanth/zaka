use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::stmt;

#[test]
fn the_mark_reads_with_or_without_spaces_and_takes_a_statement_or_a_block()
{
    let one = stmt("လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် ၁ ကို ပြသမည်။");
    assert!(one.starts_with("(defer (block"), "{one}");
    for spelling in ["လုပ်ဆောင်စရာများ ပြီးသည့် အခါတွင်", "လုပ်ဆောင်စရာများပြီးသည့်အခါတွင်"]
    {
        assert_eq!(stmt(&format!("{spelling} ၁ ကို ပြသမည်။")), one, "{spelling}");
    }
    let block = stmt(
        "လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n၁ ကို ပြသမည်။\n၂ ကို \
         ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။",
    );
    assert!(block.starts_with("(defer (block"), "{block}");
    assert_eq!(block.matches("print").count(), 2, "{block}");
}

#[test]
fn a_mark_alone_in_a_branch_is_zk0272()
{
    let arm = "အကယ်၍ မှန် လျှင် လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် ၁ ကို ပြသမည်။";
    assert_eq!(codes(arm), [Code::DeferAlone]);
    let twice = "လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် ၁ ကို ပြသမည်။";
    assert_eq!(codes(twice), [Code::DeferAlone]);
}

#[test]
fn the_failure_mark_reads_as_its_own_form()
{
    let text = "မအောင်မြင်၍ လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် ၁ ကို ပြသမည်။";
    assert!(stmt(text).starts_with("(defer-failure (block"), "{}", stmt(text));
    let glued = "မအောင်မြင်၍လုပ်ဆောင်စရာများပြီးသည့်အခါတွင် ၁ ကို ပြသမည်။";
    assert_eq!(stmt(glued), stmt(text));
}
