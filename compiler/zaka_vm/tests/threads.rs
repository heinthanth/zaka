//! Threads on the VM: a call that runs in parallel starts a thread, which takes
//! turns with the others over the one heap; a wait gives its result. A trap
//! ends its thread alone under `--trap unwind`, and the whole program under
//! `--trap abort`. Channels take values from one thread to another.

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
use crate::common::output_on;
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
fn two_threads_sum_two_halves()
{
    let text = program(
        "ကိန်းရှင် က သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၁၊ ၂၊ ၃] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၄၊ ၅၊ ၆] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
(က ကို စောင့်သော ရလဒ်) + (ခ ကို စောင့်သော ရလဒ်) ကို ပြသမည်။",
    );
    assert_eq!(output(&text), "၂၁\n");
}

#[test]
fn one_wait_takes_a_text_result_and_leaves_no_block_held()
{
    let text = program(
        "ကိန်းရှင် က သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို \"မင်္ဂလာ\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ရ သည် က ကို စောင့်သော ရလဒ် ဖြစ်ပါစေ။
ရ ကို ပြသမည်။
ရ ကို ပြသမည်။
ကိန်းရှင် ခ သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို \"ခ\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ခ ကို စောင့်မည်။",
    );
    let mut out = Vec::new();
    let options = RunOptions::default();
    let (machine, result) =
        run_and_keep(&compile(&text), &mut out, &options, &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(String::from_utf8(out).unwrap(), "မင်္ဂလာ!\nမင်္ဂလာ!\n");
    // The wait that reads no value dropped the result of its thread.
    assert_eq!(machine.leaks(), Vec::new());
}

#[test]
fn a_second_wait_for_a_thread_stops_the_program()
{
    let text = program(
        "ကိန်းရှင် က သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို \"က\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။",
    );
    let (out, result) = run_source_with(&text, &RunOptions::default());
    assert_eq!(out, "က!\n");
    assert_eq!(result.map_err(|t| t.kind), Err(TrapKind::ThreadWaited { thread: 1 }));
}

#[test]
fn the_program_waits_at_its_end_for_each_thread()
{
    let text = program(
        "ပြ ဟူသော လုပ်ဆောင်ချက်ကို \"နောက်ကွယ်\" ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
\"ရှေ့\" ဟု ပြသမည်။",
    );
    assert_eq!(output(&text), "ရှေ့\nနောက်ကွယ်\n");
}

/// A thread that divides 10 with 0, then one that divides 10 with 2 and a
/// print.
const TRAPS: &str = "ကိန်းရှင် က သည် စား ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း \
                     ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စား ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ခ ကို စောင့်သော ရလဒ် ကို ပြသမည်။";

#[test]
fn a_trap_ends_its_thread_alone_and_the_program_ends_with_code_one()
{
    let mut out = Vec::new();
    let options = RunOptions::default();
    let module = compile(&program(TRAPS));
    let (machine, result) =
        run_and_keep(&module, &mut out, &options, &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(String::from_utf8(out).unwrap(), "၅\n");
    let traps = machine.thread_traps();
    assert_eq!(traps.len(), 1);
    assert_eq!((traps[0].0, &traps[0].1.kind), (1, &TrapKind::ZeroDivisor));
    assert_eq!(machine.exit_code(), Some(1));
}

#[test]
fn a_wait_for_a_thread_that_trapped_traps()
{
    let text = program(&format!("{TRAPS}\nက ကို စောင့်မည်။"));
    let (out, result) = run_source_with(&text, &RunOptions::default());
    assert_eq!(out, "၅\n");
    assert_eq!(result.map_err(|t| t.kind), Err(TrapKind::ThreadTrapped { thread: 1 }));
}

#[test]
fn under_abort_a_trap_in_a_thread_stops_the_program()
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
fn a_receive_no_thread_can_serve_stops_the_program()
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
fn a_wait_passes_on_a_failure_of_the_thread()
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

/// The messages of the diagnostics of a program.
fn messages(text: &str) -> Vec<String>
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let (lexer, options) = (LexerOptions::default(), SemcheckOptions::default());
    #[rustfmt::skip]
    let emitted = run_phases(&file, &lexer, &options, &FoldOptions::default(),
        &Default::default(), ResultMode::default());
    emitted.diagnostics().iter().map(|d| d.message.clone()).collect()
}

#[test]
fn a_round_that_reaches_a_global_names_the_loop()
{
    let text = "ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ထွက် ၏ အကွက် i ကို i * ၁၀ ဟု ထားမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";
    let found = messages(text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].starts_with("a round of this parallel loop"), "{found:?}");
    assert!(!found[0].contains('#'), "{found:?}");
    let thread = messages(&reaching("ခေါ် ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။"));
    // The thread reaches the global through တိုး, which it calls.
    assert!(thread[0].starts_with("\"တိုး\" runs in a thread"), "{thread:?}");
}

