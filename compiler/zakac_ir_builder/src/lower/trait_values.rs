//! Trait values, `<trait> အရာ`. A trait value is a closure: its record holds
//! the value of the kind that has the trait, and its code is the dispatcher
//! of that kind for the trait, a function of the module made once for each
//! pair of trait value kind and held kind.
//!
//! A call through a trait value fills an argument pack in a hidden slot of
//! the caller, one field for each argument after the first and one for the
//! result, then calls the trait value with the number of the trait function
//! and the address of the pack. The dispatcher picks the function of the
//! held kind through that number, calls it with the held value first and
//! the fields of the pack after it, and writes the result into the pack. A
//! result of the trait's own kind comes back as a new trait value of the
//! same dispatcher.
//!
//! A trait function that changes its first parameter changes a duplicate: the
//! dispatcher duplicates the held value into a local, calls the function
//! with the address of the local, and writes a new trait value that holds
//! the local into the first field of the pack. The caller then puts that new
//! value in the place of the old one, so another trait value that shares the
//! old record keeps the value it holds.

use zakac_semcheck::fn_kind::FnParam;
use zakac_semcheck::trait_value::dispatch_shape;

use super::nested::record_field;
use super::*;

/// The kind of each field of the argument pack of a trait function: the new
/// trait value first when the function changes its first parameter, then
/// each argument after the first (a borrow of its place for a parameter the
/// function can change or fill), then the result.
///
/// # Arguments
///
/// * `value` - The trait value kind.
/// * `params` - The parameters of the trait function, the trait value kind for
///   the trait's kind.
/// * `ret` - Its result kind.
///
/// # Returns
///
/// The kinds.
fn pack_fields(
    value: DataKind, params: &[FnParam], ret: Option<DataKind>,
) -> Vec<DataKind>
{
    let changes = params.first().is_some_and(|p| p.edit);
    let arg = |p: &FnParam| {
        tern!(p.edit || p.fill, p.kind.pointer_to().as_borrowed(), p.kind)
    };
    let args = params.iter().skip(1).map(arg);
    let new = changes.then_some(value);
    new.into_iter().chain(args).chain(ret).collect()
}

/// A value with a new count of each block it holds, for a value that holds
/// text; the value itself for each other kind.
///
/// # Arguments
///
/// * `e` - The value.
///
/// # Returns
///
/// The value.
fn dup_held(e: Expr) -> Expr { tern!(e.kind.holds_text(), dup(e), e) }

/// The address of one field of a struct value.
///
/// # Arguments
///
/// * `base` - The struct value, read where it sits.
/// * `field` - The field number.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The address, a pointer to the field kind.
fn field_addr(base: &Expr, field: usize, platform: u16) -> Expr
{
    let shape = base.kind.as_struct().expect("a field address reads a struct");
    let kind = shape.fields()[field].1;
    let offset = shape.offset(field, platform);
    let node = ExprNode::FieldAddr {
        base: Box::new(base.clone()),
        field: field as u32,
        offset,
    };
    node_expr(kind.pointer_to(), node, base.span)
}

/// The work of building one dispatcher: its locals so far and what each arm
/// needs.
struct Dispatch
{
    /// The trait value kind.
    value: DataKind,
    /// The kind it holds.
    held: DataKind,
    /// The kind of its closure record.
    record: DataKind,
    /// The number of the dispatcher among the functions of the module.
    index: u32,
    /// The name of the dispatcher.
    name: String,
    /// Its locals: the number of the trait function, the address of the
    /// pack, the record, then the locals of its arms.
    locals: Vec<Slot>,
    /// The span its code takes.
    span: Span,
}

impl Dispatch
{
    /// Add a local to the dispatcher.
    ///
    /// # Arguments
    ///
    /// * `name` - Its name.
    /// * `kind` - Its kind.
    /// * `addressed` - Whether its address is taken.
    ///
    /// # Returns
    ///
    /// Its place.
    fn local(&mut self, name: &str, kind: DataKind, addressed: bool) -> Place
    {
        #[rustfmt::skip]
        let slot = Slot::builder().name(name).kind(kind).access(Access::Var).addressed(addressed).span(self.span).build();
        self.locals.push(slot);
        Place::Local(self.locals.len() as u32 - 1)
    }

