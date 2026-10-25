mod common;

use std::io;
use std::io::Write;

use common::compile;
use zaka_vm::machine::Machine;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::State;
use zaka_vm::machine::run_with;
use zaka_vm::trap::TrapKind;
use zakac_codegen::bc::Func;
use zakac_codegen::bc::Module;
use zakac_codegen::bc::Op;
use zakac_codegen::bc::Slot;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::value::Value;
use zakac_semcheck::kind::DataKind;

fn v(t: &str) -> Value { t.parse().unwrap_or_else(|e| panic!("{e}")) }

fn main_of(code: Vec<Op>) -> Func
{
    Func::builder().name("main").params(0).code(code).build()
}

fn module(funcs: Vec<Func>) -> Module
{
    let main = funcs.len() as u32 - 1;
    Module::builder().platform_bits(64).funcs(funcs).main(main).build()
}

/// `sub(10, 3)` printed from main: the call is at op 2 of main.
fn with_call() -> Module
{
    let isize = |name: &str| Slot::builder().name(name).kind(DataKind::ISIZE).build();
    let sub = Func::builder()
        .name("sub")
        .params(2)
        .ret(DataKind::ISIZE)
        .locals(vec![isize("a"), isize("b")])
        .code(vec![
            Op::LoadLocal(0),
            Op::LoadLocal(1),
            Op::Infix(InfixOp::Sub),
            Op::Ret,
        ])
        .build();
    let main = main_of(vec![
        Op::Push(v("10:isize")),
        Op::Push(v("3:isize")),
        Op::Call(0),
        Op::Print,
        Op::Ret,
    ]);
    module(vec![sub, main])
}

fn machine(m: Module) -> Machine { Machine::new(m, RunOptions::default()) }

#[test]
fn a_new_machine_stands_at_the_first_op_of_main()
{
    let m = machine(with_call());
    assert_eq!(m.position(), Some((1, 0)));
    assert_eq!(m.frames().len(), 1);
    assert_eq!(m.state(), &State::Running);
    assert_eq!(m.steps(), 0);
    assert_eq!(m.output(), "");
    assert_eq!(m.next_op(), Some(Op::Push(v("10:isize"))));
}

#[test]
fn each_step_runs_one_op()
{
    let mut m =
        machine(module(vec![main_of(vec![Op::Push(v("7:i32")), Op::Print, Op::Ret])]));
    m.step();
    assert_eq!(m.stack(), [v("7:i32")]);
    assert_eq!(m.position(), Some((0, 1)));
    m.step();
    assert_eq!(m.output(), "၇\n");
    assert_eq!(m.step(), &State::Finished);
    assert_eq!(m.position(), None);
    assert!(m.frames().is_empty());
    assert_eq!(m.steps(), 3);
}

#[test]
fn step_over_runs_a_call_to_its_return()
{
    let mut m = machine(with_call());
    m.step();
    m.step();
    assert_eq!(m.next_op(), Some(Op::Call(0)));
    m.step_over();
    assert_eq!(m.frames().len(), 1);
    assert_eq!(m.position(), Some((1, 3)));
    assert_eq!(m.stack(), [v("7:isize")]);
    m.step_over();
    assert_eq!(m.output(), "၇\n");
}

#[test]
fn step_out_returns_to_the_caller()
{
    let mut m = machine(with_call());
    do_steps(&mut m, 3);
    assert_eq!(m.frames().len(), 2);
    assert_eq!(m.position(), Some((0, 0)));
    m.step_out();
    assert_eq!(m.frames().len(), 1);
    assert_eq!(m.next_op(), Some(Op::Print));
    assert_eq!(m.step_out(), &State::Finished);
}

