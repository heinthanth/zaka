//! Compile-time conditionals and the facts of the target: the loader keeps
//! the arm that holds for the target, and the core's marked constants hold
//! its facts.

use zakac_ast::expr::Expr;
use zakac_ast::stmt::Stmt;
use zakac_ast::target::BuildTarget;
use zakac_ast::target::Target;
use zakac_ast::target::TargetAbi;
use zakac_ast::target::TargetArch;
use zakac_ast::target::TargetOs;
use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::Loaded;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// A target on an OS, 64-bit x86, for the VM.
fn on(os: TargetOs) -> Target
{
    #[rustfmt::skip]
    return Target::builder().os(os).arch(TargetArch::X86_64).bits(64).output(BuildTarget::Vm)
        .build();
}

/// Read a program of one file for a target.
fn load(text: &str, target: &Target) -> Loaded
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    Loaded::single_for(&file, &LexerOptions::default(), target)
}

/// The names of the constants a program declares, in order, the facts of the
/// core left out.
fn constants(loaded: &Loaded) -> Vec<String>
{
    let decls = loaded.program.stmts.iter().filter_map(|s| match s
    {
        Stmt::Decl(d) if !d.builtin => Some(d.name.name.to_string()),
        _ => None,
    });
    decls.collect()
}

/// The diagnostic codes of a program read for a target.
fn codes(text: &str, target: &Target) -> Vec<Code>
{
    load(text, target).diagnostics.iter().map(|d| d.code).collect()
}

/// A chain that declares one constant on Windows and another elsewhere.
const CHAIN: &str = "အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == \"windows\" ဖြစ်လျှင် \
                     အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းသေ ပြတင်း သည် ၁ ဖြစ်ပါစေ။\nလုပ်ဆောင်စရာများ \
                     ပြီးဆုံးပါပြီး။\nသို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းသေ ယူနစ် သည် ၂ \
                     ဖြစ်ပါစေ။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";

/// The arm that holds for the target stands in place of the conditional,
/// at the top level, and the other arm is gone.
#[test]
fn the_arm_for_the_target_stands_in_place()
{
    let windows = load(CHAIN, &on(TargetOs::Windows));
    assert!(windows.diagnostics.is_empty(), "{:?}", windows.diagnostics);
    assert_eq!(constants(&windows), ["ပြတင်း"]);
    assert_eq!(windows.program.target, Some(on(TargetOs::Windows)));
    assert_eq!(constants(&load(CHAIN, &on(TargetOs::Linux))), ["ယူနစ်"]);
}

/// A conditional in a block keeps its arm among the statements of the block,
/// with no scope of its own.
#[test]
fn a_conditional_in_a_block_splices_its_arm()
{
    let text = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nအကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ် >= ၃၂ \
                ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။\nလုပ်ဆောင်စရာများ \
                ပြီးဆုံးပါပြီး။\nက ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let loaded = load(text, &on(TargetOs::Linux));
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    let start =
        loaded.program.stmts.iter().find(|s| matches!(s, Stmt::Fn(f) if f.start));
    let Some(Stmt::Fn(start)) = start
    else
    {
        panic!("the start block stands");
    };
    assert!(
        matches!(start.block.stmts.first(), Some(Stmt::Decl(_))),
        "{:?}",
        start.block
    );
}

/// A condition that names no fact, compares a fact with a value it never
/// holds, or computes, is ZK0255.
#[test]
fn a_condition_outside_the_facts_is_zk0255()
{
    let with = |cond: &str| {
        format!(
            "အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် {cond} ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းသေ က သည် ၁ \
             ဖြစ်ပါစေ။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
        )
    };
    let linux = on(TargetOs::Linux);
    for cond in [
        "အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == \"window\"",
        "စနစ် == \"linux\"",
        "၁ + ၁ == ၂",
        "အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == ၂",
    ]
    {
        assert_eq!(codes(&with(cond), &linux), [Code::BadCompileTimeTest], "{cond}");
    }
    assert!(codes(&with("ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား == \"vm\""), &linux).is_empty());
}

/// A trait and its block in an arm at the top level are top-level
/// declarations, not declarations inside a block.
#[test]
fn an_arm_at_the_top_level_holds_top_level_declarations()
{
    let text = "အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ် == ၆၄ ဖြစ်လျှင် အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nပြနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ \
                ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။\nဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ \
                ပြီးဆုံးပါပြီး။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    let loaded = load(text, &on(TargetOs::Linux));
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    assert!(loaded.program.stmts.iter().any(|s| matches!(s, Stmt::Trait(_))));
}

