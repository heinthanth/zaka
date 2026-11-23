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
use zakac_semcheck::trait_value::PART_PATH;
use zakac_semcheck::trait_value::dispatch_shape;
use zakac_semcheck::trait_value::held_kind;
use zakac_semcheck::trait_value::value_fns;

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
        let fns = value_fns(value, &self.traits);
        let (decl, targs, at) =
            fns.get(index).expect("the check found the trait function");
        let shape = dispatch_shape(decl, value, targs, *at, self.structs);
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
        let fns = value_fns(value, &self.traits);
        let record = DataKind::closure_record(&[held]);
        let name = format!("{trait_name}<{}>#{index}{}", held.name(), self.suffix);
        let raw = DataKind::OPAQUE.with_outer_raw();
        #[rustfmt::skip]
        let mut d = Dispatch { value, held, record, index, name, locals: Vec::new(), span };
        d.local("%number", DataKind::USIZE, false);
        d.local("%pack", raw, false);
        d.local("%record", record.raw_pointer_to(), false);
        let mut els = vec![Stmt::Return { value: None, span }];
        let count = fns.len();
        for i in (0..count + 2).rev()
        {
            let arm = match fns.get(i)
            {
                Some((decl, targs, at)) =>
                {
                    self.dispatch_arm(&mut d, (decl, targs, *at), impls[i])
                }
                None if i == count => self.number_arm(&d),
                None => self.read_arm(&d),
            };
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
    /// * `trait_fn` - The trait, its kinds after the first, and the number of
    ///   the function in it.
    /// * `func` - The function of the held kind.
    ///
    /// # Returns
    ///
    /// The statements of the arm.
    fn dispatch_arm(
        &mut self, d: &mut Dispatch, trait_fn: (&st::TraitDecl, &[DataKind], usize),
        func: Name,
    ) -> Vec<Stmt>
    {
        let (span, platform) = (d.span, self.platform);
        let (decl, targs, i) = trait_fn;
        let shape = dispatch_shape(decl, d.value, targs, i, self.structs);
        let (params, ret) = shape.expect("the check found the trait function");
        let fields = pack_fields(d.value, &params, ret);
        let pack = DataKind::args_pack(&fields);
        // A function of a part names the fields of the path to the part after
        // it.
        let text = func.to_string();
        let mut words = text.split(PART_PATH);
        let func = Name::new(words.next().unwrap_or_default());
        let path: Vec<&str> = words.collect();
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
        let mut out = Vec::new();
        // A function that changes its first parameter changes a duplicate in a
        // local of the arm.
        let clone = changes.then(|| d.local("%clone", d.held, true));
        let held =
            node_expr(d.held, ExprNode::Load { addr: Box::new(held_at.clone()) }, span);
        do_if!(let Some(c) = clone, out.push(Stmt::Set { place: c, value: dup_held(held), span }));
        let root = match clone
        {
            Some(c) => node_expr(d.held.pointer_to(), ExprNode::AddrOf(c), span),
            None => held_at,
        };
        let at = self.base_part_addr(root, &path);
        let first = match clone
        {
            Some(_) =>
            {
                let kind =
                    at.kind.pointee().expect("an address").pointer_to().as_borrowed();
                Expr { kind, ..at }
            }
            None =>
            {
                let kind = at.kind.pointee().expect("an address");
                dup_held(node_expr(kind, ExprNode::Load { addr: Box::new(at) }, span))
            }
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

impl Lowerer<'_>
{
    /// The number that names a held data kind in the tests of trait values,
    /// the same for each trait: 1 for the first data kind asked, and so on.
    ///
    /// # Arguments
    ///
    /// * `held` - The data kind.
    ///
    /// # Returns
    ///
    /// The number.
    fn kind_number(&mut self, held: DataKind) -> u32
    {
        let next = self.kind_numbers.len() as u32 + 1;
        *self.kind_numbers.entry(held).or_insert(next)
    }

    /// The address of a part of a held value, through the fields of the parts:
    /// a shared object is read through its box.
    ///
    /// # Arguments
    ///
    /// * `root` - The address of the held value.
    /// * `path` - The fields of the parts.
    ///
    /// # Returns
    ///
    /// The address of the part; `root` for no path.
    fn base_part_addr(&self, root: Expr, path: &[&str]) -> Expr
    {
        let mut at = root;
        for name in path
        {
            let (kind, span) = (at.kind.pointee().expect("an address"), at.span);
            let value = node_expr(kind, ExprNode::Load { addr: Box::new(at) }, span);
            let fields =
                tern!(kind.as_object().is_some(), self.object_fields(value), value);
            let shape = fields.kind.as_struct().expect("a part is a struct");
            let index = shape.field(name).expect("the check found the part").0;
            at = field_addr(&fields, index, self.platform);
        }
        at
    }

    /// The arm of a dispatcher that writes the number of its held data kind
    /// into the first field of the pack.
    ///
    /// # Arguments
    ///
    /// * `d` - The dispatcher being built.
    ///
    /// # Returns
    ///
    /// The statements of the arm.
    fn number_arm(&mut self, d: &Dispatch) -> Vec<Stmt>
    {
        let pack = DataKind::args_pack(&[DataKind::USIZE]);
        let number = self.kind_number(d.held);
        let value = self.usize_const(number, d.span);
        vec![Stmt::Store {
            addr: pack_field(pack, d.span, self.platform),
            value,
            span: d.span,
        }]
    }

    /// The arm of a dispatcher that writes a new count of its held value into
    /// the first field of the pack, which the caller laid out for that data
    /// kind after a test of the number.
    ///
    /// # Arguments
    ///
    /// * `d` - The dispatcher being built.
    ///
    /// # Returns
    ///
    /// The statements of the arm.
    fn read_arm(&mut self, d: &Dispatch) -> Vec<Stmt>
    {
        let span = d.span;
        let pack = DataKind::args_pack(&[d.held]);
        let record = read(Place::Local(2), d.record.raw_pointer_to(), span);
        let record =
            node_expr(d.record, ExprNode::Load { addr: Box::new(record) }, span);
        let held_at = field_addr(&record, RECORD_HEAD, self.platform);
        let held = node_expr(d.held, ExprNode::Load { addr: Box::new(held_at) }, span);
        let addr = pack_field(pack, span, self.platform);
        vec![Stmt::Store { addr, value: dup_held(held), span }]
    }

    /// The count of the functions of the trait of a trait value kind: the
    /// number of the selector that asks for the held data kind.
    ///
    /// # Arguments
    ///
    /// * `value` - The trait value kind.
    ///
    /// # Returns
    ///
    /// The count.
    fn trait_fn_count(&self, value: DataKind) -> usize
    {
        value_fns(value, &self.traits).len()
    }

    /// Call a trait value with a selector past its trait functions and a pack
    /// of one field of a data kind: the statements go to [`Self::pre`].
    ///
    /// # Arguments
    ///
    /// * `subject` - The trait value, read where it sits.
    /// * `selector` - The selector.
    /// * `field` - The data kind of the field.
    /// * `span` - The use.
    ///
    /// # Returns
    ///
    /// A read of the field, after the call.
    fn ask_held(
        &mut self, subject: &Expr, selector: usize, field: DataKind, span: Span,
    ) -> Expr
    {
        let pack = DataKind::args_pack(&[field]);
        let slot = self.hidden(pack, span);
        if pack.holds_text()
        {
            let blank = default_constant(pack, span);
            self.pre_init.push(Stmt::Set { place: slot, value: blank, span });
            self.drops.last_mut().expect("a scope is open").push((slot, pack));
        }
        let cells =
            node_expr(pack, ExprNode::List(vec![default_constant(field, span)]), span);
        self.pre.push(Stmt::Set { place: slot, value: cells, span });
        self.slot(slot).addressed = true;
        let at = node_expr(pack.raw_pointer_to(), ExprNode::AddrOf(slot), span);
        let raw = DataKind::OPAQUE.with_outer_raw();
        let at = node_expr(
            raw,
            ExprNode::Convert { operand: Box::new(at), implicit: false },
            span,
        );
        let args = vec![self.usize_const(selector as u32, span), at];
        self.pre.push(Stmt::CallValue { callee: subject.clone(), args, span });
        let base = read(slot, pack, span);
        let addr = field_addr(&base, 0, self.platform);
        node_expr(field, ExprNode::Load { addr: Box::new(addr) }, span)
    }

    /// The test whether a trait value holds a value of the data kind a name
    /// names: the number of its held data kind `==` the number of that one.
    ///
    /// # Arguments
    ///
    /// * `subject` - The trait value, read where it sits.
    /// * `name` - The name of the data kind.
    /// * `span` - The test.
    ///
    /// # Returns
    ///
    /// The test, a boolean.
    pub(super) fn held_test(&mut self, subject: &Expr, name: &str, span: Span) -> Expr
    {
        let held =
            held_kind(name, self.structs).expect("the check found the data kind");
        let count = self.trait_fn_count(subject.kind);
        let number = self.ask_held(subject, count, DataKind::USIZE, span);
        let number = self.spill(number);
        let wanted = self.kind_number(held);
        let want = self.usize_const(wanted, span);
        node_expr(DataKind::Bool, compare_node(CmpOp::Eq, number, want), span)
    }

    /// The read of the value a trait value holds, as the data kind a name
    /// names; the caller tests the data kind first.
    ///
    /// # Arguments
    ///
    /// * `subject` - The trait value, read where it sits.
    /// * `name` - The name of the data kind.
    /// * `span` - The read.
    ///
    /// # Returns
    ///
    /// A read of the value, held in a hidden slot its scope drops.
    pub(super) fn held_read(&mut self, subject: &Expr, name: &str, span: Span) -> Expr
    {
        let held =
            held_kind(name, self.structs).expect("the check found the data kind");
        let count = self.trait_fn_count(subject.kind);
        let value = self.ask_held(subject, count + 1, held, span);
        self.spill(value)
    }

    /// The read of the value of a data kind out of a trait value, which stops
    /// the run when the trait value holds a value of another data kind.
    ///
    /// # Arguments
    ///
    /// * `value` - The trait value.
    /// * `name` - The name of the data kind.
    /// * `span` - The read.
    ///
    /// # Returns
    ///
    /// The value.
    pub(super) fn held_data(
        &mut self, value: &ast::Expr, name: &str, span: Span,
    ) -> ExprNode
    {
        let lowered = self.expr(value);
        let subject = self.held_subject(lowered);
        let test = self.held_test(&subject, name, span);
        let wrong = node_expr(DataKind::Bool, prefix_node(PrefixOp::Not, test), span);
        let halt = Stmt::Halt { cause: HaltCause::WrongVariant, value: None, span };
        self.pre.push(Stmt::If {
            cond: wrong,
            then: vec![halt],
            els: Vec::new(),
            span,
        });
        self.held_read(&subject, name, span).node
    }

    /// The block of an arm that names the value of the data kind it tests:
    /// the read runs inside the arm, after the test passed.
    ///
    /// # Arguments
    ///
    /// * `subject` - The trait value, read where it sits.
    /// * `bind` - The name and the data kind it names.
    /// * `stmts` - The statements of the arm.
    ///
    /// # Returns
    ///
    /// The statements of the arm.
    pub(super) fn held_block(
        &mut self, subject: &Expr, bind: (&ast::Ident, &str), stmts: &[st::Stmt],
    ) -> Vec<Stmt>
    {
        let (name, held) = bind;
        let (mut out, data) = self.with_pre(|l| l.held_read(subject, held, name.span));
        out.extend(self.bound_block(Some((name, data)), stmts));
        out
    }

    /// Lower a switch over a trait value: the number of its held data kind
    /// once, then an arm for each case of a data kind, which can name the
    /// value.
    ///
    /// # Arguments
    ///
    /// * `n` - The switch.
    /// * `out` - The list the statements go to.
    pub(super) fn held_switch(&mut self, n: &st::SwitchStmt, out: &mut Vec<Stmt>)
    {
        let lowered = self.expr(&n.subject);
        let subject = self.switch_subject(lowered, out);
        let count = self.trait_fn_count(subject.kind);
        let number = self.ask_held(&subject, count, DataKind::USIZE, n.span);
        let number = self.spill(number);
        out.extend(std::mem::take(&mut self.pre));
        let mut arms = Vec::new();
        for arm in &n.arms
        {
            let names: Vec<&str> =
                arm.cases.iter().filter_map(case_kind_name).collect();
            let tests = names.iter().map(|name| {
                let held = held_kind(name, self.structs)
                    .expect("the check found the data kind");
                let wanted = self.kind_number(held);
                let want = self.usize_const(wanted, arm.span);
                node_expr(
                    DataKind::Bool,
                    compare_node(CmpOp::Eq, number.clone(), want),
                    arm.span,
                )
            });
            let tests: Vec<Expr> = tests.collect();
            let cond = tests.into_iter().reduce(|lhs, rhs| {
                let node = ExprNode::Logic {
                    op: LogicOp::Or,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                node_expr(DataKind::Bool, node, arm.span)
            });
            let cond = cond.expect("an arm has a case");
            let bind = match arm.cases.as_slice()
            {
                [ast::Expr::Variant(v)] => v.bind.as_ref().map(|b| (b, v.variant.name)),
                _ => None,
            };
            let block = match bind
            {
                Some((b, held)) =>
                {
                    self.held_block(&subject, (b, held.as_str()), &arm.block.stmts)
                }
                None => self.block(&arm.block.stmts),
            };
            arms.push((Vec::new(), (cond, block, arm.span)));
        }
        let els =
            n.default.as_ref().map_or_else(Vec::new, |d| self.block(&d.block.stmts));
        out.push(if_chain(arms, els, n.span));
    }
}

/// The name of the data kind a case of a switch over a trait value names.
///
/// # Arguments
///
/// * `case` - The case.
///
/// # Returns
///
/// The name, or `None` for another case.
fn case_kind_name(case: &ast::Expr) -> Option<&str>
{
    match case
    {
        ast::Expr::Name(n) => Some(n.name.as_str()),
        ast::Expr::Variant(v) => Some(v.variant.name.as_str()),
        _ => None,
    }
}

/// The address of the first field of a pack, through the raw address the
/// dispatcher takes as its second parameter.
///
/// # Arguments
///
/// * `pack` - The pack kind.
/// * `span` - The span the code takes.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The address.
fn pack_field(pack: DataKind, span: Span, platform: u16) -> Expr
{
    let raw = read(Place::Local(1), DataKind::OPAQUE.with_outer_raw(), span);
    let shaped = ExprNode::Convert { operand: Box::new(raw), implicit: false };
    let shaped = node_expr(pack.raw_pointer_to(), shaped, span);
    let base = node_expr(pack, ExprNode::Load { addr: Box::new(shaped) }, span);
    field_addr(&base, 0, platform)
}
