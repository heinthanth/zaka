use std::collections::HashMap;

use zakac_ast::stmt::CountMode;
use zakac_borrow_checker::each_stmt;
use zakac_ir_builder::ir;
use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::FmtPiece;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::LogicOp;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::PrefixOp;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::SliceFrom;
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

use crate::bc::FmtArg;
use crate::bc::Func;
use crate::bc::Module;
use crate::bc::Op;
use crate::bc::SliceSource;
use crate::bc::Slot;
use crate::bc::TrapCode;
use crate::code::CodeBuilder;
use crate::code::Label;
use crate::options::EmitOptions;
use crate::options::ResultMode;
use crate::options::ScratchMode;

/// The name of the main function.
pub const MAIN_NAME: &str = "main";

/// The names of the slots that hold scratch values: values a statement
/// builds and uses before it ends. The slots of a counted loop live across
/// its block and are not among them.
const SCRATCH: &[&str] = &["%list", "%slice", "%ret", "%at", "%text"];

/// Generate stack machine ops for a checked, folded module.
///
/// # Arguments
///
/// * `module` - The IR module.
/// * `platform_bits` - Bit count of the platform-sized integer kinds.
/// * `options` - How results reach the caller and where scratch values live.
///
/// # Returns
///
/// The op module. Its functions keep their IR indices; the main function
/// comes last.
pub fn emit(module: &ir::Module, platform_bits: u16, options: EmitOptions) -> Module
{
    let rets = module.funcs.iter().map(|f| f.ret).collect();
    let globals = module.globals.iter().map(slot).collect();
    #[rustfmt::skip]
    let mut shared = Shared::builder().platform(platform_bits).rets(rets)
        .globals(globals).results(options.results).scratch(options.scratch).build();
    let mut funcs: Vec<Func> =
        module.funcs.iter().map(|f| func(&mut shared, f)).collect();
    funcs.push(main_func(&mut shared, module));
    let main = funcs.len() as u32 - 1;
    #[rustfmt::skip]
    return Module::builder().platform_bits(platform_bits).globals(shared.globals)
        .texts(shared.texts).formats(shared.formats).funcs(funcs).main(main).build();
}

