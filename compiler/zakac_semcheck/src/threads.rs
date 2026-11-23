//! A call that runs as a thread: its value is the thread, and each value that
//! goes to the thread, or comes back from it, must be one other threads can
//! use (ZK0494).

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::name::Name;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::EachStmt;
use zakac_ast::stmt::LockStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_add;
use zakac_ast::visit::walk_assign;
use zakac_ast::visit::walk_counted;
use zakac_ast::visit::walk_decl;
use zakac_ast::visit::walk_each;
use zakac_ast::visit::walk_expr;
use zakac_ast::visit::walk_remove;
use zakac_ast::visit::walk_replace;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::ownership as od;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::kind::DataKind;
use crate::proof::result_var;
use crate::scope::FnSig;
use crate::semcheck::Checker;
use crate::semcheck::each_list;
use crate::task;
use crate::trait_value::trait_call_index;

/// The verb of the core that waits for a thread.
pub const WAIT: &str = "စောင့်";

/// The preposition of a sleep counted in milliseconds: `<n> မီလီစက္ကန့် စောင့်မည်`.
pub const MILLIS: &str = "မီလီစက္ကန့်";

/// The preposition of a sleep counted in seconds: `<n> စက္ကန့် စောင့်မည်`.
pub const SECONDS: &str = "စက္ကန့်";

/// The milliseconds in one unit of a sleep, from the preposition of a call
/// of the core's `စောင့်` with one argument.
///
/// # Arguments
///
/// * `callee` - The name at the call.
/// * `place` - The value a call on a value names, or `None`.
/// * `prepositions` - The preposition after each argument.
///
/// # Returns
///
/// 1 for milliseconds, 1000 for seconds, or `None` for a call that is no
/// sleep.
pub fn sleep_unit(
    callee: &Ident, place: Option<&Expr>, prepositions: &[Option<Ident>],
) -> Option<u64>
{
    ret_if!(place.is_some() || callee.name.as_str() != WAIT, None);
    let [Some(unit)] = prepositions
    else
    {
        return None;
    };
    match unit.name.as_str()
    {
        MILLIS => Some(1),
        SECONDS => Some(1000),
        _ => None,
    }
}

/// The verb of the core that sends a value to a channel.
pub const SEND: &str = "ပို့";

/// The verb of the core that closes a channel.
pub const CLOSE: &str = "ပိတ်";

/// The struct of the core that holds the stats of the ring collector.
pub const RING_STATS: &str = "အလိုအလျောက်မင်မိုရီရှင်းလင်းမှုအခြေအနေပြကိန်းဂဏန်းများ";

/// The verb of the core that adds to an atomic number.
pub const ATOMIC_ADD: &str = "ပေါင်းထည့်";

/// The verb of the core that subtracts from an atomic number.
pub const ATOMIC_SUB: &str = "နုတ်ယူ";

/// The verb of the core that sets an atomic number when it holds an expected
/// value: `<atomic> ထဲရှိ <expected> ကို <new> ဖြင့် အစားထိုးသော ရလဒ်`.
pub const ATOMIC_CAS: &str = "အစားထိုး";

/// Finds a read of one place inside a value.
struct SamePlace
{
    /// The place, as [`place_text`] writes it.
    place: Option<String>,
    /// Whether a read of it came.
    found: bool,
}

impl Visitor for SamePlace
{
    fn visit_expr(&mut self, e: &Expr)
    {
        self.found |= self.place.is_some() && place_text(e) == self.place;
        walk_expr(self, e);
    }
}

/// A place as text: a name, or a field of a name, `<name>.<field>`.
///
/// # Arguments
///
/// * `e` - The place.
///
/// # Returns
///
/// The text, or `None` for another value.
fn place_text(e: &Expr) -> Option<String>
{
    match e
    {
        Expr::Group(g) => place_text(&g.inner),
        Expr::Name(n) => Some(n.name.to_string()),
        Expr::Field(f) => Some(format!("{}.{}", place_text(&f.base)?, f.field.name)),
        _ => None,
    }
}

/// Finds the writes of a round of a parallel loop to variables from outside
/// the loop.
struct RoundWrites
{
    /// The variable of the number of the round.
    number: Option<Name>,
    /// The variables the round declares.
    inner: Vec<Name>,
    /// Each write outside, with the name it writes.
    outside: Vec<(Span, String)>,
}

