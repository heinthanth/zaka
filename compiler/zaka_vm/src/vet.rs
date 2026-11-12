//! The check a loaded artifact passes before it runs. Ops the compiler has just
//! made are sound; an artifact read from a file can be cut or edited, and
//! must not panic the VM, hand C a wrong value or claim unbounded storage.
//!
//! The check walks each path through each function and tracks the stack: its
//! height, and a coarse class of each value (an integer, a float, a
//! character, a boolean, an address or a function value) where the ops tell
//! it. Each jump lands on an op; each path ends in `ret`, `halt`, `trap` or a
//! jump; paths that meet agree on the height; no op pops past the bottom; each
//! op that reads an address, a boolean or a value of a kind finds a value of
//! that class when the class is known; and `ret` leaves the values the result
//! needs. Each index names a slot, function, text, static list, format or C
//! call that exists, and each kind fits the storage limits.

use std::fmt;

use bon::Builder;
use zakac_codegen_target_vm::bc::FmtArg;
use zakac_codegen_target_vm::bc::Func;
use zakac_codegen_target_vm::bc::Module;
use zakac_codegen_target_vm::bc::Op;
use zakac_i18n::t;
use zakac_ir_builder::value::Value;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

/// The platform widths the compiler builds for.
const PLATFORM_WIDTHS: &[u16] = &[8, 16, 32, 64];

/// The reason a loaded artifact cannot run: where the fault lies and what it
/// is.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Malformed
{
    /// The name of the function, or `None` for a fault of the module.
    pub func: Option<String>,
    /// The index of the op, or `None` for a fault of the function itself.
    pub pc: Option<u32>,
    /// What is wrong.
    #[builder(into)]
    pub reason: String,
}

impl fmt::Display for Malformed
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let reason = &self.reason;
        let text = match (&self.func, self.pc)
        {
            (Some(func), Some(pc)) =>
            {
                t!("vm.vet.op", func = func, pc = pc, reason = reason)
            }
            (Some(func), None) => t!("vm.vet.func", func = func, reason = reason),
            _ => t!("vm.vet.module", reason = reason),
        };
        f.write_str(&text)
    }
}

/// Check a loaded module before it runs.
///
/// # Arguments
///
/// * `module` - The module.
/// * `limit` - The most octets a value of one kind, or the globals together,
///   can take: the larger of the heap and stack limits.
///
/// # Returns
///
/// `Ok`, or the first fault found.
pub fn vet_module(module: &Module, limit: u64) -> Result<(), Malformed>
{
    let at_module = |reason: String| Malformed::builder().reason(reason).build();
    let vet = Vet { module, limit };
    vet.whole().map_err(at_module)?;
    module.funcs.iter().try_for_each(|func| vet.func(func))
}

/// The coarse class of a value on the stack.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Class
{
    /// A value the ops do not tell.
    Unknown,
    /// An integer or a member of an enumeration.
    Int,
    /// A float.
    Float,
    /// A character.
    Char,
    /// A boolean.
    Bool,
    /// An address.
    Addr,
    /// A function value with this result kind, which tells what
    /// `call_value` leaves.
    Fn(Option<DataKind>),
}

/// What the result of a call leaves on the stack of the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gives
{
    /// Nothing: the function has no result.
    Nothing,
    /// One value.
    Value,
    /// Nothing more: the caller's slot under the arguments holds the list or
    /// struct, and its address remains.
    Slot,
}

/// The result shape of a result kind.
///
/// # Arguments
///
/// * `ret` - The result kind.
///
/// # Returns
///
/// What a call leaves.
fn gives(ret: Option<DataKind>) -> Gives
{
    match ret
    {
        None => Gives::Nothing,
        Some(k) if k.is_aggregate() => Gives::Slot,
        Some(_) => Gives::Value,
    }
}

/// The class of a value of a kind on the stack, as a read from storage
/// gives it.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The class; `Unknown` for a list, a struct or text, which no read gives.
fn class_of(kind: DataKind) -> Class
{
    match kind.repr()
    {
        DataKind::Int(_) => Class::Int,
        DataKind::Float(_) => Class::Float,
        DataKind::Char => Class::Char,
        DataKind::Bool => Class::Bool,
        DataKind::Ptr(_) | DataKind::Null => Class::Addr,
        DataKind::Fn(f) => Class::Fn(f.ret.map(DataKind::repr)),
        _ => Class::Unknown,
    }
}