    /// The trait value that holds a value: a new record of the dispatcher in
    /// a local of its own, which the closure moves to a heap block.
    ///
    /// # Arguments
    ///
    /// * `held` - The value, owned.
    /// * `out` - The statements of the arm.
    ///
    /// # Returns
    ///
    /// The trait value.
    fn wrap(&mut self, held: Expr, out: &mut Vec<Stmt>) -> Expr
    {
        let (record, span) = (self.record, self.span);
        let slot = self.local("%record", record, true);
        let code = default_constant(record_field(record, 0), span);
        let info = default_constant(record_field(record, 1), span);
        let cells = node_expr(record, ExprNode::List(vec![code, info, held]), span);
        out.push(Stmt::Set { place: slot, value: cells, span });
        let addr = node_expr(record.raw_pointer_to(), ExprNode::AddrOf(slot), span);
        #[rustfmt::skip]
        let made = ExprNode::Closure { func: self.index, name: self.name.clone(), record: Box::new(addr) };
        node_expr(self.value, made, span)
    }
}

impl Lowerer<'_>
{
    /// Lower the call the check puts around a value that goes into a trait
    /// value: the value, then the address of each function that gives its
    /// kind a function of the trait.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    /// * `kind` - The trait value kind.
    ///
    /// # Returns
    ///
    /// The trait value.
    pub(super) fn to_trait_node(
        &mut self, n: &ast::CallExpr, kind: DataKind,
    ) -> ExprNode
    {
        let (value, span) = (&n.args[0], n.span);
        let held = self.kind_of(value);
        let impls: Vec<Name> = n.args[1..]
            .iter()
            .filter_map(
                |a| tern!(let ast::Expr::FnAddr(f) = a, Some(f.name.name), None),
            )
            .collect();
        let lowered = self.owned(value, held);
        let index = self.dispatcher(kind, held, &impls, span);
        let record = DataKind::closure_record(&[held]);
        let slot = self.hidden(record, span);
        let code = default_constant(record_field(record, 0), span);
        let info = default_constant(record_field(record, 1), span);
        let cells = node_expr(record, ExprNode::List(vec![code, info, lowered]), span);
        self.pre.push(Stmt::Set { place: slot, value: cells, span });
        self.slot(slot).addressed = true;
        let addr = node_expr(record.raw_pointer_to(), ExprNode::AddrOf(slot), span);
        let name = self.lifted_name(index);
        ExprNode::Closure { func: index, name, record: Box::new(addr) }
    }

    /// The name of a function the lowering made, from its number.
    ///
    /// # Arguments
    ///
    /// * `index` - The number of the function.
    ///
    /// # Returns
    ///
    /// The name.
    fn lifted_name(&self, index: u32) -> String
    {
        let at = (index - self.top_count) as usize;
        self.lifted[at].as_ref().map_or_else(String::new, |f| f.name.clone())
    }