impl RoundWrites
{
    /// Note a write of a place, unless it is a variable of the round or the
    /// cell of the round's own number.
    ///
    /// # Arguments
    ///
    /// * `place` - The place written.
    fn write(&mut self, place: &Expr)
    {
        let own_cell = match place
        {
            Expr::Index(n) =>
            {
                matches!(n.index.as_ref(), Expr::Name(i) if Some(i.name) == self.number)
            }
            _ => false,
        };
        let root = root_name(place);
        let inner = root.as_ref().is_some_and(|(n, _)| self.inner.contains(n));
        let Some((name, span)) = root.filter(|_| !own_cell && !inner)
        else
        {
            return;
        };
        self.outside.push((span, name.to_string()));
    }
}

impl Visitor for RoundWrites
{
    fn visit_decl(&mut self, n: &zakac_ast::stmt::DeclStmt)
    {
        self.inner.push(n.name.name);
        walk_decl(self, n);
    }

    fn visit_counted(&mut self, n: &zakac_ast::stmt::CountedStmt)
    {
        self.inner.push(n.var.name);
        walk_counted(self, n);
    }

    fn visit_each(&mut self, n: &EachStmt)
    {
        self.inner.push(n.var.name);
        do_if!(let Some(p) = &n.place, self.inner.push(p.name));
        walk_each(self, n);
    }

    fn visit_assign(&mut self, n: &zakac_ast::stmt::AssignStmt)
    {
        self.write(&n.target);
        walk_assign(self, n);
    }

    fn visit_add(&mut self, n: &zakac_ast::stmt::AddStmt)
    {
        self.write(&n.list);
        walk_add(self, n);
    }

    fn visit_remove(&mut self, n: &zakac_ast::stmt::RemoveStmt)
    {
        self.write(&n.list);
        walk_remove(self, n);
    }

    fn visit_replace(&mut self, n: &zakac_ast::stmt::ReplaceStmt)
    {
        self.write(&n.text);
        walk_replace(self, n);
    }

    fn visit_verb(&mut self, n: &zakac_ast::expr::VerbExpr)
    {
        // A removal of a row writes its ဇယား.
        let removed = n.args.first().filter(|_| n.op == VerbOp::RemoveRow);
        do_if!(let Some(row) = removed, self.write(row));
        zakac_ast::visit::walk_verb(self, n);
    }
}

/// The round of a parallel loop that a check is inside.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RoundCtx
{
    /// The loop: a variable declared inside it belongs to the round.
    pub span: Span,
    /// The variable of the number of the round, if the loop has one.
    pub number: Option<Name>,
    /// The count of loops around the block of the round, inside the function.
    pub depth: u32,
}

impl RoundCtx
{
    /// The round of a loop, before its depth is known.
    ///
    /// # Arguments
    ///
    /// * `span` - The loop.
    /// * `number` - The variable of the number of the round.
    ///
    /// # Returns
    ///
    /// The round.
    pub(crate) fn of(span: Span, number: Option<Name>) -> RoundCtx
    {
        RoundCtx { span, number, depth: 0 }
    }
}

/// The variable at the root of a place, through fields and cells.
///
/// # Arguments
///
/// * `place` - The place.
///
/// # Returns
///
/// The name and its span, or `None` for no place.
fn root_name(place: &Expr) -> Option<(Name, Span)>
{
    match place
    {
        Expr::Group(g) => root_name(&g.inner),
        Expr::Name(n) => Some((n.name, n.span)),
        Expr::Field(f) => root_name(&f.base),
        Expr::Index(i) => root_name(&i.list),
        // A write through a row writes its ဇယား.
        Expr::Verb(v) if matches!(v.op, VerbOp::Row | VerbOp::RowValue) =>
        {
            root_name(v.args.first()?)
        }
        _ => None,
    }
}

impl Checker
{
    /// The thread a call that runs in parallel gives, after a report of each
    /// argument, and of the result, that threads cannot use.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments of the call.
    /// * `ret` - The kind its function gives back, or `None`.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The thread kind.
    pub(crate) fn start_thread(
        &mut self, args: &[Expr], ret: Option<DataKind>, span: Span,
    ) -> DataKind
    {
        for arg in args
        {
            let kind = self.kinds.get(&arg.id()).copied();
            do_if!(let Some(kind) = kind, self.goes_to_thread(kind, arg.span()));
        }
        do_if!(let Some(kind) = ret, self.goes_to_thread(kind, span));
        DataKind::Task(task::make(false, ret))
    }