/// [`REACH`], then a start block of these lines.
fn reaching(lines: &str) -> String
{
    format!("{REACH}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{lines}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

#[test]
fn a_thread_reads_a_constant_and_no_global()
{
    let constant = reaching("ဖတ် ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။");
    assert_eq!(codes(&constant), []);
    let global = reaching("ခေါ် ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။");
    assert_eq!(codes(&global), [Code::GlobalInThread]);
}

#[test]
fn a_thread_starts_in_a_declared_function_that_changes_no_parameter()
{
    let value = reaching(
        "ကိန်းရှင် ဖ သည် ဖတ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။\nဖ ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။",
    );
    assert_eq!(codes(&value), [Code::ThreadOfNoDeclaredFn]);
    let changes =
        reaching("ကိန်းရှင် ဂ သည် ၁ ဖြစ်ပါစေ။\nပြင် ဟူသော လုပ်ဆောင်ချက်ကို ဂ ဖြင့် အပြိုင် လုပ်ဆောင်မည်။");
    assert_eq!(codes(&changes), [Code::ThreadChangesArgs]);
}

#[test]
fn a_lock_block_keeps_the_changes_of_two_threads_apart()
{
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../zakac_codegen_target_c/tests/inputs/locks.zaka"
    ))
    .unwrap();
    assert_eq!(output(&text), "၂၀၀၀\n၂၀၀၀\n");
}

/// A struct threads share, and a function that divides its field inside a lock
/// block.
const SPLIT: &str = "အကောင့် ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် \
                     လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ခွဲ ဟူသော လုပ်ဆောင်ချက် သည် ဘဏ် ဟူသော အကောင့် တစ်ခု ကို လည်းကောင်း က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဘဏ် ကို ခေတ္တယူ၍ ဘဏ် ၏ လက်ကျန် ကို (ဘဏ် ၏ လက်ကျန်) / က ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

#[test]
fn a_lock_whose_holder_trapped_stops_the_next_thread_that_takes_it()
{
    let text = format!(
        "{SPLIT}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘဏ် သည် အကောင့်(လက်ကျန်: ၁၀) ဖြစ်ပါစေ။
ခွဲ ဟူသော လုပ်ဆောင်ချက်ကို ဘဏ် နှင့် ၀ တို့ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ခွဲ ဟူသော လုပ်ဆောင်ချက်ကို ဘဏ် နှင့် ၂ တို့ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။\n"
    );
    let mut out = Vec::new();
    let (machine, result) = run_and_keep(
        &compile(&text),
        &mut out,
        &RunOptions::default(),
        &HostData::default(),
    );
    assert!(result.is_ok(), "{result:?}");
    let kinds: Vec<&TrapKind> =
        machine.thread_traps().iter().map(|(_, t)| &t.kind).collect();
    assert_eq!(kinds, [&TrapKind::ZeroDivisor, &TrapKind::LockBroken]);
}

#[test]
fn a_field_of_an_object_threads_share_is_reached_inside_a_lock_block_alone()
{
    let outside = format!(
        "{SPLIT}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘဏ် သည် အကောင့်(လက်ကျန်: ၁၀) ဖြစ်ပါစေ။
ဘဏ် ၏ လက်ကျန် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။\n"
    );
    assert_eq!(codes(&outside), [Code::OutsideLock]);
    let value = program("ကိန်းရှင် ဂ သည် ၅ ဖြစ်ပါစေ။\nဂ ကို ခေတ္တယူ၍ ဂ ကို ပြသမည်။");
    assert_eq!(codes(&value), [Code::OutsideLock]);
}

