use std::time::Duration;

use zaka_explorer::compile::Compiled;
use zaka_explorer::compile::Options;
use zaka_explorer::compile::asm::find_spot;
use zaka_explorer::compile::asm::has_ops;
use zaka_explorer::compile::asm::line_stops;
use zaka_explorer::compile::asm::op_spot;
use zaka_explorer::compile::compile;
use zaka_explorer::compile::samples::MISTAKES;
use zaka_explorer::compile::samples::SAMPLES;
use zaka_explorer::compile::samples::find_sample;
use zaka_explorer::debug::BlockRow;
use zaka_explorer::debug::BlockState;
use zaka_explorer::debug::MapPart;
use zaka_explorer::debug::RunStatus;
use zaka_explorer::debug::Session;
use zaka_explorer::debug::cell_width;
use zaka_vm::machine::FrameMode;
use zaka_vm::mem::Origin;
use zakac_borrow_checker::options::MemCheck;
use zakac_ir_builder::ir::FoldOptions;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::tern;

/// Compile a sample and start a session over its ops.
fn session_of(path: &str) -> (Compiled, Session)
{
    session_with(path, &Options::default())
}

/// Compile a sample with some options and start a session over its ops.
fn session_with(path: &str, options: &Options) -> (Compiled, Session)
{
    let out = compile(find_sample(path).unwrap().text, options);
    let session = Session::new(out.ops.as_ref());
    (out, session)
}

/// Options that build a program the ownership check finds fault with.
fn unchecked() -> Options { Options::builder().check(MemCheck::Off).build() }

/// The main function index and the op indices of its calls.
/// The heap blocks of a session, without the blocks of its variables.
fn heap_blocks(s: &Session) -> Vec<BlockRow>
{
    s.heap().blocks.into_iter().filter(|b| b.origin == Origin::Heap).collect()
}

fn main_calls(out: &Compiled) -> (u32, Vec<u32>)
{
    let main = out.asm.last().expect("main");
    let calls = main.rows.iter().filter(|r| r.text.starts_with("call "));
    (main.index, calls.map(|r| r.pc).collect())
}

#[test]
fn a_session_without_ops_does_nothing()
{
    let mut s = Session::new(None);
    assert!(!s.is_live());
    s.step();
    s.resume();
    assert_eq!(s.status(), RunStatus::Start);
    assert!(s.frames().is_empty() && s.output().is_empty());
    assert_eq!(s.viewed_func(), None);
}

#[test]
fn hello_runs_to_its_end_and_goes_back_to_the_first_op()
{
    let (out, mut s) = session_of("basics/hello");
    let (main, _) = main_calls(&out);
    assert_eq!(s.status(), RunStatus::Start);
    s.resume();
    assert_eq!(s.status(), RunStatus::Finished);
    assert!(!s.output().is_empty());
    assert!(s.steps() > 0);
    assert_eq!(s.position(), Some((main, 0)));
    assert_eq!(s.marked_pc(), Some(0));
    assert_eq!(s.trap(), None);
    let written = s.output();
    s.step();
    assert_eq!(s.status(), RunStatus::Paused);
    assert_eq!(s.steps(), 1);
    assert_ne!(s.output(), written);
}

#[test]
fn a_step_runs_one_op()
{
    let (out, mut s) = session_of("basics/hello");
    let (main, _) = main_calls(&out);
    assert_eq!(s.position(), Some((main, 0)));
    assert_eq!(s.marked_pc(), Some(0));
    s.step();
    assert_eq!(s.position(), Some((main, 1)));
    assert_eq!(s.steps(), 1);
    assert_eq!(s.status(), RunStatus::Paused);
}

#[test]
fn a_break_point_stops_the_run_and_step_over_runs_the_call()
{
    let (out, mut s) = session_of("functions/procedures");
    let (main, calls) = main_calls(&out);
    let call = calls[1];
    s.toggle_break(main, call);
    assert_eq!(s.breaks_in(main), [call]);
    s.resume();
    assert_eq!(s.position(), Some((main, call)));
    let before = s.output();
    s.step_over();
    assert_eq!(s.position(), Some((main, call + 1)));
    assert_eq!(s.frames().len(), 1);
    assert!(s.output().len() > before.len());
}