    /// Lower a call through a trait value: fill the argument pack, call the
    /// trait value with the number of the trait function and the address of
    /// the pack, put a new trait value in the place of the old one for a
    /// function that changes its first parameter, and read the result.
    ///
    /// # Arguments
    ///
    /// * `index` - The number of the trait function.
    /// * `args` - The arguments, the trait value first.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// A read of the result, held in a hidden slot, or `None` for a function
    /// with no result.
    pub(super) fn dispatch(
        &mut self, index: usize, args: &[ast::Expr], span: Span,
    ) -> Option<Expr>
    {
        let value = self.kind_of(&args[0]);
        let (name, _) = value.as_trait_value().expect("the check found a trait value");
        let decl = self.traits.get(&name).cloned().expect("the check found the trait");
        let shape = dispatch_shape(&decl, value, index, self.structs);
        let (params, ret) = shape.expect("the check found the trait function");
        let fields = pack_fields(value, &params, ret);
        let pack = DataKind::args_pack(&fields);
        let changes = params[0].edit;
        let callee = self.expr(&args[0]);
        let place = matches!(callee.node, ExprNode::Read(_) | ExprNode::Load { .. });
        let callee = tern!(place, callee, self.spill(callee));
        let slot = self.hidden(pack, span);
        if pack.holds_text()
        {
            let blank = default_constant(pack, span);
            self.pre_init.push(Stmt::Set { place: slot, value: blank, span });
            self.drops.last_mut().expect("a scope is open").push((slot, pack));
        }
        let mut cells = Vec::new();
        do_if!(changes, cells.push(default_constant(value, span)));
        for (a, p) in args[1..].iter().zip(&params[1..])
        {
            let borrow = p.kind.pointer_to().as_borrowed();
            let edit = p.edit || p.fill;
            cells.push(tern!(
                edit,
                self.edit_arg(a, borrow, p.fill),
                self.owned(a, p.kind)
            ));
        }
        do_if!(let Some(r) = ret, cells.push(default_constant(r, span)));
        self.pre.push(Stmt::Set {
            place: slot,
            value: node_expr(pack, ExprNode::List(cells), span),
            span,
        });
        self.slot(slot).addressed = true;
        let at = node_expr(pack.raw_pointer_to(), ExprNode::AddrOf(slot), span);
        let raw = DataKind::OPAQUE.with_outer_raw();
        let at = node_expr(
            raw,
            ExprNode::Convert { operand: Box::new(at), implicit: false },
            span,
        );
        let passed = vec![self.usize_const(index as u32, span), at];
        self.pre.push(Stmt::CallValue { callee, args: passed, span });
        self.pre.append(&mut self.post);
        let base = read(slot, pack, span);
        do_if!(changes, self.write_back(&args[0], &base, span));
        let ret_at = fields.len() - 1;
        ret.map(|r| {
            let addr = field_addr(&base, ret_at, self.platform);
            self.spill(node_expr(r, ExprNode::Load { addr: Box::new(addr) }, span))
        })
    }

    /// Put the new trait value a call that changes its first parameter wrote
    /// into the first field of the pack in the place of the old one, which is
    /// dropped.
    ///
    /// # Arguments
    ///
    /// * `target` - The place of the trait value.
    /// * `pack` - The pack, read where it sits.
    /// * `span` - The call.
    fn write_back(&mut self, target: &ast::Expr, pack: &Expr, span: Span)
    {
        let addr = field_addr(pack, 0, self.platform);
        let kind = self.kind_of(target);
        let new = node_expr(kind, ExprNode::Take { addr: Box::new(addr) }, span);
        let dest = self.dest(target);
        let mut out = Vec::new();
        let new = self.replace(&dest, new, &mut out);
        self.pre.append(&mut out);
        self.pre.push(match dest
        {
            Dest::Slot(place) => Stmt::Set { place, value: new, span },
            Dest::Mem(addr) => Stmt::Store { addr, value: new, span },
        });
    }

    /// The dispatcher of a held kind for a trait value kind, made the first
    /// time a value of the kind goes into a trait value of the kind.
    ///
    /// # Arguments
    ///
    /// * `value` - The trait value kind.
    /// * `held` - The held kind.
    /// * `impls` - The function that gives the held kind each function of the
    ///   trait, in the trait's order.
    /// * `span` - The span its code takes.
    ///
    /// # Returns
    ///
    /// The number of the dispatcher among the functions of the module.
    fn dispatcher(
        &mut self, value: DataKind, held: DataKind, impls: &[Name], span: Span,
    ) -> u32
    {
        ret_if!(let Some(index) = self.dispatchers.get(&(value, held)), *index);
        let index = self.top_count + self.lifted.len() as u32;
        self.dispatchers.insert((value, held), index);
        self.lifted.push(None);
        let (trait_name, _) = value.as_trait_value().expect("a trait value kind");
        let decl =
            self.traits.get(&trait_name).cloned().expect("the check found the trait");
        let record = DataKind::closure_record(&[held]);
        let name = format!("{trait_name}<{}>#{index}{}", held.name(), self.suffix);
        let raw = DataKind::OPAQUE.with_outer_raw();
        #[rustfmt::skip]
        let mut d = Dispatch { value, held, record, index, name, locals: Vec::new(), span };
        d.local("%number", DataKind::USIZE, false);
        d.local("%pack", raw, false);
        d.local("%record", record.raw_pointer_to(), false);
        let mut els = vec![Stmt::Return { value: None, span }];
        for i in (0..decl.fns.len()).rev()
        {
            let arm = self.dispatch_arm(&mut d, &decl, i, impls[i]);
            let number = read(Place::Local(0), DataKind::USIZE, span);
            let cond =
                compare_node(CmpOp::Eq, number, self.usize_const(i as u32, span));
            let cond = node_expr(DataKind::Bool, cond, span);
            els = vec![Stmt::If { cond, then: arm, els, span }];
        }
        #[rustfmt::skip]
        let func = Func::builder().name(d.name.clone()).params(3).locals(d.locals).block(els).span(span).build();
        self.lifted[(index - self.top_count) as usize] = Some(func);
        index
    }