#[test]
fn locals_follow_each_frame()
{
    let mut m = machine(with_call());
    do_steps(&mut m, 3);
    assert_eq!(m.locals(1), [v("10:isize"), v("3:isize")]);
    assert!(m.locals(0).is_empty());
    assert!(m.locals(9).is_empty());
    let g = Slot::builder().name("g").kind(DataKind::Bool).build();
    let m = Machine::new(
        Module::builder()
            .platform_bits(64)
            .globals(vec![g])
            .funcs(vec![main_of(vec![Op::Ret])])
            .main(0)
            .build(),
        RunOptions::default(),
    );
    assert_eq!(m.globals(), [Value::Bool(false)]);
}

#[test]
fn a_frame_keeps_the_stack_depth_of_its_start()
{
    let mut m = machine(with_call());
    assert_eq!(m.frames()[0].depth, 0);
    do_steps(&mut m, 4);
    assert_eq!(m.frames()[1].depth, 0);
    assert_eq!(m.stack(), [v("10:isize")]);
}

#[test]
fn run_until_stops_at_a_break_point()
{
    let mut m = machine(with_call());
    m.run_until(&[(1, 3)]);
    assert_eq!(m.position(), Some((1, 3)));
    assert!(m.running());
    m.run_until(&[(1, 3)]);
    assert_eq!(m.state(), &State::Finished);
    assert_eq!(m.output(), "၇\n");
}

#[test]
fn a_trap_is_final()
{
    let text = "ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n\"အရင်\" ဟု ပြသပါ။\n၁ / သုည ကို ပြသပါ။";
    let mut m = machine(compile(text));
    m.run_until(&[]);
    let trap = m.state().trap().cloned().expect("trapped");
    assert_eq!(trap.kind, TrapKind::ZeroDivisor);
    let (func, pc) = m.position().expect("place");
    assert_eq!((m.module().funcs[func as usize].name.as_str(), pc), ("main", trap.pc));
    let steps = m.steps();
    m.step();
    m.step_over();
    assert_eq!(m.steps(), steps);
    assert_eq!(m.state().trap(), Some(&trap));
    assert_eq!(m.next_op(), None);
    assert_eq!(m.take_output(), "အရင်\n");
    assert_eq!(m.output(), "");
}

#[test]
fn the_op_budget_ends_a_run_until()
{
    let limits = RunOptions::builder().max_steps(1000).build();
    let mut m = Machine::new(compile("မှန် ဖြစ်နေသရွေ့ \"ထပ်\" ဟု ပြသပါ။"), limits);
    m.run_until(&[]);
    let kind = m.state().trap().map(|t| t.kind.clone());
    assert_eq!(kind, Some(TrapKind::StepLimit { steps: 1000 }));
}

#[test]
fn a_depth_limit_of_zero_traps_at_the_start()
{
    let limits = RunOptions::builder().max_depth(0).build();
    let m = Machine::new(with_call(), limits);
    let kind = m.state().trap().map(|t| t.kind.clone());
    assert_eq!(kind, Some(TrapKind::StackOverflow { depth: 0 }));
}

/// A writer that fails from its second write on.
#[derive(Default)]
struct Brittle
{
    /// What it took.
    taken: Vec<u8>,
    /// Writes so far.
    calls: u32,
}

impl Write for Brittle
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>
    {
        self.calls += 1;
        (self.calls > 1)
            .then_some(())
            .map_or(Ok(()), |_| Err(io::Error::other("full")))?;
        self.taken.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

#[test]
fn run_with_writes_as_it_goes_and_places_write_traps()
{
    let mut m =
        module(vec![main_of(vec![Op::PrintText(0), Op::PrintText(0), Op::Ret])]);
    m.texts = vec!["က".to_string()];
    let mut out = Brittle::default();
    let trap = run_with(&m, &mut out, &RunOptions::default()).unwrap_err();
    assert_eq!(String::from_utf8(out.taken).unwrap(), "က\n");
    assert!(matches!(trap.kind, TrapKind::Write { .. }), "{trap:?}");
    assert_eq!(trap.pc, 1);
}

/// Run `n` single steps.
fn do_steps(m: &mut Machine, n: usize) { (0..n).for_each(|_| _ = m.step()); }