/// The class of a value passed for a parameter or a piece of a kind: the
/// address of the value for a list, a struct or text.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The class.
fn arg_class(kind: DataKind) -> Class
{
    tern!(kind.is_aggregate(), Class::Addr, class_of(kind))
}

/// The class of a constant.
///
/// # Arguments
///
/// * `v` - The constant.
///
/// # Returns
///
/// The class.
fn class_of_value(v: Value) -> Class
{
    match v
    {
        Value::Int { .. } => Class::Int,
        Value::Float { .. } => Class::Float,
        Value::Char(_) => Class::Char,
        Value::Bool(_) => Class::Bool,
        Value::Ptr(_) => Class::Addr,
    }
}

/// The class of the result of an operator on a left operand of a class: an
/// integer, float or boolean operator gives the class of its operands.
///
/// # Arguments
///
/// * `left` - The class of the left operand.
///
/// # Returns
///
/// The class.
fn arith(left: Class) -> Class
{
    let same = matches!(left, Class::Int | Class::Float | Class::Bool);
    tern!(same, left, Class::Unknown)
}

/// Whether a value of one class can stand where an op reads another. An
/// unknown class fits each; an address and a function value are both one
/// word.
///
/// # Arguments
///
/// * `have` - The class of the value.
/// * `want` - The class the op reads.
///
/// # Returns
///
/// `true` when the value fits.
fn fits(have: Class, want: Class) -> bool
{
    let word = |c: Class| matches!(c, Class::Addr | Class::Fn(_));
    ret_true!(have == Class::Unknown || want == Class::Unknown);
    ret_true!(word(have) && word(want));
    std::mem::discriminant(&have) == std::mem::discriminant(&want)
}

/// The class two paths agree on where the paths meet: the first, when both
/// are function values whose calls leave the same, or the class both hold,
/// else unknown.
///
/// # Arguments
///
/// * `a` - The class on one path.
/// * `b` - The class on the other.
///
/// # Returns
///
/// The class.
fn meet(a: Class, b: Class) -> Class
{
    match (a, b)
    {
        (Class::Fn(l), Class::Fn(r)) if gives(l) == gives(r) => a,
        _ if a == b => a,
        _ => Class::Unknown,
    }
}

/// Where control goes after an op.
enum Flow
{
    /// To the next op.
    Next,
    /// To this op.
    Goto(u32),
    /// To this op or the next.
    Branch(u32),
    /// Nowhere: the call or the run ends.
    End,
}

/// The flow of an op.
///
/// # Arguments
///
/// * `op` - The op.
///
/// # Returns
///
/// Where control goes.
fn flow(op: Op) -> Flow
{
    match op
    {
        Op::Jump(t) => Flow::Goto(t),
        Op::JumpIfFalse(t) | Op::JumpIfTrue(t) => Flow::Branch(t),
        Op::Ret | Op::Halt { .. } | Op::Trap(_) => Flow::End,
        _ => Flow::Next,
    }
}

/// What an op does to the stack.
#[derive(Builder)]
struct Effect
{
    /// The count of values it pops.
    #[builder(default)]
    pops: usize,
    /// The class of each value it reads, the top first: the values it pops,
    /// and those it reads and leaves, such as the slot under the arguments
    /// of a call.
    #[builder(default)]
    takes: Vec<Class>,
    /// The class of the value it pushes, if it pushes one.
    push: Option<Class>,
}

/// An effect that pops values of these classes, the top first, and pushes
/// one value or none.
///
/// # Arguments
///
/// * `takes` - The class of each value popped.
/// * `push` - The class of the value pushed.
///
/// # Returns
///
/// The effect.
fn effect(takes: &[Class], push: Option<Class>) -> Effect
{
    #[rustfmt::skip]
    let effect = Effect::builder().pops(takes.len()).takes(takes.to_vec())
        .maybe_push(push).build();
    effect
}

/// Shorthands for the classes of [`effect`]: a value of unknown class.
const U: Class = Class::Unknown;
/// An address.
const A: Class = Class::Addr;
/// An integer.
const I: Class = Class::Int;

/// The module under check and the size limit.
struct Vet<'m>
{
    /// The module.
    module: &'m Module,
    /// The most octets one kind can take.
    limit: u64,
}

/// The fault of an op, as a reason.
type Fault<T> = Result<T, String>;

