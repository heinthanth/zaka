use zakac_explorer::compile::Compiled;
use zakac_explorer::compile::Options;
use zakac_explorer::compile::compile;
use zakac_explorer::compile::samples::find_sample;
use zakac_explorer::debug::RunStatus;
use zakac_explorer::debug::Session;

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
    use zakac_explorer::debug::Speed;
    let rates: Vec<u64> = Speed::iter().map(Speed::rate).collect();
    assert_eq!(rates, [1, 2, 5, 10, 20, 50, 100]);
    assert_eq!(Speed::default(), Speed::Ops5);
    assert_eq!(Speed::Ops1.millis(), 1000);
    assert_eq!(Speed::Ops100.millis(), 10);
    assert_eq!("20".parse::<Speed>(), Ok(Speed::Ops20));
}
