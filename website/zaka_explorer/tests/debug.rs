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
use zaka_explorer::debug::BlockState;
use zaka_explorer::debug::RunStatus;
use zaka_explorer::debug::Session;
use zaka_vm::mem::Origin;
use zakac_ir_builder::ir::FoldOptions;
use zakac_rust_sugars::do_while;

/// Compile a sample and start a session over its ops.
fn session_of(path: &str) -> (Compiled, Session)
{
    let out = compile(find_sample(path).unwrap().text, &Options::default());
    let session = Session::new(out.ops.as_ref());
    (out, session)
}

/// The main function index and the op indices of its calls.
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
    let text = "ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n\"အရင်\" ဟု ပြသပါ။\n၁ / သုည ကို ပြသပါ။";
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
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၇ ဖြစ်ပါစေ။\nက + ၁ ကို ပြသပါ။";
    let out = compile(text, &Options::default());
    let mut s = Session::new(out.ops.as_ref());
    let globals = s.globals();
    assert_eq!(globals.len(), 1);
    assert_eq!((globals[0].name.as_str(), globals[0].kind.as_str()), ("က", "isize"));
    s.step();
    assert_eq!(s.stack().len(), 1);
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
    let text = "ကိန်းသေ က သည် ကိန်းပြည့် (၂ + ၃) * ၄ ဖြစ်ပါစေ။\nက ကို ပြသပါ။\n\"x\" ဟု ပြသပါ။";
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
fn an_addressed_variable_shows_its_value_then_its_cell()
{
    let text = "ကိန်းရှင် x သည် ကိန်းပြည့် ၇ ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ x ၏ လိပ်စာ ဖြစ်ပါစေ။";
    let out = compile(text, &Options::default());
    let mut s = Session::new(out.ops.as_ref());
    let set =
        |s: &Session| s.globals().get(1).is_some_and(|r| r.value.starts_with("0x"));
    do_while!(!set(&s) && s.steps() < 64, s.step());
    let rows = s.globals();
    let (x, p) = (&rows[0], &rows[1]);
    assert_eq!(x.name, "x");
    assert_eq!(x.value, format!("၇ @{}", p.value));
}

#[test]
fn the_heap_lists_blocks_as_the_program_asks_for_them_and_frees_them()
{
    let (_, mut s) = session_of("heap/growable");
    assert!(s.heap().blocks.is_empty());
    while s.heap().blocks.is_empty()
    {
        s.step();
    }
    let first = s.heap().blocks[0].clone();
    assert_eq!(
        (first.origin, first.state, first.len),
        (Origin::Heap, BlockState::Live, 32)
    );
    assert_eq!(first.data.len(), 32);
    while s.heap().freed == 0
    {
        s.step();
    }
    let heap = s.heap();
    assert_eq!(heap.blocks[0].state, BlockState::Freed);
    assert_eq!((heap.blocks[1].state, heap.blocks[1].len), (BlockState::Live, 64));
    assert_eq!((heap.live, heap.live_heap), (1, 64));
}

#[test]
fn a_finished_run_keeps_its_last_heap_until_the_next_run()
{
    let (_, mut s) = session_of("heap/growable");
    while s.run_for(100, false)
    {}
    assert_eq!(s.status(), RunStatus::Finished);
    let states: Vec<BlockState> = s.heap().blocks.iter().map(|b| b.state).collect();
    assert_eq!(states, [BlockState::Freed; 3]);
    s.reset();
    assert!(s.heap().blocks.is_empty());
}

#[test]
fn heap_room_never_freed_shows_as_leaked_once_the_run_finishes()
{
    let text = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။\nမင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် \
                သိမ်းထားမည်။\nပ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားပါ။";
    let out = compile(text, &Options::default());
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
fn a_pointer_leads_into_a_cell_that_names_its_maker()
{
    let (_, mut s) = session_of("heap/swap");
    s.step();
    let frames = s.frame_blocks();
    let values = frames.iter().flat_map(|f| f.slots.iter().chain(&f.temps));
    let addr = values.filter_map(|v| v.points_at).next().expect("a pointer");
    let heap = s.heap();
    let start = heap.block_of(addr).expect("its block");
    let block = heap.blocks.iter().find(|b| b.start == start).unwrap();
    assert_eq!((block.origin, block.maker.as_deref()), (Origin::Cell, Some("main")));
}

#[test]
fn each_mistake_traps_or_leaks()
{
    let mistakes: Vec<_> = SAMPLES.iter().filter(|s| s.group == MISTAKES).collect();
    assert!(mistakes.len() >= 8);
    for sample in mistakes
    {
        let (_, mut s) = session_of(&sample.path());
        while s.run_for(10_000, false)
        {}
        let leaked = s.heap().blocks.iter().any(|b| b.state == BlockState::Leaked);
        let trapped = s.status() == RunStatus::Trapped;
        assert!(trapped != leaked, "{}: {:?}", sample.path(), s.status());
    }
}