    /// A wait for a thread, `<thread> ကို စောင့်မည်` or `<thread> ကို စောင့်သော ရလဒ်`: a
    /// call of the core's `စောင့်` on one value of a thread kind.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, or `None`.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// `Some` with the kind the function of the thread gives back, or `None`
    /// for a call that is no wait.
    pub(crate) fn wait_call(
        &mut self, callee: &Ident, place: Option<&Expr>, args: &[Expr],
    ) -> Option<Option<DataKind>>
    {
        let [thread] = args
        else
        {
            return None;
        };
        ret_if!(place.is_some() || callee.name.as_str() != WAIT, None);
        let task = self.infer(thread, None)?.as_task().filter(|t| t.is_thread())?;
        Some(task.of)
    }

    /// A sleep, `<n> မီလီစက္ကန့် စောင့်မည်` or `<n> စက္ကန့် စောင့်မည်`: a call of the
    /// core's `စောင့်` whose one argument is a count of the unsigned kind of the
    /// platform.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, or `None`.
    /// * `args` - The arguments.
    /// * `prepositions` - The preposition after each argument.
    ///
    /// # Returns
    ///
    /// `Some(None)`, for a sleep gives no value, or `None` for a call that is
    /// no sleep.
    pub(crate) fn sleep_call(
        &mut self, callee: &Ident, place: Option<&Expr>, args: &[Expr],
        prepositions: &[Option<Ident>],
    ) -> Option<Option<DataKind>>
    {
        sleep_unit(callee, place, prepositions)?;
        let [count] = args
        else
        {
            return None;
        };
        self.expect_kind(count, Some(DataKind::USIZE));
        Some(None)
    }

    /// A send, `<pipe> ထဲသို့ <value> ကို ပို့မည်`, or a close, `<pipe> ကို
    /// ပိတ်မည်`: a call of the core's `ပို့` or `ပိတ်` on a channel.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, or `None`.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// `Some(None)`, for neither gives a value, or `None` for a call that is
    /// neither.
    pub(crate) fn pipe_call(
        &mut self, callee: &Ident, place: Option<&Expr>, args: &[Expr],
    ) -> Option<Option<DataKind>>
    {
        ret_if!(place.is_some(), None);
        let name = callee.name.as_str();
        match args
        {
            [pipe, value] if name == SEND && self.names_pipe(pipe) =>
            {
                let carried = self.pipe_of(pipe);
                self.expect_kind(value, carried);
                do_if!(let Some(kind) = carried, self.goes_to_thread(kind, value.span()));
                Some(None)
            }
            [pipe] if name == CLOSE && self.names_pipe(pipe) => Some(None),
            _ => None,
        }
    }

    /// Whether a value is a channel, which a call of the core's `ပို့` or
    /// `ပိတ်` takes in place of the overloads a program declares.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    ///
    /// # Returns
    ///
    /// `true` for a channel.
    fn names_pipe(&mut self, e: &Expr) -> bool
    {
        self.infer(e, None).and_then(DataKind::as_task).is_some_and(|t| t.pipe)
    }

    /// The kind a channel carries, after a report of a value that is no
    /// channel.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after a report.
    fn pipe_of(&mut self, e: &Expr) -> Option<DataKind>
    {
        let kind = self.infer(e, None)?;
        let task = kind.as_task().filter(|t| t.pipe);
        ret_if!(
            task.is_none(),
            self.report(kd::not_a_pipe(e.span(), &kind.name())),
            None
        );
        task.and_then(|t| t.of)
    }

    /// Infer a receive, `<pipe> မှ ရရှိသော ဒေတာ`: an optional of the kind the
    /// channel carries.
    ///
    /// # Arguments
    ///
    /// * `pipe` - The channel.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after an error.
    pub(crate) fn infer_receive(&mut self, pipe: &Expr) -> Option<DataKind>
    {
        self.pipe_of(pipe).map(DataKind::optional)
    }

