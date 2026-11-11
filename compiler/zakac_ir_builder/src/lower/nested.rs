//! Functions declared inside a function. Each becomes a function of the
//! module of its own, after the top-level ones, that takes each capture it
//! names after its declared parameters.
//!
//! A capture is shared: the function and the block around it read and write
//! one variable. A local that a nested function names lives in a box, a
//! counted block of one cell, so it outlives the call that declares it; each
//! use of the name, in the function around and inside the nested one, reads
//! or writes that cell. A captured closure is held as a value.
//!
//! A function with captures is a closure, made once where it is declared: a
//! record of a code word, an info word and one field for each capture (a box,
//! or a closure) moves to a counted heap block, and a hidden local under the
//! name of the function holds the function value. Each call of it, and its
//! address, goes through that value; a thunk takes the declared parameters
//! and the record, and calls the function with each field. A call of itself
//! inside itself passes its own captures on.

use zakac_ast::visit::Visitor;
use zakac_semcheck::fn_kind::FnKind;
use zakac_semcheck::fn_kind::FnParam;

use super::*;

/// A function declared inside a function, as its calls and its address see
/// it.
#[derive(Debug, Clone)]
pub(super) struct Nested
{
    /// The signature of its function: the declared parameters, its own
    /// closure, then each capture under [`Sig::captures`]: a box, or a
    /// closure.
    pub sig: Sig,
    /// The written name.
    pub written: Name,
    /// The name of its function in the module.
    pub name: String,
    /// The kind of its function values.
    pub value_kind: DataKind,
    /// The declaration.
    pub span: Span,
}

/// The number of words of a closure record before its captures: the code
/// word and the info word.
pub(crate) const RECORD_HEAD: usize = 2;

/// The kind of one field of a closure record.
///
/// # Arguments
///
/// * `record` - The record kind.
/// * `n` - The field number; 0 is the code word.
///
/// # Returns
///
/// The kind.
fn record_field(record: DataKind, n: usize) -> DataKind
{
    record.as_struct().expect("a closure record is a struct").fields()[n].1
}

/// The name of the hidden local that holds the closure of a function
/// declared inside a function: the function name and a `%`, which no
/// written name holds.
///
/// # Arguments
///
/// * `name` - The function name.
///
/// # Returns
///
/// The name of the local.
pub(super) fn closure_name(name: &str) -> Name { Name::new(&format!("{name}%")) }

impl Lowerer<'_>
{
    /// Lower a function declared inside a function into a function of its
    /// own, with the state of the function around it set aside meanwhile;
    /// a function with captures then makes its closure where it stands.
    ///
    /// # Arguments
    ///
    /// * `f` - The declaration.
    /// * `out` - The statements of the block that declares it.
    pub(super) fn nested_fn(&mut self, f: &st::FnDecl, out: &mut Vec<Stmt>)
    {
        let names: Vec<Name> =
            f.captures.iter().map(|c| self.capture_name(c.name.name)).collect();
        let kinds: Vec<DataKind> =
            names.iter().map(|n| self.captured_kind(n)).collect();
        let index = self.top_count + self.lifted.len() as u32;
        self.lifted.push(None);
        let value_kind = self.value_kind_of(f);
        // A closure takes its own value after its parameters, so it can name
        // itself as each other place names it.
        let own = (!names.is_empty())
            .then(|| (closure_name(&f.name.name), value_kind.repr()));
        let declared = f.params.iter().map(|p| param_kind(p, self.structs));
        let own_kind = own.map(|(_, k)| k);
        let held = names.iter().zip(&kinds).map(|(n, k)| held_kind(n, *k));
        let params = declared.chain(own_kind).chain(held).collect();
        let own_edit = own.map(|_| false);
        let edits = f.params.iter().map(|p| p.edit).chain(own_edit);
        let edits = edits.chain(names.iter().map(|_| false));
        let owned = f.params.iter().map(|p| p.owned).collect();
        let ret = f.ret.as_ref().map(|k| DataKind::from_ast(k, self.structs).repr());
        let own_name = own.map(|(n, _)| n);
        let fills = f.params.iter().map(|p| p.fill).collect();
        #[rustfmt::skip]
        let sig = Sig { index, params, edits: edits.collect(), fills, owned, ret, foreign: None, captures: names.clone(), own: own_name };
        let name = format!("{}#{index}{}", f.name.name, self.suffix);
        #[rustfmt::skip]
        let nested = Nested { sig, written: f.name.name, name: name.clone(), value_kind, span: f.span };
        let scope = self.nested.last_mut().expect("a scope is open");
        scope.insert(f.name.name, nested.clone());
        let saved = self.set_aside();
        self.own_fns.push(f.name.name);
        let pairs: Vec<(Name, DataKind)> = names.into_iter().zip(kinds).collect();
        let mut func = self.func(f, own, &pairs);
        self.own_fns.pop();
        self.restore(saved);
        func.name = name;
        self.lifted[(index - self.top_count) as usize] = Some(func);
        do_if!(!nested.sig.captures.is_empty(), self.make_closure(&nested, out));
    }

