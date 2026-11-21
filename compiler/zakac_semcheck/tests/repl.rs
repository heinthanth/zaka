use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Code;
use zakac_semcheck::semcheck::SemcheckOptions;

use crate::common::Checked;
use crate::common::check_as_is;
use crate::common::codes_as_is;

/// Check `text` as it stands in REPL mode.
fn repl(text: &str) -> Checked
{
    check_as_is(text, &SemcheckOptions::builder().repl(true).build())
}

/// The codes of `text` in REPL mode.
fn repl_codes(text: &str) -> Vec<Code>
{
    repl(text).out.diagnostics.iter().map(|d| d.code).collect()
}

/// The statements of the start block of a checked program.
fn start_stmts(checked: &Checked) -> &[Stmt]
{
    let start = checked.program.stmts.iter().find_map(|s| match s
    {
        Stmt::Fn(f) if f.start => Some(f),
        _ => None,
    });
    &start.expect("a start block").block.stmts
}

const LOOSE: &str = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
က ကို ပြသမည်။
ကိန်းရှင် ခ သည် ၂ ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
";

#[test]
fn statements_at_the_top_level_form_the_start_block_in_repl_mode()
{
    assert_eq!(repl_codes(LOOSE), []);
    assert!(codes_as_is(LOOSE).contains(&Code::TopLevelStmt));
}

#[test]
fn a_declaration_after_a_statement_moves_in_order()
{
    let checked = repl(LOOSE);
    let moved = start_stmts(&checked);
    assert_eq!(moved.len(), 3);
    assert!(matches!(moved[1], Stmt::Decl(_)));
    let top = checked.program.stmts.iter().filter(|s| matches!(s, Stmt::Decl(_)));
    assert_eq!(top.count(), 1);
}

#[test]
fn declarations_alone_get_a_blank_start_block()
{
    let checked = repl("ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။\n");
    assert_eq!(checked.out.diagnostics.len(), 0);
    assert!(start_stmts(&checked).is_empty());
}

#[test]
fn a_program_with_a_start_block_remains_as_it_is()
{
    let text = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n၁ ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let plain = check_as_is(text, &SemcheckOptions::default()).program;
    assert_eq!(repl(text).program, plain);
    let loose = format!("၂ ကို ပြသမည်။\n{text}");
    assert_eq!(repl_codes(&loose), [Code::TopLevelStmt]);
}