    /// The kind of the variable of a loop over each value of a channel.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value the loop walks.
    /// * `n` - The loop.
    ///
    /// # Returns
    ///
    /// The kinds of the two names of the loop, or `None` after a report.
    pub(crate) fn received_var_kind(
        &mut self, kind: DataKind, n: &EachStmt,
    ) -> Option<(Option<DataKind>, DataKind)>
    {
        let carried = kind.as_task().filter(|t| t.pipe).and_then(|t| t.of);
        ret_if!(let Some(carried) = carried, Some((None, carried)));
        self.report(kd::not_a_pipe(n.source.span(), &kind.name()));
        None
    }

    /// Check the condition of an arm of `အကယ်၍`, where a test of a channel for
    /// data can stand alone.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    ///
    /// # Returns
    ///
    /// The channel the condition tests, whose data its branch can read.
    pub(crate) fn check_arm_cond(&mut self, cond: &Expr) -> Option<Name>
    {
        let tested = match cond
        {
            Expr::Verb(v) if v.op == VerbOp::HasData && v.args.len() == 1 =>
            {
                Some(&v.args[0])
            }
            _ => None,
        };
        let outer = std::mem::replace(&mut self.pipe_test, tested.map(Expr::id));
        self.expect_bool(cond);
        self.pipe_test = outer;
        let pipe = tested.filter(|t| {
            let kind = self.kinds.get(&t.id()).copied();
            kind.and_then(DataKind::as_task).is_some_and(|k| k.pipe)
        });
        pipe.and_then(result_var)
    }

    /// Warn on a while loop whose block is one wait on several channels with
    /// the arm of no data and no sleep in any arm: it never waits (ZK0418).
    ///
    /// # Arguments
    ///
    /// * `block` - The block of the loop.
    pub(crate) fn check_select_spin(&mut self, block: &BlockStmt)
    {
        let [Stmt::Select(wait)] = block.stmts.as_slice()
        else
        {
            return;
        };
        let Some(idle) = &wait.idle
        else
        {
            return;
        };
        let arms = wait.arms.iter().map(|a| &a.block).chain(&wait.closed);
        let sleeps = |b: &BlockStmt| b.stmts.iter().any(is_sleep);
        ret_if!(arms.chain([idle]).any(sleeps));
        self.report(kd::select_spins(idle.span));
    }

    /// Check the test of an arm of a wait on several channels: it tests a
    /// channel for data, as the condition of `အကယ်၍` does, and the channel is
    /// a name (ZK0495 for a value of another kind).
    ///
    /// # Arguments
    ///
    /// * `cond` - The test.
    ///
    /// # Returns
    ///
    /// The channel, whose data the block of the arm can read.
    pub(crate) fn check_select_arm(&mut self, cond: &Expr) -> Option<Name>
    {
        let pipe = self.check_arm_cond(cond);
        ret_if!(pipe.is_some(), pipe);
        let tested = match cond
        {
            Expr::Verb(v) if v.op == VerbOp::HasData => v.args.first(),
            _ => None,
        };
        let tested = tested.unwrap_or(cond);
        let kind = self.kinds.get(&tested.id()).copied();
        let name = kind.map_or_else(|| "?".to_string(), |k| k.name());
        // A channel that is no plain name has ZK0496 at this point.
        let pipe_kind = kind.and_then(DataKind::as_task).is_some_and(|k| k.pipe);
        do_if!(!pipe_kind, self.report(kd::not_a_pipe(tested.span(), &name)));
        None
    }

    /// Infer a test of a channel for data, or a read of its data: the test
    /// stands alone as the condition of `အကယ်၍` (ZK0496 elsewhere), and the read
    /// inside the branch of that test.
    ///
    /// # Arguments
    ///
    /// * `op` - The verb.
    /// * `value` - The channel.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The kind of the test or the read.
    pub(crate) fn pipe_part(
        &mut self, op: VerbOp, value: &Expr, kind: DataKind,
    ) -> Option<DataKind>
    {
        let name = result_var(value);
        let ok = match op
        {
            VerbOp::HasData => self.pipe_test == Some(value.id()) && name.is_some(),
            VerbOp::Unwrap => name.is_some_and(|n| self.pipe_reads.contains(&n)),
            _ => false,
        };
        do_if!(!ok, self.report(kd::pipe_data_outside(value.span())));
        let carried = kind.as_task().and_then(|t| t.of);
        tern!(op == VerbOp::Unwrap, carried, Some(DataKind::Bool))
    }

