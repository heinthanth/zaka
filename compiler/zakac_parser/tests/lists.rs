mod common;

use common::codes;
use common::expr;
use common::stmt;
use zakac_diagnostics::Code;

#[test]
fn list_kinds_take_each_spelling()
{
    let decl = |kind: &str| stmt(&format!("ကိန်းရှင် က သည် {kind} ဖြစ်ပါစေ။"));
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း"), "(var က [3]isize)");
    assert_eq!(decl("အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း"), "(var က [3]isize)");
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း"), "(var က [3]isize)");
    assert_eq!(decl("အကွက် ၃ ရှိသော ကိန်းပြည့် ၏ စာရင်း"), "(var က [3]isize)");
    assert_eq!(decl("ကိန်းပြည့် စာရင်း"), "(var က []isize)");
    assert_eq!(decl("ကိန်းပြည့် ၏ စာရင်း"), "(var က []isize)");
}

#[test]
fn list_kinds_nest_and_mix_with_pointers()
{
    let decl = |kind: &str| stmt(&format!("ကိန်းရှင် က သည် {kind} ဖြစ်ပါစေ။"));
    assert_eq!(
        decl("အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း"),
        "(var က [2][3]isize)"
    );
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း"), "(var က [][3]isize)");
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း"), "(var က [3]*isize)");
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ"), "(var က *[3]isize)");
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ၏ လိပ်စာ"), "(var က *[3]isize)");
    assert_eq!(decl("အကွက် ၂ ခုရှိသော လိပ်စာ စာရင်း"), "(var က [2]ptr)");
}

#[test]
fn a_list_count_is_a_whole_number_above_zero()
{
    let bad = |kind: &str| codes(&format!("ကိန်းရှင် က သည် {kind} ဖြစ်ပါစေ။"));
    assert_eq!(bad("အကွက် ၀ ခုရှိသော ကိန်းပြည့် စာရင်း"), [Code::BadListCount]);
    assert_eq!(bad("အကွက် ၂ ခုရှိသော ကိန်းပြည့်"), [Code::ExpectedToken]);
}

#[test]
fn cells_count_from_zero_or_from_one()
{
    assert_eq!(expr("က ၏ အကွက် ၀"), "(cell က 0)");
    assert_eq!(expr("က ၏ အကွက် န + ၁"), "(cell က (+ န 1))");
    assert_eq!(expr("က ၏ ၂ ခုမြောက်အကွက်"), "(cell1 က 2)");
    assert_eq!(expr("က ၏ ၂ ခုမြောက် အကွက်"), "(cell1 က 2)");
    assert_eq!(expr("က ၏ အကွက် ၁ ၏ အကွက် ၂"), "(cell (cell က 1) 2)");
    assert_eq!(expr("က ၏ အကွက် ၁ ၏ ၃ ခုမြောက်အကွက်"), "(cell1 (cell က 1) 3)");
    assert_eq!(expr("က ၏ အကွက်အရေအတွက်"), "(cell_count က)");
    assert_eq!(expr("က ၏ အကွက် ၀ ၏ လိပ်စာ"), "(addr (cell က 0))");
}

#[test]
fn a_cell_is_a_place()
{
    assert_eq!(stmt("က ၏ အကွက် ၀ ကို ၅ ဟု ထားပါ။"), "(set (cell က 0) 5)");
    assert_eq!(stmt("က ၏ ၁ ခုမြောက်အကွက် ကို ပြသပါ။"), "(print (cell1 က 1))");
}

#[test]
fn list_literals_close_with_a_list_word()
{
    assert_eq!(expr("၁၊ ၂၊ ၃ တို့ပါသော စာရင်း"), "(list 1 2 3)");
    assert_eq!(expr("၁ တို့ပါသော စာရင်း"), "(list 1)");
    assert_eq!(
        expr("(၁၊ ၂ တို့ပါသော စာရင်း)၊ (၃၊ ၄ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း"),
        "(list [(list 1 2)] [(list 3 4)])"
    );
    assert_eq!(expr("စာရင်းအလွတ်"), "blank_list");
    assert_eq!(
        stmt("ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ တို့ပါသော စာရင်း ဖြစ်ပါစေ။"),
        "(var က []isize (list 1 2))"
    );
    assert_eq!(stmt("၁၊ ၂ တို့ပါသော စာရင်း ကို ပြသပါ။"), "(print (list 1 2))");
}

#[test]
fn a_list_literal_in_call_arguments()
{
    assert_eq!(stmt("လုပ်ဆောင်ချက် f ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။"), "(call f 1 2)");
    assert_eq!(
        stmt("လုပ်ဆောင်ချက် f ကို ၁၊ ၂ တို့ပါသော စာရင်း တို့ဖြင့် လုပ်ဆောင်ပါ။"),
        "(call f (list 1 2))"
    );
    assert_eq!(
        stmt("လုပ်ဆောင်ချက် f ကို (၁၊ ၂ တို့ပါသော စာရင်း)၊ န တို့ဖြင့် လုပ်ဆောင်ပါ။"),
        "(call f [(list 1 2)] န)"
    );
    assert_eq!(
        stmt("လုပ်ဆောင်ချက် f ကို ၁၊ ၂ တို့ပါသော စာရင်း၊ န တို့ဖြင့် လုပ်ဆောင်ပါ။"),
        "(call f (list 1 2) န)"
    );
}