#[test]
fn step_goes_into_a_call_and_step_out_comes_back()
{
    let (out, mut s) = session_of("functions/procedures");
    let (main, calls) = main_calls(&out);
    let call = calls[1];
    s.toggle_break(main, call);
    s.resume();
    s.step();
    let frames = s.frames();
    assert_eq!(frames.len(), 2);
    assert_ne!(frames[0].func, main);
    assert_eq!(frames[1].pc, call);
    assert_eq!(s.viewed(), 1);
    let callee = &out.asm[frames[0].func as usize];
    let names: Vec<String> = s.locals().into_iter().map(|v| v.name).collect();
    let want: Vec<String> = callee.locals.iter().map(|l| l.name.clone()).collect();
    assert_eq!(names, want);
    assert_eq!(s.locals()[0].value, "၃");
    s.view_frame(0);
    assert_eq!(s.viewed_func(), Some(main));
    assert_eq!(s.marked_pc(), Some(call));
    s.step_out();
    assert_eq!(s.position(), Some((main, call + 1)));
    assert_eq!(s.viewed(), 0);
}

#[test]
fn reset_keeps_the_break_points()
{
    let (out, mut s) = session_of("functions/procedures");
    let (main, calls) = main_calls(&out);
    s.toggle_break(main, calls[2]);
    s.resume();
    s.reset();
    assert_eq!(s.status(), RunStatus::Start);
    assert!(s.output().is_empty());
    s.resume();
    assert_eq!(s.position(), Some((main, calls[2])));
    s.toggle_break(main, calls[2]);
    assert!(s.breaks_in(main).is_empty());
}

#[test]
fn the_timer_stops_at_a_break_point()
{
    let (out, mut s) = session_of("functions/procedures");
    let (main, calls) = main_calls(&out);
    s.toggle_break(main, calls[1]);
    let mut ticks = 0;
    while s.tick()
    {
        ticks += 1;
    }
    assert!(ticks > 0);
    assert_eq!(s.position(), Some((main, calls[1])));
    assert_eq!(s.status(), RunStatus::Paused);
}

#[test]
fn a_trap_is_final()
{
    let text = "ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n\"အရင်\" ဟု ပြသပါ။\n၁ / သုည ကို ပြသပါ။";
    let out = compile(text, &Options::default());
    let mut s = Session::new(out.ops.as_ref());
    s.resume();
    assert_eq!(s.status(), RunStatus::Trapped);
    let trap = s.trap().expect("trap");
    assert!(trap.starts_with("The program stopped: "), "{trap}");
    assert_eq!(s.output(), "အရင်\n");
    let (steps, at) = (s.steps(), s.position());
    assert!(at.is_some());
    s.step();
    s.step_out();
    assert_eq!((s.steps(), s.position()), (steps, at));
    assert!(!s.tick());
}

#[test]
fn globals_and_the_stack_show_their_values()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၇ ဖြစ်ပါစေ။\nက + ၁ ကို ပြသပါ။";
    let out = compile(text, &Options::default());
    let mut s = Session::new(out.ops.as_ref());
    let globals = s.globals();
    assert_eq!(globals.len(), 1);
    assert_eq!((globals[0].name.as_str(), globals[0].kind.as_str()), ("က", "isize"));
    s.step();
    assert_eq!(s.stack().len(), 1, "the address of က");
    s.step();
    assert_eq!(s.stack().len(), 2, "then the value");
    s.step();
    assert_eq!(s.globals()[0].value, "၇");
    assert!(s.stack().is_empty());
}

#[test]
fn a_batch_stops_at_a_break_point_when_asked()
{
    let (out, mut s) = session_of("functions/procedures");
    let (main, calls) = main_calls(&out);
    s.toggle_break(main, calls[1]);
    while s.run_for(3, true)
    {}
    assert_eq!(s.position(), Some((main, calls[1])));
    while s.run_for(3, false)
    {}
    assert_eq!(s.status(), RunStatus::Finished);
}