    /// Report the function a call that runs as a thread starts in when it is no
    /// function the program declares in Zaka (ZK0497), or when it changes or
    /// fills a parameter (ZK0498). A call through a trait value runs the
    /// function of its trait, which must change and fill no parameter either.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, or `None`.
    /// * `first` - The first argument, or `None`.
    pub(crate) fn check_thread_callee(
        &mut self, callee: &Ident, place: Option<&Expr>, first: Option<&Expr>,
    )
    {
        let name = callee.name.as_str();
        let sigs = self.fns.get(&callee.name).filter(|_| place.is_none());
        let chosen = self.chosen.get(&(self.top, callee.span)).copied();
        if let Some(index) = chosen.and_then(|c| trait_call_index(&c))
        {
            let value = first.and_then(|a| self.kinds.get(&a.id()).copied());
            let func = value.and_then(|v| self.trait_fn_at(v, index));
            let changes =
                func.is_some_and(|f| f.params.iter().any(|p| p.edit || p.fill));
            do_if!(changes, self.report(kd::thread_changes_args(callee.span, name)));
            return;
        }
        let sig = sigs.and_then(|s| {
            let one = s.first().filter(|_| s.len() == 1);
            s.iter().find(|f| Some(f.inner) == chosen).or(one).cloned()
        });
        let plain =
            sigs.is_some() && !sig.as_ref().is_some_and(|s| s.external || s.closure);
        ret_if!(!plain, self.report(kd::thread_of_no_declared_fn(callee.span, name)));
        let changes = sig.is_some_and(|s| s.edits.iter().chain(&s.fills).any(|e| *e));
        do_if!(changes, self.report(kd::thread_changes_args(callee.span, name)));
    }

    /// Check a lock block: its name holds a shared object, and the fields of
    /// that object can be reached inside it.
    ///
    /// # Arguments
    ///
    /// * `n` - The block.
    pub(crate) fn check_lock(&mut self, n: &LockStmt)
    {
        let binding = self.scopes.lookup(&n.obj.name);
        do_if!(let Some(b) = &binding, self.refer(&n.obj.name, n.obj.span, b.home()));
        let object =
            binding.and_then(|b| b.kind).is_some_and(|k| k.as_object().is_some());
        do_if!(!object, self.report(kd::outside_lock(n.obj.span, &n.obj.name, true)));
        self.held_locks.push(n.obj.name);
        self.visit_block(&n.block);
        self.held_locks.pop();
    }

    /// Report a field of an object threads share that is reached outside a lock
    /// block of the object.
    ///
    /// # Arguments
    ///
    /// * `base` - The object.
    /// * `kind` - Its kind.
    /// * `field` - The field.
    pub(crate) fn check_locked_field(
        &mut self, base: &Expr, kind: DataKind, field: &Ident,
    )
    {
        let shared = kind.cell_kind().is_some_and(|c| self.marked_for_threads(c));
        ret_if!(!kind.is_box() || !shared);
        let held = result_var(base).is_some_and(|n| self.held_locks.contains(&n));
        do_if!(!held, self.report(kd::outside_lock(field.span, &field.name, false)));
    }

    /// An add to an atomic number, `<atomic> ထဲသို့ <value> ကို ပေါင်းထည့်မည်`, or a
    /// subtraction, `<atomic> ထဲမှ <value> ကို နုတ်ယူမည်`: a call of the core's
    /// `ပေါင်းထည့်` or `နုတ်ယူ` on an atomic number.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `place` - The value a call on a value names, or `None`.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// `Some(None)`, for neither gives a value, or `None` for a call that is
    /// neither.
    pub(crate) fn atomic_call(
        &mut self, callee: &Ident, place: Option<&Expr>, args: &[Expr],
    ) -> Option<Option<DataKind>>
    {
        let name = callee.name.as_str();
        ret_if!(place.is_some(), None);
        if let ([target, expected, new], ATOMIC_CAS) = (args, name)
        {
            return self.cas_call(target, expected, new);
        }
        ret_if!(name != ATOMIC_ADD && name != ATOMIC_SUB, None);
        let [target, value] = args
        else
        {
            return None;
        };
        let int = self.atomic_place(target)?;
        // The add writes the number, so a closure shares it and copies none.
        self.note_written(target);
        self.infer(target, None);
        self.expect_kind(value, Some(int));
        Some(None)
    }