impl Vet<'_>
{
    /// Check the parts of the module that sit outside the functions.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the first fault.
    fn whole(&self) -> Fault<()>
    {
        let m = self.module;
        ret_if!(
            !PLATFORM_WIDTHS.contains(&m.platform_bits),
            Err(format!("the VM knows no platform width {}", m.platform_bits))
        );
        let main = m.funcs.get(m.main as usize).ok_or("no main function")?;
        ret_if!(main.params != 0, Err("the main function takes parameters".into()));
        let host = [m.host.args, m.host.env, m.host.exit];
        do_for!(g in host.into_iter().flatten(), self.global(g)?);
        do_for!(s in &m.globals, self.sized(s.kind)?);
        let total = m.globals.iter().map(|s| u64::from(s.kind.size(m.platform_bits)));
        let total = total.fold(0u64, u64::saturating_add);
        ret_if!(total > self.limit, Err(self.too_large(total)));
        do_for!(s in &m.seqs, self.sized(s.kind)?);
        do_for!(f in &m.foreign, self.signature(f.params.iter().copied().chain(f.ret))?);
        do_for!(c in &m.foreign_calls, self.foreign_call(c.func, &c.extra)?);
        m.formats.iter().flatten().try_for_each(|a| self.fmt_arg(*a))
    }

    /// Check the kinds of a signature.
    ///
    /// # Arguments
    ///
    /// * `kinds` - The kinds.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the first fault.
    fn signature(&self, mut kinds: impl Iterator<Item = DataKind>) -> Fault<()>
    {
        kinds.try_for_each(|k| self.sized(k))
    }

    /// Check a call of a C function: its index and the kinds of its other
    /// values.
    ///
    /// # Arguments
    ///
    /// * `func` - The index of the C function.
    /// * `extra` - The kinds of the other values.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the fault.
    fn foreign_call(&self, func: u32, extra: &[DataKind]) -> Fault<()>
    {
        ret_if!(
            func as usize >= self.module.foreign.len(),
            Err(format!("a C call names C function {func}, which does not exist"))
        );
        self.signature(extra.iter().copied())
    }

    /// Check one piece of a format.
    ///
    /// # Arguments
    ///
    /// * `arg` - The piece.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the fault.
    fn fmt_arg(&self, arg: FmtArg) -> Fault<()>
    {
        match arg
        {
            FmtArg::Text(t) => self.text(t),
            FmtArg::Value(kind) => self.sized(kind),
        }
    }

    /// Check one function: its slots, its result kind and each path through
    /// its ops.
    ///
    /// # Arguments
    ///
    /// * `func` - The function.
    ///
    /// # Returns
    ///
    /// `Ok`, or the first fault.
    fn func(&self, func: &Func) -> Result<(), Malformed>
    {
        let name = &func.name;
        let at_func = |reason: String| {
            Malformed::builder().func(name.clone()).reason(reason).build()
        };
        let params_fit = func.params as usize <= func.locals.len();
        ret_if!(!params_fit, Err(at_func("more parameters than slots".into())));
        let kinds = func.locals.iter().map(|s| s.kind).chain(func.ret);
        self.signature(kinds).map_err(at_func)?;
        let at_op = |(pc, reason): (u32, String)| {
            Malformed::builder().func(name.clone()).pc(pc).reason(reason).build()
        };
        let paths =
            Paths { vet: self, func, seen: vec![None; func.code.len()], work: vec![] };
        paths.walk().map_err(at_op)
    }

    /// Check that a kind fits the storage limits.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the fault.
    fn sized(&self, kind: DataKind) -> Fault<()>
    {
        let size = kind.checked_size(self.module.platform_bits);
        let size = size.ok_or_else(|| format!("kind {} has no size", kind.name()))?;
        ret_if!(u64::from(size) > self.limit, Err(self.too_large(u64::from(size))));
        Ok(())
    }

    /// The reason for a size past the limit.
    ///
    /// # Arguments
    ///
    /// * `size` - The size, in octets.
    ///
    /// # Returns
    ///
    /// The reason.
    fn too_large(&self, size: u64) -> String
    {
        format!("{size} octets pass the limit of {} octets", self.limit)
    }

    /// Check a global index.
    ///
    /// # Arguments
    ///
    /// * `n` - The index.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the fault.
    fn global(&self, n: u32) -> Fault<()>
    {
        let fits = (n as usize) < self.module.globals.len();
        tern!(fits, Ok(()), Err(format!("global slot {n} does not exist")))
    }