#[test]
fn stop_goes_back_to_the_first_op_and_keeps_the_output()
{
    let (out, mut s) = session_of("functions/procedures");
    let (main, calls) = main_calls(&out);
    s.stop();
    assert_eq!(s.status(), RunStatus::Start);
    s.toggle_break(main, calls[2]);
    s.resume();
    let (written, steps) = (s.output(), s.steps());
    assert!(!written.is_empty());
    s.stop();
    assert_eq!(s.status(), RunStatus::Stopped);
    assert_eq!(s.position(), Some((main, 0)));
    assert_eq!(s.steps(), steps);
    assert_eq!(s.output(), written);
    s.step();
    assert_eq!(s.status(), RunStatus::Paused);
    assert_eq!(s.steps(), 1);
}

#[test]
fn frames_stack_up_and_the_registers_point_into_them()
{
    let (out, mut s) = session_of("functions/procedures");
    let (main, calls) = main_calls(&out);
    s.toggle_break(main, calls[1]);
    s.resume();
    s.step();
    s.step();
    let blocks = s.frame_blocks();
    assert_eq!(blocks.len(), 2);
    let (inner, outer) = (&blocks[0], &blocks[1]);
    assert_eq!(outer.start, 0);
    assert_eq!(inner.start, outer.slots.len() + outer.temps.len());
    assert_eq!(inner.slots[0].value, "၃");
    assert_eq!(inner.temps.len(), 1);
    let regs = s.registers();
    assert_eq!(regs.bp, Some(inner.start));
    assert_eq!(regs.sp, inner.start + inner.slots.len() + inner.temps.len());
    let (func, pc) = s.position().unwrap();
    assert_eq!(regs.pc, Some(out.asm[func as usize].start + pc));
    assert_eq!(s.frames()[0].addr, regs.pc.unwrap());
}

#[test]
fn op_addresses_run_on_from_one_function_to_the_next()
{
    let (out, _) = session_of("functions/procedures");
    let addrs: Vec<u32> =
        out.asm.iter().flat_map(|f| f.rows.iter().map(|r| r.addr)).collect();
    let want: Vec<u32> = (0..addrs.len() as u32).collect();
    assert_eq!(addrs, want);
    let call =
        out.asm.iter().flat_map(|f| &f.rows).find(|r| r.text.starts_with("call "));
    assert!(call.unwrap().note.as_ref().unwrap().contains(" @ "));
}

#[test]
fn each_speed_steps_at_its_rate()
{
    use strum::IntoEnumIterator;
    use zaka_explorer::debug::Speed;
    let rates: Vec<u64> = Speed::iter().map(Speed::rate).collect();
    assert_eq!(rates, [1, 2, 5, 10, 20, 50, 100]);
    assert_eq!(Speed::default(), Speed::Ops5);
    assert_eq!(Speed::Ops1.millis(), 1000);
    assert_eq!(Speed::Ops100.millis(), 10);
    assert_eq!("20".parse::<Speed>(), Ok(Speed::Ops20));
}

#[test]
fn endless_loops_stop_at_the_budget()
{
    let out = compile("မှန် ဖြစ်နေသရွေ့ \"ထပ်\" ဟု ပြသပါ။", &Options::default());
    let mut s = Session::new(out.ops.as_ref());
    s.resume();
    assert_eq!(s.status(), RunStatus::Trapped);
    let trap = s.trap().expect("trap");
    assert!(trap.contains("stopped after"), "{trap}");
}

#[test]
fn a_line_stops_where_the_run_comes_onto_it()
{
    let (out, _) = session_of("programs/gcd");
    let stops = line_stops(&out.asm, 7);
    assert!(!stops.is_empty());
    let head =
        |&(f, pc): &(u32, u32)| out.asm[f as usize].rows[pc as usize].label.is_some();
    assert!(stops.iter().any(head), "{stops:?}");
    assert!(line_stops(&out.asm, 2).is_empty());
    assert!(!has_ops(&out.asm, 1) && !has_ops(&out.asm, 2));
    assert!(has_ops(&out.asm, 7));
}

#[test]
fn a_break_on_a_loop_line_pauses_on_each_pass()
{
    let (out, mut s) = session_of("programs/gcd");
    s.set_line_stops(line_stops(&out.asm, 7));
    s.resume();
    let first = s.position();
    let mut steps = s.steps();
    assert_eq!(s.status(), RunStatus::Paused);
    for _ in 0..3
    {
        s.resume();
        assert_eq!(s.position(), first);
        assert!(s.steps() > steps);
        steps = s.steps();
    }
}