    /// A compare-and-set of an atomic number, `<atomic> ထဲရှိ <expected> ကို
    /// <new> ဖြင့် အစားထိုးသော ရလဒ်`: a result of its integer kind on each side,
    /// as Rust's `compare_exchange` gives: the good side holds what the number
    /// held, the expected value, and the error side what it held in its place.
    /// Each value can read the same number: the set itself checks it.
    ///
    /// # Arguments
    ///
    /// * `target` - The atomic number.
    /// * `expected` - The value it must hold.
    /// * `new` - The value it takes.
    ///
    /// # Returns
    ///
    /// `Some(Some(result))`, or `None` for a place that holds no atomic number,
    /// which leaves the call to the replace of text.
    fn cas_call(
        &mut self, target: &Expr, expected: &Expr, new: &Expr,
    ) -> Option<Option<DataKind>>
    {
        let int = self.atomic_place(target)?;
        self.note_written(target);
        self.infer(target, None);
        self.expect_kind(expected, Some(int));
        self.expect_kind(new, Some(int));
        Some(Some(DataKind::fallible(Some(int), Some(int))))
    }

    /// The integer kind of an atomic number a place holds: a variable or a
    /// field of one.
    ///
    /// # Arguments
    ///
    /// * `e` - The place.
    ///
    /// # Returns
    ///
    /// The integer kind, or `None` for a place that holds no atomic number.
    pub(crate) fn atomic_place(&mut self, e: &Expr) -> Option<DataKind>
    {
        let held = match e
        {
            Expr::Group(g) => return self.atomic_place(&g.inner),
            Expr::Name(n) => self.scopes.lookup(&n.name)?.kind?,
            Expr::Field(f) =>
            {
                let base = self.infer(&f.base, None)?;
                base.object_struct()?.field(&f.field.name)?.1
            }
            _ => return None,
        };
        held.as_task().filter(|t| t.atomic).and_then(|t| t.of)
    }

    /// Report a set of an atomic number whose value reads the same number: a
    /// thread between the read and the write would lose its change (ZK0727).
    ///
    /// # Arguments
    ///
    /// * `target` - The place set.
    /// * `value` - The value.
    pub(crate) fn check_atomic_set(&mut self, target: &Expr, value: &Expr)
    {
        ret_if!(self.atomic_place(target).is_none());
        let mut reads = SamePlace { place: place_text(target), found: false };
        reads.visit_expr(value);
        do_if!(reads.found, self.report(od::atomic_read_write(target.span())));
    }

    /// Infer a parallel map, `<list> ၏ အကွက်တိုင်း ကို <fn> ဟူသော လုပ်ဆောင်ချက်
    /// ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ`: a plain list of what the function gives
    /// back. The function is one the program declares, of one parameter the
    /// cells fit, as for a thread.
    ///
    /// # Arguments
    ///
    /// * `list` - The list.
    /// * `f` - The name of the function.
    ///
    /// # Returns
    ///
    /// The plain list kind, or `None` after an error.
    pub(crate) fn infer_par_map(
        &mut self, list: &Expr, f: &NameExpr,
    ) -> Option<DataKind>
    {
        let kind = self.infer(list, None)?;
        let cell = each_list(kind, false).cell_kind();
        let Some(cell) = cell
        else
        {
            self.report(kd::each_source_wrong(list.span(), &kind.name()));
            return None;
        };
        let callee = Ident::builder().name(f.name.as_str()).span(f.span).build();
        let sigs = self.fns.get(&f.name).cloned().unwrap_or_default();
        let fits = |s: &&FnSig| {
            s.params.len() == 1 && cell.widens_to(s.params[0], self.platform)
        };
        let found: Vec<&FnSig> = sigs.iter().filter(fits).collect();
        let [sig] = found.as_slice()
        else
        {
            let list = sigs
                .iter()
                .map(|s| {
                    s.params.iter().map(|k| k.name()).collect::<Vec<_>>().join(", ")
                })
                .collect::<Vec<_>>()
                .join(" / ");
            self.report(rd::no_overload_fits(f.span, &f.name, &list));
            return None;
        };
        self.refer(&f.name, f.span, sig.decl);
        self.check_thread_callee(&callee, None, None);
        self.goes_to_thread(cell, list.span());
        let ret = sig.ret?;
        self.goes_to_thread(ret, f.span);
        Some(DataKind::seq_of(ret))
    }