#[test]
fn threads_add_to_an_atomic_number_with_no_lock()
{
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../zakac_codegen_target_c/tests/inputs/atomics.zaka"
    ))
    .unwrap();
    assert_eq!(
        output(&text),
        "၁၉၈၀\n၇\n၄\nရလဒ်ကောင်း(၃)\nအမှားအယွင်း(၉)\nရလဒ်ကောင်း(၇)\n၈\n၅၀၀၅၀၀\n၅၀၀၅၀၀၀\n"
    );
}

#[test]
fn a_set_of_an_atomic_number_that_reads_it_is_reported()
{
    let read_write = program(
        "ကိန်းရှင် ဒေသ သည် အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။\nဒေသ ကို ဒေသ + ၁ \
         ဟု ထားမည်။",
    );
    assert_eq!(codes(&read_write), [Code::AtomicReadWrite]);
    let narrow =
        program("ကိန်းရှင် ဒေသ သည် အက်တမ်မစ် ဖြစ်သော ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။");
    assert!(codes(&narrow).contains(&Code::KindMismatch), "{:?}", codes(&narrow));
}

#[test]
fn a_parallel_map_gives_a_list_in_the_order_of_the_cells()
{
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../zakac_codegen_target_c/tests/inputs/par_map.zaka"
    ))
    .unwrap();
    let mut out = Vec::new();
    let (machine, result) = run_and_keep(
        &compile(&text),
        &mut out,
        &RunOptions::default(),
        &HostData::default(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "၁၀၈၈၉၄\n၁၂၃၄၆!\n၃၃၈၃၅၀\n၁၀၀၀၀\n၃၉၆၉\n[၂၅၊ ၃၆]\n"
    );
    assert_eq!(machine.leaks(), Vec::new());
}

#[test]
fn a_round_of_a_parallel_loop_writes_the_cell_of_its_own_number_alone()
{
    let own = program(
        "ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ထွက် ၏ အကွက် i ကို i * i ဟု ထားမည်။
ထွက် ကို ပြသမည်။",
    );
    assert_eq!(output(&own), "[၀၊ ၁၊ ၄၊ ၉၊ ၁၆]\n");
    let outside = program(
        "ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။\n၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ စု ကို စု + i ဟု ထားမည်။",
    );
    assert_eq!(codes(&outside), [Code::RoundWritesOutside]);
}

/// The head of a program with a function that changes its parameter.
const BUMP: &str = "တိုး ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

#[test]
fn a_round_that_leaves_its_loop_or_its_function_is_refused()
{
    let leave = program("၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ကွင်းဆက် ကို ရပ်မည်။");
    assert_eq!(codes(&leave), [Code::RoundLeaves]);
    let back = format!(
        "ပြန် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ \
         ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n{}",
        program("")
    );
    assert_eq!(codes(&back), [Code::RoundLeaves]);
    let inner =
        program("၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ၀ မှ ၄ အထိ j အနေဖြင့် ရေတွက်၍ ကွင်းဆက် ကို ရပ်မည်။");
    assert_eq!(codes(&inner), []);
}

#[test]
fn a_continue_ends_the_round_alone()
{
    let text = program(
        "ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ i သည် ၂ ထက် ကြီးလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။
ထွက် ၏ အကွက် i ကို i + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ထွက် ကို ပြသမည်။",
    );
    assert_eq!(output(&text), "[၁၊ ၂၊ ၃၊ ၀၊ ၀]\n");
}

#[test]
fn a_round_reads_from_outside_values_threads_can_use_alone()
{
    let text = format!(
        "ကောင်တာ ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် n ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။\n{}",
        program(
            "ကိန်းရှင် c သည် ကောင်တာ(n: ၁) ဖြစ်ပါစေ။
ကိန်းရှင် ထွက် သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၁ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ထွက် ၏ အကွက် i ကို c ၏ n ဟု ထားမည်။"
        )
    );
    assert_eq!(codes(&text), [Code::NotForThreads]);
}