#[test]
fn line_stops_join_the_op_breaks_and_replace_as_a_whole()
{
    let (out, mut s) = session_of("programs/gcd");
    let stops = line_stops(&out.asm, 7);
    let (f, pc) = stops[0];
    s.toggle_break(f, 0);
    s.set_line_stops(stops.clone());
    assert!(s.from_line(f, pc) && !s.from_line(f, 0));
    assert!(s.breaks_in(f).contains(&pc) && s.breaks_in(f).contains(&0));
    s.set_line_stops([]);
    assert_eq!(s.breaks_in(f), [0]);
}

#[test]
fn each_op_has_a_spot_that_finds_it_again()
{
    let (out, _) = session_of("programs/gcd");
    for func in &out.asm
    {
        for row in &func.rows
        {
            let spot = op_spot(&out.asm, func.index, row.pc).expect("spot");
            assert_eq!(find_spot(&out.asm, &spot), Some((func.index, row.pc)));
        }
    }
    assert_eq!(op_spot(&out.asm, 99, 0), None);
}

#[test]
fn a_spot_finds_its_op_after_the_folding_changes()
{
    let text = "ကိန်းသေ က သည် ကိန်းပြည့် (၂ + ၃) * ၄ ဖြစ်ပါစေ။\nက ကို ပြသပါ။\n\"x\" ဟု ပြသပါ။";
    let folded = compile(text, &Options::default());
    let none = Options::builder().fold(FoldOptions::none()).build();
    let plain = compile(text, &none);
    let main = folded.asm.last().expect("main");
    let print =
        main.rows.iter().find(|r| r.text.starts_with("print_text")).expect("print");
    let spot = op_spot(&folded.asm, main.index, print.pc).expect("spot");
    let (f, pc) = find_spot(&plain.asm, &spot).expect("found");
    let row = &plain.asm[f as usize].rows[pc as usize];
    assert_eq!((row.text.as_str(), row.range), (print.text.as_str(), print.range));
    assert_ne!(pc, print.pc, "the folding should move the op");
}

#[test]
fn each_variable_shows_its_value_and_the_block_it_lives_in()
{
    let text = "ကိန်းရှင် x သည် ကိန်းပြည့် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ငှားထားသော ကိန်းပြည့် ၏ လိပ်စာ x ၏ လိပ်စာ ဖြစ်ပါစေ။";
    let out = compile(text, &Options::default());
    let mut s = Session::new(out.ops.as_ref());
    let set =
        |s: &Session| s.globals().get(1).is_some_and(|r| r.value.starts_with("0x"));
    do_while!(!set(&s) && s.steps() < 64, s.step());
    let rows = s.globals();
    let (x, p) = (&rows[0], &rows[1]);
    assert_eq!((x.name.as_str(), x.value.as_str()), ("x", "၇"));
    let home = x.home.expect("x lives in a block");
    assert_eq!(p.value, format!("{home:#x}"));
    assert_eq!(p.points_at, Some(home));
    let heap = s.heap();
    let block = heap.blocks.iter().find(|b| b.start == home).expect("its block");
    assert_eq!(
        (block.origin, block.owner.as_deref()),
        (Origin::Static, Some("Global x"))
    );
}

#[test]
fn the_heap_lists_blocks_as_the_program_asks_for_them_and_frees_them()
{
    let (_, mut s) = session_of("heap/growable");
    assert!(heap_blocks(&s).is_empty());
    while heap_blocks(&s).is_empty()
    {
        s.step();
    }
    let first = heap_blocks(&s)[0].clone();
    assert_eq!(
        (first.origin, first.state, first.len),
        (Origin::Heap, BlockState::Live, 32)
    );
    assert_eq!(first.data.len(), 32);
    while s.heap().freed == 0
    {
        s.step();
    }
    let blocks = heap_blocks(&s);
    assert_eq!(blocks[0].state, BlockState::Freed);
    assert_eq!((blocks[1].state, blocks[1].len), (BlockState::Live, 64));
    assert_eq!(s.heap().live_heap, 64);
}

