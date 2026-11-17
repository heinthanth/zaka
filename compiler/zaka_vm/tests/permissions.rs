//! The rights of a run: a denied file or variable read fails with no
//! permission, a denied C call or environment stops the run, and an asker
//! can grant one effect or each effect of a right.

use zaka_host::Answer;
use zaka_host::Gate;
use zaka_host::Grant;
use zaka_host::Grants;
use zaka_host::MemWorld;
use zaka_host::ScriptAsker;
use zaka_vm::machine::HostData;
use zaka_vm::machine::Machine;
use zaka_vm::machine::RunOptions;

use crate::common::compile;

/// A program that reads a variable and a file, then calls C.
const PROGRAM: &str = "\"စံကား/အဝင်အထွက်\" မှ ဖတ် နှင့် ဖိုင်ကိုဖတ် တို့ကို ထည့်သွင်းမည်။\nစိတ်မချရသော လုပ်ဆောင်ချက် \
                       abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍\nn ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု \
                       ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\nပရိုဂရမ်သည် အောက်ပါတို့ကို \
                       လုပ်ဆောင်မည်။\n\"V\" ဟူသည့် ပတ်ဝန်းကျင်ကိန်းရှင် ကို ဖတ်သည့်ရလဒ် ကို \
                       ပြသမည်။\n\"x.txt\" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ် ကို ပြသမည်။\nအောက်ပါတို့ကို \
                       စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။\nabs ဟူသော လုပ်ဆောင်ချက်ကို -၄၂ ဖြင့် လုပ်ဆောင်သော \
                       ရလဒ် ကို ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";

/// The text a denied read prints.
const DENIED: &str = "အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ ခွင့်ပြုချက်မရှိခြင်း)\n";

/// Run the program in a world with the variable `V` and the file `x.txt`,
/// with a gate: what it printed, and the trap it stopped with.
fn run(gate: Gate) -> (String, Option<String>)
{
    let mut world = MemWorld::default();
    world.env.insert("V".into(), "v".into());
    world.files.insert("x.txt".into(), "x".into());
    let mut machine = Machine::new(compile(PROGRAM), RunOptions::default());
    machine.set_world(Box::new(world));
    machine.set_gate(gate);
    machine.set_host(&HostData::default());
    while machine.running()
    {
        machine.step();
    }
    let trap = machine.state().trap().map(|t| t.kind.to_string());
    (machine.output().to_string(), trap)
}

/// With no right, the reads fail with no permission and the C call stops.
#[test]
fn a_run_with_no_right_is_denied()
{
    let (out, trap) = run(Gate::new(Grants::default()));
    assert_eq!(out, format!("{DENIED}{DENIED}"));
    assert_eq!(trap.as_deref(), Some("calling the C function abs needs --allow-ffi"));
}

/// A list of variables grants those alone.
#[test]
fn a_list_grants_the_variables_it_names()
{
    let grants = Grants::builder().env(Grant::Some(vec!["V".into()])).build();
    let (out, _) = run(Gate::new(grants));
    assert!(out.starts_with("ရလဒ်ကောင်း(\"v\")\n"), "{out}");
}

/// The asker grants one effect, each effect of a right, or none.
#[test]
fn the_asker_grants_what_it_answers()
{
    let answers = vec![Answer::Once, Answer::No, Answer::All];
    let asker = Box::new(ScriptAsker { answers, asked: Vec::new() });
    let gate = Gate::new(Grants::builder().prompt(true).build()).with_asker(asker);
    let (out, trap) = run(gate);
    assert!(out.starts_with(&format!("ရလဒ်ကောင်း(\"v\")\n{DENIED}")), "{out}");
    // The C call holds the right; a build with no C runs none at all.
    let trap = trap.unwrap_or_default();
    assert!(!trap.contains("--allow-ffi"), "{trap}");
}
