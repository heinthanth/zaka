//! Functions declared inside a function. Each becomes a function of the
//! module of its own, after the top-level ones, that takes a borrow of each
//! capture it lists after its declared parameters, and reads and writes the
//! capture through it, as a parameter the function can change.
//!
//! A function with captures is a closure, made once where it is declared: a
//! record of a code word and one field for each capture (the value for a
//! duplicate or a move, the address of the local for a borrow) moves to a
//! heap block, and a hidden local under the name of the function holds the
//! function value. Each call of it, and its address, goes through that
//! value; a thunk takes the declared parameters and the record, and calls the
//! function with the address of each field, or the address a borrowed field
//! holds. A call of itself inside itself passes its own captures on.

use zakac_ast::stmt::CaptureMode;
use zakac_semcheck::fn_kind::FnKind;
use zakac_semcheck::fn_kind::FnParam;

use super::*;

/// A function declared inside a function, as its calls and its address see
/// it.
#[derive(Debug, Clone)]
pub(super) struct Nested
{
    /// The signature of its function: the declared parameters, then a borrow
    /// of each capture, under [`Sig::captures`].
    pub sig: Sig,
    /// The written name.
    pub written: Name,
    /// The name of its function in the module.
    pub name: String,
    /// The mode of each capture, in the order of [`Sig::captures`].
    pub modes: Vec<CaptureMode>,
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
pub(super) fn closure_key(name: &str) -> Name { Name::new(&format!("{name}%")) }

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
            f.captures.iter().map(|c| self.capture_key(c.name.name)).collect();
        let modes: Vec<CaptureMode> = f.captures.iter().map(|c| c.mode).collect();
        let kinds: Vec<DataKind> =
            names.iter().map(|n| self.captured_kind(n)).collect();
        let index = self.top_count + self.lifted.len() as u32;
        self.lifted.push(None);
        let value_kind = self.value_kind_of(f);
        // A closure takes its own value after its parameters, so it can name
        // itself as each other place names it.
        let own =
            (!names.is_empty()).then(|| (closure_key(&f.name.name), value_kind.repr()));
        let declared = f.params.iter().map(|p| param_kind(p, self.structs));
        let own_kind = own.map(|(_, k)| k);
        let borrows = kinds.iter().map(|k| k.pointer_to().as_borrowed());
        let params = declared.chain(own_kind).chain(borrows).collect();
        let own_edit = own.map(|_| false);
        let edits = f.params.iter().map(|p| p.edit).chain(own_edit);
        let edits = edits.chain(names.iter().map(|_| true));
        let ret = f.ret.as_ref().map(|k| DataKind::from_ast(k, self.structs).repr());
        let own_name = own.map(|(n, _)| n);
        #[rustfmt::skip]
        let sig = Sig { index, params, edits: edits.collect(), ret, foreign: None, captures: names.clone(), own: own_name };
        let name = format!("{}#{index}{}", f.name.name, self.suffix);
        #[rustfmt::skip]
        let nested = Nested { sig, written: f.name.name, name: name.clone(), modes, value_kind, span: f.span };
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
    /// names by the function name.
    ///
    /// # Arguments
    ///
    /// * `name` - The listed name.
    ///
    /// # Returns
    ///
    /// The name in scope.
    fn capture_key(&self, name: Name) -> Name
    {
        let local = self.scopes.iter().any(|s| s.contains_key(&name));
        tern!(local, name, closure_key(&name))
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
        let captures = nested.sig.captures.iter().zip(&nested.modes);
        let fields: Vec<Expr> =
            captures.map(|(c, m)| self.capture_field(*c, *m, span)).collect();
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
        let key =
            ast::Ident::builder().name(closure_key(&nested.written)).span(span).build();
        let place = self.declare(&key, nested.value_kind, Access::Var);
        let addr =
            Box::new(node_expr(record.raw_pointer_to(), ExprNode::AddrOf(slot), span));
        let label = nested.name.clone();
        let made = ExprNode::Closure { func: thunk, name: label, record: addr };
        let value = node_expr(nested.value_kind.repr(), made, span);
        out.push(Stmt::Set { place, value, span });
    }