#[test]
fn a_finished_run_keeps_its_last_heap_until_the_next_run()
{
    let (_, mut s) = session_of("heap/growable");
    while s.run_for(100, false)
    {}
    assert_eq!(s.status(), RunStatus::Finished);
    let states: Vec<BlockState> = heap_blocks(&s).iter().map(|b| b.state).collect();
    assert_eq!(states, [BlockState::Freed; 3]);
    s.reset();
    assert!(heap_blocks(&s).is_empty());
}

#[test]
fn heap_room_never_freed_shows_as_leaked_once_the_run_finishes()
{
    let text = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။\nမင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် \
                သိမ်းထားမည်။\nပ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားပါ။";
    let out = compile(text, &unchecked());
    let mut s = Session::new(out.ops.as_ref());
    s.resume();
    assert_eq!(s.status(), RunStatus::Finished);
    let heap = s.heap();
    let leaked: Vec<_> =
        heap.blocks.iter().filter(|b| b.origin == Origin::Heap).collect();
    assert_eq!(leaked.len(), 1);
    assert_eq!((leaked[0].state, leaked[0].len), (BlockState::Leaked, 8));
    assert!(leaked[0].data.contains(&42));
}

#[test]
fn a_local_lives_in_a_stack_block_that_names_it_and_its_call()
{
    let (_, mut s) = session_of("heap/swap");
    do_while!(s.frame_blocks().len() < 2, s.step());
    let frames = s.frame_blocks();
    let first = &frames[0].slots[0];
    let heap = s.heap();
    let named = |addr: u64| {
        let start = heap.block_of(addr).expect("a block");
        let b = heap.blocks.iter().find(|b| b.start == start).unwrap();
        (b.origin, b.owner.clone().unwrap_or_default())
    };
    let home = first.home.expect("a home");
    assert_eq!(named(home), (Origin::Stack, "က of လဲလှယ်".to_string()));
    let target = first.points_at.expect("a pointer");
    assert_eq!(named(target), (Origin::Static, "Global ပထမ".to_string()));
}

#[test]
fn packed_frames_put_the_locals_of_a_call_in_one_block()
{
    let (_, mut s) = session_of("heap/swap");
    s.set_frames(FrameMode::Packed);
    do_while!(s.frame_blocks().len() < 2, s.step());
    let frames = s.frame_blocks();
    let homes: Vec<u64> = frames[0].slots.iter().filter_map(|v| v.home).collect();
    let heap = s.heap();
    let blocks: Vec<Option<u64>> = homes.iter().map(|h| heap.block_of(*h)).collect();
    assert_eq!(homes.len(), 3, "က, ခ and ယာယီ");
    assert!(blocks.iter().all(|b| b.is_some() && *b == blocks[0]), "{blocks:?}");
    let start = blocks[0].unwrap();
    let block = heap.blocks.iter().find(|b| b.start == start).unwrap();
    assert_eq!(block.owner.as_deref(), Some("Frame of လဲလှယ်"));
    let names: Vec<&str> = block.vars.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, ["က", "ခ", "ယာယီ"], "the map shows each variable of the frame");
    let rows = block.rows();
    let owners: Vec<&str> = rows.iter().filter_map(|r| r.owner.as_deref()).collect();
    assert_eq!(owners, ["က of လဲလှယ်", "ခ of လဲလှယ်", "ယာယီ of လဲလှယ်"]);
    assert_eq!(rows.iter().map(|r| r.len).collect::<Vec<_>>(), [8, 8, 8]);
    assert_eq!(rows[1].start, start + 8, "a row of its own at the variable");
    assert_eq!(heap.row_of(homes[1]), Some(homes[1]), "a pointer finds its variable");
    s.set_frames(FrameMode::Checked);
    assert_eq!(s.steps(), 0, "a new form starts the run again");
}

#[test]
fn each_mistake_traps_or_leaks()
{
    let mistakes: Vec<_> = SAMPLES.iter().filter(|s| s.group == MISTAKES).collect();
    assert!(mistakes.len() >= 8);
    let warn = Options::builder().check(MemCheck::Warn).build();
    for sample in mistakes
    {
        let (_, mut s) = session_with(&sample.path(), &warn);
        while s.run_for(10_000, false)
        {}
        let leaked = s.heap().blocks.iter().any(|b| b.state == BlockState::Leaked);
        let trapped = s.status() == RunStatus::Trapped;
        assert!(trapped != leaked, "{}: {:?}", sample.path(), s.status());
    }
}