#[test]
fn a_round_lets_a_call_change_no_variable_from_outside()
{
    let text = format!(
        "{BUMP}{}",
        program(
            "ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ တိုး ဟူသော လုပ်ဆောင်ချက်ကို စု ဖြင့် လုပ်ဆောင်မည်။"
        )
    );
    assert_eq!(codes(&text), [Code::RoundWritesOutside]);
}

#[test]
fn a_parallel_loop_with_a_step_of_zero_stops()
{
    let text = program(
        "ကိန်းရှင် ခြား သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၄ အထိ ခြား စီ i အနေဖြင့် အပြိုင် ရေတွက်၍ ထွက် ၏ အကွက် i ကို i ဟု ထားမည်။",
    );
    let options = RunOptions::default();
    let (_, result) = run_source_with(&text, &options);
    let trap = result.expect_err("a step of zero stops");
    assert!(matches!(trap.kind, TrapKind::StepNotPositive), "{trap}");
}

#[test]
fn a_round_writes_no_row_of_a_table_from_outside()
{
    let text = program(
        "ကိန်းရှင် ဇ သည် ကိန်းပြည့် ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [:] ဖြစ်ပါစေ။
၀ မှ ၉ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ဇ ထဲမှ အညွှန်း i ၏ တန်ဖိုး ကို i ဟု ထားမည်။",
    );
    assert_eq!(codes(&text), [Code::RoundWritesOutside]);
}

#[test]
fn a_count_of_threads_of_zero_stops()
{
    let text = program(
        "ကိန်းရှင် ချည် သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင်လုပ်ငန်း ချည် ခုခွဲ၍ ရေတွက်၍ ထွက် ၏ အကွက် i ကို i ဟု ထားမည်။",
    );
    let (_, result) = run_source_with(&text, &RunOptions::default());
    let trap = result.expect_err("a count of zero stops");
    assert!(matches!(trap.kind, TrapKind::ThreadsNotPositive), "{trap}");
}

#[test]
fn a_count_of_threads_that_is_no_integer_is_refused()
{
    let text = program(
        "ကိန်းရှင် ထွက် သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၄ အထိ i အနေဖြင့် အပြိုင်လုပ်ငန်း \"နှစ်\" ခုခွဲ၍ ရေတွက်၍ ထွက် ၏ အကွက် i ကို i ဟု ထားမည်။",
    );
    assert_eq!(codes(&text), [Code::LoopBoundNotInt]);
}

#[test]
fn a_parallel_loop_over_each_value_of_a_narrow_kind_counts_with_no_overflow()
{
    let signed = program(
        "ကိန်းရှင် နိမ့် သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် -၁၂၈ ဖြစ်ပါစေ။
ကိန်းရှင် မြင့် သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၇ ဖြစ်ပါစေ။
နိမ့် မှ မြင့် အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် y သည် i ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
\"ok\" ဟု ပြသမည်။",
    );
    assert_eq!(codes(&signed), []);
    assert_eq!(output(&signed), "ok\n");
    let unsigned = program(
        "ကိန်းရှင် နိမ့် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် မြင့် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅၅ ဖြစ်ပါစေ။
ကိန်းရှင် ထွက် သည် အကွက် ၂၅၆ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
မြင့် မှ နိမ့် အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ထွက် ၏ အကွက် i ကို ၁ ဟု ထားမည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
ထွက် ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ စု ကို စု + x ဟု ထားမည်။
စု ကို ပြသမည်။",
    );
    assert_eq!(output(&unsigned), "၂၅၆\n");
}

#[test]
fn a_trap_in_a_round_names_the_function_of_its_loop()
{
    let text = program(
        "ကိန်းရှင် ထွက် သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၀ မှ ၁ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ထွက် ၏ အကွက် i ကို ၁၀ ကို (၁ - i) ဖြင့် စားသော စားလဒ် ဟု ထားမည်။",
    );
    // Fewer rounds than cores run in order, on the thread of the loop.
    let (_, result) = run_source_with(&text, &RunOptions::default());
    let trap = result.expect_err("the second round divides by zero");
    assert_eq!((trap.kind, trap.func.as_str()), (TrapKind::ZeroDivisor, "ပရိုဂရမ်"));
}