/// A file that names a fact reads the core, whose marked constant holds
/// the value of the target, and imports it with no import written.
#[test]
fn a_named_fact_holds_the_target()
{
    let text =
        "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nအလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let loaded = load(text, &on(TargetOs::Windows));
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    let value = loaded.program.stmts.iter().find_map(|s| match s
    {
        Stmt::Decl(d) if d.builtin && d.name.name.as_str() == "အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ်" =>
        {
            d.init.clone()
        }
        _ => None,
    });
    let Some(Expr::Str(text)) = value
    else
    {
        panic!("the fact holds text: {value:?}");
    };
    assert_eq!(text.plain(), Some("windows"));
    let imported = loaded
        .program
        .stmts
        .iter()
        .any(|s| matches!(s, Stmt::Import(n) if n.unit.is_some()));
    assert!(imported);
    assert_eq!(loaded.program.target, Some(on(TargetOs::Windows)));
}

/// The ABI fact holds the ABI of the target: the arm on `"msvc"` stands for
/// an MSVC target and the other arm for a GNU one, and an OS with no ABI
/// given takes its usual one.
#[test]
fn the_abi_fact_holds_the_abi()
{
    let chain = CHAIN.replace(
        "အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == \"windows\"",
        "အလုပ်လုပ်မည့်အပလီကေးရှင်းဘိုင်နရီကြားခံစနစ် == \"msvc\"",
    );
    let with = |abi| {
        #[rustfmt::skip]
        let target = Target::builder().os(TargetOs::Windows).arch(TargetArch::X86_64)
            .abi(abi).bits(64).output(BuildTarget::Vm).build();
        load(&chain, &target)
    };
    let msvc = with(TargetAbi::Msvc);
    assert!(msvc.diagnostics.is_empty(), "{:?}", msvc.diagnostics);
    assert_eq!(constants(&msvc), ["ပြတင်း"]);
    assert_eq!(constants(&with(TargetAbi::Gnu)), ["ယူနစ်"]);
    assert_eq!(on(TargetOs::Macos).abi, TargetAbi::Apple);
    assert_eq!(on(TargetOs::Linux).abi, TargetAbi::Gnu);
}

/// An import in the arm the target keeps is read; one in a dropped arm is
/// not.
#[test]
fn an_import_in_the_kept_arm_is_read()
{
    let text = "အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် == \"linux\" ဖြစ်လျှင် အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\n\"စံကား/စာသား\" မှ ခွဲ ကို ထည့်သွင်းမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    let units = |os| load(text, &on(os)).program.units.len();
    assert!(units(TargetOs::Linux) > units(TargetOs::Windows));
}

/// The comptime mark sets the flag of a function at the top level, after
/// the export mark too.
#[test]
fn the_comptime_mark_marks_a_function()
{
    let text = "တစ် ဟူသော အများသုံးနိုင်သော၊ ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nအောက်ပါတို့ကို \
                လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\n၁ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ \
                ပြီးဆုံးပါပြီး။\n";
    let loaded = load(text, &on(TargetOs::Linux));
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    let marked = loaded
        .program
        .stmts
        .iter()
        .any(|s| matches!(s, Stmt::Fn(f) if f.comptime && f.exported));
    assert!(marked);
}

/// The comptime mark on the start block, on a C function or inside a
/// function is ZK0256.
#[test]
fn the_comptime_mark_elsewhere_is_zk0256()
{
    let mark = "ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော";
    let start = format!("{mark} ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n");
    let c_fn = format!(
        "abs ဟူသော {mark} လုပ်ဆောင်ချက် သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍\nn ဟူသော ၃၂ ဘစ် \
         ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\n"
    );
    let nested = format!(
        "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nအတွင်း ဟူသော {mark} လုပ်ဆောင်ချက် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ \
         ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\n၁ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nပရိုဂရမ် \
         ပြီးဆုံးပါပြီး။\n"
    );
    for text in [start, c_fn, nested]
    {
        let codes = codes(&text, &on(TargetOs::Linux));
        assert!(codes.contains(&Code::ComptimeMarkMisplaced), "{codes:?}: {text}");
    }
}