#[test]
fn a_finished_run_keeps_its_time_and_usage_until_a_reset()
{
    let (_, mut s) = session_of("heap/growable");
    assert_eq!(s.run_time(), Duration::ZERO);
    s.resume();
    assert_eq!(s.status(), RunStatus::Finished);
    let (heap, stack) = s.usage();
    assert!(heap > 0, "the list takes heap");
    assert!(stack > 0);
    assert!(s.run_time() > Duration::ZERO);
    s.reset();
    assert_eq!(s.run_time(), Duration::ZERO);
    assert_eq!(s.usage().0, 0);
}

#[test]
fn batches_add_up_their_time()
{
    let (_, mut s) = session_of("heap/growable");
    assert!(s.run_for(5, false));
    let first = s.run_time();
    assert!(first > Duration::ZERO);
    assert!(s.run_for(5, false));
    assert!(s.run_time() > first);
    assert_eq!(s.steps(), 10);
}

#[test]
fn the_map_shows_the_gap_after_each_variable_and_none_inside_a_packed_frame()
{
    let (_, mut s) = session_of("heap/swap");
    do_while!(s.frame_blocks().len() < 2, s.step());
    let gaps = |s: &Session| {
        let shot = s.heap();
        let lens = shot.parts().into_iter().filter_map(|p| match p
        {
            MapPart::Gap { len, .. } => Some(len),
            MapPart::Block(_) => None,
        });
        lens.collect::<Vec<u64>>()
    };
    let checked = gaps(&s);
    assert!(checked.contains(&24), "an 8-octet local, then 24 octets: {checked:?}");
    s.set_frames(FrameMode::Packed);
    do_while!(s.frame_blocks().len() < 2, s.step());
    let blocks = s.heap().blocks.len();
    assert_eq!(gaps(&s).len(), blocks, "a gap after each block, none within one");
}

#[test]
fn a_trap_in_a_gap_is_found_in_the_map()
{
    let (_, mut s) = session_with("mistakes/past_the_end", &unchecked());
    while s.run_for(10_000, false)
    {}
    assert_eq!(s.status(), RunStatus::Trapped);
    let shot = s.heap();
    let fault = shot.fault.expect("the address of the trap");
    let found = shot.parts().into_iter().find(|p| match *p
    {
        MapPart::Gap { start, len } => shot.faults_in(start, len),
        MapPart::Block(_) => false,
    });
    assert!(found.is_some(), "{fault:#x} lies in a gap");
}

#[test]
fn the_map_holds_each_block_in_address_order()
{
    let text = "ကိန်းရှင် ကိန်းများ သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်ပါစေ။\nမင်မိုရီ ကိန်းပြည့် လိပ်စာ ၃၀၀ \
                ခုစာ နေရာယူ၍ လိပ်စာကို ကိန်းများ တွင် သိမ်းထားမည်။\nကိန်းရှင် အရွယ် သည် လက္ခဏာမဲ့ ကိန်းပြည့် \
                ဖြစ်ပါစေ။\n၀ မှ ၂၉၉ အထိ နံပါတ် အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။\nအရွယ် ကို \
                (နံပါတ် % ၈ + ၁) ကို လက္ခဏာမဲ့ ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။\nကိန်းရှင် ပ \
                သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။\nမင်မိုရီ ကိန်းပြည့် အရွယ် ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် \
                သိမ်းထားမည်။\nပ ၏ တန်ဖိုး ကို နံပါတ် ဟု ထားပါ။\nကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို ပ ဟု \
                ထားပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။\n\n၀ မှ ၂၉၉ အထိ ၃ စီ နံပါတ် အနေဖြင့် တိုး၍ \
                အောက်ပါတို့ကို လုပ်ဆောင်ပါ။\nကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ကိန်းများ + နံပါတ် \
                ၏ တန်ဖိုး ဖြစ်ပါစေ။\nပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။\nကိန်းများ + နံပါတ် ၏ တန်ဖိုး ကို \
                ဗလာ ဟု ထားပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။\n\"ပြီးပါပြီ\" ဟု ပြသပါ။";
    let out = compile(text, &unchecked());
    let mut s = Session::new(out.ops.as_ref());
    s.resume();
    assert_eq!(s.status(), RunStatus::Finished);
    let shot = s.heap();
    let parts = shot.parts();
    let blocks: Vec<usize> = parts
        .iter()
        .filter_map(|p| tern!(let MapPart::Block(i) = p, Some(*i), None))
        .collect();
    assert_eq!(blocks.len(), 307, "six globals, the list and its 300 blocks");
    assert_eq!(
        parts.len() - blocks.len(),
        blocks.len(),
        "each block has a gap after it"
    );
    let starts: Vec<u64> = blocks.iter().map(|i| shot.blocks[*i].start).collect();
    assert!(starts.is_sorted(), "the map follows the addresses");
}