    /// The name a capture is kept under: the local of the name, or the
    /// closure of a function declared inside a function, which the list
    /// names with the function name.
    ///
    /// # Arguments
    ///
    /// * `name` - The listed name.
    ///
    /// # Returns
    ///
    /// The name in scope.
    fn capture_name(&self, name: Name) -> Name
    {
        let local = self.scopes.iter().any(|s| s.contains_key(&name));
        tern!(local, name, closure_name(&name))
    }

    /// The kind of the function values of a function declared inside a
    /// function: its declared parameters and its result.
    ///
    /// # Arguments
    ///
    /// * `f` - The declaration.
    ///
    /// # Returns
    ///
    /// The function kind.
    fn value_kind_of(&self, f: &st::FnDecl) -> DataKind
    {
        let param = |p: &st::Param| FnParam {
            kind: DataKind::from_ast(&p.kind, self.structs),
            edit: p.edit,
            mark: p.mark.as_ref().map(|m| m.name),
        };
        let ret = f.ret.as_ref().map(|k| DataKind::from_ast(k, self.structs));
        DataKind::Fn(FnKind::make(f.params.iter().map(param).collect(), ret))
    }

    /// Make the closure of a function with captures where it is declared:
    /// fill its record in a hidden slot, then move the record to the heap
    /// into the function value of the hidden local of the function.
    ///
    /// # Arguments
    ///
    /// * `nested` - The function.
    /// * `out` - The statements of the block that declares it.
    fn make_closure(&mut self, nested: &Nested, out: &mut Vec<Stmt>)
    {
        let span = nested.span;
        let captures = nested.sig.captures.clone();
        let fields: Vec<Expr> =
            captures.iter().map(|c| self.capture_arg(c, span)).collect();
        let kinds: Vec<DataKind> = fields.iter().map(|e| e.kind).collect();
        let record = DataKind::closure_record(&kinds);
        let thunk = self.thunk_of(nested, record);
        let code = default_constant(record_field(record, 0), span);
        let info = default_constant(record_field(record, 1), span);
        let cells: Vec<Expr> = [code, info].into_iter().chain(fields).collect();
        let slot = self.hidden(record, span);
        out.append(&mut self.pre);
        let value = node_expr(record, ExprNode::List(cells), span);
        out.push(Stmt::Set { place: slot, value, span });
        self.slot(slot).addressed = true;
        let ident = ast::Ident::builder()
            .name(closure_name(&nested.written))
            .span(span)
            .build();
        let place = self.declare(&ident, nested.value_kind, Access::Var);
        let addr =
            Box::new(node_expr(record.raw_pointer_to(), ExprNode::AddrOf(slot), span));
        let label = nested.name.clone();
        let made = ExprNode::Closure { func: thunk, name: label, record: addr };
        let value = node_expr(nested.value_kind.repr(), made, span);
        out.push(Stmt::Set { place, value, span });
    }

    /// The kind of a local a function declared inside a function names: the
    /// kind of the variable, behind its box, or behind the borrow a
    /// parameter the function can change holds.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The kind.
    fn captured_kind(&mut self, name: &str) -> DataKind
    {
        let place = self.lookup(name);
        self.reached_kind(place)
    }

    /// Set aside the state of the function being lowered, leaving the
    /// top-level names of the file in scope.
    ///
    /// # Returns
    ///
    /// The state.
    fn set_aside(&mut self) -> Aside
    {
        let top = self.scopes.first().cloned().unwrap_or_default();
        Aside {
            scopes: std::mem::replace(&mut self.scopes, vec![top]),
            drops: std::mem::replace(&mut self.drops, vec![Vec::new()]),
            loops: std::mem::take(&mut self.loops),
            pre: std::mem::take(&mut self.pre),
            pre_init: std::mem::take(&mut self.pre_init),
            pending: std::mem::take(&mut self.pending),
            spilling: std::mem::take(&mut self.spilling),
        }
    }