    /// The field of a closure record for one capture: the address of the
    /// local for a borrow; a duplicate of the value, or the value moved out
    /// for a move. A number is copied either way, and an owner always
    /// moves.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the capture in scope.
    /// * `mode` - How it is captured.
    /// * `span` - The declaration.
    ///
    /// # Returns
    ///
    /// The field value.
    fn capture_field(&mut self, key: Name, mode: CaptureMode, span: Span) -> Expr
    {
        ret_if!(mode == CaptureMode::Lend, self.capture_arg(&key, span));
        let kind = self.captured_kind(&key);
        let node = self.read_name(&key, span);
        let read = node_expr(kind, node, span);
        let moved =
            kind.is_move_kind() || (mode == CaptureMode::Move && kind.holds_text());
        ret_if!(moved, take(read));
        tern!(kind.holds_text(), dup(read), read)
    }

    /// The kind of a local a function declared inside a function names: the
    /// kind of its slot, or of the caller's place for a parameter that holds
    /// a borrow of it.
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
        let kind = self.slot_kind(place);
        tern!(self.is_edit(place), kind.pointee().unwrap_or(kind), kind)
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
        let node = self.read_name(&closure_key(&nested.written), span);
        node_expr(nested.value_kind.repr(), node, span)
    }

    /// Declare a parameter that holds a borrow of a capture of a function
    /// declared inside a function, read and written as a parameter the
    /// function can change.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the capture in scope.
    /// * `kind` - Its kind.
    /// * `span` - The declared name of the function.
    pub(super) fn declare_capture(&mut self, name: Name, kind: DataKind, span: Span)
    {
        let ident = ast::Ident::builder().name(name).span(span).build();
        let place = self.declare(&ident, kind.pointer_to().as_borrowed(), Access::Var);
        let local = tern!(let Place::Local(n) = place, Some(n), None);
        let frame = self.frame.as_mut();
        do_if!(let (Some(n), Some(f)) = (local, frame), { f.edits.insert(n); });
    }

    /// Declare the parameter that holds the closure of the function being
    /// lowered, lent by each call, so its drop frees nothing.
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

    /// The borrow of a local that a call of a function declared inside a
    /// function passes: the address of the slot, or the borrow a parameter
    /// holds for it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the local.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The borrow.
    pub(super) fn capture_arg(&mut self, name: &str, span: Span) -> Expr
    {
        let place = self.lookup(name);
        ret_if!(self.is_edit(place), self.edit_addr(place, span));
        let kind = self.slot_kind(place).pointer_to().as_borrowed();
        self.slot(place).addressed = true;
        node_expr(kind, ExprNode::AddrOf(place), span)
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
        let fields = nested.modes.iter().enumerate().map(|(j, mode)| {
            let field = (j + RECORD_HEAD) as u32;
            let offset = shape.offset(j + RECORD_HEAD, platform);
            let kind = shape.fields()[j + RECORD_HEAD].1;
            let at =
                ExprNode::FieldAddr { base: Box::new(base.clone()), field, offset };
            let at = node_expr(kind.pointer_to(), at, span);
            let held =
                node_expr(kind, ExprNode::Load { addr: Box::new(at.clone()) }, span);
            let arg = tern!(*mode == CaptureMode::Lend, held, at);
            coerce_lowered(arg, sig.params[declared + 1 + j])
        });
        // The function takes its own closure next, lent: the record itself.
        let own = node_expr(
            nested.value_kind.repr(),
            ExprNode::Read(Place::Local(declared as u32)),
            span,
        );
        let own = node_expr(
            own.kind,
            ExprNode::Dup { value: Box::new(own), lent: true },
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
