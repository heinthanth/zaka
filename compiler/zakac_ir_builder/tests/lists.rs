mod common;

use common::codes;
use common::folded;
use common::plain;
use zakac_diagnostics::Code;

/// `l` a list of three integers.
const LIST: &str = "ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။\n";

#[test]
fn lists_lower_to_list_nodes_and_cell_addresses()
{
    let forms = plain(&format!("{LIST}l ၏ အကွက် ၁ ကို ၅ ဟု ထားပါ။\nl ကို ပြသပါ။"));
    assert_eq!(forms[0], "(set g0 blank_list)");
    assert_eq!(forms[1], "(store (cell@ g0 1:usize 3) 5:isize)");
    assert_eq!(forms[2], "(print g0)");
    let lit = plain("ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ တို့ပါသော စာရင်း ဖြစ်ပါစေ။");
    assert_eq!(lit, ["(set g0 (list 1:isize 2:isize))"]);
}

#[test]
fn a_cell_read_loads_and_a_count_is_a_constant()
{
    let text = format!(
        "{LIST}ကိန်းရှင် i သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\nl ၏ အကွက် i ကို ပြသပါ။\nl ၏ အကွက်အရေအတွက် ကို \
         ပြသပါ။"
    );
    let forms = plain(&text);
    assert_eq!(forms[2], "(print (load (cell@ g0 g1 3)))");
    assert_eq!(forms[3], "(print 3:usize)");
}

#[test]
fn a_constant_cell_number_is_checked_when_folded()
{
    let forms = folded(&format!("{LIST}l ၏ ၃ ခုမြောက်အကွက် ကို ပြသပါ။"));
    assert_eq!(forms[1], "(print (load (cell1@ g0 3:usize 3 checked)))");
    let pointer =
        "ကိန်းရှင် p သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ ဖြစ်ပါစေ။\np ၏ အကွက် ၀ ကို ပြသပါ။";
    assert_eq!(folded(pointer)[1], "(print (load (cell@ g0 0:usize 3 checked)))");
}

#[test]
fn a_constant_cell_number_outside_the_list_is_an_error()
{
    assert_eq!(codes(&format!("{LIST}l ၏ အကွက် ၃ ကို ပြသပါ။")), [Code::CellOutOfRange]);
    assert_eq!(
        codes(&format!("{LIST}l ၏ ၀ ခုမြောက်အကွက် ကို ပြသပါ။")),
        [Code::CellOutOfRange]
    );
    assert_eq!(codes(&format!("{LIST}l ၏ အကွက် ၁ + ၂ ကို ပြသပါ။")), [Code::CellOutOfRange]);
    let negative = "ကိန်းသေ n သည် ကိန်းပြည့် -၁ ဖြစ်ပါစေ။\n";
    assert_eq!(
        codes(&format!("{LIST}{negative}l ၏ အကွက် n ကို ပြသပါ။")),
        [Code::CellOutOfRange]
    );
}

#[test]
fn whole_lists_compare_with_a_comparison()
{
    let forms = plain(&format!("{LIST}(l == l) ကို ပြသပါ။"));
    assert_eq!(forms[1], "(print (== g0 g0))");
}