#[test]
fn a_thread_that_ended_gives_its_stack_region_to_the_next()
{
    // Each region takes 1 MiB of the 4 GiB of a 32-bit VM: 8000 threads fit
    // when the regions of the ended ones come back.
    let text = program(
        "ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
၁ မှ ၈၀၀၀ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် j သည် စား ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
စု ကို စု + (j ကို စောင့်သော ရလဒ်) ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စု ကို ပြသမည်။",
    );
    assert_eq!(output_on(&text, 32), "၁၆၀၀၀\n");
}

#[test]
fn a_list_result_comes_to_its_wait_after_its_region_went_to_another_thread()
{
    // က ends while ခ runs, and ဂ then takes a region of an ended thread.
    let text = program(
        "ကိန်းရှင် က သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို \"က\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို \"ခ\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
(ခ ကို စောင့်သော ရလဒ်) ကို ပြသမည်။
ကိန်းရှင် ဂ သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို \"ဂ\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
(ဂ ကို စောင့်သော ရလဒ်) ကို ပြသမည်။
(က ကို စောင့်သော ရလဒ်) ကို ပြသမည်။",
    );
    assert_eq!(output(&text), "ခ!\nဂ!\nက!\n");
}

#[test]
fn a_seed_shuffles_the_turns_and_gives_the_same_order_again()
{
    let text = program(
        "ကိန်းရှင် က သည် ပြ ဟူသော လုပ်ဆောင်ချက်ကို \"က\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ပြ ဟူသော လုပ်ဆောင်ချက်ကို \"ခ\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ပြ ဟူသော လုပ်ဆောင်ချက်ကို \"ဂ\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
\"မ\" ဟု ပြသမည်။",
    );
    let run = |seed: u64| {
        let options = RunOptions::builder().shuffle(seed).build();
        let (out, result) = run_source_with(&text, &options);
        assert!(result.is_ok(), "seed {seed}: {result:?}");
        out
    };
    let lines = |out: &str| {
        let mut l: Vec<String> = out.lines().map(str::to_string).collect();
        l.sort();
        l
    };
    let fixed = lines(&output(&text));
    let orders: std::collections::BTreeSet<String> = (0..20).map(run).collect();
    // Each order holds the same lines, and the seeds give more than one order.
    assert!(orders.iter().all(|o| lines(o) == fixed), "{orders:?}");
    assert!(orders.len() > 1, "{orders:?}");
    assert_eq!(run(7), run(7));
}

/// Three threads claim an object through a compare-and-set of its owner field.
const CLAIM: &str = "ကောင်တာ ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် \
                     ပိုင်ရှင် ဟူသော အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

ယူ ဟူသော လုပ်ဆောင်ချက် သည် ဘူး ဟူသော ကောင်တာ တစ်ခု ကို လည်းကောင်း န ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ရ သည် ဘူး ၏ ပိုင်ရှင် ထဲရှိ ၀ ကို န ဖြင့် အစားထိုးသော ရလဒ် ဖြစ်ပါစေ။
အကယ်၍ ရ ထဲတွင် အောင်မြင်သည့်သတင်း ရှိလျှင် ၁ ကို ပြန်ပေးမည်။
၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဘူး သည် ကောင်တာ(ပိုင်ရှင်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ဘူး နှင့် ၁ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ဘူး နှင့် ၂ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ဘူး နှင့် ၃ တို့ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
(က ကို စောင့်သော ရလဒ်) + (ခ ကို စောင့်သော ရလဒ်) + (ဂ ကို စောင့်သော ရလဒ်) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

#[test]
fn a_compare_and_set_lets_one_thread_alone_claim_an_object()
{
    // Each order of the turns gives one winner.
    for seed in 0..20
    {
        let options = RunOptions::builder().shuffle(seed).build();
        let (out, result) = run_source_with(CLAIM, &options);
        assert!(result.is_ok(), "seed {seed}: {result:?}");
        assert_eq!(out, "၁\n", "seed {seed}");
    }
}