    /// Bring back the state [`Self::set_aside`] kept.
    ///
    /// # Arguments
    ///
    /// * `a` - The state.
    fn restore(&mut self, a: Aside)
    {
        self.scopes = a.scopes;
        self.drops = a.drops;
        self.loops = a.loops;
        self.pre = a.pre;
        self.pre_init = a.pre_init;
        self.pending = a.pending;
        self.spilling = a.spilling;
    }

    /// The function declared inside a function that a name calls, innermost
    /// scope first.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The function, or `None`.
    pub(super) fn nested_named(&self, name: &str) -> Option<&Nested>
    {
        self.nested.iter().rev().find_map(|s| s.get(name))
    }

    /// Whether a call of a function declared inside a function goes through
    /// its closure: it has captures, and the call is no call of itself
    /// inside itself.
    ///
    /// # Arguments
    ///
    /// * `nested` - The function.
    ///
    /// # Returns
    ///
    /// `true` for a call through the closure.
    pub(super) fn through_closure(&self, nested: &Nested) -> bool
    {
        let own = self.own_fns.last() == Some(&nested.written);
        !nested.sig.captures.is_empty() && !own
    }

    /// The function value of a function with captures, read from its hidden
    /// local, or from the capture of it that the function being lowered
    /// lists.
    ///
    /// # Arguments
    ///
    /// * `nested` - The function.
    /// * `span` - The use.
    ///
    /// # Returns
    ///
    /// The read, of its function kind.
    pub(super) fn closure_value(&mut self, nested: &Nested, span: Span) -> Expr
    {
        let node = self.read_name(&closure_name(&nested.written), span);
        node_expr(nested.value_kind.repr(), node, span)
    }

    /// Declare a parameter that holds a capture of a function declared inside
    /// a function: the box of a variable, each use of which reads or writes
    /// its cell, or a closure, held as a value.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the capture in scope.
    /// * `kind` - Its kind.
    /// * `span` - The declared name of the function.
    pub(super) fn declare_capture(&mut self, name: Name, kind: DataKind, span: Span)
    {
        let ident = ast::Ident::builder().name(name).span(span).build();
        let place = self.declare(&ident, held_kind(&name, kind), Access::Var);
        do_if!(!is_closure_tag(&name), self.mark_box(place));
    }

    /// Declare the parameter that holds the closure of the function being
    /// lowered, lent for each call, so its drop frees nothing.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the closure.
    /// * `kind` - Its function kind.
    /// * `span` - The declared name of the function.
    pub(super) fn declare_own(&mut self, name: Name, kind: DataKind, span: Span)
    {
        let ident = ast::Ident::builder().name(name).span(span).build();
        self.declare(&ident, kind, Access::Var);
    }

    /// A capture as a closure record or a call of a function declared inside
    /// a function takes it: a new count of the box of the variable, or of the
    /// closure, so the holder shares it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the local.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The shared value.
    pub(super) fn capture_arg(&mut self, name: &str, span: Span) -> Expr
    {
        let place = self.lookup(name);
        let kind = self.slot_kind(place);
        dup(read(place, kind, span))
    }

    /// The address of a function declared inside a function: the function
    /// itself when it has no captures, else its closure value.
    ///
    /// # Arguments
    ///
    /// * `name` - The function name at the address.
    ///
    /// # Returns
    ///
    /// The operation, or `None` for a top-level function.
    pub(super) fn nested_addr(&mut self, name: &ast::Ident) -> Option<ExprNode>
    {
        let nested = self.nested_named(&name.name)?.clone();
        let (func, label) = (nested.sig.index, nested.name.clone());
        ret_if!(
            nested.sig.captures.is_empty(),
            Some(ExprNode::FnAddr { func, name: label })
        );
        Some(self.closure_value(&nested, name.span).node)
    }