/// Make the slot of an IR slot.
///
/// # Arguments
///
/// * `s` - The IR slot.
///
/// # Returns
///
/// The slot.
fn slot(s: &ir::Slot) -> Slot { Slot::builder().name(&s.name).kind(s.kind).build() }

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
    let locals = f.locals.iter().map(slot).collect();
    let in_place = shared.results == ResultMode::InPlace;
    let mut g = FuncGen::new(shared, locals, f.span.shrink_to_hi());
    g.in_slot = in_place.then(|| given_back(f)).flatten();
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
    /// How a list or struct result reaches the caller.
    results: ResultMode,
    /// Where the scratch values of a function live.
    scratch: ScratchMode,
    /// The global slots.
    globals: Vec<Slot>,
    /// Pooled print strings.
    #[builder(default)]
    texts: Vec<String>,
    /// The pool index of each string.
    #[builder(default)]
    text_index: HashMap<String, u32>,
    /// The pieces of each text built when the program runs.
    #[builder(default)]
    formats: Vec<Vec<FmtArg>>,
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

    /// The index of a format, added as a new one.
    ///
    /// # Arguments
    ///
    /// * `args` - The pieces.
    ///
    /// # Returns
    ///
    /// The index.
    fn format(&mut self, args: Vec<FmtArg>) -> u32
    {
        self.formats.push(args);
        self.formats.len() as u32 - 1
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
    /// The local that lives in the caller's slot for the result.
    in_slot: Option<u32>,
    /// Scratch slots a statement before this one used, free to take again.
    free: Vec<u32>,
    /// Scratch slots the statements since the last one began have taken.
    taken: Vec<u32>,
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
        #[rustfmt::skip]
        return Self { shared, code, locals, loops: Vec::new(), in_slot: None,
            free: Vec::new(), taken: Vec::new() };
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

    /// Add a local slot for a value the generator keeps. With shared scratch
    /// slots, a scratch value takes a free slot of its name and kind when a
    /// statement before this one left one.
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
        let scratch = SCRATCH.contains(&name);
        let shared = scratch && self.shared.scratch == ScratchMode::Shared;
        let fits = |n: &u32| {
            let slot = &self.locals[*n as usize];
            slot.name == name && slot.kind == kind
        };
        let found = shared.then(|| self.free.iter().position(fits)).flatten();
        let n = found.map_or_else(
            || {
                self.locals.push(Slot::builder().name(name).kind(kind).build());
                self.locals.len() as u32 - 1
            },
            |at| self.free.swap_remove(at),
        );
        do_if!(scratch, self.taken.push(n));
        n
    }

    /// The slot at a place.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    ///
    /// # Returns
    ///
    /// The slot.
    fn slot_at(&self, place: Place) -> &Slot
    {
        match place
        {
            Place::Global(n) => &self.shared.globals[n as usize],
            Place::Local(n) => &self.locals[n as usize],
        }
    }

    /// Push the address of the block that holds a slot.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    fn addr_of(&mut self, place: Place)
    {
        let op = self.addr_op(place);
        self.op(op);
    }

    /// The op that pushes the address of the block that holds a slot.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The op.
    fn addr_op(&self, place: Place) -> Op
    {
        match place
        {
            Place::Global(n) => Op::GlobalAddr(n),
            Place::Local(n) if self.in_slot == Some(n) => Op::ResultAddr,
            Place::Local(n) => Op::LocalAddr(n),
        }
    }

    /// Set a slot that holds a list, a struct or a slice. A slice, and a
    /// literal whose cells read nothing of the slot, are built in the slot
    /// itself; each other value is built elsewhere and duplicated in, so a
    /// value that reads the slot, as `l = [l[1], l[0]]` does, sees it whole.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `value` - The value.
    fn set_block(&mut self, place: Place, value: &Expr)
    {
        let base = self.addr_op(place);
        match &value.node
        {
            ExprNode::List(cells) if cells.iter().all(|c| apart_from(c, place)) =>
            {
                self.fill_at(base, value.kind, Some(cells))
            }
            ExprNode::MakeSlice { .. } =>
            {
                self.make_slice(base, value);
                self.op(Op::Pop);
            }
            ExprNode::Text(_) | ExprNode::Format(_) =>
            {
                self.text_at(base, value);
                self.op(Op::Pop);
            }
            _ =>
            {
                self.op(base);
                self.fill_list(value);
            }
        }
    }

    /// Make a slice in a block and push the address of the block. The source,
    /// the start and the length run first, and the op reads the source
    /// before it writes the block, so the block can be the source itself.
    ///
    /// # Arguments
    ///
    /// * `dest` - The op that pushes the address of the block.
    /// * `e` - The [`ExprNode::MakeSlice`] expression.
    fn make_slice(&mut self, dest: Op, e: &Expr)
    {
        let ExprNode::MakeSlice { source, start, len, from, checked } = &e.node
        else
        {
            panic!("make_slice takes a slice expression");
        };
        self.op(dest);
        self.expr(source);
        self.expr(start);
        self.expr(len);
        let cell = e.kind.as_slice().expect("a slice has cells").cell;
        let stride = cell.size(self.shared.platform);
        let from = match from
        {
            SliceFrom::List(_) if *checked => SliceSource::Checked,
            SliceFrom::List(n) => SliceSource::List(*n),
            SliceFrom::Slice => SliceSource::Slice,
            SliceFrom::Block => SliceSource::Block,
        };
        self.op(Op::MakeSlice { stride, from });
        // A range of text keeps whole characters; the whole text keeps them
        // too.
        do_if!(e.kind.is_text_view() && !checked, self.op(Op::TextEdges));
    }

    /// Write text in a slot and push the address of the slot: static text,
    /// or text built from its pieces. A piece that is a fresh text of its
    /// own is dropped once its text is written.
    ///
    /// # Arguments
    ///
    /// * `dest` - The op that pushes the address of the slot.
    /// * `e` - The [`ExprNode::Text`] or [`ExprNode::Format`] expression.
    fn text_at(&mut self, dest: Op, e: &Expr)
    {
        self.op(dest);
        ret_if!(let ExprNode::Text(text) = &e.node, {
            let t = self.shared.text(text);
            self.op(Op::Text(t));
        });
        let ExprNode::Format(pieces) = &e.node
        else
        {
            panic!("text_at takes text");
        };
        let mut args = Vec::with_capacity(pieces.len());
        let mut fresh = Vec::new();
        for piece in pieces
        {
            args.push(match piece
            {
                FmtPiece::Text(text) => FmtArg::Text(self.shared.text(text)),
                FmtPiece::Value(v) =>
                {
                    fresh.extend(self.read_operand(v));
                    FmtArg::Value(v.kind)
                }
            });
        }
        let f = self.shared.format(args);
        self.op(Op::Format(f));
        do_for!((slot, kind) in fresh, self.drop_local(slot, kind));
    }

    /// Push a value that is read and not kept: a value that holds text and
    /// is made for the read alone, as a format, a call or a list literal
    /// makes it, goes to a slot the caller drops after the read.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    ///
    /// # Returns
    ///
    /// The slot to drop and its kind, for a fresh value that holds text.
    fn read_operand(&mut self, e: &Expr) -> Option<(u32, DataKind)>
    {
        let name = match &e.node
        {
            ExprNode::Format(_) => "%text",
            ExprNode::Call { .. } => "%ret",
            ExprNode::List(_) => "%list",
            _ => "",
        };
        ret_if!(name.is_empty() || !e.kind.holds_text(), self.expr(e), None);
        let slot = self.temp(name, e.kind);
        let at = Op::LocalAddr(slot);
        match &e.node
        {
            ExprNode::Call { func, args, .. } => self.call_into(Some(at), *func, args),
            ExprNode::List(cells) => self.build_list(at, e.kind, Some(cells)),
            _ => self.text_at(at, e),
        }
        Some((slot, e.kind))
    }

    /// Drop the value in a local slot.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot.
    /// * `kind` - Its kind, which holds text.
    fn drop_local(&mut self, slot: u32, kind: DataKind)
    {
        self.op(Op::LocalAddr(slot));
        self.op(Op::Drop(kind));
    }

    /// Push the value of a slot, read from its block.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    fn load(&mut self, place: Place)
    {
        let kind = self.slot_at(place).kind;
        self.addr_of(place);
        do_if!(!kind.is_aggregate(), self.op(Op::Load(kind)));
    }

    /// Set the value of a slot, written to its block.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `value` - Pushes the value.
    fn store(&mut self, place: Place, value: impl FnOnce(&mut Self))
    {
        let kind = self.slot_at(place).kind;
        self.addr_of(place);
        value(self);
        self.op(Op::Store(kind));
    }

    /// Fill the list or struct at the address on the stack with a value of
    /// its kind: zero octets for a blank one, else a duplicate of the value.
    ///
    /// # Arguments
    ///
    /// * `value` - The list value.
    fn fill_list(&mut self, value: &Expr)
    {
        ret_if!(value.node == ExprNode::BlankList, self.op(Op::Zero(value.kind)));
        self.expr(value);
        self.op(Op::Blit(value.kind));
    }

    /// Write a value at the address on the stack: a list or a struct is
    /// copied, each other value stored.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    fn put(&mut self, value: &Expr)
    {
        ret_if!(value.kind.is_aggregate(), self.fill_list(value));
        self.expr(value);
        self.op(Op::Store(value.kind));
    }

    /// Build a list or struct value in a slot of its own and push its address.
    ///
    /// # Arguments
    ///
    /// * `kind` - The list kind.
    /// * `cells` - The cells; none for a blank list.
    fn spill_list(&mut self, kind: DataKind, cells: Option<&[Expr]>)
    {
        let slot = self.temp("%list", kind);
        self.build_list(Op::LocalAddr(slot), kind, cells);
    }

    /// Build a list or struct value at an address and push the address.
    ///
    /// # Arguments
    ///
    /// * `base` - The op that pushes the address.
    /// * `kind` - The list kind.
    /// * `cells` - The cells; none for a blank list.
    fn build_list(&mut self, base: Op, kind: DataKind, cells: Option<&[Expr]>)
    {
        self.fill_at(base, kind, cells);
        self.op(base);
    }

    /// Build a list or struct value at an address.
    ///
    /// # Arguments
    ///
    /// * `base` - The op that pushes the address.
    /// * `kind` - The list kind.
    /// * `cells` - The cells; none for a blank list.
    fn fill_at(&mut self, base: Op, kind: DataKind, cells: Option<&[Expr]>)
    {
        self.op(base);
        let Some(cells) = cells
        else
        {
            return self.op(Op::Zero(kind));
        };
        let platform = self.shared.platform;
        let stride = kind.as_list().map_or(0, |l| l.cell.size(platform));
        // A cell steps `i` strides; a field steps its offset in octets.
        let step = |i: usize| {
            kind.as_struct().map_or((i as u32, stride), |s| (s.offset(i, platform), 1))
        };
        for (i, cell) in cells.iter().enumerate()
        {
            do_if!(i > 0, {
                self.op(base);
                let (count, scale) = step(i);
                self.op(Op::Push(usize_value(u128::from(count), platform)));
                self.op(Op::PtrAdd(scale));
            });
            self.put(cell);
        }
    }

    /// Push the address of a list cell. A cell number not proved in range is
    /// checked first.
    ///
    /// # Arguments
    ///
    /// * `e` - The address expression, a pointer to the cell kind.
    /// * `list` - The list, or a pointer to it.
    /// * `index` - The cell number.
    /// * `bounds` - The number of cells, and of the first cell.
    /// * `checked` - Whether the number is known to be in range.
    fn cell_addr(
        &mut self, e: &Expr, list: &Expr, index: &Expr, bounds: (u32, u8),
        checked: bool,
    )
    {
        let (len, first) = bounds;
        self.expr(list);
        self.expr(index);
        do_if!(!checked || first != 0, self.op(Op::Index { len, first }));
        self.op(Op::PtrAdd(self.stride(e.kind)));
    }

    /// Set the address a heap statement keeps.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the address goes.
    /// * `value` - Pushes the address.
    fn store_dest(&mut self, dest: &Dest, value: impl FnOnce(&mut Self))
    {
        match dest
        {
            Dest::Slot(place) => self.store(*place, value),
            Dest::Mem(at) => self.store_mem(at, value),
        }
    }

    /// Replace the address a heap statement keeps with one worked out from
    /// it.
    ///
    /// # Arguments
    ///
    /// * `dest` - Where the address lives.
    /// * `then` - Turns the address on the stack into the new one.
    fn update_dest(&mut self, dest: &Dest, then: impl FnOnce(&mut Self))
    {
        match dest
        {
            Dest::Slot(place) => self.update_slot(*place, then),
            Dest::Mem(at) => self.update_mem(at, then),
        }
    }

    /// Replace the value of a slot with one worked out from it.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `then` - Turns the value on the stack into the new one.
    fn update_slot(&mut self, place: Place, then: impl FnOnce(&mut Self))
    {
        self.store(place, |g| {
            g.load(place);
            then(g);
        });
    }

    /// Replace the pointer at an address with one worked out from it. The
    /// address is worked out once.
    ///
    /// # Arguments
    ///
    /// * `at` - The address, of a pointer to a pointer.
    /// * `then` - Turns the pointer on the stack into the new one.
    fn update_mem(&mut self, at: &Expr, then: impl FnOnce(&mut Self))
    {
        let kind = held_kind(at);
        self.store_mem(at, |g| {
            g.op(Op::Dup);
            g.op(Op::Load(kind));
            then(g);
        });
    }

    /// Write a pointer at an address.
    ///
    /// # Arguments
    ///
    /// * `at` - The address, of a pointer to a pointer.
    /// * `value` - Pushes the pointer.
    fn store_mem(&mut self, at: &Expr, value: impl FnOnce(&mut Self))
    {
        self.expr(at);
        value(self);
        self.op(Op::Store(held_kind(at)));
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
        // The scratch values of the statements before this one are dead.
        let done = std::mem::take(&mut self.taken);
        self.free.extend(done);
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
            Stmt::Set { place, value, .. } if value.kind.is_aggregate() =>
            {
                self.set_block(*place, value)
            }
            Stmt::Set { place, value, .. } => self.store(*place, |g| g.expr(value)),
            Stmt::Print { value: PrintValue::Expr(e), .. } =>
            {
                let fresh = self.read_operand(e);
                self.op(tern!(e.kind.is_aggregate(), Op::PrintList(e.kind), Op::Print));
                do_if!(let Some((slot, kind)) = fresh, self.drop_local(slot, kind));
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
                do_if!(let Some(e) = value, self.result(e));
                self.op(Op::Ret);
            }
            Stmt::Call { func, args, .. } =>
            {
                let ret = self.shared.rets[*func as usize];
                let held = ret.filter(|k| k.holds_text());
                let slot = held.map(|kind| (self.temp("%ret", kind), kind));
                let into = slot.map(|(n, _)| Op::LocalAddr(n));
                tern!(
                    slot.is_some(),
                    self.call_into(into, *func, args),
                    self.call(*func, args)
                );
                do_if!(ret.is_some(), self.op(Op::Pop));
                do_if!(let Some((n, kind)) = slot, self.drop_local(n, kind));
            }
            Stmt::Store { addr, value, .. } =>
            {
                self.expr(addr);
                self.put(value);
            }
            Stmt::Alloc { dest, count, unit, .. } =>
            {
                let stride = self.unit_size(*unit);
                self.store_dest(dest, |g| g.count(count, Op::Alloc(stride)));
            }
            Stmt::Resize { dest, count, unit, .. } =>
            {
                let stride = self.unit_size(*unit);
                self.update_dest(dest, |g| g.count(count, Op::Realloc(stride)));
            }
            Stmt::Free { dest, .. } => self.update_dest(dest, |g| g.free()),
            Stmt::Drop { dest, kind, .. } =>
            {
                match dest
                {
                    Dest::Slot(place) => self.addr_of(*place),
                    Dest::Mem(at) => self.expr(at),
                }
                self.op(Op::Drop(*kind));
            }
        }
    }

    /// Push an element count, then add a heap op that takes it.
    ///
    /// # Arguments
    ///
    /// * `count` - The count.
    /// * `op` - The heap op.
    fn count(&mut self, count: &Expr, op: Op)
    {
        self.expr(count);
        self.op(op);
    }

    /// Release the heap block whose address is on the stack and push null in
    /// its place.
    fn free(&mut self)
    {
        self.op(Op::Free);
        self.op(Op::Push(Value::NULL));
    }

    /// The stride of a heap statement.
    ///
    /// # Arguments
    ///
    /// * `unit` - The element kind; none for a count in octets.
    ///
    /// # Returns
    ///
    /// The element size, or 1.
    fn unit_size(&self, unit: Option<DataKind>) -> u32
    {
        unit.map_or(1, |k| k.size(self.shared.platform))
    }

    /// The element size of a pointer kind.
    ///
    /// # Arguments
    ///
    /// * `ptr` - The pointer kind, not the opaque one.
    ///
    /// # Returns
    ///
    /// The size of the pointee.
    fn stride(&self, ptr: DataKind) -> u32
    {
        extract!(Some(pointee) = ptr.pointee(), "pointer math needs a pointee");
        pointee.size(self.shared.platform)
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
        self.store(var, |g| g.expr(from));
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
        self.store(Place::Local(n), |g| g.expr(e));
        Operand::Local(n)
    }

    /// Push a counted-loop operand.
    ///
    /// # Arguments
    ///
    /// * `operand` - The operand.
    fn push(&mut self, operand: Operand)
    {
        match operand
        {
            Operand::Const(v) => self.op(Op::Push(v)),
            Operand::Local(n) => self.load(Place::Local(n)),
        }
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
    /// left is worked out in the unsigned kind with wrapping ops, which give
    /// the true distance even for bounds of each sign, so a step never
    /// moves the variable past the last value.
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
        self.op(Op::Wrap(InfixOp::Sub));
        self.push_unsigned(c.step, c);
        self.op(Op::Compare(CmpOp::Lt));
        self.code.jump_if_true(exit);
        self.update_slot(c.var, |g| {
            g.push(c.step);
            g.op(Op::Wrap(tern!(up, InfixOp::Add, InfixOp::Sub)));
        });
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
        let dir = Place::Local(self.temp("%dir", DataKind::Bool));
        self.store(dir, |g| {
            g.load(c.var);
            g.push(c.to);
            g.op(Op::Compare(CmpOp::Le));
        });
        let (head, next, exit) =
            (self.code.label(), self.code.label(), self.code.label());
        let (down_test, down_next) = (self.code.label(), self.code.label());
        self.load(dir);
        self.code.jump_if_false(down_test);
        self.skip_if_past(c, true, exit);
        self.code.jump(head);
        self.code.bind(down_test);
        self.skip_if_past(c, false, exit);
        self.code.bind(head);
        self.in_loop(LoopLabels { next, exit }, block);
        self.code.bind(next);
        self.load(dir);
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
            ExprNode::Compare { op, lhs, rhs } if lhs.kind.is_list() =>
            {
                self.expr(lhs);
                self.expr(rhs);
                self.op(Op::ListEq(lhs.kind));
                do_if!(*op == CmpOp::Ne, self.op(Op::Prefix(PrefixOp::Not)));
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
                // A view of text widens to a slice of u8s with the same two
                // words.
                let same = operand.kind == e.kind || e.kind.is_aggregate();
                do_if!(!same, self.op(Op::Convert(e.kind)));
                let checked = e.kind.is_text_view() && !operand.kind.is_text_view();
                do_if!(checked, self.op(Op::CheckUtf8));
            }
            ExprNode::Call { func, args, .. } => self.call(*func, args),
            ExprNode::AddrOf(place) => self.addr_of(*place),
            ExprNode::Load { addr } =>
            {
                self.expr(addr);
                do_if!(!e.kind.is_aggregate(), self.op(Op::Load(e.kind)));
            }
            ExprNode::Take { addr } if e.kind.is_aggregate() =>
            {
                self.take_list(e.kind, addr)
            }
            ExprNode::Take { addr } =>
            {
                self.expr(addr);
                self.op(Op::Take(e.kind));
            }
            ExprNode::Lend { operand } => self.expr(operand),
            ExprNode::PtrAdd { base, offset } =>
            {
                self.expr(base);
                self.expr(offset);
                self.op(Op::PtrAdd(self.stride(base.kind)));
            }
            ExprNode::PtrSub { lhs, rhs } =>
            {
                self.expr(lhs);
                self.expr(rhs);
                self.op(Op::PtrSub(self.stride(lhs.kind)));
            }
            ExprNode::SizeOf(kind) => self.size_of(*kind),
            ExprNode::List(cells) => self.spill_list(e.kind, Some(cells)),
            ExprNode::BlankList => self.spill_list(e.kind, None),
            ExprNode::CellAddr { list, index, len: None, first: 0, checked: true } =>
            {
                // A cell proved in range steps from the first cell of the
                // slice, which its first word holds, with no check.
                self.expr(list);
                self.op(Op::Load(e.kind));
                self.expr(index);
                self.op(Op::PtrAdd(self.stride(e.kind)));
            }
            ExprNode::CellAddr { list, index, len: None, first, .. } =>
            {
                self.expr(list);
                self.expr(index);
                self.op(Op::SliceCell { stride: self.stride(e.kind), first: *first });
            }
            ExprNode::CellAddr { list, index, len: Some(len), first, checked } =>
            {
                self.cell_addr(e, list, index, (*len, *first), *checked)
            }
            ExprNode::MakeSlice { .. } =>
            {
                let slot = self.temp("%slice", e.kind);
                self.make_slice(Op::LocalAddr(slot), e)
            }
            ExprNode::Text(_) | ExprNode::Format(_) =>
            {
                let slot = self.temp("%text", e.kind);
                self.text_at(Op::LocalAddr(slot), e)
            }
            ExprNode::Chars { text, index, first } =>
            {
                let fresh = self.read_operand(text);
                do_if!(let Some(i) = index, self.expr(i));
                self.op(tern!(
                    index.is_some(),
                    Op::CharAt { first: *first },
                    Op::CharCount
                ));
                do_if!(let Some((slot, kind)) = fresh, self.drop_local(slot, kind));
            }
            ExprNode::FieldAddr { base, offset, .. } =>
            {
                self.expr(base);
                self.op(Op::Push(usize_value(
                    u128::from(*offset),
                    self.shared.platform,
                )));
                self.op(Op::PtrAdd(1));
            }
        }
    }

    /// Move a list or a struct out of the place at an address: its octets go
    /// to a slot of their own, the place is zeroed, which leaves `ဗလာ` in each
    /// owner cell or field, and the address of the new slot is pushed. The
    /// address is worked out once and kept in a slot of its own, so the
    /// source is read once.
    ///
    /// # Arguments
    ///
    /// * `kind` - The list or struct kind.
    /// * `addr` - The address of the place.
    fn take_list(&mut self, kind: DataKind, addr: &Expr)
    {
        let at = Place::Local(self.temp("%at", addr.kind));
        self.store(at, |g| g.expr(addr));
        let slot = self.temp("%list", kind);
        self.op(Op::LocalAddr(slot));
        self.load(at);
        self.op(Op::Blit(kind));
        self.load(at);
        self.op(Op::Zero(kind));
        self.op(Op::LocalAddr(slot));
    }

    /// Push the size of a kind in octets, a `usize`.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    fn size_of(&mut self, kind: DataKind)
    {
        let platform = self.shared.platform;
        let size = u128::from(kind.size(platform));
        self.op(Op::Push(Value::int(int_kind(DataKind::USIZE), size, platform)));
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

    /// Generate a call: the arguments in order, then the call. A list or
    /// struct result gets a slot of its own in the caller.
    ///
    /// # Arguments
    ///
    /// * `func` - The function index.
    /// * `args` - The arguments.
    fn call(&mut self, func: u32, args: &[Expr])
    {
        let ret = self.shared.rets[func as usize].filter(|k| k.is_aggregate());
        let slot = ret.map(|kind| Op::LocalAddr(self.temp("%ret", kind)));
        self.call_into(slot, func, args);
    }

    /// Generate a call whose list or struct result goes to a slot.
    ///
    /// # Arguments
    ///
    /// * `slot` - The op that pushes the address of the slot, for a list or
    ///   struct result.
    /// * `func` - The function index.
    /// * `args` - The arguments.
    fn call_into(&mut self, slot: Option<Op>, func: u32, args: &[Expr])
    {
        do_if!(let Some(op) = slot, self.op(op));
        do_for!(a in args, self.expr(a));
        self.op(Op::Call(func));
    }

    /// Push the value a function gives back. A list or struct local is not
    /// moved out first, since its frame ends with the return. In place, a
    /// literal or the result of a call is built in the caller's slot.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    fn result(&mut self, e: &Expr)
    {
        ret_if!(!e.kind.is_aggregate(), self.expr(e));
        ret_if!(let Some(n) = local_given(e), self.addr_of(Place::Local(n)));
        let in_place = self.shared.results == ResultMode::InPlace;
        match &e.node
        {
            ExprNode::List(cells) if in_place =>
            {
                self.build_list(Op::ResultAddr, e.kind, Some(cells))
            }
            ExprNode::BlankList if in_place =>
            {
                self.build_list(Op::ResultAddr, e.kind, None)
            }
            ExprNode::Call { func, args, .. } if in_place =>
            {
                self.call_into(Some(Op::ResultAddr), *func, args)
            }
            _ => self.expr(e),
        }
    }
}

