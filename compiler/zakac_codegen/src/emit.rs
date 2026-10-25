use std::collections::HashMap;

use zakac_ast::stmt::CountMode;
use zakac_ir_builder::ir;
use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::LogicOp;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::Stmt;
use zakac_ir_builder::value::Value;
use zakac_ir_builder::value::compare;
use zakac_ir_builder::value::convert;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;
use zakac_span::span::Span;

use crate::bc::Func;
use crate::bc::Module;
use crate::bc::Op;
use crate::bc::Slot;
use crate::bc::TrapCode;
use crate::code::CodeBuilder;
use crate::code::Label;

/// The name of the main function.
pub const MAIN_NAME: &str = "main";

/// Generate stack machine ops for a checked, folded module.
///
/// # Arguments
///
/// * `module` - The IR module.
/// * `platform_bits` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The op module. Its functions keep their IR indices; the main function
/// comes last.
pub fn emit(module: &ir::Module, platform_bits: u16) -> Module
{
    let rets = module.funcs.iter().map(|f| f.ret).collect();
    let mut shared = Shared::builder().platform(platform_bits).rets(rets).build();
    let mut funcs: Vec<Func> =
        module.funcs.iter().map(|f| func(&mut shared, f)).collect();
    funcs.push(main_func(&mut shared, module));
    let globals = module.globals.iter().map(|s| slot(&s.name, s.kind)).collect();
    let main = funcs.len() as u32 - 1;
    #[rustfmt::skip]
    return Module::builder().platform_bits(platform_bits).globals(globals)
        .texts(shared.texts).funcs(funcs).main(main).build();
}

/// Make a slot.
///
/// # Arguments
///
/// * `name` - The name.
/// * `kind` - The data kind.
///
/// # Returns
///
/// The slot.
fn slot(name: &str, kind: DataKind) -> Slot
{
    Slot::builder().name(name).kind(kind).build()
}

/// Generate one IR function.
///
/// # Arguments
///
/// * `shared` - State shared across functions.
/// * `f` - The function.
///
/// # Returns
///
/// The op function.
fn func(shared: &mut Shared, f: &ir::Func) -> Func
{
    let locals = f.locals.iter().map(|s| slot(&s.name, s.kind)).collect();
    let mut g = FuncGen::new(shared, locals, f.span.shrink_to_hi());
    g.block(&f.block);
    let returns = matches!(f.block.last(), Some(Stmt::Return { .. }));
    do_if!(!returns, g.op(Op::Ret));
    g.finish(&f.name, f.params, f.ret)
}

/// Generate the main function from the top-level statements.
///
/// # Arguments
///
/// * `shared` - State shared across functions.
/// * `module` - The IR module that holds the statements.
///
/// # Returns
///
/// The op function.
fn main_func(shared: &mut Shared, module: &ir::Module) -> Func
{
    let mut g = FuncGen::new(shared, Vec::new(), module.span.shrink_to_hi());
    g.block(&module.main);
    g.op(Op::Ret);
    g.finish(MAIN_NAME, 0, None)
}

/// State shared across the functions of a module.
#[derive(bon::Builder)]
struct Shared
{
    /// Bit count of the platform-sized integer kinds.
    platform: u16,
    /// The result kind of each IR function.
    rets: Vec<Option<DataKind>>,
    /// Pooled print strings.
    #[builder(default)]
    texts: Vec<String>,
    /// The pool index of each string.
    #[builder(default)]
    text_index: HashMap<String, u32>,
}

impl Shared
{
    /// The pool index of a string, adding it on first use.
    ///
    /// # Arguments
    ///
    /// * `text` - The string.
    ///
    /// # Returns
    ///
    /// The index.
    fn text(&mut self, text: &str) -> u32
    {
        ret_if!(let Some(i) = self.text_index.get(text), *i);
        let i = self.texts.len() as u32;
        self.texts.push(text.to_string());
        self.text_index.insert(text.to_string(), i);
        i
    }
}

/// The jump targets of the innermost loop.
#[derive(Debug, Clone, Copy)]
struct LoopLabels
{
    /// Where `continue` goes.
    next: Label,
    /// Where `break` goes.
    exit: Label,
}

/// A counted-loop bound or step: a constant, or a local slot that holds
/// the value worked out once before the loop.
#[derive(Debug, Clone, Copy)]
enum Operand
{
    /// A constant.
    Const(Value),
    /// A local slot.
    Local(u32),
}

