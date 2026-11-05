mod common;

use common::codes;
use common::parse_text;
use common::stmt;
use zakac_diagnostics::Code;

#[test]
fn heap_statements()
{
    assert_eq!(stmt("မင်မိုရီတွင် ၆၄ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။"), "(alloc p 64)");
    assert_eq!(
        stmt("မှတ်ဉာဏ်တွင် ကိန်းပြည့် ၁၀ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။"),
        "(alloc p 10 isize)"
    );
    assert_eq!(
        stmt("မင်မိုရီတွင် ကိန်းပြည့် ၏ အရွယ်အစား * ၂ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။"),
        "(alloc p (* (sizeof isize) 2))"
    );
    assert_eq!(
        stmt("မင်မိုရီတွင် ကိန်းပြည့် ၏ လိပ်စာ န ခုစာ နေရာယူ၍ လိပ်စာကို p ၏ တန်ဖိုး တွင် သိမ်းထားမည်။"),
        "(alloc (deref p) န isize*)"
    );
    assert_eq!(
        stmt("မင်မိုရီတွင် ကိန်းပြည့် လိပ်စာ န ခုစာ နေရာယူ၍ လိပ်စာကို p ၏ တန်ဖိုး တွင် သိမ်းထားမည်။"),
        "(alloc (deref p) န isize*)"
    );
    assert_eq!(
        stmt("လိပ်စာ p ရှိ မင်မိုရီကို န * ၂ ဘိုက်စာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။"),
        "(resize p (* န 2))"
    );
    assert_eq!(
        stmt("လိပ်စာ p ရှိ မင်မိုရီကို ကိန်းပြည့် ဆ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။"),
        "(resize p ဆ isize)"
    );
    assert_eq!(stmt("လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။"), "(free p)");
    assert_eq!(stmt("လိပ်စာ q ၏ တန်ဖိုး ရှိ မှတ်ဉာဏ်ကို ပြန်လည်စွန့်လွှတ်မည်။"), "(free (deref q))");
}

#[test]
fn a_kind_prefixed_literal_in_a_size_needs_parentheses()
{
    let found = codes("မင်မိုရီတွင် ကိန်းပြည့် ၁၀ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။");
    assert_eq!(found, [Code::ExpectedToken]);
    let diag =
        &parse_text("မင်မိုရီတွင် ကိန်းပြည့် ၁၀ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။").diagnostics[0];
    assert!(diag.help.is_some(), "the message suggests parentheses");
    assert_eq!(
        stmt("မင်မိုရီတွင် (ကိန်းပြည့် ၁၀) ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။"),
        "(alloc p [10isize])"
    );
}

#[test]
fn blits_and_fills()
{
    assert_eq!(
        stmt("လိပ်စာ p ရှိ ကိန်းပြည့် ၁၀ ခုစာ မင်မိုရီကို လိပ်စာ q ဆီသို့ ကူးယူမည်။"),
        "(blit p 10 isize q)"
    );
    assert_eq!(
        stmt("လိပ်စာ p ရှိ ကိန်းပြည့် ၉ ခုစာ မင်မိုရီကို လိပ်စာ p + ၁ ဆီသို့ ပြောင်းထည့်မည်။"),
        "(shift p 9 isize (+ p 1))"
    );
    assert_eq!(
        stmt("လိပ်စာ p ရှိ မင်မိုရီကို -၁ ဖြင့် ကိန်းပြည့် ၁၀ ခုစာ ဖြည့်မည်။"),
        "(fill p 10 isize (neg 1))"
    );
    assert_eq!(stmt("လိပ်စာ p ရှိ မင်မိုရီကို ၀ ဖြင့် ၆၄ ဘိုက်စာ ဖြည့်မည်။"), "(fill p 64 0)");
    assert_eq!(stmt("လိပ်စာ p ရှိ မင်မိုရီကို ၆၄ ဘိုက်စာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။"), "(resize p 64)");
    assert_eq!(
        stmt("လိပ်စာ p ရှိ မင်မိုရီကို ကိန်းပြည့် ၁၀ ခုစာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။"),
        "(resize p 10 isize)"
    );
    assert_eq!(
        stmt("လိပ်စာ p ရှိ မင်မိုရီကို (က ကို ၂ ဖြင့် စားသော စားလဒ်) ဖြင့် ၁ ဘိုက်စာ ဖြည့်မည်။"),
        "(fill p 1 [(div က 2)])"
    );
    assert_eq!(
        stmt("လိပ်စာ p ရှိ မင်မိုရီကို က ကို ၂ ဖြင့် စားသော စားလဒ် ဖြင့် ၁ ဘိုက်စာ ဖြည့်မည်။"),
        "(fill p 1 (div က 2))"
    );
    // A value with a kind word in front reads as a value: the verb at the end
    // makes the statement a fill.
    assert_eq!(
        stmt("လိပ်စာ p ရှိ မင်မိုရီကို ၈ ဘစ် ကိန်းပြည့် ၅ ဖြင့် ၈ ဘစ် ကိန်းပြည့် ၄ ခုစာ ဖြည့်မည်။"),
        "(fill p 4 i8 5i8)"
    );
    assert_eq!(stmt("လိပ်စာ p ရှိ မင်မိုရီကို ကိန်းပြည့် ၁ ဖြင့် ၈ ဘိုက်စာ ဖြည့်မည်။"), "(fill p 8 1isize)");
}
