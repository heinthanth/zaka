mod common;

use common::run_module;
use zaka_vm::machine::RunOptions;
use zaka_vm::trap::TrapKind;
use zakac_codegen::bc::Func;
use zakac_codegen::bc::Module;
use zakac_codegen::bc::Op;
use zakac_codegen::bc::Slot;
use zakac_ir_builder::ir::CmpOp;
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

fn run(m: &Module) -> (String, Result<(), TrapKind>)
{
    let (out, result) = run_module(m, &RunOptions::default());
    (out, result.map_err(|t| t.kind))
}

#[test]
fn push_and_print()
{
    let m = module(vec![main_of(vec![Op::Push(v("7:i32")), Op::Print, Op::Ret])]);
    assert_eq!(run(&m), ("၇\n".to_string(), Ok(())));
}

#[test]
fn calls_pass_arguments_and_return_a_result()
{
    let sub = Func::builder()
        .name("sub")
        .params(2)
        .ret(DataKind::ISIZE)
        .locals(vec![
            Slot::builder().name("a").kind(DataKind::ISIZE).build(),
            Slot::builder().name("b").kind(DataKind::ISIZE).build(),
        ])
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
    assert_eq!(run(&module(vec![sub, main])), ("၇\n".to_string(), Ok(())));
}

#[test]
fn locals_and_globals_start_at_their_defaults()
{
    let main = Func::builder()
        .name("main")
        .params(0)
        .locals(vec![Slot::builder().name("x").kind(DataKind::Bool).build()])
        .code(vec![Op::LoadLocal(0), Op::Print, Op::LoadGlobal(0), Op::Print, Op::Ret])
        .build();
    let m = Module::builder()
        .platform_bits(64)
        .globals(vec![Slot::builder().name("g").kind(DataKind::F64).build()])
        .funcs(vec![main])
        .main(0)
        .build();
    assert_eq!(run(&m), ("မမှန်\n၀.၀\n".to_string(), Ok(())));
}

#[test]
fn jumps_pop_their_condition()
{
    let m = module(vec![main_of(vec![
        Op::Push(v("1:i32")),
        Op::Push(v("2:i32")),
        Op::Compare(CmpOp::Lt),
        Op::JumpIfFalse(6),
        Op::PrintText(0),
        Op::Jump(7),
        Op::PrintText(1),
        Op::Ret,
    ])]);
    let m = Module { texts: vec!["ok".into(), "no".into()], ..m };
    assert_eq!(run(&m), ("ok\n".to_string(), Ok(())));
}

#[test]
fn malformed_ops_trap_instead_of_panicking()
{
    let cases = [
        vec![Op::Pop, Op::Ret],
        vec![Op::LoadLocal(3), Op::Ret],
        vec![Op::LoadGlobal(0), Op::Ret],
        vec![Op::Call(9), Op::Ret],
        vec![Op::PrintText(0), Op::Ret],
        vec![Op::Push(v("1:i32")), Op::JumpIfTrue(0), Op::Ret],
        vec![
            Op::Push(v("1:i32")),
            Op::Push(v("1.0:f64")),
            Op::Infix(InfixOp::Add),
            Op::Ret,
        ],
        vec![Op::Jump(9)],
    ];
    for code in cases
    {
        let m = module(vec![main_of(code.clone())]);
        let (_, result) = run(&m);
        assert!(
            matches!(result, Err(TrapKind::Malformed { .. })),
            "{code:?}: {result:?}"
        );
    }
}
