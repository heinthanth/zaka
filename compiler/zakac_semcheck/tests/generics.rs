use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_call_expr;
use zakac_ast::visit::walk_call_stmt;
use zakac_diagnostics::Code;

use crate::common::Checked;
use crate::common::check_as_is;
use crate::common::kind_at;

/// A generic function that gives back its argument.
const SAME: &str = "လုပ်ဆောင်ချက် ထပ်<T> သည်\nက ဟူသော T တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ T \
                    တစ်ခု ကို ပြန်ပေးမည်။\nက ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";

/// A program of the declarations `decls` and a start block of `stmts`.
fn program(decls: &str, stmts: &str) -> String
{
    format!("{decls}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

/// Check a program of `decls` and `stmts`.
fn check(decls: &str, stmts: &str) -> Checked
{
    check_as_is(&program(decls, stmts), &Default::default())
}

/// The diagnostic codes of a program of `decls` and `stmts`.
fn codes(decls: &str, stmts: &str) -> Vec<Code>
{
    check(decls, stmts).out.diagnostics.iter().map(|d| d.code).collect()
}

/// The walker that lists the name each call runs, in each top-level function.
#[derive(Default)]
struct Callees(Vec<(String, String)>, String);

impl Visitor for Callees
{
    fn visit_call_expr(&mut self, n: &zakac_ast::expr::CallExpr)
    {
        self.0.push((self.1.clone(), n.callee.name.to_string()));
        walk_call_expr(self, n);
    }

    fn visit_call_stmt(&mut self, n: &CallStmt)
    {
        self.0.push((self.1.clone(), n.callee.name.to_string()));
        walk_call_stmt(self, n);
    }
}

/// The function and the callee of each call in a checked program.
fn callees(checked: &Checked) -> Vec<(String, String)>
{
    let mut walk = Callees::default();
    for stmt in &checked.program.stmts
    {
        walk.1 = match stmt
        {
            Stmt::Fn(d) => d.name.name.to_string(),
            _ => String::new(),
        };
        walk.visit_stmt(stmt);
    }
    walk.0
}

/// The names of the top-level functions of a checked program.
fn fn_names(checked: &Checked) -> Vec<String>
{
    let fns = checked.program.stmts.iter().filter_map(|s| match s
    {
        Stmt::Fn(d) => Some(d.name.name.to_string()),
        _ => None,
    });
    fns.collect()
}

/// Each call of a generic function asks for an instance for the kinds of
/// its arguments, or for the kinds written; the call runs the instance.
#[test]
fn calls_run_an_instance_for_their_kinds()
{
    let calls = "ထပ်(၁) ကို ပြသမည်။\nထပ်(\"က\") ကို ပြသမည်။\nထပ်::<ဒဿမကိန်း>(၂.၅) ကို \
                 ပြသမည်။\nကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်မည်။";
    let checked = check(SAME, calls);
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    let names = fn_names(&checked);
    for inst in ["ထပ်<isize>", "ထပ်<text>", "ထပ်<f64>"]
    {
        assert!(names.iter().any(|n| n == inst), "{inst} in {names:?}");
    }
    let called: Vec<String> = callees(&checked).into_iter().map(|(_, c)| c).collect();
    assert_eq!(called, ["ထပ်<isize>", "ထပ်<text>", "ထပ်<f64>", "ထပ်<isize>"]);
}

/// An instance checks its calls with its own kinds, so one call in the
/// generic code runs another overload in each instance, and a call of
/// another generic function asks for the instance its kinds give.
#[test]
fn each_instance_chooses_its_own_overloads()
{
    let show = |kind: &str| {
        format!(
            "လုပ်ဆောင်ချက် ပြ သည်\nက ဟူသော {kind} တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nက ကို \
             ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
        )
    };
    let twice = "လုပ်ဆောင်ချက် နှစ်ခါ<T> သည်\nက ဟူသော T တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nက ကို \
                 သုံး၍ ပြ မည်။\nထပ်(က) ကို ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    let decls = format!("{}{}{SAME}{twice}", show("ကိန်းပြည့်"), show("စာသား"));
    let checked = check(&decls, "၁ ကို သုံး၍ နှစ်ခါ မည်။\n\"က\" ကို သုံး၍ နှစ်ခါ မည်။");
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    let calls = callees(&checked);
    let inside = |f: &str| -> Vec<String> {
        calls.iter().filter(|(top, _)| top == f).map(|(_, c)| c.clone()).collect()
    };
    assert_eq!(inside("နှစ်ခါ<isize>"), ["ပြ", "ထပ်<isize>"]);
    assert_eq!(inside("နှစ်ခါ<text>"), ["ပြ#2", "ထပ်<text>"]);
}

/// Another count of kinds than the function declares is ZK0340, and so are
/// kinds written for a function that is not generic.
#[test]
fn another_count_of_kinds_is_zk0340()
{
    let plain =
        "လုပ်ဆောင်ချက် ပြ သည်\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\n၁ ကို ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    let decls = format!("{SAME}{plain}");
    let two = "ထပ်::<ကိန်းပြည့်၊ စာသား>(၁) ကို ပြသမည်။";
    assert_eq!(codes(&decls, two), [Code::KindArgCount]);
    assert_eq!(codes(&decls, "ပြ::<ကိန်းပြည့်>() ကို ပြသမည်။"), [Code::KindArgCount]);
}

/// A kind no argument shows is ZK0341, as are two kinds for one parameter.
#[test]
fn kinds_the_arguments_do_not_show_are_zk0341()
{
    let none = "လုပ်ဆောင်ချက် ဗလာ<T> သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။\nကနဦးတန်ဖိုး ကို \
                ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    assert_eq!(codes(none, "ဗလာ() ကို ပြသမည်။"), [Code::KindsUnknown]);
    assert!(codes(none, "ဗလာ::<ကိန်းပြည့်>() ကို ပြသမည်။").is_empty());
    let pair = "လုပ်ဆောင်ချက် တူ<T> သည်\nက ဟူသော T တစ်ခု ကို လည်းကောင်း ခ ဟူသော T တစ်ခု ကို လည်းကောင်း \
                လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။\nက ကို \
                ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    assert_eq!(codes(pair, "တူ(၁၊ \"က\") ကို ပြသမည်။"), [Code::KindsUnknown]);
}

/// A fault in the generic code is found in the instance with the kinds
/// that do not fit; a fault that each instance shares is reported once.
#[test]
fn a_fault_in_an_instance_is_reported_once()
{
    let add = SAME.replace("က ကို ပြန်ပေးမည်။", "က + ၁ ကို ပြန်ပေးမည်။");
    assert!(codes(&add, "ထပ်(၁) ကို ပြသမည်။").is_empty());
    assert_eq!(codes(&add, "ထပ်(\"က\") ကို ပြသမည်။"), [Code::OperandKind]);
    let shared = SAME.replace("က ကို ပြန်ပေးမည်။", "\"က\" + ၁ ကို ပြသမည်။\nက ကို ပြန်ပေးမည်။");
    let found = codes(&shared, "ထပ်(၁) ကို ပြသမည်။\nထပ်(\"က\") ကို ပြသမည်။");
    assert_eq!(found, [Code::OperandKind]);
}

/// Instances that each ask for a deeper one stop at a limit, ZK0342.
#[test]
fn instances_without_end_are_zk0342()
{
    let deep = "လုပ်ဆောင်ချက် နက်<T> သည်\nက ဟူသော T တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\n[က] ကို \
                သုံး၍ နက် မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    assert_eq!(codes(deep, "၁ ကို သုံး၍ နက် မည်။"), [Code::InstancesEndless]);
}

/// A generic struct with two kind parameters.
const PAIR: &str =
    "အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင် ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု တို့ပါဝင်သည်။\n";

/// The names of the top-level structs of a checked program.
fn struct_names(checked: &Checked) -> Vec<String>
{
    let structs = checked.program.stmts.iter().filter_map(|s| match s
    {
        Stmt::Struct(d) => Some(d.name.name.to_string()),
        _ => None,
    });
    structs.collect()
}

/// A kind that names a generic struct with its kinds, nested ones too, and
/// a struct value with its kinds written, each make an instance.
#[test]
fn kinds_with_their_kinds_make_struct_instances()
{
    let calls = "ကိန်းရှင် က သည် အတွဲ<အတွဲ<စာသား၊ ကိန်းပြည့်>၊ ကိန်းပြည့်> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး \
                 ဖြစ်ပါစေ။\nကိန်းရှင် ခ သည် အတွဲ::<ဒဿမကိန်း၊ စာလုံး>(ပထမ: ၁.၅၊ ဒုတိယ: 'က') \
                 ဖြစ်ပါစေ။\nက ၏ ပထမ ၏ ပထမ ကို ပြသမည်။";
    let checked = check(PAIR, calls);
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    let names = struct_names(&checked);
    for inst in ["အတွဲ<text၊ isize>", "အတွဲ<အတွဲ<text၊ isize>၊ isize>", "အတွဲ<f64၊ char>"]
    {
        assert!(names.iter().any(|n| n == inst), "{inst} in {names:?}");
    }
    assert_eq!(kind_at(&checked, "က ၏ ပထမ ၏ ပထမ"), "text");
}

/// A struct value with no kinds takes the instance its place expects, or
/// else the one its field values give.
#[test]
fn struct_values_work_out_their_instance()
{
    let calls = "ကိန်းရှင် က သည် အတွဲ(ပထမ: \"မမ\"၊ ဒုတိယ: ၂၄) ဖြစ်ပါစေ။\nကိန်းရှင် ခ သည် အတွဲ<ဒဿမကိန်း၊ \
                 ကိန်းပြည့်> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အတွဲ(ပထမ: ၁၊ ဒုတိယ: ၂) ဖြစ်ပါစေ။\nက ၏ ပထမ ကို ပြသမည်။";
    let checked = check(PAIR, calls);
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "က"), "အတွဲ<text၊ isize>");
    assert_eq!(kind_at(&checked, "အတွဲ(ပထမ: ၁၊ ဒုတိယ: ၂)"), "အတွဲ<f64၊ isize>");
    let missing = codes(PAIR, "ကိန်းရှင် က သည် အတွဲ(ပထမ: ၁) ဖြစ်ပါစေ။");
    assert_eq!(missing, [Code::KindsUnknown]);
}