#[test]
fn a_cell_grows_one_step_for_each_doubling_of_its_block()
{
    let widths: Vec<u64> = [0, 8, 16, 24, 64, 2400, 1 << 30].map(cell_width).to_vec();
    assert_eq!(widths, [12, 12, 24, 24, 48, 108, 120]);
}

#[test]
fn the_stack_sits_between_the_static_data_and_the_heap_on_the_map()
{
    let (_, mut s) = session_of("programs/gradebook");
    do_while!(s.heap().blocks.iter().all(|b| b.origin != Origin::Heap), s.step());
    do_while!(s.frame_blocks().len() < 2, s.step());
    let shot = s.heap();
    let rank = |o: Origin| {
        [Origin::Static, Origin::Stack, Origin::Heap].iter().position(|r| *r == o)
    };
    let order: Vec<Option<usize>> = shot
        .parts()
        .iter()
        .filter_map(|p| tern!(let MapPart::Block(i) = p, Some(rank(shot.blocks[*i].origin)), None))
        .collect();
    assert!(order.is_sorted(), "static, then stack, then heap: {order:?}");
    assert!(order.contains(&Some(1)) && order.contains(&Some(2)));
    assert_eq!(shot.stack_cap, 1 << 20, "the stack region of a run");
    assert!(shot.stack_used > 0, "a call in progress takes stack room");
}

#[test]
fn each_address_is_padded_to_the_width_of_the_highest_one()
{
    let (_, mut s) = session_of("heap/growable");
    assert_eq!(s.heap().addr_digits(), 4, "globals alone sit under 0x10000");
    do_while!(heap_blocks(&s).is_empty(), s.step());
    assert_eq!(
        s.heap().addr_digits(),
        6,
        "the heap starts past the 1 MiB stack region"
    );
}

#[test]
fn a_list_block_knows_its_cells_in_both_frame_forms()
{
    let (_, mut s) = session_of("lists/grid");
    s.step();
    let heap = s.heap();
    let grid = heap.blocks.iter().find(|b| b.owner.as_deref() == Some("Global ဇယား"));
    let grid = grid.expect("the grid global");
    assert_eq!(grid.cells, [32, 8], "rows of 4 integers, cells of 8 octets");
    assert_eq!(grid.cell_path(0), [0, 0]);
    assert_eq!(grid.cell_path(40), [1, 1], "octet 40 sits in row 1, cell 1");
    assert_eq!(grid.cell_path(95), [2, 3]);

    let (_, mut s) = session_of("lists/copies");
    s.set_frames(FrameMode::Packed);
    do_while!(s.frame_blocks().len() < 2, s.step());
    let heap = s.heap();
    let frame =
        heap.blocks.iter().find(|b| !b.vars.is_empty()).expect("a packed frame");
    let rows = frame.rows();
    assert_eq!(rows[0].cells, [8], "the list parameter of the call");
    let rows: Vec<BlockRow> = heap.blocks.iter().flat_map(BlockRow::rows).collect();
    let global = rows.iter().find(|r| r.owner.as_deref() == Some("Global မူရင်း"));
    assert_eq!(global.map(|r| r.cells.clone()), Some(vec![8]), "a packed global too");
}