/// The parts of a counted loop the generator needs after its set-up.
#[derive(Debug, Clone, Copy)]
struct Counter
{
    /// The loop variable.
    var: Place,
    /// The kind of the loop variable.
    kind: DataKind,
    /// The unsigned kind of the same width.
    unsigned: DataKind,
    /// The last value.
    to: Operand,
    /// The step, positive.
    step: Operand,
}

/// Generator state for one function.
struct FuncGen<'s>
{
    /// State shared across functions.
    shared: &'s mut Shared,
    /// The ops.
    code: CodeBuilder,
    /// The local slots, growing as the generator adds temp slots.
    locals: Vec<Slot>,
    /// The enclosing loops, innermost last.
    loops: Vec<LoopLabels>,
}

impl<'s> FuncGen<'s>
{
    /// Start a function.
    ///
    /// # Arguments
    ///
    /// * `shared` - State shared across functions.
    /// * `locals` - The local slots from the IR.
    /// * `end` - The source region of ops outside each statement: a blank
    ///   region at the end of the declaration.
    ///
    /// # Returns
    ///
    /// The generator.
    fn new(shared: &'s mut Shared, locals: Vec<Slot>, end: Span) -> Self
    {
        let mut code = CodeBuilder::new();
        code.at(end);
        Self { shared, code, locals, loops: Vec::new() }
    }

    /// Finish the function.
    ///
    /// # Arguments
    ///
    /// * `name` - The function name.
    /// * `params` - The number of parameters.
    /// * `ret` - The result kind.
    ///
    /// # Returns
    ///
    /// The op function.
    fn finish(self, name: &str, params: u32, ret: Option<DataKind>) -> Func
    {
        let (code, spans) = self.code.finish().expect("each label is bound");
        #[rustfmt::skip]
        return Func::builder().name(name).params(params).maybe_ret(ret)
            .locals(self.locals).code(code).spans(spans).build();
    }

    /// Add an op.
    ///
    /// # Arguments
    ///
    /// * `op` - The op.
    fn op(&mut self, op: Op) { self.code.emit(op); }

    /// Add a local slot for a value the generator keeps.
    ///
    /// # Arguments
    ///
    /// * `name` - The slot name, starting with `%`.
    /// * `kind` - The data kind.
    ///
    /// # Returns
    ///
    /// The slot index.
    fn temp(&mut self, name: &str, kind: DataKind) -> u32
    {
        self.locals.push(slot(name, kind));
        self.locals.len() as u32 - 1
    }

    /// Push the value of a slot.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    fn load(&mut self, place: Place)
    {
        let op = match place
        {
            Place::Global(n) => Op::LoadGlobal(n),
            Place::Local(n) => Op::LoadLocal(n),
        };
        self.op(op);
    }

    /// Pop a value into a slot.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    fn store(&mut self, place: Place)
    {
        let op = match place
        {
            Place::Global(n) => Op::StoreGlobal(n),
            Place::Local(n) => Op::StoreLocal(n),
        };
        self.op(op);
    }

    /// Generate statements in order.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    fn block(&mut self, stmts: &[Stmt])
    {
        do_for!(s in stmts, self.stmt(s));
    }

    /// Generate a statement; its ops get its source region.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn stmt(&mut self, s: &Stmt)
    {
        let outer = self.code.at(s.span());
        self.stmt_ops(s);
        self.code.at(outer);
    }

    /// Generate the ops of a statement.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn stmt_ops(&mut self, s: &Stmt)
    {
        match s
        {
            Stmt::Set { place, value, .. } =>
            {
                self.expr(value);
                self.store(*place);
            }
            Stmt::Print { value: PrintValue::Expr(e), .. } =>
            {
                self.expr(e);
                self.op(Op::Print);
            }
            Stmt::Print { value: PrintValue::Text(text), .. } =>
            {
                let i = self.shared.text(text);
                self.op(Op::PrintText(i));
            }
            Stmt::If { cond, then, els, .. } => self.if_stmt(cond, then, els),
            Stmt::Loop { init, cond, update, block, .. } =>
            {
                self.loop_stmt(init, cond, update, block)
            }
            Stmt::Counted { var, from, to, step, mode, block, .. } =>
            {
                self.counted(*var, from, to, step.as_ref(), *mode, block)
            }
            Stmt::Break { .. } => self.leave(|l| l.exit),
            Stmt::Continue { .. } => self.leave(|l| l.next),
            Stmt::Return { value, .. } =>
            {
                do_if!(let Some(e) = value, self.expr(e));
                self.op(Op::Ret);
            }
            Stmt::Call { func, args, .. } =>
            {
                self.call(*func, args);
                let has_result = self.shared.rets[*func as usize].is_some();
                do_if!(has_result, self.op(Op::Pop));
            }
        }
    }

    /// Jump out of the innermost loop.
    ///
    /// # Arguments
    ///
    /// * `pick` - Picks the target from the loop labels.
    fn leave(&mut self, pick: impl Fn(LoopLabels) -> Label)
    {
        extract!(Some(labels) = self.loops.last().copied(), "break outside a loop");
        self.code.jump(pick(labels));
    }

    /// Generate an `if` statement.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    /// * `then` - Statements when it holds.
    /// * `els` - Statements when it does not.
    fn if_stmt(&mut self, cond: &Expr, then: &[Stmt], els: &[Stmt])
    {
        let other = self.code.label();
        self.expr(cond);
        self.code.jump_if_false(other);
        self.block(then);
        ret_if!(els.is_empty(), self.code.bind(other));
        let end = self.code.label();
        do_if!(!ends_in_jump(then), self.code.jump(end));
        self.code.bind(other);
        self.block(els);
        self.code.bind(end);
    }

    /// Generate a while or generic loop.
    ///
    /// # Arguments
    ///
    /// * `init` - Statements before the first test.
    /// * `cond` - The condition.
    /// * `update` - Statements after each round.
    /// * `block` - The loop statements.
    fn loop_stmt(&mut self, init: &[Stmt], cond: &Expr, update: &[Stmt], block: &[Stmt])
    {
        self.block(init);
        let head = self.code.label();
        let next = self.code.label();
        let exit = self.code.label();
        self.code.bind(head);
        self.expr(cond);
        self.code.jump_if_false(exit);
        self.in_loop(LoopLabels { next, exit }, block);
        self.code.bind(next);
        self.block(update);
        self.code.jump(head);
        self.code.bind(exit);
    }

    /// Generate loop statements with `break` and `continue` going to
    /// `labels`.
    ///
    /// # Arguments
    ///
    /// * `labels` - The loop labels.
    /// * `block` - The loop statements.
    fn in_loop(&mut self, labels: LoopLabels, block: &[Stmt])
    {
        self.loops.push(labels);
        self.block(block);
        self.loops.pop();
    }

    /// Generate a counted loop. The bounds and the step are worked out once,
    /// in source order, before the first round. A step that is not positive
    /// stops the program.
    ///
    /// # Arguments
    ///
    /// * `var` - The loop variable.
    /// * `from` - The first value.
    /// * `to` - The last value.
    /// * `step` - The step; one when absent.
    /// * `mode` - The direction.
    /// * `block` - The loop statements.
    fn counted(
        &mut self, var: Place, from: &Expr, to: &Expr, step: Option<&Expr>,
        mode: CountMode, block: &[Stmt],
    )
    {
        let kind = from.kind;
        self.expr(from);
        self.store(var);
        let to_op = self.operand("%to", to);
        let one = Value::int(int_kind(kind), 1, self.shared.platform);
        let step_op = step.map_or(Operand::Const(one), |s| self.operand("%step", s));
        self.check_step(step_op, kind);
        let unsigned = unsigned_of(kind);
        let c = Counter { var, kind, unsigned, to: to_op, step: step_op };
        let up = match mode
        {
            CountMode::Increase => Some(true),
            CountMode::Decrease => Some(false),
            CountMode::Count => self.static_direction(from, to),
        };
        match up
        {
            Some(up) => self.known_direction(c, up, block),
            None => self.runtime_direction(c, block),
        }
    }

    /// The direction of a counting loop whose bounds are both constants.
    ///
    /// # Arguments
    ///
    /// * `from` - The first value.
    /// * `to` - The last value.
    ///
    /// # Returns
    ///
    /// `Some(true)` to count up, `Some(false)` to count down, `None` when the
    /// direction is known at run time alone.
    fn static_direction(&self, from: &Expr, to: &Expr) -> Option<bool>
    {
        let (from, to) = (from.as_const()?, to.as_const()?);
        compare(CmpOp::Le, from, to).ok()
    }

    /// Evaluate a counted-loop operand once.
    ///
    /// # Arguments
    ///
    /// * `name` - The temp slot name for a value that is not constant.
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The constant, or the local slot that holds the value.
    fn operand(&mut self, name: &str, e: &Expr) -> Operand
    {
        ret_if!(let Some(v) = e.as_const(), Operand::Const(v));
        let n = self.temp(name, e.kind);
        self.expr(e);
        self.op(Op::StoreLocal(n));
        Operand::Local(n)
    }

    /// Push a counted-loop operand.
    ///
    /// # Arguments
    ///
    /// * `operand` - The operand.
    fn push(&mut self, operand: Operand)
    {
        let op = match operand
        {
            Operand::Const(v) => Op::Push(v),
            Operand::Local(n) => Op::LoadLocal(n),
        };
        self.op(op);
    }

    /// Push a counted-loop operand converted to an unsigned kind.
    ///
    /// # Arguments
    ///
    /// * `operand` - The operand.
    /// * `c` - The loop.
    fn push_unsigned(&mut self, operand: Operand, c: Counter)
    {
        let platform = self.shared.platform;
        let as_unsigned = |v| convert(v, c.unsigned, platform).unwrap_or(v);
        ret_if!(let Operand::Const(v) = operand, self.op(Op::Push(as_unsigned(v))));
        self.push(operand);
        do_if!(c.kind != c.unsigned, self.op(Op::Convert(c.unsigned)));
    }

    /// Push the loop variable converted to an unsigned kind.
    ///
    /// # Arguments
    ///
    /// * `c` - The loop.
    fn load_unsigned(&mut self, c: Counter)
    {
        self.load(c.var);
        do_if!(c.kind != c.unsigned, self.op(Op::Convert(c.unsigned)));
    }

    /// Stop the program when a step that is not a constant is not positive.
    ///
    /// # Arguments
    ///
    /// * `step` - The step.
    /// * `kind` - Its kind.
    fn check_step(&mut self, step: Operand, kind: DataKind)
    {
        ret_if!(let Operand::Const(_) = step);
        let fine = self.code.label();
        self.push(step);
        self.op(Op::Push(Value::default_of(kind)));
        self.op(Op::Compare(CmpOp::Gt));
        self.code.jump_if_true(fine);
        self.op(Op::Trap(TrapCode::StepNotPositive));
        self.code.bind(fine);
    }

    /// Jump to `exit` when the loop variable is past the last
    /// value.
    ///
    /// # Arguments
    ///
    /// * `c` - The loop.
    /// * `up` - `true` when counting up.
    /// * `exit` - The loop exit.
    fn skip_if_past(&mut self, c: Counter, up: bool, exit: Label)
    {
        self.load(c.var);
        self.push(c.to);
        self.op(Op::Compare(tern!(up, CmpOp::Gt, CmpOp::Lt)));
        self.code.jump_if_true(exit);
    }

    /// Move the loop variable one step on and go back to `head`, or jump to
    /// `exit` when one more step would pass the last value. The distance
    /// left is worked out in the unsigned kind, so it never wraps.
    ///
    /// # Arguments
    ///
    /// * `c` - The loop.
    /// * `up` - `true` when counting up.
    /// * `head` - The first op of the loop statements.
    /// * `exit` - The loop exit.
    fn step_on(&mut self, c: Counter, up: bool, head: Label, exit: Label)
    {
        tern!(up, self.push_unsigned(c.to, c), self.load_unsigned(c));
        tern!(up, self.load_unsigned(c), self.push_unsigned(c.to, c));
        self.op(Op::Infix(InfixOp::Sub));
        self.push_unsigned(c.step, c);
        self.op(Op::Compare(CmpOp::Lt));
        self.code.jump_if_true(exit);
        self.load(c.var);
        self.push(c.step);
        self.op(Op::Infix(tern!(up, InfixOp::Add, InfixOp::Sub)));
        self.store(c.var);
        self.code.jump(head);
    }

    /// Generate a counted loop whose direction is known.
    ///
    /// # Arguments
    ///
    /// * `c` - The loop.
    /// * `up` - `true` when counting up.
    /// * `block` - The loop statements.
    fn known_direction(&mut self, c: Counter, up: bool, block: &[Stmt])
    {
        let head = self.code.label();
        let next = self.code.label();
        let exit = self.code.label();
        self.skip_if_past(c, up, exit);
        self.code.bind(head);
        self.in_loop(LoopLabels { next, exit }, block);
        self.code.bind(next);
        self.step_on(c, up, head, exit);
        self.code.bind(exit);
    }

    /// Generate a counting loop whose direction is known at run time alone:
    /// it counts up when the first value is not above the last one.
    ///
    /// # Arguments
    ///
    /// * `c` - The loop.
    /// * `block` - The loop statements.
    fn runtime_direction(&mut self, c: Counter, block: &[Stmt])
    {
        let dir = self.temp("%dir", DataKind::Bool);
        self.load(c.var);
        self.push(c.to);
        self.op(Op::Compare(CmpOp::Le));
        self.op(Op::StoreLocal(dir));
        let (head, next, exit) =
            (self.code.label(), self.code.label(), self.code.label());
        let (down_test, down_next) = (self.code.label(), self.code.label());
        self.op(Op::LoadLocal(dir));
        self.code.jump_if_false(down_test);
        self.skip_if_past(c, true, exit);
        self.code.jump(head);
        self.code.bind(down_test);
        self.skip_if_past(c, false, exit);
        self.code.bind(head);
        self.in_loop(LoopLabels { next, exit }, block);
        self.code.bind(next);
        self.op(Op::LoadLocal(dir));
        self.code.jump_if_false(down_next);
        self.step_on(c, true, head, exit);
        self.code.bind(down_next);
        self.step_on(c, false, head, exit);
        self.code.bind(exit);
    }

    /// Generate an expression; its value ends on the stack and its ops get
    /// its source region.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    fn expr(&mut self, e: &Expr)
    {
        let outer = self.code.at(e.span);
        self.expr_ops(e);
        self.code.at(outer);
    }

    /// Generate the ops of an expression.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    fn expr_ops(&mut self, e: &Expr)
    {
        match &e.node
        {
            ExprNode::Const(v) => self.op(Op::Push(*v)),
            ExprNode::Read(place) => self.load(*place),
            ExprNode::Prefix { op, operand } =>
            {
                self.expr(operand);
                self.op(Op::Prefix(*op));
            }
            ExprNode::Infix { op, lhs, rhs } =>
            {
                self.expr(lhs);
                self.expr(rhs);
                self.op(Op::Infix(*op));
            }
            ExprNode::Compare { op, lhs, rhs } =>
            {
                self.expr(lhs);
                self.expr(rhs);
                self.op(Op::Compare(*op));
            }
            ExprNode::Logic { op, lhs, rhs } => self.logic(*op, lhs, rhs),
            ExprNode::Convert { operand, .. } =>
            {
                self.expr(operand);
                do_if!(operand.kind != e.kind, self.op(Op::Convert(e.kind)));
            }
            ExprNode::Call { func, args, .. } => self.call(*func, args),
        }
    }

    /// Generate `and` or `or`; the right operand runs just when needed.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    fn logic(&mut self, op: LogicOp, lhs: &Expr, rhs: &Expr)
    {
        let and = op == LogicOp::And;
        let short = self.code.label();
        let end = self.code.label();
        self.expr(lhs);
        tern!(and, self.code.jump_if_false(short), self.code.jump_if_true(short));
        self.expr(rhs);
        self.code.jump(end);
        self.code.bind(short);
        self.op(Op::Push(Value::Bool(!and)));
        self.code.bind(end);
    }

    /// Generate a call: the arguments in order, then the call.
    ///
    /// # Arguments
    ///
    /// * `func` - The function index.
    /// * `args` - The arguments.
    fn call(&mut self, func: u32, args: &[Expr])
    {
        do_for!(a in args, self.expr(a));
        self.op(Op::Call(func));
    }
}

/// Whether statements end in a jump, so no op after them runs.
///
/// # Arguments
///
/// * `stmts` - The statements.
///
/// # Returns
///
/// `true` when the last one is a return, `break` or `continue`.
fn ends_in_jump(stmts: &[Stmt]) -> bool
{
    let last = stmts.last();
    matches!(
        last,
        Some(Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. })
    )
}

/// The integer kind of a loop variable.
///
/// # Arguments
///
/// * `kind` - The data kind, an integer kind.
///
/// # Returns
///
/// The integer kind.
fn int_kind(kind: DataKind) -> IntKind
{
    extract!(Some(int) = kind.int_kind(), "counted loops count integers");
    int
}

/// The unsigned integer kind of the same width.
///
/// # Arguments
///
/// * `kind` - The data kind, an integer kind.
///
/// # Returns
///
/// The unsigned kind.
fn unsigned_of(kind: DataKind) -> DataKind
{
    let int = int_kind(kind);
    DataKind::Int(IntKind::new(false, int.width))
}
