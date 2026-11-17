use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::folded;
use crate::common::plain;

/// `l` a list of three integers.
const LIST: &str = "ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။\n";

#[test]
fn lists_lower_to_list_nodes_and_cell_addresses()
{
    let forms = plain(&format!("{LIST}l ၏ အကွက် ၁ ကို ၅ ဟု ထားမည်။\nl ကို ပြသမည်။"));
    assert_eq!(forms[0], "(set g0 blank_list)");
    assert_eq!(forms[1], "(store (cell@ g0 1:usize 3) 5:isize)");
    assert_eq!(forms[2], "(print g0)");
    let lit = plain(
        "ကိန်းရှင် l သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ နှင့် ၂ တို့ပါသော စာရင်း \
         ဖြစ်ပါစေ။",
    );
    assert_eq!(lit, ["(set g0 (list 1:isize 2:isize))"]);
}

#[test]
fn a_cell_read_loads_and_a_count_is_a_constant()
{
    let text = format!(
        "{LIST}ကိန်းရှင် i သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nl ၏ အကွက် i ကို ပြသမည်။\nl ၏ \
         အကွက်အရေအတွက် ကို ပြသမည်။"
    );
    let forms = plain(&text);
    assert_eq!(forms[2], "(print (load (cell@ g0 g1 3)))");
    assert_eq!(forms[3], "(print 3:usize)");
}

#[test]
fn a_constant_cell_number_is_checked_when_folded()
{
    let forms = folded(&format!("{LIST}l ၏ ၃ ခုမြောက်အကွက် ကို ပြသမည်။"));
    assert_eq!(forms[1], "(print (load (cell1@ g0 3:usize 3 checked)))");
    let pointer =
        "ကိန်းရှင် p သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ဖြစ်ပါစေ။\np ၏ အကွက် ၀ ကို ပြသမည်။";
    assert_eq!(folded(pointer)[1], "(print (load (cell@ g0 0:usize 3 checked)))");
}

#[test]
fn a_constant_cell_number_outside_the_list_is_an_error()
{
    assert_eq!(codes(&format!("{LIST}l ၏ အကွက် ၃ ကို ပြသမည်။")), [Code::CellOutOfRange]);
    assert_eq!(
        codes(&format!("{LIST}l ၏ ၀ ခုမြောက်အကွက် ကို ပြသမည်။")),
        [Code::CellOutOfRange]
    );
    assert_eq!(codes(&format!("{LIST}l ၏ အကွက် ၁ + ၂ ကို ပြသမည်။")), [Code::CellOutOfRange]);
    let negative = "ကိန်းသေ n သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် -၁ ဖြစ်ပါစေ။\n";
    assert_eq!(
        codes(&format!("{LIST}{negative}l ၏ အကွက် n ကို ပြသမည်။")),
        [Code::CellOutOfRange]
    );
}

#[test]
fn whole_lists_compare_with_a_comparison()
{
    let forms = plain(&format!("{LIST}(l == l) ကို ပြသမည်။"));
    assert_eq!(forms[1], "(print (== g0 g0))");
}

/// A long list of an enumeration set to `ကနဦးတန်ဖိုး` holds its initial cell
/// once, not once for each cell.
#[test]
fn a_long_initial_list_holds_one_cell()
{
    let text = "အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။
ကိန်းရှင် ရောင်များ သည် အကွက် ၁၀၀၀၀၀ ခုရှိသော အရောင် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။";
    let forms = plain(text).concat();
    assert!(forms.contains("(repeat 100000 "), "{forms}");
    assert!(forms.len() < 400, "{forms}");
}