    /// Check a static text index.
    ///
    /// # Arguments
    ///
    /// * `t` - The index.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the fault.
    fn text(&self, t: u32) -> Fault<()>
    {
        let fits = (t as usize) < self.module.texts.len();
        tern!(fits, Ok(()), Err(format!("static text {t} does not exist")))
    }

    /// Check a static list index.
    ///
    /// # Arguments
    ///
    /// * `s` - The index.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the fault.
    fn seq(&self, s: u32) -> Fault<()>
    {
        let fits = (s as usize) < self.module.seqs.len();
        tern!(fits, Ok(()), Err(format!("static list {s} does not exist")))
    }

    /// The function at an index.
    ///
    /// # Arguments
    ///
    /// * `f` - The index.
    ///
    /// # Returns
    ///
    /// The function, or the reason of the fault.
    fn callee(&self, f: u32) -> Fault<&Func>
    {
        let found = self.module.funcs.get(f as usize);
        found.ok_or_else(|| format!("function {f} does not exist"))
    }
}

/// The walk of each path through the ops of one function.
struct Paths<'v, 'm>
{
    /// The module under check.
    vet: &'v Vet<'m>,
    /// The function.
    func: &'v Func,
    /// The stack on entering each op, once a path reaches it.
    seen: Vec<Option<Vec<Class>>>,
    /// The ops whose stack changed and must be walked again.
    work: Vec<u32>,
}

impl Paths<'_, '_>
{
    /// Walk each path from the first op until no stack changes.
    ///
    /// # Returns
    ///
    /// `Ok`, or the index of the op at fault and the reason.
    fn walk(mut self) -> Result<(), (u32, String)>
    {
        self.reach(0, Vec::new()).map_err(|r| (0, r))?;
        while let Some(pc) = self.work.pop()
        {
            self.step(pc).map_err(|r| (pc, r))?;
        }
        Ok(())
    }

    /// Run one op on the stack it enters with and pass the stack on.
    ///
    /// # Arguments
    ///
    /// * `pc` - The index of the op.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the fault.
    fn step(&mut self, pc: u32) -> Fault<()>
    {
        let op = self.func.code[pc as usize];
        let mut stack = self.seen[pc as usize].clone().unwrap_or_default();
        let effect = self.effect(op, &stack)?;
        let need = effect.pops.max(effect.takes.len());
        let held = stack.len();
        ret_if!(
            held < need,
            Err(format!("{op} needs {need} values, the stack holds {held}"))
        );
        let read = stack.iter().rev().zip(&effect.takes);
        let wrong = read.enumerate().find(|(_, (have, want))| !fits(**have, **want));
        do_if!(let Some((n, (have, want))) = wrong, return Err(misfit(op, n, *have, *want)));
        stack.truncate(held - effect.pops);
        do_if!(let Some(class) = effect.push, stack.push(class));
        match flow(op)
        {
            Flow::Next => self.reach(pc + 1, stack),
            Flow::Goto(t) => self.reach(t, stack),
            Flow::Branch(t) =>
            {
                self.reach(t, stack.clone())?;
                self.reach(pc + 1, stack)
            }
            Flow::End => Ok(()),
        }
    }

    /// Enter an op with a stack: note it on the first visit, check that the
    /// height agrees on a later one, and walk the op again when a class
    /// changes.
    ///
    /// # Arguments
    ///
    /// * `pc` - The index of the op.
    /// * `stack` - The stack.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the fault.
    fn reach(&mut self, pc: u32, stack: Vec<Class>) -> Fault<()>
    {
        let len = self.func.code.len();
        ret_if!(
            pc as usize >= len,
            Err(format!("control runs to op {pc}, past the end of {len} ops"))
        );
        let Some(before) = &self.seen[pc as usize]
        else
        {
            self.seen[pc as usize] = Some(stack);
            self.work.push(pc);
            return Ok(());
        };
        let (was, now) = (before.len(), stack.len());
        ret_if!(
            was != now,
            Err(format!(
                "paths meet at op {pc} with {was} and {now} values on the stack"
            ))
        );
        let met: Vec<Class> =
            before.iter().zip(&stack).map(|(a, b)| meet(*a, *b)).collect();
        ret_if!(met == *before, Ok(()));
        self.seen[pc as usize] = Some(met);
        self.work.push(pc);
        Ok(())
    }