    /// Report each write of a round of a parallel loop to a variable from
    /// outside the loop, but the cell of the round's own number (ZK0728).
    ///
    /// # Arguments
    ///
    /// * `number` - The variable of the number of the round, if the loop has
    ///   one.
    /// * `block` - The block of the round.
    pub(crate) fn check_round(&mut self, number: Option<Name>, block: &BlockStmt)
    {
        let mut walk = RoundWrites { number, inner: Vec::new(), outside: Vec::new() };
        walk.visit_block(block);
        let found = std::mem::take(&mut walk.outside);
        found
            .into_iter()
            .for_each(|(span, n)| self.report(od::round_writes_outside(span, &n)));
    }

    /// Check the count of threads of a parallel loop or map: an integer, as a
    /// bound of a counted loop is (ZK0405).
    ///
    /// # Arguments
    ///
    /// * `threads` - The count.
    pub(crate) fn check_thread_count(&mut self, threads: &Expr)
    {
        let found = self.infer(threads, None);
        do_if!(let Some(k) = found, _ = self.need_bound(k, threads));
    }

    /// Report a variable from outside a round of a parallel loop that the
    /// round reads, when it holds a value threads cannot use (ZK0494): the
    /// rounds run on several threads at once.
    ///
    /// # Arguments
    ///
    /// * `n` - The name read.
    pub(crate) fn round_read(&mut self, n: &NameExpr)
    {
        let Some(round) = self.round
        else
        {
            return;
        };
        let local =
            self.scopes.local(&n.name).filter(|b| !round.span.contains(&b.span));
        let Some(kind) = local.and_then(|b| b.kind)
        else
        {
            return;
        };
        self.goes_to_thread(kind, n.span);
    }

    /// Report a call in a round of a parallel loop of a closure declared
    /// outside the round (ZK0494): its captures are shared, and the rounds
    /// run on several threads at once.
    ///
    /// # Arguments
    ///
    /// * `sig` - The function called.
    /// * `callee` - The name of the callee.
    pub(crate) fn round_call(&mut self, sig: &FnSig, callee: &Ident)
    {
        let outside = self.round.is_some_and(|r| !r.span.contains(&sig.decl));
        do_if!(
            outside && sig.closure,
            self.report(kd::not_for_threads(callee.span, &callee.name))
        );
    }

    /// Report an argument a call in a round of a parallel loop lets the
    /// function change, when it is a variable from outside the round other
    /// than the cell of the round's own number (ZK0728).
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument.
    pub(crate) fn round_edit_arg(&mut self, arg: &Expr)
    {
        let Some(round) = self.round
        else
        {
            return;
        };
        let own_cell = matches!(arg, Expr::Index(n)
            if matches!(n.index.as_ref(), Expr::Name(i) if Some(i.name) == round.number));
        let Some((name, span)) = root_name(arg).filter(|_| !own_cell)
        else
        {
            return;
        };
        let outside =
            self.scopes.local(&name).is_some_and(|b| !round.span.contains(&b.span));
        do_if!(outside, self.report(od::round_writes_outside(span, &name)));
    }

    /// Report a kind that threads cannot use, where a value of it goes to a
    /// thread or comes back from one.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `span` - The value.
    pub(crate) fn goes_to_thread(&mut self, kind: DataKind, span: Span)
    {
        do_if!(
            !self.usable_in_threads(kind),
            self.report(kd::not_for_threads(span, &kind.name()))
        );
    }
}

/// Whether a statement is a sleep, `<n> မီလီစက္ကန့် စောင့်မည်။`.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// `true` if it is.
fn is_sleep(s: &Stmt) -> bool
{
    let Stmt::Call(c) = s
    else
    {
        return false;
    };
    sleep_unit(&c.callee, c.place.as_deref(), &c.prepositions).is_some()
}
