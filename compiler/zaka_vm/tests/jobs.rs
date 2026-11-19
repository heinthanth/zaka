//! Jobs on the VM: a call that runs in parallel starts a job, which takes
//! turns with the others over the one heap; a wait gives its result. A trap
//! ends its job alone under `--trap unwind`, and the whole program under
//! `--trap abort`. Channels take values from one job to another.

use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::TrapMode;
use zaka_vm::machine::run_and_keep;
use zaka_vm::trap::TrapKind;
use zakac_codegen_target_vm::artifact::run_phases;
use zakac_codegen_target_vm::options::ResultMode;
use zakac_diagnostics::Code;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

use crate::common::compile;
use crate::common::output;
use crate::common::run_source_with;

/// A function that sums a list, one that joins a mark to text, one that
/// divides 10, and one that prints its text.
const FNS: &str = "စုပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် ဂဏန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
ဂဏန်းများ ၏ အကွက်တိုင်း ကို ဂ အနေဖြင့် ယူ၍ စု ကို စု + ဂ ဟု ထားမည်။
စု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အော် ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
\"{စာ}!\" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စား ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁၀ / က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// The functions, then a start block of these lines.
fn program(lines: &str) -> String
{
    format!("{FNS}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{lines}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

#[test]
fn two_jobs_sum_two_halves()
{
    let text = program(
        "ကိန်းရှင် က သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၁၊ ၂၊ ၃] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၄၊ ၅၊ ၆] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
(က ကို စောင့်သော ရလဒ်) + (ခ ကို စောင့်သော ရလဒ်) ကို ပြသမည်။",
    );
    assert_eq!(output(&text), "၂၁\n");
}

#[test]
fn a_text_result_comes_to_each_wait_and_leaves_no_block_held()
{
    let text = program(
        "ကိန်းရှင် က သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို \"မင်္ဂလာ\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။
က ကို စောင့်မည်။",
    );
    let mut out = Vec::new();
    let options = RunOptions::default();
    let (machine, result) =
        run_and_keep(&compile(&text), &mut out, &options, &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(String::from_utf8(out).unwrap(), "မင်္ဂလာ!\nမင်္ဂလာ!\n");
    assert_eq!(machine.leaks(), Vec::new());
}

#[test]
fn the_program_waits_at_its_end_for_each_job()
{
    let text = program(
        "ပြ ဟူသော လုပ်ဆောင်ချက်ကို \"နောက်ကွယ်\" ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
\"ရှေ့\" ဟု ပြသမည်။",
    );
    assert_eq!(output(&text), "ရှေ့\nနောက်ကွယ်\n");
}

/// A job that divides 10 with 0, then one that divides 10 with 2 and a print.
const TRAPS: &str = "ကိန်းရှင် က သည် စား ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း \
                     ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စား ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ခ ကို စောင့်သော ရလဒ် ကို ပြသမည်။";

#[test]
fn a_trap_ends_its_job_alone_and_the_program_ends_with_code_one()
{
    let mut out = Vec::new();
    let options = RunOptions::default();
    let module = compile(&program(TRAPS));
    let (machine, result) =
        run_and_keep(&module, &mut out, &options, &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(String::from_utf8(out).unwrap(), "၅\n");
    let traps = machine.job_traps();
    assert_eq!(traps.len(), 1);
    assert_eq!((traps[0].0, &traps[0].1.kind), (1, &TrapKind::ZeroDivisor));
    assert_eq!(machine.exit_code(), Some(1));
}

#[test]
fn a_wait_for_a_job_that_trapped_traps()
{
    let text = program(&format!("{TRAPS}\nက ကို စောင့်မည်။"));
    let (out, result) = run_source_with(&text, &RunOptions::default());
    assert_eq!(out, "၅\n");
    assert_eq!(result.map_err(|t| t.kind), Err(TrapKind::JobTrapped { job: 1 }));
}

#[test]
fn under_abort_a_trap_in_a_job_stops_the_program()
{
    let options = RunOptions::builder().trap(TrapMode::Abort).build();
    let (out, result) = run_source_with(&program(TRAPS), &options);
    assert_eq!(out, "");
    assert_eq!(result.map_err(|t| t.kind), Err(TrapKind::ZeroDivisor));
}

/// A function that sends the numbers 1 to 5 to a channel, printing each
/// send, then closes it.
const SENDS: &str = "ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို \
                     လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၅ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လမ်း ထဲသို့ န ကို ပို့မည်။
\"ပို့ {န}\" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လမ်း ကို ပိတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// [`SENDS`], then a start block of these lines.
fn with_sends(lines: &str) -> String
{
    format!("{SENDS}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{lines}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

#[test]
fn a_loop_takes_each_value_of_a_channel_until_it_is_closed()
{
    let text = with_sends(
        "ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို လမ်း ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
လမ်း မှ ရရှိသော ဒေတာတိုင်းကို တန် အနေဖြင့် ယူ၍ \"ရ {တန်}\" ဟု ပြသမည်။",
    );
    let lines: Vec<String> = (1..=5).map(|n| format!("ပို့ {}", burmese(n))).collect();
    let got: Vec<String> = (1..=5).map(|n| format!("ရ {}", burmese(n))).collect();
    let out = output(&text);
    assert_eq!(out.lines().filter(|l| l.starts_with("ပို့")).collect::<Vec<_>>(), lines);
    assert_eq!(out.lines().filter(|l| l.starts_with("ရ")).collect::<Vec<_>>(), got);
}

/// The digits of a number in Burmese.
fn burmese(n: u32) -> String
{
    n.to_string()
        .chars()
        .map(|c| char::from_u32(0x1040 + c.to_digit(10).unwrap()).unwrap())
        .collect()
}

#[test]
fn a_send_waits_while_a_channel_with_a_room_is_full()
{
    let text = with_sends(
        "ကိန်းရှင် ကန့် သည် ကိန်းပြည့် ၂ ခု ပို့ဆောင်နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး \
         ဖြစ်ပါစေ။
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို ကန့် ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကန့် မှ ရရှိသော ဒေတာ ကို ပြသမည်။
အကယ်၍ ကန့် ထဲတွင် ဒေတာ ရှိလျှင် \"ရ {ကန့် ထဲမှ ဒေတာ}\" ဟု ပြသမည်။
ကန့် မှ လက်ခံရရှိသော ဒေတာတိုင်း ကို တန် အနေဖြင့် ယူ၍ \"ကျန် {တန်}\" ဟု ပြသမည်။
ကန့် မှ ရရှိသော ဒေတာ ကို ပြသမည်။",
    );
    let want = "ပို့ ၁\nပို့ ၂\nဒေတာ(၁)\nရ ၂\nပို့ ၃\nပို့ ၄\nကျန် ၃\nကျန် ၄\nပို့ ၅\nကျန် ၅\nအလွတ်\n";
    assert_eq!(output(&text), want);
}

#[test]
fn text_sent_through_a_channel_leaves_no_block_held()
{
    let text = program(
        "ကိန်းရှင် လမ်း သည် စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
လမ်း ထဲသို့ \"က\" ကို ပို့မည်။
လမ်း ထဲသို့ \"ခ\" ကို ပို့မည်။
လမ်း ထဲသို့ \"ဂ\" ကို ပို့မည်။
လမ်း ကို ပိတ်မည်။
အကယ်၍ လမ်း ထဲတွင် ဒေတာ ရှိလျှင် လမ်း ထဲမှ ဒေတာ ကို ပြသမည်။",
    );
    let mut out = Vec::new();
    let options = RunOptions::default();
    let (machine, result) =
        run_and_keep(&compile(&text), &mut out, &options, &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(String::from_utf8(out).unwrap(), "က\n");
    assert_eq!(machine.leaks(), Vec::new());
}

/// A start block that declares the channel `လမ်း` of numbers, then these
/// lines.
fn with_pipe(lines: &str) -> String
{
    program(&format!(
        "ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး \
         ဖြစ်ပါစေ။\n{lines}"
    ))
}

#[test]
fn a_receive_no_job_can_serve_stops_the_program()
{
    let text = with_pipe("လမ်း မှ ရရှိသော ဒေတာ ကို ပြသမည်။");
    let (_, result) = run_source_with(&text, &RunOptions::default());
    assert_eq!(result.map_err(|t| t.kind), Err(TrapKind::Deadlock));
}

#[test]
fn a_send_to_a_closed_channel_stops_the_program()
{
    let text = with_pipe("လမ်း ကို ပိတ်မည်။\nလမ်း ထဲသို့ ၁ ကို ပို့မည်။");
    let (_, result) = run_source_with(&text, &RunOptions::default());
    assert_eq!(result.map_err(|t| t.kind), Err(TrapKind::PipeClosed { pipe: 1 }));
}

#[test]
fn a_wait_passes_on_a_failure_of_the_job()
{
    let text = "စမ်း ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ယူ ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် က သည် စမ်း ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် က ကို စောင့်သော ရလဒ်ကောင်း ဖြစ်ပါစေ။
ခ + ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ယူ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";
    assert_eq!(output(text), "၈\n");
}

/// The codes of the diagnostics of a program.
fn codes(text: &str) -> Vec<Code>
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let (lexer, options) = (LexerOptions::default(), SemcheckOptions::default());
    let mem = Default::default();
    let emitted = run_phases(
        &file,
        &lexer,
        &options,
        &FoldOptions::default(),
        &mem,
        ResultMode::default(),
    );
    emitted.diagnostics().iter().map(|d| d.code).collect()
}

#[test]
fn the_data_of_a_channel_is_read_inside_its_test_alone()
{
    let outside = with_pipe("လမ်း ထဲမှ ဒေတာ ကို ပြသမည်။");
    assert_eq!(codes(&outside), [Code::PipeDataOutside]);
    let kept = with_pipe("ကိန်းရှင် ခ သည် လမ်း ထဲတွင် ဒေတာ ရှိ ဖြစ်ပါစေ။");
    assert_eq!(codes(&kept), [Code::PipeDataOutside]);
}

#[test]
fn a_receive_from_a_value_that_is_no_channel_is_reported()
{
    let text = program("ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။\nက မှ ရရှိသော ဒေတာ ကို ပြသမည်။");
    assert_eq!(codes(&text), [Code::NotAPipe]);
}

/// A global, a constant, a function that writes the global, one that reads
/// the constant, one that calls the writer, and one that changes its
/// parameter.
const REACH: &str = "ကိန်းရှင် ရေ သည် ၀ ဖြစ်ပါစေ။
ကိန်းသေ အများ သည် ၁၀ ဖြစ်ပါစေ။

တိုး ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရေ ကို ရေ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဖတ် ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အများ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ခေါ် ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြင် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// [`REACH`], then a start block of these lines.
fn reaching(lines: &str) -> String
{
    format!("{REACH}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{lines}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

#[test]
fn a_job_reads_a_constant_and_no_global()
{
    let constant = reaching("ဖတ် ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။");
    assert_eq!(codes(&constant), []);
    let global = reaching("ခေါ် ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။");
    assert_eq!(codes(&global), [Code::GlobalInJob]);
}

#[test]
fn a_job_starts_in_a_declared_function_that_changes_no_parameter()
{
    let value = reaching(
        "ကိန်းရှင် ဖ သည် ဖတ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။\nဖ ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။",
    );
    assert_eq!(codes(&value), [Code::JobOfNoDeclaredFn]);
    let changes =
        reaching("ကိန်းရှင် ဂ သည် ၁ ဖြစ်ပါစေ။\nပြင် ဟူသော လုပ်ဆောင်ချက်ကို ဂ ဖြင့် အပြိုင် လုပ်ဆောင်မည်။");
    assert_eq!(codes(&changes), [Code::JobChangesArgs]);
}