    /// The arm of a dispatcher for one trait function: call the function of
    /// the held kind and write what it gives into the pack.
    ///
    /// # Arguments
    ///
    /// * `d` - The dispatcher being built.
    /// * `decl` - The trait.
    /// * `i` - The number of the trait function.
    /// * `func` - The function of the held kind.
    ///
    /// # Returns
    ///
    /// The statements of the arm.
    fn dispatch_arm(
        &mut self, d: &mut Dispatch, decl: &st::TraitDecl, i: usize, func: Name,
    ) -> Vec<Stmt>
    {
        let (span, platform) = (d.span, self.platform);
        let shape = dispatch_shape(decl, d.value, i, self.structs);
        let (params, ret) = shape.expect("the check found the trait function");
        let fields = pack_fields(d.value, &params, ret);
        let pack = DataKind::args_pack(&fields);
        let sig = self
            .sigs
            .get(&func)
            .cloned()
            .expect("the check named the function of the kind");
        let changes = params[0].edit;
        let raw = read(Place::Local(1), DataKind::OPAQUE.with_outer_raw(), span);
        let shaped = ExprNode::Convert { operand: Box::new(raw), implicit: false };
        let shaped = node_expr(pack.raw_pointer_to(), shaped, span);
        let base = node_expr(pack, ExprNode::Load { addr: Box::new(shaped) }, span);
        let record = read(Place::Local(2), d.record.raw_pointer_to(), span);
        let record =
            node_expr(d.record, ExprNode::Load { addr: Box::new(record) }, span);
        let held_at = field_addr(&record, RECORD_HEAD, platform);
        let held = node_expr(d.held, ExprNode::Load { addr: Box::new(held_at) }, span);
        let mut out = Vec::new();
        // A function that changes its first parameter changes a duplicate in a
        // local of the arm.
        let clone = changes.then(|| d.local("%clone", d.held, true));
        do_if!(let Some(c) = clone, out.push(Stmt::Set { place: c, value: dup_held(held.clone()), span }));
        let first = match clone
        {
            Some(c) =>
            {
                node_expr(d.held.pointer_to().as_borrowed(), ExprNode::AddrOf(c), span)
            }
            None => dup_held(held),
        };
        let first = coerce_lowered(first, sig.params[0], platform);
        let skip = usize::from(changes);
        let rest = params.iter().enumerate().skip(1).map(|(j, p)| {
            let at = field_addr(&base, skip + j - 1, platform);
            let v = node_expr(
                fields[skip + j - 1],
                ExprNode::Load { addr: Box::new(at) },
                span,
            );
            let v = tern!(p.edit || p.fill, v, dup_held(v));
            coerce_lowered(v, sig.params[j], platform)
        });
        let args: Vec<Expr> = std::iter::once(first).chain(rest).collect();
        let call = ExprNode::Call { func: sig.index, name: func.to_string(), args };
        match (ret, sig.ret)
        {
            (Some(r), Some(given)) =>
            {
                let value = node_expr(given, call, span);
                let value =
                    tern!(r == d.value && given != r, d.wrap(value, &mut out), value);
                let addr = field_addr(&base, fields.len() - 1, platform);
                out.push(Stmt::Store { addr, value, span });
            }
            _ =>
            {
                extract!(ExprNode::Call { func, name, args } = call, "a call");
                out.push(Stmt::Call { func, name, args, span });
            }
        }
        let Some(c) = clone
        else
        {
            return out;
        };
        let new = d.wrap(read(c, d.held, span), &mut out);
        out.push(Stmt::Store {
            addr: field_addr(&base, 0, platform),
            value: new,
            span,
        });
        out
    }
}