    /// The thunk of a closure: it takes the declared parameters and the
    /// address of the record, and calls the function with the address of
    /// each field, or the address a borrowed field holds.
    ///
    /// # Arguments
    ///
    /// * `nested` - The function.
    /// * `record` - The kind of its record.
    ///
    /// # Returns
    ///
    /// The index of the thunk among the functions of the module.
    fn thunk_of(&mut self, nested: &Nested, record: DataKind) -> u32
    {
        let index = self.top_count + self.lifted.len() as u32;
        let sig = &nested.sig;
        let declared =
            sig.params.len() - sig.captures.len() - usize::from(sig.own.is_some());
        let span = nested.span;
        let slot = |name: String, kind: DataKind| {
            Slot::builder().name(name).kind(kind).access(Access::Var).span(span).build()
        };
        let at_record = record.raw_pointer_to();
        let mut locals: Vec<Slot> =
            (0..declared).map(|i| slot(format!("%p{i}"), sig.params[i])).collect();
        locals.push(slot("%record".into(), at_record));
        let base = read(Place::Local(declared as u32), at_record, span);
        let base = node_expr(record, ExprNode::Load { addr: Box::new(base) }, span);
        let passed =
            (0..declared).map(|i| read(Place::Local(i as u32), sig.params[i], span));
        let shape = record.as_struct().expect("a closure record is a struct");
        let platform = self.platform;
        // Each field goes to the function as a new count of its box or
        // closure, which the function lets go as it returns.
        let fields = (0..sig.captures.len()).map(|j| {
            let field = (j + RECORD_HEAD) as u32;
            let offset = shape.offset(j + RECORD_HEAD, platform);
            let kind = shape.fields()[j + RECORD_HEAD].1;
            let at =
                ExprNode::FieldAddr { base: Box::new(base.clone()), field, offset };
            let at = node_expr(kind.pointer_to(), at, span);
            let held = node_expr(kind, ExprNode::Load { addr: Box::new(at) }, span);
            coerce_lowered(dup(held), sig.params[declared + 1 + j], self.platform)
        });
        // The function takes its own closure next, lent: the record itself.
        let own = node_expr(
            nested.value_kind.repr(),
            ExprNode::Read(Place::Local(declared as u32)),
            span,
        );
        let own = node_expr(
            own.kind,
            ExprNode::Dup { value: Box::new(own), lent: true, deep: false },
            span,
        );
        let args: Vec<Expr> =
            passed.chain(std::iter::once(own)).chain(fields).collect();
        let call = ExprNode::Call { func: sig.index, name: nested.name.clone(), args };
        let (block, result) = match sig.ret
        {
            Some(kind) =>
            {
                let at = locals.len() as u32;
                locals.push(slot(RESULT_NAME.into(), kind));
                let value = node_expr(kind, call, span);
                let set = Stmt::Set { place: Place::Local(at), value, span };
                (vec![set, read_return(at, kind, span)], Some(at))
            }
            None =>
            {
                extract!(ExprNode::Call { func, name, args } = call, "a call");
                let run = Stmt::Call { func, name, args, span };
                (vec![run, Stmt::Return { value: None, span }], None)
            }
        };
        #[rustfmt::skip]
        let func = Func::builder().name(format!("{}#record{index}{}", nested.written, self.suffix))
            .params(declared as u32 + 1).maybe_result(result).maybe_ret(sig.ret).locals(locals)
            .block(block).span(span).build();
        self.lifted.push(Some(func));
        index
    }
}

/// The state of the function being lowered, set aside while a function
/// declared inside it is lowered.
pub(super) struct Aside
{
    /// Names in scope.
    scopes: Vec<HashMap<Name, Place>>,
    /// The places each scope drops.
    drops: Vec<Vec<(Place, DataKind)>>,
    /// The scope of each open loop.
    loops: Vec<usize>,
    /// Statements that run before the statement being lowered.
    pre: Vec<Stmt>,
    /// Statements that run before those.
    pre_init: Vec<Stmt>,
    /// The calls that pass a failure on still to lower.
    pending: usize,
    /// Whether an operand is being lowered into a hidden slot.
    spilling: bool,
}

impl Lowerer<'_>
{
    /// Mark a slot as the box of a variable that a closure shares, so each
    /// use of the name reaches its cell.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    pub(super) fn mark_box(&mut self, place: Place)
    {
        let local = tern!(let Place::Local(n) = place, Some(n), None);
        let frame = self.frame.as_mut();
        do_if!(let (Some(n), Some(f)) = (local, frame), { f.boxes.insert(n); });
    }

    /// A new box that holds a value: a counted block of one cell.
    ///
    /// # Arguments
    ///
    /// * `value` - The value, its own.
    /// * `span` - Where the box is made.
    ///
    /// # Returns
    ///
    /// The box, of the box kind of the value.
    pub(super) fn make_box(&self, value: Expr, span: Span) -> Expr
    {
        let cell = value.kind;
        let one =
            node_expr(DataKind::list_of(cell, 1), ExprNode::List(vec![value]), span);
        let node = ExprNode::Convert { operand: Box::new(one), implicit: false };
        node_expr(DataKind::box_of(cell), node, span)
    }

