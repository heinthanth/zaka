use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::expr;
use crate::common::stmt;

#[test]
fn list_kinds_take_each_spelling()
{
    let decl = |kind: &str| stmt(&format!("ကိန်းရှင် က သည် {kind} တစ်ခု ဖြစ်ပါစေ။"));
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း"), "(var က isize[3])");
    assert_eq!(decl("အကွက် ၃ ရှိသော ကိန်းပြည့် စာရင်း"), "(var က isize[3])");
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် ၏ စာရင်း"), "(var က isize[3])");
    assert_eq!(decl("အကွက် ၃ ရှိသော ကိန်းပြည့် ၏ စာရင်း"), "(var က isize[3])");
    assert_eq!(decl("ကိန်းပြည့် စာရင်း"), "(var က isize[])");
    assert_eq!(decl("ကိန်းပြည့် ၏ စာရင်း"), "(var က isize[])");
}

#[test]
fn list_kinds_nest_and_mix_with_pointers()
{
    let decl = |kind: &str| stmt(&format!("ကိန်းရှင် က သည် {kind} တစ်ခု ဖြစ်ပါစေ။"));
    assert_eq!(
        decl("အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း"),
        "(var က isize[3][2])"
    );
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း"), "(var က isize[3][])");
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း"), "(var က isize*[3])");
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ"), "(var က isize[3]*)");
    assert_eq!(decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ၏ လိပ်စာ"), "(var က isize[3]*)");
    assert_eq!(decl("အကွက် ၂ ခုရှိသော လိပ်စာ စာရင်း"), "(var က ptr[2])");
}

#[test]
fn a_range_reads_its_start_and_length()
{
    assert_eq!(expr("က ၏ အကွက် ၁ မှစ၍ ၂ ကွက်"), "(range က 1 2)");
    assert_eq!(expr("က ၏ အကွက် i + ၁ မှစ၍ န - ၁ ကွက်"), "(range က (+ i 1) (- န 1))");
    assert_eq!(expr("က ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ၏ အကွက် ၀"), "(cell (range က 1 2) 0)");
    assert_eq!(codes("က ၏ အကွက် ၁ မှစ၍ ၂ ကို ပြသမည်။"), [Code::ExpectedToken]);
}

#[test]
fn a_struct_can_name_the_cell_word_as_its_classifier()
{
    let decl =
        "ကွက်လပ် ဟူသော ဒေတာအစုအဝေးတွင် အမှတ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ကွက် ဖြစ်သည်။";
    assert_eq!(codes(decl), []);
    let heap = format!(
        "{decl}\nကိန်းရှင် ပ သည် ကွက်လပ် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။\nမင်မိုရီတွင် ကွက်လပ် ၃ ကွက်စာ နေရာယူ၍ လိပ်စာကို ပ \
         တွင် သိမ်းထားမည်။"
    );
    assert_eq!(codes(&heap), []);
    let lit = format!("{decl}\nအမှတ် ၁ ရှိသော ကွက်လပ်တစ်ကွက် ကို ပြသမည်။");
    assert_eq!(codes(&lit), []);
}

#[test]
fn a_list_count_can_be_an_expression()
{
    let decl = |kind: &str| stmt(&format!("ကိန်းရှင် က သည် {kind} တစ်ခု ဖြစ်ပါစေ။"));
    assert_eq!(decl("အကွက် N ခုရှိသော ကိန်းပြည့် စာရင်း"), "(var က isize[N])");
    assert_eq!(decl("အကွက် N * ၂ ရှိသော ကိန်းပြည့် စာရင်း"), "(var က isize[(* N 2)])");
    assert_eq!(
        decl("အကွက် (N + ၁) * ၂ ခုရှိသော အကွက် M ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း"),
        "(var က isize[M][(* [(+ N 1)] 2)])"
    );
    let declared =
        stmt("ကိန်းရှင် က သည် အကွက် N ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁] ဖြစ်ပါစေ။");
    assert!(declared.starts_with("(var က isize[N]"), "{declared}");
}

#[test]
fn a_list_count_is_a_whole_number_above_zero()
{
    let bad = |kind: &str| codes(&format!("ကိန်းရှင် က သည် {kind} တစ်ခု ဖြစ်ပါစေ။"));
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
    assert_eq!(stmt("က ၏ အကွက် ၀ ကို ၅ ဟု ထားမည်။"), "(set (cell က 0) 5)");
    assert_eq!(stmt("က ၏ ၁ ခုမြောက်အကွက် ကို ပြသမည်။"), "(print (cell1 က 1))");
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
        stmt("ကိန်းရှင် က သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ တို့ပါသော စာရင်း ဖြစ်ပါစေ။"),
        "(var က isize[] (list 1 2))"
    );
    assert_eq!(stmt("၁၊ ၂ တို့ပါသော စာရင်း ကို ပြသမည်။"), "(print (list 1 2))");
}

#[test]
fn list_literals_take_brackets()
{
    assert_eq!(expr("[၁၊ ၂၊ ၃]"), "(list 1 2 3)");
    assert_eq!(expr("[၁]"), "(list 1)");
    assert_eq!(expr("[]"), "blank_list");
    assert_eq!(expr("[[၁၊ ၂]၊ [၃၊ ၄]]"), "(list (list 1 2) (list 3 4))");
    assert_eq!(expr("[၁ + ၂၊ -က]"), "(list (+ 1 2) (neg က))");
    assert_eq!(expr("[၁၊ ၂]၊ [၃] တို့ပါသော စာရင်း"), "(list (list 1 2) (list 3))");
    assert_eq!(stmt("[၁၊ ၂] ကို ပြသမည်။"), "(print (list 1 2))");
    assert_eq!(
        stmt("f ဟူသော လုပ်ဆောင်ချက်ကို [၁၊ ၂]၊ န တို့ဖြင့် လုပ်ဆောင်မည်။"),
        "(call f (list 1 2) န)"
    );
}

#[test]
fn a_bracket_list_closes()
{
    assert_eq!(codes("[၁၊ ၂ ကို ပြသမည်။"), [Code::ExpectedToken]);
}

#[test]
fn a_list_literal_in_call_arguments()
{
    assert_eq!(stmt("f ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်မည်။"), "(call f 1 2)");
    assert_eq!(
        stmt("f ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ ၂ တို့ပါသော စာရင်း တို့ဖြင့် လုပ်ဆောင်မည်။"),
        "(call f (list 1 2))"
    );
    assert_eq!(
        stmt("f ဟူသော လုပ်ဆောင်ချက်ကို (၁၊ ၂ တို့ပါသော စာရင်း)၊ န တို့ဖြင့် လုပ်ဆောင်မည်။"),
        "(call f [(list 1 2)] န)"
    );
    assert_eq!(
        stmt("f ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ ၂ တို့ပါသော စာရင်း၊ န တို့ဖြင့် လုပ်ဆောင်မည်။"),
        "(call f (list 1 2) န)"
    );
}