/// A generic function works out its kinds through a generic struct
/// parameter, and its result names the instance its kinds give.
#[test]
fn a_generic_function_takes_a_generic_struct()
{
    let swap = "လုပ်ဆောင်ချက် လှည့်<A၊ B> သည်\nတွဲ ဟူသော အတွဲ<A၊ B> တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို \
                လုပ်ဆောင်ကာ အတွဲ<B၊ A> တစ်ခု ကို ပြန်ပေးမည်။\nအတွဲ(ပထမ: (တွဲ ၏ ဒုတိယ)၊ ဒုတိယ: (တွဲ ၏ ပထမ)) \
                ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    let calls = "ကိန်းရှင် က သည် လှည့်(အတွဲ(ပထမ: \"မမ\"၊ ဒုတိယ: ၂၄)) ဖြစ်ပါစေ။\nက ၏ ပထမ ကို ပြသမည်။";
    let checked = check(&format!("{PAIR}{swap}"), calls);
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert!(fn_names(&checked).iter().any(|n| n == "လှည့်<text၊ isize>"));
    assert_eq!(kind_at(&checked, "က"), "အတွဲ<isize၊ text>");
}

/// Another count of kinds for a generic struct is ZK0340, and a struct
/// whose instances each hold a deeper one stops at the limit, ZK0342.
#[test]
fn struct_kinds_with_faults_have_their_codes()
{
    let one = "ကိန်းရှင် က သည် အတွဲ<ကိန်းပြည့်> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။";
    assert_eq!(codes(PAIR, one), [Code::KindArgCount]);
    let deep = "နက်<T> ဟူသော ဒေတာအစုအဝေးတွင် နောက် ဟူသော နက်<T စာရင်း> လိပ်စာ တစ်ခု ပါဝင်သည်။\n";
    let calls = "ကိန်းရှင် က သည် နက်<ကိန်းပြည့်> လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။";
    assert_eq!(codes(deep, calls), [Code::InstancesEndless]);
}

/// A trait with one function, `ငယ်သလား`, on two values of its kind.
const ORDER: &str = "စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် \
                     အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။\nလုပ်ဆောင်ချက် ငယ်သလား သည်\nက ဟူသော A တစ်ခု \
                     ကို လည်းကောင်း ခ ဟူသော A တစ်ခု ကို လည်းကောင်း\nလက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို \
                     ပြန်ပေးမည်။\nဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။\n";

/// A generic function whose kind needs [`ORDER`].
const SMALLER: &str = "လုပ်ဆောင်ချက် ငယ်ရာ<T: စီနိုင်သော> သည်\nက ဟူသော T တစ်ခု ကို လည်းကောင်း ခ ဟူသော T \
                       တစ်ခု ကို လည်းကောင်း\nလက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို \
                       ပြန်ပေးမည်။\nအကယ်၍ ငယ်သလား(က၊ ခ) ဖြစ်လျှင် က ကို ပြန်ပေးမည်။\nခ ကို \
                       ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";

/// `ငယ်သလား` for a kind, as a plain function.
fn less_for(kind: &str) -> String
{
    format!(
        "လုပ်ဆောင်ချက် ငယ်သလား သည်\nက ဟူသော {kind} တစ်ခု ကို လည်းကောင်း ခ ဟူသော {kind} တစ်ခု ကို \
         လည်းကောင်း\nလက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။\nက < ခ ကို \
         ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
    )
}

/// A kind has a trait through a plain function of the trait's name and
/// kinds, with no block; a kind without one does not meet the constraint,
/// ZK0343, and the compiler makes no instance for it.
#[test]
fn a_kind_meets_a_constraint_through_its_functions()
{
    let decls = format!("{ORDER}{SMALLER}{}", less_for("ကိန်းပြည့်"));
    let checked = check(&decls, "ငယ်ရာ(၃၊ ၅) ကို ပြသမည်။");
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    let missing = check(&decls, "ငယ်ရာ(၁.၅၊ ၂.၅) ကို ပြသမည်။");
    let found: Vec<Code> = missing.out.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(found, [Code::BoundNotMet]);
    assert!(!fn_names(&missing).iter().any(|n| n.starts_with("ငယ်ရာ<")));
}

/// The built-in traits follow the operators a kind takes, and a name that
/// is no trait is ZK0344.
#[test]
fn built_in_traits_follow_the_operators()
{
    let sum = SMALLER.replace("စီနိုင်သော", "နံပါတ်နှင့်တူသော");
    let decls = format!("{sum}{}", less_for("ကိန်းပြည့်"));
    assert!(codes(&decls, "ငယ်ရာ(၃၊ ၅) ကို ပြသမည်။").is_empty());
    assert_eq!(codes(&decls, "ငယ်ရာ(\"က\"၊ \"ခ\") ကို ပြသမည်။"), [Code::BoundNotMet]);
    let none = SMALLER.replace("စီနိုင်သော", "မသိသော");
    let decls = format!("{none}{}", less_for("ကိန်းပြည့်"));
    assert!(codes(&decls, "ငယ်ရာ(၃၊ ၅) ကို ပြသမည်။").contains(&Code::UnknownTrait));
}

/// A trait block holds each function of its trait with the kinds the trait
/// gives it: one left out is ZK0345, one that is not the trait's ZK0346.
#[test]
fn a_trait_block_holds_the_functions_of_its_trait()
{
    let block = |fns: &str| {
        format!(
            "{ORDER}ကိန်းပြည့် တွင် စီနိုင်သော ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို \
             ထည့်သွင်းမည်။\n{fns}ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။\n"
        )
    };
    assert!(codes(&block(&less_for("ကိန်းပြည့်")), "").is_empty());
    let wrong = block(&less_for("ဒဿမကိန်း"));
    assert_eq!(codes(&wrong, ""), [Code::TraitFnMissing, Code::TraitFnForeign]);
}

/// A trait can need another: a kind meets it when it has both.
#[test]
fn a_trait_needs_the_traits_of_its_notes()
{
    let show = "ပြနိုင်သော<A> ဟူသည့် ဂုဏ်သတ္တိ တွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။\nဤနေရာတွင် \
                သတိပြုရန်မှာ\nA တွင် စီနိုင်သော ဂုဏ်သတ္တိ ရှိသည်။\nသတိပြုစရာများ ပြီးဆုံးပါပြီး။\nဂုဏ်သတ္တိတွင် \
                ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။\n";
    let shown = SMALLER.replace("<T: စီနိုင်သော>", "<T: ပြနိုင်သော>");
    let decls = format!("{ORDER}{show}{shown}{}", less_for("ကိန်းပြည့်"));
    assert!(codes(&decls, "ငယ်ရာ(၃၊ ၅) ကို ပြသမည်။").is_empty());
    assert_eq!(codes(&decls, "ငယ်ရာ('က'၊ 'ခ') ကို ပြသမည်။"), [Code::BoundNotMet]);
}

/// A fault inside an instance points at the code of its generic, with a
/// label at the use that asked for the instance.
#[test]
fn a_fault_in_an_instance_is_labeled_at_its_use()
{
    let add = SAME.replace("က ကို ပြန်ပေးမည်။", "က + ၁ ကို ပြန်ပေးမည်။");
    let checked = check(&add, "ထပ်(\"က\") ကို ပြသမည်။");
    let [fault] = checked.out.diagnostics.as_slice()
    else
    {
        panic!("{:?}", checked.out.diagnostics);
    };
    assert_eq!(fault.labels.len(), 1, "{fault:?}");
    assert!(fault.labels[0].text.contains("ထပ်<text>"), "{fault:?}");
}

/// A function value can stand for a kind parameter: its function kind is the
/// kind.
#[test]
fn a_function_kind_is_a_kind_argument()
{
    let twice = "လုပ်ဆောင်ချက် ကြိမ်<F> သည်\nလုပ် ဟူသော F တစ်ခု ကို လည်းကောင်း က ဟူသော ကိန်းပြည့် တစ်ခု ကို \
                 လည်းကောင်း\nလက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\nလုပ်(လုပ်(က)) \
                 ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    let double = "လုပ်ဆောင်ချက် နှစ်ဆ သည်\nက ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ \
                  ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\nက * ၂ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ \
                  ပြီးဆုံးပါပြီး။\n";
    let found =
        codes(&format!("{twice}{double}"), "ကြိမ်(နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်၊ ၃) ကို ပြသမည်။");
    assert!(found.is_empty(), "{found:?}");
}

/// The compiler mark belongs to the prelude: in another file it is ZK0349.
#[test]
fn the_compiler_mark_outside_the_prelude_is_zk0349()
{
    let marked = "လုပ်ဆောင်ချက် ပေါင်း<T> သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော လုပ်ဆောင်ချက် ဖြစ်၍\nက \
                  ဟူသော T တစ်ခု ကို လည်းကောင်း ခ ဟူသော T တစ်ခု ကို လည်းကောင်း\nလက်ခံပြီး T တစ်ခု ကို \
                  ပြန်ပေးမည်။\n";
    assert_eq!(codes(marked, "၁ ကို ပြသမည်။"), [Code::BuiltinOutsidePrelude]);
}

/// An operator on two values of a struct calls the function of the struct
/// it stands for; `!=` and `>=` negate it, and a struct with no `ညီမညီစစ်`
/// compares one field at a time.
#[test]
fn operators_on_a_struct_call_its_functions()
{
    let point = "ဗက်တာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။\n";
    let add = "လုပ်ဆောင်ချက် ပေါင်း သည်\nက ဟူသော ဗက်တာ တစ်ခု ကို လည်းကောင်း ခ ဟူသော ဗက်တာ တစ်ခု ကို \
               လည်းကောင်း\nလက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဗက်တာ တစ်ခု ကို ပြန်ပေးမည်။\nက ကို \
               ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    let calls =
        "ကိန်းရှင် က သည် ဗက်တာ(x: ၁) ဖြစ်ပါစေ။\n(က + က) ၏ x ကို ပြသမည်။\n(က == က) ကို ပြသမည်။";
    let checked = check(&format!("{point}{add}"), calls);
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    let called: Vec<String> = callees(&checked).into_iter().map(|(_, c)| c).collect();
    assert!(called.contains(&"ပေါင်း".to_string()), "{called:?}");
    let none = check(point, "ကိန်းရှင် က သည် ဗက်တာ(x: ၁) ဖြစ်ပါစေ။\n(က + က) ကို ပြသမည်။");
    assert!(!none.out.diagnostics.is_empty());
}