    /// Move a parameter that a function declared inside this one names into
    /// a box, so the closure shares it: the name stands for the box from
    /// here on. A parameter marked `ဒေတာ ပြောင်းလဲရန်` is noted, so each exit of
    /// the function copies the box back to the caller.
    ///
    /// # Arguments
    ///
    /// * `p` - The parameter.
    /// * `block` - The statements of the function so far.
    pub(super) fn box_param(&mut self, p: &st::Param, block: &mut Vec<Stmt>)
    {
        ret_if!(!self.is_boxed_name(&p.name.name));
        let place = self.lookup(&p.name.name);
        let kind = self.reached_kind(place);
        let node = self.read_name(&p.name.name, p.span);
        let read = node_expr(kind, node, p.span);
        let value = tern!(kind.holds_text(), dup(read), read);
        let boxed = self.declare(&p.name, DataKind::box_of(kind), Access::Var);
        self.mark_box(boxed);
        let value = self.make_box(value, p.span);
        block.push(Stmt::Set { place: boxed, value, span: p.span });
        // The function changes its own box; each exit copies it back.
        let frame = self.frame.as_mut().expect("a function is open");
        do_if!(p.edit, frame.write_backs.push((place, boxed)));
    }

    /// Give a loop variable a box of its own for one round, so a closure made
    /// in the round keeps the value of that round: the name stands for the
    /// box until the round ends, and the round drops it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable.
    /// * `var` - The slot of the variable.
    /// * `kind` - Its kind.
    /// * `out` - The statements of the round.
    pub(super) fn rebox(
        &mut self, name: &ast::Ident, var: Place, kind: DataKind, out: &mut Vec<Stmt>,
    )
    {
        let read = read(var, kind, name.span);
        let value = tern!(kind.holds_text(), dup(read), read);
        let boxed = self.declare(name, DataKind::box_of(kind), Access::Var);
        self.mark_box(boxed);
        let value = self.make_box(value, name.span);
        out.push(Stmt::Set { place: boxed, value, span: name.span });
    }

    /// The struct an object holds, read through its box: cell 0, checked, so
    /// an object that is the zero word stops the program.
    ///
    /// # Arguments
    ///
    /// * `object` - The object.
    ///
    /// # Returns
    ///
    /// The struct, of the struct kind.
    pub(super) fn object_fields(&self, object: Expr) -> Expr
    {
        let span = object.span;
        let shape = object.kind.as_object().expect("an object holds a struct");
        let index = Box::new(self.usize_const(0, span));
        #[rustfmt::skip]
        let node = ExprNode::CellAddr { list: Box::new(object), index, len: None, first: 0, checked: false };
        let addr = node_expr(DataKind::Struct(shape).pointer_to(), node, span);
        node_expr(
            DataKind::Struct(shape),
            ExprNode::Load { addr: Box::new(addr) },
            span,
        )
    }

    /// Whether a local of a name lives in a box in the function being
    /// lowered: a function declared inside it names the local.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// `true` for such a name.
    pub(super) fn is_boxed_name(&self, name: &str) -> bool
    {
        self.frame.as_ref().is_some_and(|f| f.boxed.iter().any(|n| n.as_str() == name))
    }
}

/// Whether a capture names a closure: the hidden local of a function declared
/// inside a function, whose name ends in `%`.
///
/// # Arguments
///
/// * `name` - The name of the capture in scope.
///
/// # Returns
///
/// `true` for a closure.
fn is_closure_tag(name: &str) -> bool { name.ends_with('%') }

/// The kind a capture is held as: the box of a variable, or the closure
/// itself.
///
/// # Arguments
///
/// * `name` - The name of the capture in scope.
/// * `kind` - The kind of the variable or the closure.
///
/// # Returns
///
/// The kind.
fn held_kind(name: &str, kind: DataKind) -> DataKind
{
    tern!(is_closure_tag(name), kind, DataKind::box_of(kind))
}

/// The names the functions declared inside a function capture, so the
/// function keeps each local of these names in a box.
///
/// # Arguments
///
/// * `f` - The function.
///
/// # Returns
///
/// The names.
pub(super) fn boxed_in(f: &st::FnDecl) -> HashSet<Name>
{
    let mut walk = Captures(HashSet::new());
    do_for!(s in &f.block.stmts, walk.visit_stmt(s));
    walk.0
}

/// The walker that gathers the captures of the functions declared inside
/// one, without going inside them: what a deeper one captures from around,
/// the one around it captures too.
struct Captures(HashSet<Name>);

impl Visitor for Captures
{
    fn visit_fn(&mut self, n: &st::FnDecl)
    {
        self.0.extend(n.captures.iter().map(|c| c.name.name));
    }
}
