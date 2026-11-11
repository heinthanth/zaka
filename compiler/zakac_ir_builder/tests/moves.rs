mod common;

use common::plain;

/// `p` an owner of an integer.
const OWNER: &str = "ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။\n";

/// The owner prelude, then `rest`.
fn owner(rest: &str) -> String { format!("{OWNER}{rest}") }

#[test]
fn reads_that_do_not_move_remain_reads()
{
    let text = owner(
        "p ကို ပြသမည်။
p + ၁ ၏ တန်ဖိုး ကို ပြသမည်။
p == မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ ကို ပြသမည်။",
    );
    let forms = plain(&text);
    assert_eq!(forms[1], "(print g0)");
    assert_eq!(forms[2], "(print (load (ptr+ g0 1:isize)))");
    assert!(forms[3].starts_with("(print (== g0 "), "{}", forms[3]);
    assert!(!forms[3].contains("take"), "{}", forms[3]);
}

#[test]
fn a_borrow_variable_is_not_taken()
{
    let text = owner(
        "ကိန်းရှင် q သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် p ဖြစ်ပါစေ။
ကိန်းရှင် r သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် q ဖြစ်ပါစေ။",
    );
    assert_eq!(plain(&text)[2], "(set g2 g1)");
}

#[test]
fn an_owner_compared_with_its_borrow_needs_no_conversion()
{
    let text = owner(
        "ကိန်းရှင် q သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
p == q ကို ပြသမည်။
q == p ကို ပြသမည်။",
    );
    let forms = plain(&text);
    assert_eq!(forms[2], "(print (== g0 g1))");
    assert_eq!(forms[3], "(print (== g1 g0))");
}
