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
        "(alloc (deref p) န *isize)"
    );
    assert_eq!(
        stmt("မင်မိုရီတွင် ကိန်းပြည့် လိပ်စာ န ခုစာ နေရာယူ၍ လိပ်စာကို p ၏ တန်ဖိုး တွင် သိမ်းထားမည်။"),
        "(alloc (deref p) န *isize)"
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