    /// Check a local slot index.
    ///
    /// # Arguments
    ///
    /// * `n` - The index.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason of the fault.
    fn local(&self, n: u32) -> Fault<()>
    {
        let fits = (n as usize) < self.func.locals.len();
        tern!(fits, Ok(()), Err(format!("local slot {n} does not exist")))
    }

    /// Check the kind of an op and give its effect.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `takes` - The class of each value popped, the top first.
    /// * `push` - The class of the value pushed.
    ///
    /// # Returns
    ///
    /// The effect, or the reason of the fault.
    fn kinded(
        &self, kind: DataKind, takes: &[Class], push: Option<Class>,
    ) -> Fault<Effect>
    {
        self.vet.sized(kind)?;
        Ok(effect(takes, push))
    }

    /// What an op does to the stack, once its operands pass their checks.
    /// The executor of the VM is the rule each arm follows.
    ///
    /// # Arguments
    ///
    /// * `op` - The op.
    /// * `stack` - The stack it enters with.
    ///
    /// # Returns
    ///
    /// The effect, or the reason of the fault.
    fn effect(&self, op: Op, stack: &[Class]) -> Fault<Effect>
    {
        let vet = self.vet;
        let top = |n: usize| stack.len().checked_sub(n + 1).map_or(U, |at| stack[at]);
        match op
        {
            Op::Push(v) => Ok(effect(&[], Some(class_of_value(v)))),
            Op::LocalAddr(n) | Op::LocalRef(n) =>
            {
                self.local(n).map(|()| effect(&[], Some(A)))
            }
            Op::GlobalAddr(n) => vet.global(n).map(|()| effect(&[], Some(A))),
            Op::Prefix(_) => Ok(effect(&[U], Some(arith(top(0))))),
            Op::Infix(_) | Op::Wrap(_) => Ok(effect(&[U, U], Some(arith(top(1))))),
            Op::Compare(_) => Ok(effect(&[U, U], Some(Class::Bool))),
            Op::Convert(kind) => self.kinded(kind, &[U], Some(class_of(kind))),
            Op::Jump(_) => Ok(effect(&[], None)),
            Op::JumpIfFalse(_) | Op::JumpIfTrue(_) => Ok(effect(&[Class::Bool], None)),
            Op::Call(f) => self.call(f),
            Op::FnAddr(f) => Ok(effect(&[], Some(Class::Fn(vet.callee(f)?.ret)))),
            Op::CallValue(n) => self.call_value(n as usize, stack),
            Op::Closure { func, record } =>
            {
                let ret = vet.callee(func)?.ret;
                self.kinded(record, &[A], Some(Class::Fn(ret)))
            }
            Op::Host(_) => self.host(),
            Op::Foreign(i) => self.foreign(i),
            Op::Ret => self.ret(stack.len()),
            Op::Print | Op::Pop | Op::PrintEnum(_) => Ok(effect(&[U], None)),
            Op::Free | Op::CRevoke | Op::Pin => Ok(effect(&[A], None)),
            Op::PrintText(t) => vet.text(t).map(|()| effect(&[], None)),
            Op::Trap(_) => Ok(effect(&[], None)),
            Op::Load(kind) => self.kinded(kind, &[A], Some(class_of(kind))),
            Op::Store(kind) => self.kinded(kind, &[class_of(kind), A], None),
            Op::Take(kind) if kind.is_aggregate() =>
            {
                Err(format!("take of {}, a list or struct", kind.name()))
            }
            Op::Take(kind) => self.kinded(kind, &[A], Some(class_of(kind))),
            Op::PtrAdd(_) | Op::Realloc(_) => Ok(effect(&[I, A], Some(A))),
            Op::PtrSub(_) => Ok(effect(&[A, A], Some(I))),
            Op::Alloc(_) => Ok(effect(&[I], Some(A))),
            Op::CLeave(kind) => self.kinded(kind, &[A], None),
            Op::CLoad { kind, .. } =>
            {
                let takes = tern!(kind.is_aggregate(), &[A, A][..], &[A][..]);
                self.kinded(kind, takes, Some(arg_class(kind)))
            }
            Op::CStore { kind, .. } => self.kinded(kind, &[U, A], None),
            Op::MemBlit(_) | Op::MemMove(_) => Ok(effect(&[A, I, A], None)),
            Op::MemFill(kind) => self.kinded(kind, &[I, class_of(kind), A], None),
            Op::Dup =>
            {
                let dup =
                    Effect::builder().takes(vec![U]).maybe_push(stack.last().copied());
                Ok(dup.build())
            }
            Op::Index { .. } => Ok(effect(&[U], Some(I))),
            Op::Blit(kind) | Op::ToHeap(kind) | Op::AddCell(kind) =>
            {
                self.kinded(kind, &[A, A], None)
            }
            Op::AddText(kind) => self.kinded(kind, &[arg_class(kind), A], None),
            Op::Zero(kind)
            | Op::PrintList(kind)
            | Op::Drop(kind)
            | Op::DupSeqs(kind)
            | Op::DeepSeqs(kind)
            | Op::LendSeqs(kind)
            | Op::Own(kind)
            | Op::RemoveCell(kind) => self.kinded(kind, &[A], None),
            Op::ListEq(kind) | Op::SeqEq(kind) =>
            {
                self.kinded(kind, &[A, A], Some(Class::Bool))
            }
            Op::RowFind(kind) => self.kinded(kind, &[A, A], Some(I)),
            Op::SetRow(kind) => self.kinded(kind, &[A, A, A], None),
            Op::RemoveRow(kind) => self.kinded(kind, &[A, A], None),
            Op::Halt { kind: None, .. } => Ok(effect(&[], None)),
            Op::Halt { kind: Some(kind), .. } =>
            {
                self.kinded(kind, &[arg_class(kind)], None)
            }
            Op::ResultAddr if gives(self.func.ret) != Gives::Slot =>
            {
                Err("result_addr needs a list or struct result".into())
            }
            Op::ResultAddr => Ok(effect(&[], Some(A))),
            Op::MakeSlice { .. } => Ok(effect(&[I, I, A, A], Some(A))),
            Op::SliceCell { .. } | Op::ObjectCell { .. } =>
            {
                Ok(effect(&[U, A], Some(A)))
            }
            Op::Text(t) => vet.text(t).map(|()| effect(&[A], Some(A))),
            Op::StaticSeq(s) => vet.seq(s).map(|()| effect(&[A], Some(A))),
            Op::SeqAddr(s) => vet.seq(s).map(|()| effect(&[], Some(A))),
            Op::Format(f) => self.format(f),
            Op::SetChar { .. } => Ok(effect(&[Class::Char, U, A], None)),
            Op::InsertCell { kind, .. } => self.kinded(kind, &[A, U, A], None),
            Op::InsertText { kind, .. } =>
            {
                self.kinded(kind, &[arg_class(kind), U, A], None)
            }
            Op::ReplaceText { kind, .. } =>
            {
                self.kinded(kind, &[arg_class(kind), U, U, A], None)
            }
            Op::CharCount => Ok(effect(&[A], Some(I))),
            Op::CharAtOctet | Op::CharAt { .. } =>
            {
                Ok(effect(&[U, A], Some(Class::Char)))
            }
            Op::TextEq => Ok(effect(&[A, A], Some(Class::Bool))),
            Op::TextEdges | Op::CheckUtf8 =>
            {
                Ok(Effect::builder().takes(vec![A]).build())
            }
            Op::ObjToPtr => Ok(effect(&[A], Some(A))),
            Op::PtrToObj => Ok(effect(&[A, A], None)),
        }
    }