/// The local a value reads or moves out of.
///
/// # Arguments
///
/// * `e` - The value.
///
/// # Returns
///
/// The local slot, or `None` for each other value.
fn local_given(e: &Expr) -> Option<u32>
{
    match &e.node
    {
        ExprNode::Read(Place::Local(n)) => Some(*n),
        ExprNode::Take { addr } =>
        {
            tern!(let ExprNode::AddrOf(Place::Local(n)) = addr.node, Some(n), None)
        }
        _ => None,
    }
}

/// The local that each return of a function with a list or struct result
/// gives back, so it can live in the caller's slot. A parameter does not
/// count: the caller fills it.
///
/// # Arguments
///
/// * `f` - The function.
///
/// # Returns
///
/// The local slot, or `None`.
fn given_back(f: &ir::Func) -> Option<u32>
{
    ret_if!(!f.ret.is_some_and(DataKind::is_aggregate), None);
    let mut given = Vec::new();
    each_stmt(&f.block, &mut |s| {
        do_if!(let Stmt::Return { value, .. } = s, given.push(value.as_ref().and_then(local_given)));
    });
    let first = given.first().copied().flatten()?;
    let same = given.iter().all(|g| *g == Some(first));
    (same && first >= f.params).then_some(first)
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

/// The kind of the pointer kept at an address.
///
/// # Arguments
///
/// * `at` - The address, of a pointer to a pointer.
///
/// # Returns
///
/// The pointer kind.
fn held_kind(at: &Expr) -> DataKind
{
    extract!(Some(kind) = at.kind.pointee(), "a heap destination holds a pointer");
    kind
}

/// Whether a value reads nothing that a write to a slot could change, so the
/// slot can be built in place while the value runs: constants, sizes, blank
/// values, reads of and moves out of other slots, and operators and literals
/// over these. A load, a call or a cell can reach the slot through an
/// address, so each counts as reading it.
///
/// # Arguments
///
/// * `e` - The value.
/// * `place` - The slot.
///
/// # Returns
///
/// `true` when the value stands apart from the slot.
fn apart_from(e: &Expr, place: Place) -> bool
{
    match &e.node
    {
        ExprNode::Const(_) | ExprNode::SizeOf(_) | ExprNode::BlankList => true,
        ExprNode::Read(p) => *p != place,
        ExprNode::Take { addr } =>
        {
            matches!(addr.node, ExprNode::AddrOf(p) if p != place)
        }
        ExprNode::Prefix { operand, .. } | ExprNode::Convert { operand, .. } =>
        {
            apart_from(operand, place)
        }
        ExprNode::Infix { lhs, rhs, .. }
        | ExprNode::Compare { lhs, rhs, .. }
        | ExprNode::Logic { lhs, rhs, .. } =>
        {
            apart_from(lhs, place) && apart_from(rhs, place)
        }
        ExprNode::List(cells) => cells.iter().all(|c| apart_from(c, place)),
        _ => false,
    }
}

/// A `usize` constant.
///
/// # Arguments
///
/// * `n` - The number.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The value.
fn usize_value(n: u128, platform: u16) -> Value
{
    Value::int(int_kind(DataKind::USIZE), n, platform)
}

/// The integer kind of an integer data kind.
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
    extract!(Some(int) = kind.int_kind(), "the kind is an integer kind");
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
