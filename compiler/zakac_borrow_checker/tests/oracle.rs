mod common;

use std::mem::discriminant;
use std::path::Path;
use std::path::PathBuf;

use common::repo_root;
use common::zaka_files;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_with;
use zaka_vm::trap::TrapKind;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_codegen::artifact::run_phases;
use zakac_diagnostics::has_errors;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::Stmt;
use zakac_lexer::lexer::LexerOptions;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Whether a kind has a raw address level, at whatever depth, or is a
/// list whose cells have one.
fn raw_kind(kind: DataKind) -> bool
{
    ret_if!(let Some(list) = kind.as_list(), raw_kind(list.cell));
    kind.ptr_kind().is_some_and(|p| p.raw != 0)
}

/// Whether an expression, or one of its operands, has a raw kind.
fn raw_expr(e: &Expr) -> bool
{
    ret_if!(raw_kind(e.kind), true);
    match &e.node
    {
        ExprNode::Const(_)
        | ExprNode::Read(_)
        | ExprNode::AddrOf(_)
        | ExprNode::SizeOf(_)
        | ExprNode::BlankList => false,
        ExprNode::Prefix { operand, .. }
        | ExprNode::Convert { operand, .. }
        | ExprNode::Load { addr: operand }
        | ExprNode::Lend { operand }
        | ExprNode::Take { addr: operand } => raw_expr(operand),
        ExprNode::Infix { lhs, rhs, .. }
        | ExprNode::Compare { lhs, rhs, .. }
        | ExprNode::Logic { lhs, rhs, .. }
        | ExprNode::PtrAdd { base: lhs, offset: rhs }
        | ExprNode::PtrSub { lhs, rhs }
        | ExprNode::CellAddr { list: lhs, index: rhs, .. } =>
        {
            raw_expr(lhs) || raw_expr(rhs)
        }
        ExprNode::Call { args, .. } | ExprNode::List(args) => args.iter().any(raw_expr),
    }
}

/// Whether a heap destination names its address with a raw expression.
fn raw_dest(dest: &Dest) -> bool { matches!(dest, Dest::Mem(e) if raw_expr(e)) }

/// Whether a statement, or one nested in it, has a raw kind.
fn raw_stmt(s: &Stmt) -> bool
{
    match s
    {
        Stmt::Set { value, .. } => raw_expr(value),
        Stmt::Print { value: PrintValue::Expr(e), .. } => raw_expr(e),
        Stmt::If { cond, then, els, .. } =>
        {
            raw_expr(cond) || raw_block(then) || raw_block(els)
        }
        Stmt::Loop { init, cond, update, block, .. } =>
        {
            raw_block(init) || raw_expr(cond) || raw_block(update) || raw_block(block)
        }
        Stmt::Counted { from, to, step, block, .. } =>
        {
            raw_expr(from)
                || raw_expr(to)
                || step.iter().any(raw_expr)
                || raw_block(block)
        }
        Stmt::Return { value, .. } => value.iter().any(raw_expr),
        Stmt::Call { args, .. } => args.iter().any(raw_expr),
        Stmt::Store { addr, value, .. } => raw_expr(addr) || raw_expr(value),
        Stmt::Alloc { dest, count, .. } | Stmt::Resize { dest, count, .. } =>
        {
            raw_dest(dest) || raw_expr(count)
        }
        Stmt::Free { dest, .. } => raw_dest(dest),
        Stmt::Print { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => false,
    }
}

/// Whether a statement list has a raw kind.
fn raw_block(stmts: &[Stmt]) -> bool { stmts.iter().any(raw_stmt) }

/// Whether a module has a raw address kind in a slot, a result or an
/// expression. Such a program is outside what the check promises. The
/// module is the one before folding, where a raw cast of a constant still
/// shows.
fn has_raw_kinds(module: &Module) -> bool
{
    let funcs = module.funcs.iter();
    let locals = funcs.clone().flat_map(|f| &f.locals);
    let results = funcs.clone().filter_map(|f| f.ret);
    let kinds = module.globals.iter().chain(locals).map(|s| s.kind).chain(results);
    ret_if!(kinds.into_iter().any(raw_kind), true);
    raw_block(&module.main) || module.funcs.iter().any(|f| raw_block(&f.block))
}

/// The traps the check must rule out.
fn temporal_traps() -> [TrapKind; 3]
{
    [
        TrapKind::UseAfterFree { addr: 0 },
        TrapKind::DoubleFree { addr: 0 },
        TrapKind::BadFree { addr: 0 },
    ]
}

/// How one program fares in the VM.
enum Outcome
{
    /// Not run: the check reports an error, or the program has a raw kind.
    Skipped,
    /// Run with no temporal trap.
    Ran,
    /// Run into a temporal trap, with its message.
    Trapped(String),
}

/// Check one program and, when it passes with no raw kind, run it.
fn outcome(path: &Path) -> Outcome
{
    let text = std::fs::read_to_string(path).unwrap();
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom(path.display().to_string()), text);
    let emitted = run_phases(
        &file,
        &LexerOptions::default(),
        &SemcheckOptions::default(),
        &FoldOptions::default(),
        &BorrowCheckOptions::default(),
    );
    ret_if!(has_errors(emitted.diagnostics()), Outcome::Skipped);
    let lowered = emitted.borrowchecked.lowered.plain.as_ref().unwrap();
    ret_if!(has_raw_kinds(lowered), Outcome::Skipped);
    let options = RunOptions::builder().max_steps(10_000_000).build();
    let mut out = Vec::new();
    let Err(trap) = run_with(emitted.module.as_ref().unwrap(), &mut out, &options)
    else
    {
        return Outcome::Ran;
    };
    let kind = discriminant(&trap.kind);
    let temporal = temporal_traps().iter().any(|t| discriminant(t) == kind);
    tern!(temporal, Outcome::Trapped(trap.to_string()), Outcome::Ran)
}

/// Each sample and each input of this crate that passes the check with no
/// raw kind runs in the VM without a use after free, a double free or a
/// bad free. Enough of them must run, so that a change that skips each
/// program cannot pass.
#[test]
fn checked_programs_never_hit_temporal_traps()
{
    let inputs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inputs");
    let mut files = zaka_files(&repo_root().join("samples"));
    files.extend(zaka_files(&inputs));
    let outcomes: Vec<(String, Outcome)> =
        files.iter().map(|f| (f.display().to_string(), outcome(f))).collect();
    let ran = outcomes.iter().filter(|(_, o)| matches!(o, Outcome::Ran)).count();
    let found: Vec<(&String, &String)> = outcomes
        .iter()
        .filter_map(|(f, o)| match o
        {
            Outcome::Trapped(t) => Some((f, t)),
            _ => None,
        })
        .collect();
    assert!(found.is_empty(), "{found:#?}");
    assert!(ran >= 50, "{ran} of {} programs ran", files.len());
}