    /// The effect of `ret`: the stack holds the result alone, the address of
    /// it for a list or struct, or nothing for a function with no result.
    ///
    /// # Arguments
    ///
    /// * `held` - The height of the stack.
    ///
    /// # Returns
    ///
    /// The effect, or the reason of the fault.
    fn ret(&self, held: usize) -> Fault<Effect>
    {
        let ret = self.func.ret;
        let want = usize::from(ret.is_some());
        ret_if!(
            held != want,
            Err(format!("ret leaves {held} values, the function gives {want}"))
        );
        Ok(effect(&ret.map(arg_class).into_iter().collect::<Vec<_>>(), None))
    }

    /// The effect of `host`: it pops the address of each parameter of the
    /// running function and the address of the result slot under them, and
    /// pushes that address.
    ///
    /// # Returns
    ///
    /// The effect, or the reason of the fault.
    fn host(&self) -> Fault<Effect>
    {
        ret_if!(
            self.func.ret.is_none(),
            Err("host needs a function with a result".into())
        );
        Ok(effect(&vec![A; self.func.params as usize + 1], Some(A)))
    }

    /// The effect of `format`: it pops the value of each value piece, the last
    /// on top, then the address of a text slot, and pushes the address.
    ///
    /// # Arguments
    ///
    /// * `f` - The index of the format.
    ///
    /// # Returns
    ///
    /// The effect, or the reason of the fault.
    fn format(&self, f: u32) -> Fault<Effect>
    {
        let pieces = self.vet.module.formats.get(f as usize);
        let pieces = pieces.ok_or_else(|| format!("format {f} does not exist"))?;
        let values = pieces
            .iter()
            .filter_map(|a| tern!(let FmtArg::Value(k) = a, Some(*k), None));
        let mut takes: Vec<Class> = values.map(arg_class).collect();
        takes.reverse();
        takes.push(A);
        Ok(effect(&takes, Some(A)))
    }

