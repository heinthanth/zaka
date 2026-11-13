use std::collections::HashSet;

use zaka_host::HOST_FNS;
use zaka_host::HostFn;
use zaka_host::MemWorld;
use zaka_vm::machine::HostData;
use zaka_vm::machine::Machine;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::State;
use zaka_vm::machine::run_in;
use zakac_codegen_target_vm::bc::Op;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::tern;

use crate::common::compile_checked;

/// The import of each name of the stdlib module of input and output.
const IMPORT: &str = "\"စံကား/အဝင်အထွက်\" မှ အချက်အလက်အဝင်အထွက်အမှားအယွင်း၊ စံပြုအဝင်၊ စံပြုအထွက်၊ \
                      စံပြုအမှား၊ စာကြောင်းတစ်ကြောင်းဖတ်၊ ဖိုင်ကိုဖတ်၊ ရေး တို့ကို ထည့်သွင်းမည်။";

/// A function that can fail, which does each host call once, and a start
/// block that prints its whole result.
const WORK: &str = "\
လုပ်ဆောင်ချက် အလုပ် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် \
                    အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
\"နာမည်: \" ကို စံပြုအထွက် သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
ကိန်းရှင် နာမည် သည် စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ်ကောင်း ဖြစ်ပါစေ။
\"မင်္ဂလာပါ {နာမည်}\" ဟု ပြသမည်။
နာမည် ကို \"out.txt\" သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
\"out.txt\" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ်ကောင်း ကို ပြသမည်။
\"missing.txt\" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ် ကို ပြသမည်။
\"oops\" ကို စံပြုအမှား သို့ ရေး မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
ကိန်းရှင် နောက် သည် စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ဖြစ်ပါစေ။
နောက် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် အလုပ် သည့် ရလဒ် ဖြစ်ပါစေ။
ရ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// The host functions read the input of a world in storage, keep the files
/// it writes there, and give each failure as a member of the error
/// enumeration; the writes to standard output keep their order with what the
/// program prints, and the writes to standard error go apart.
#[test]
fn host_functions_serve_a_program_from_a_world_in_storage()
{
    let module = compile_checked(&format!("{IMPORT}\n{WORK}"));
    let world = Box::new(MemWorld::builder().input("Hein\n").build());
    let mut out = Vec::new();
    let (_, result) =
        run_in(&module, &mut out, &RunOptions::default(), &HostData::default(), world);
    result.expect("the run ends");
    let want = "\
နာမည်: မင်္ဂလာပါ Hein
Hein
အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ မတွေ့ခြင်း)
အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း)
အောင်မြင်သည့်သတင်း
";
    assert_eq!(String::from_utf8(out).unwrap(), want);
}

/// The stdlib declares a function for each row of the host table, so the
/// table and the stdlib remain in step: each host function is the op of one of
/// its external functions.
#[test]
fn the_stdlib_declares_each_host_function()
{
    let module = compile_checked(&format!("{IMPORT}\n{WORK}"));
    let ops = module.funcs.iter().flat_map(|f| f.code.iter());
    let served: HashSet<HostFn> =
        ops.filter_map(|op| tern!(let Op::Host(f) = op, Some(*f), None)).collect();
    let rows: HashSet<HostFn> = HOST_FNS.iter().map(|d| d.func).collect();
    assert_eq!(served, rows);
}

/// A program that reads two lines of standard input and prints each whole
/// result.
const TWO_LINES: &str = "\
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ကို ပြသမည်။
စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// In a world whose input remains open, as in the explorer, a read with no
/// line waits; a line given goes on with the run, and closing the input
/// makes the read fail with the end of the input.
#[test]
fn an_open_world_waits_for_each_line()
{
    let module = compile_checked(&format!("{IMPORT}\n{TWO_LINES}"));
    let mut machine = Machine::new(module, RunOptions::default());
    machine.set_world(Box::new(MemWorld::builder().open(true).build()));
    let run = |m: &mut Machine| do_while!(m.running(), _ = m.step());
    run(&mut machine);
    assert_eq!(machine.state(), &State::Waiting);
    assert_eq!(machine.output(), "");
    machine.give_input("Hein\n");
    run(&mut machine);
    assert!(machine.waiting());
    assert_eq!(machine.output(), "ရလဒ်ကောင်း(\"Hein\")\n");
    machine.close_input();
    run(&mut machine);
    assert_eq!(machine.state(), &State::Finished);
    let end = "အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ အဝင်အချက်အလက်အဆုံးကိုရောက်ခြင်း)";
    assert_eq!(machine.output(), format!("ရလဒ်ကောင်း(\"Hein\")\n{end}\n"));
}