    /// The effect of a call of a function of the module: it pops the
    /// arguments; a list or struct result needs the caller's slot under
    /// them, and another result is pushed.
    ///
    /// # Arguments
    ///
    /// * `f` - The function index.
    ///
    /// # Returns
    ///
    /// The effect, or the reason of the fault.
    fn call(&self, f: u32) -> Fault<Effect>
    {
        let callee = self.vet.callee(f)?;
        let params = callee.locals.iter().take(callee.params as usize);
        let takes: Vec<Class> = params.rev().map(|s| arg_class(s.kind)).collect();
        Ok(call_effect(takes, callee.ret))
    }

    /// The effect of a call through a function value: the value sits under
    /// the arguments, and its kind tells what the call leaves.
    ///
    /// # Arguments
    ///
    /// * `args` - The count of arguments.
    /// * `stack` - The stack the op enters with.
    ///
    /// # Returns
    ///
    /// The effect, or the reason of the fault.
    fn call_value(&self, args: usize, stack: &[Class]) -> Fault<Effect>
    {
        let at = stack.len().checked_sub(args + 1);
        let value = at.map(|at| stack[at]);
        let value = value.ok_or("call_value finds no function value")?;
        let Class::Fn(ret) = value
        else
        {
            return Err("call_value finds no function value of a known kind".into());
        };
        let mut takes = vec![U; args];
        takes.push(value);
        Ok(call_effect(takes, ret))
    }

    /// The effect of a C call: it pops the value of each parameter and other
    /// value, the last on top, and the address of the slot for a struct
    /// result under them; a result is pushed.
    ///
    /// # Arguments
    ///
    /// * `i` - The index of the C call.
    ///
    /// # Returns
    ///
    /// The effect, or the reason of the fault.
    fn foreign(&self, i: u32) -> Fault<Effect>
    {
        let m = self.vet.module;
        let call = m.foreign_calls.get(i as usize);
        let call = call.ok_or_else(|| format!("C call {i} does not exist"))?;
        let def = m.foreign.get(call.func as usize);
        let def =
            def.ok_or_else(|| format!("C function {} does not exist", call.func))?;
        let kinds = def.params.iter().chain(&call.extra);
        let mut takes: Vec<Class> = kinds.rev().map(|k| arg_class(*k)).collect();
        do_if!(def.ret.is_some_and(DataKind::is_aggregate), takes.push(A));
        Ok(effect(&takes, def.ret.map(arg_class)))
    }
}

/// The effect of a call that pops values of these classes and leaves its
/// result.
///
/// # Arguments
///
/// * `takes` - The class of each value popped, the top first: the arguments,
///   and the function value for `call_value`.
/// * `ret` - The result kind.
///
/// # Returns
///
/// The effect.
fn call_effect(mut takes: Vec<Class>, ret: Option<DataKind>) -> Effect
{
    let n = takes.len();
    do_if!(gives(ret) == Gives::Slot, takes.push(A));
    let push = tern!(gives(ret) == Gives::Value, ret.map(class_of), None);
    Effect::builder().pops(n).takes(takes).maybe_push(push).build()
}

/// The reason a value does not suit the op that reads it.
///
/// # Arguments
///
/// * `op` - The op.
/// * `n` - How far the value sits from the top, 0 for the top.
/// * `have` - Its class.
/// * `want` - The class the op reads.
///
/// # Returns
///
/// The reason.
fn misfit(op: Op, n: usize, have: Class, want: Class) -> String
{
    format!("{op} reads {want:?} at depth {n} of the stack, which holds {have:?}")
}
