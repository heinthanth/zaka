//! The rounds of a parallel loop. Each round becomes a function of the module
//! of its own, which takes the number of the round, from 0, what the loop
//! gives each round alike, and the address of each variable from outside the
//! loop that the round names. The loop calls the function once for each
//! round: the VM runs those calls in order, and a native build hands them to
//! the runtime, which runs them on several threads at once.
//!
//! A round writes no variable from outside but the cell of its own number,
//! so the addresses let the rounds share the variables with no race. A plain
//! list a round writes a cell of gets a block of its own before the loop, so
//! no round copies it while another writes.

use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_call_expr;
use zakac_ast::visit::walk_name;

use super::*;
use crate::ir::Par;
use crate::ir::ParRounds;
use crate::lower::nested::closure_name;

/// What a round takes from its loop, besides its number and the variables
/// from outside.
enum RoundHead<'a>
{
    /// A counted loop: its variable, of a kind, from a first value, up or
    /// down, with a step.
    Counted(&'a st::CountedStmt, DataKind),
    /// A loop over each cell of a list: the slot of the list and its kind.
    Each(&'a st::EachStmt, Place, DataKind),
    /// A loop over each character of text: the kind of the text and of the
    /// list of where each character starts.
    Chars(&'a st::EachStmt, DataKind, DataKind),
    /// A loop over each row of a ဇယား: its kind.
    Rows(&'a st::EachStmt, DataKind),
}

impl RoundHead<'_>
{
    /// The count of threads the loop was given.
    ///
    /// # Returns
    ///
    /// The expression, or `None` for one thread for each core.
    fn threads(&self) -> Option<&ast::Expr>
    {
        match self
        {
            RoundHead::Counted(n, _) => n.threads.as_ref(),
            RoundHead::Each(n, ..)
            | RoundHead::Chars(n, ..)
            | RoundHead::Rows(n, _) => n.threads.as_ref(),
        }
    }
}

impl Lowerer<'_>
{
    /// Lower a parallel counted loop: a slot for each bound and for the step,
    /// the direction and the count of its rounds, and the loop of the calls of
    /// its round.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `out` - The list the statements go to.
    pub(super) fn parallel_counted(&mut self, n: &st::CountedStmt, out: &mut Vec<Stmt>)
    {
        let span = n.span;
        let kind = self.common(self.kind_of(&n.from), self.kind_of(&n.to));
        let (from, to) = (self.coerce(&n.from, kind), self.coerce(&n.to, kind));
        let one = node_expr(kind, convert_node(self.usize_const(1, span)), span);
        let step = n.step.as_ref().map_or(one, |s| self.coerce(s, kind));
        let [first, last, stride] =
            [self.hidden(kind, span), self.hidden(kind, span), self.hidden(kind, span)];
        out.push(Stmt::Set { place: first, value: from, span });
        out.push(Stmt::Set { place: last, value: to, span });
        out.push(Stmt::Set { place: stride, value: step, span });
        let [first_read, last_read, stride_read] =
            [first, last, stride].map(|p| read(p, kind, span));
        // A step of zero or below stops the program, as each counted loop does.
        let zero = node_expr(kind, convert_node(self.usize_const(0, span)), span);
        let bad = node_expr(
            DataKind::Bool,
            compare_node(CmpOp::Le, stride_read.clone(), zero),
            span,
        );
        let halt = Stmt::Halt { cause: HaltCause::StepNotPositive, value: None, span };
        let check = Stmt::If { cond: bad, then: vec![halt], els: Vec::new(), span };
        do_if!(n.step.is_some(), out.push(check));
        let le = |a: &Expr, b: &Expr| {
            node_expr(
                DataKind::Bool,
                compare_node(CmpOp::Le, a.clone(), b.clone()),
                span,
            )
        };
        let on = node_expr(DataKind::Bool, ExprNode::Const(Value::Bool(true)), span);
        let no = node_expr(DataKind::Bool, ExprNode::Const(Value::Bool(false)), span);
        // `ရေတွက်၍` goes toward the last bound; `တိုး၍` goes up and `လျှော့၍` down,
        // with no round when the last bound lies on the other side.
        let (up_value, valid) = match n.mode
        {
            st::CountMode::Count => (le(&first_read, &last_read), on),
            st::CountMode::Increase => (on, le(&first_read, &last_read)),
            st::CountMode::Decrease => (no, le(&last_read, &first_read)),
        };
        let up = self.hidden(DataKind::Bool, span);
        out.push(Stmt::Set { place: up, value: up_value, span });
        let up_read = read(up, DataKind::Bool, span);
        // The rounds: |last - first| / step + 1.
        let count = self.hidden(DataKind::USIZE, span);
        out.push(Stmt::Set { place: count, value: self.usize_const(0, span), span });
        // A word of the sign of the counter holds the gap of each two values of
        // a narrower kind, so the count never overflows where the loop does
        // not.
        let word = word_of(kind);
        let widen = |e: &Expr| node_expr(word, convert_node(e.clone()), span);
        let rounds = |a: &Expr, b: &Expr, l: &Self| {
            let gap =
                node_expr(word, infix_node(InfixOp::Sub, widen(a), widen(b)), span);
            let steps = node_expr(
                word,
                infix_node(InfixOp::Div, gap, widen(&stride_read)),
                span,
            );
            let wide = node_expr(DataKind::USIZE, convert_node(steps), span);
            let one = l.usize_const(1, span);
            let value =
                node_expr(DataKind::USIZE, infix_node(InfixOp::Add, wide, one), span);
            vec![Stmt::Set { place: count, value, span }]
        };
        let then = rounds(&last_read, &first_read, self);
        let els = rounds(&first_read, &last_read, self);
        let set = Stmt::If { cond: up_read.clone(), then, els, span };
        out.push(Stmt::If { cond: valid, then: vec![set], els: Vec::new(), span });
        let head_args = vec![first_read, up_read, stride_read];
        let head = RoundHead::Counted(n, kind);
        self.round_loop(head, head_args, &n.block, count, span, out);
    }

    /// Lower a parallel loop over each cell of a list: a slot of the list,
    /// the count of its cells, and the loop of the calls of its round.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop, over the cells of a list or a slice.
    /// * `out` - The list the statements go to.
    pub(super) fn parallel_each(&mut self, n: &st::EachStmt, out: &mut Vec<Stmt>)
    {
        let span = n.source.span();
        let given = self.kind_of(&n.source);
        let kind = each_list(given, false);
        let source = tern!(
            kind == given,
            self.owned(&n.source, kind),
            self.through(&n.source, kind)
        );
        let list = self.hidden(kind, span);
        self.slot(list).addressed = true;
        out.push(Stmt::Set { place: list, value: source, span });
        // The loop drops the list once each round ended.
        let count = self.hidden(DataKind::USIZE, span);
        let cells = self.each_count(list, kind, span);
        out.push(Stmt::Set { place: count, value: cells, span });
        let at =
            node_expr(kind.pointer_to().as_borrowed(), ExprNode::AddrOf(list), span);
        let head = RoundHead::Each(n, list, kind);
        self.round_loop(head, vec![at], &n.block, count, n.span, out);
        let drop = Stmt::Drop { dest: Dest::Slot(list), kind, span };
        do_if!(kind.holds_text(), out.push(drop));
    }

    /// Lower a parallel loop over each character of text: a slot of the text,
    /// a first pass that keeps where each character starts, and the loop of
    /// the calls of its round, one for each character.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `out` - The list the statements go to.
    pub(super) fn parallel_chars(&mut self, n: &st::EachStmt, out: &mut Vec<Stmt>)
    {
        let span = n.source.span();
        let given = self.kind_of(&n.source);
        let kind = each_list(given, true);
        let source = tern!(
            kind == given,
            self.owned(&n.source, kind),
            self.through(&n.source, kind)
        );
        let text = self.hidden(kind, span);
        self.slot(text).addressed = true;
        out.push(Stmt::Set { place: text, value: source, span });
        let starts_kind = DataKind::seq_of(DataKind::USIZE);
        let starts = self.hidden(starts_kind, span);
        self.slot(starts).addressed = true;
        let blank = node_expr(starts_kind, ExprNode::BlankList, span);
        out.push(Stmt::Set { place: starts, value: blank, span });
        out.push(self.char_starts(text, kind, starts, span));
        let count = self.hidden(DataKind::USIZE, span);
        let chars = self.each_count(starts, starts_kind, span);
        out.push(Stmt::Set { place: count, value: chars, span });
        let at = |p: Place, k: DataKind| {
            node_expr(k.pointer_to().as_borrowed(), ExprNode::AddrOf(p), span)
        };
        let head_args = vec![at(text, kind), at(starts, starts_kind)];
        let head = RoundHead::Chars(n, kind, starts_kind);
        self.round_loop(head, head_args, &n.block, count, n.span, out);
        out.push(Stmt::Drop { dest: Dest::Slot(starts), kind: starts_kind, span });
        do_if!(
            kind.holds_text(),
            out.push(Stmt::Drop { dest: Dest::Slot(text), kind, span })
        );
    }

    /// The first pass of a parallel loop over characters: a loop that adds
    /// where each character of the text starts to a list.
    ///
    /// # Arguments
    ///
    /// * `text` - The slot of the text.
    /// * `kind` - Its kind.
    /// * `starts` - The slot of the list, blank before.
    /// * `span` - The source of the loop.
    ///
    /// # Returns
    ///
    /// The loop.
    fn char_starts(
        &mut self, text: Place, kind: DataKind, starts: Place, span: Span,
    ) -> Stmt
    {
        let at = self.hidden(DataKind::USIZE, span);
        let at_read = read(at, DataKind::USIZE, span);
        let count = self.each_count(text, kind, span);
        let cond = node_expr(
            DataKind::Bool,
            compare_node(CmpOp::Lt, at_read.clone(), count),
            span,
        );
        let (value, cell) = each_char(text, kind, at, span);
        let c = self.hidden(cell, span);
        let starts_kind = self.slot_kind(starts);
        let add = Stmt::AddCell {
            dest: Dest::Slot(starts),
            kind: starts_kind,
            value: at_read.clone(),
            span,
        };
        let width = char_width(read(c, cell, span));
        let next =
            node_expr(DataKind::USIZE, infix_node(InfixOp::Add, at_read, width), span);
        Stmt::Loop {
            init: vec![Stmt::Set { place: at, value: self.usize_const(0, span), span }],
            cond,
            update: vec![Stmt::Set { place: at, value: next, span }],
            block: vec![Stmt::Set { place: c, value, span }, add],
            span,
            par: None,
        }
    }

    /// Lower a parallel loop over each row of a ဇယား: a slot of the ဇယား, and
    /// the loop of the calls of its round, one for each room for a row; a
    /// round of a removed row ends at once.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `out` - The list the statements go to.
    pub(super) fn parallel_rows(&mut self, n: &st::EachStmt, out: &mut Vec<Stmt>)
    {
        let kind = self.kind_of(&n.source);
        let span = n.source.span();
        let source = self.owned(&n.source, kind);
        let table = self.hidden(kind, span);
        self.slot(table).addressed = true;
        out.push(Stmt::Set { place: table, value: source, span });
        let count = self.hidden(DataKind::USIZE, span);
        let rows = self.row_slots(table, span);
        out.push(Stmt::Set { place: count, value: rows, span });
        let at =
            node_expr(kind.pointer_to().as_borrowed(), ExprNode::AddrOf(table), span);
        let head = RoundHead::Rows(n, kind);
        self.round_loop(head, vec![at], &n.block, count, n.span, out);
        do_if!(
            kind.holds_text(),
            out.push(Stmt::Drop { dest: Dest::Slot(table), kind, span })
        );
    }

    /// The loop of the calls of a round, marked for a native build to run at
    /// once, after a block of its own for each plain list a round writes.
    ///
    /// # Arguments
    ///
    /// * `head` - What a round takes from its loop.
    /// * `head_args` - The values of it, the same for each round.
    /// * `block` - The statements of a round.
    /// * `count` - The slot of the count of rounds.
    /// * `span` - The loop.
    /// * `out` - The list the statements go to.
    fn round_loop(
        &mut self, head: RoundHead<'_>, head_args: Vec<Expr>, block: &st::BlockStmt,
        count: Place, span: Span, out: &mut Vec<Stmt>,
    )
    {
        let threads = head.threads().map(|t| self.thread_count(t, out));
        let free = self.round_names(block);
        out.extend(
            free.iter().filter_map(|(name, p)| self.own_before(block, *name, *p, span)),
        );
        let shared: Vec<Expr> =
            free.iter().map(|(_, p)| self.round_arg(*p, span)).collect();
        let args: Vec<Expr> = head_args.into_iter().chain(shared).collect();
        let func = self.round_fn(head, &args, &free, block, span);
        let at = self.hidden(DataKind::USIZE, span);
        let at_read = read(at, DataKind::USIZE, span);
        let zero = self.usize_const(0, span);
        let one = self.usize_const(1, span);
        let cond = compare_node(
            CmpOp::Lt,
            at_read.clone(),
            read(count, DataKind::USIZE, span),
        );
        let next = node_expr(
            DataKind::USIZE,
            infix_node(InfixOp::Add, at_read.clone(), one),
            span,
        );
        let call_args = std::iter::once(at_read).chain(args.iter().cloned()).collect();
        let name = self.round_name(func);
        let call = Stmt::Call { func, name, args: call_args, span };
        let par = Par::Rounds(ParRounds { func, count, args, threads });
        out.push(Stmt::Loop {
            init: vec![Stmt::Set { place: at, value: zero, span }],
            cond: node_expr(DataKind::Bool, cond, span),
            update: vec![Stmt::Set { place: at, value: next, span }],
            block: vec![call],
            span,
            par: Some(Box::new(par)),
        });
    }

    /// The count of threads of a parallel loop or map in a `usize` slot, after
    /// a stop of the program for zero or below.
    ///
    /// # Arguments
    ///
    /// * `threads` - The expression of the count, an integer.
    /// * `out` - The list the statements go to.
    ///
    /// # Returns
    ///
    /// The slot.
    pub(super) fn thread_count(
        &mut self, threads: &ast::Expr, out: &mut Vec<Stmt>,
    ) -> Place
    {
        let span = threads.span();
        let kind = self.kind_of(threads);
        let given = self.hidden(kind, span);
        let value = self.coerce(threads, kind);
        out.push(Stmt::Set { place: given, value, span });
        let zero = node_expr(kind, convert_node(self.usize_const(0, span)), span);
        let bad = compare_node(CmpOp::Le, read(given, kind, span), zero);
        let halt =
            Stmt::Halt { cause: HaltCause::ThreadsNotPositive, value: None, span };
        let cond = node_expr(DataKind::Bool, bad, span);
        out.push(Stmt::If { cond, then: vec![halt], els: Vec::new(), span });
        let count = self.hidden(DataKind::USIZE, span);
        let wide =
            node_expr(DataKind::USIZE, convert_node(read(given, kind, span)), span);
        out.push(Stmt::Set { place: count, value: wide, span });
        count
    }

    /// The variables from outside the loop that a round names, each with its
    /// slot: the locals of the function around it, and the hidden variable of
    /// the closure of a function declared inside it. A name the round
    /// declares again takes a slot it does not use.
    ///
    /// # Arguments
    ///
    /// * `block` - The statements of a round.
    ///
    /// # Returns
    ///
    /// The names and slots, each once, in the order of their first use.
    fn round_names(&self, block: &st::BlockStmt) -> Vec<(Name, Place)>
    {
        let mut walk = Named(Vec::new());
        walk.visit_block(block);
        let local = |name: &Name| {
            let mut inner = self.scopes.iter().skip(1).rev();
            inner.find_map(|s| s.get(name)).copied().map(|p| (*name, p))
        };
        let closure = |name: &Name| local(&closure_name(name));
        let found = walk.0.iter().flat_map(|n| local(n).into_iter().chain(closure(n)));
        let mut seen: Vec<(Name, Place)> = Vec::new();
        found.for_each(|f| do_if!(!seen.contains(&f), seen.push(f)));
        seen
    }

    /// What the loop passes a round for a variable from outside: the box of
    /// one a closure shares, with a new count, or else its address.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot of the variable.
    /// * `span` - The loop.
    ///
    /// # Returns
    ///
    /// The argument.
    fn round_arg(&mut self, place: Place, span: Span) -> Expr
    {
        let kind = self.slot_kind(place);
        ret_if!(self.is_box(place), dup(read(place, kind, span)));
        let want = self.reached_kind(place).pointer_to().as_borrowed();
        if self.is_edit(place)
        {
            let at = self.edit_addr(place, span);
            return Expr { kind: want, ..at };
        }
        self.slot(place).addressed = true;
        node_expr(want, ExprNode::AddrOf(place), span)
    }

    /// The statement that gives a plain list a round writes a cell of a block
    /// of its own before the loop, so no round copies it while another
    /// writes; `None` for a variable no round writes or one that is no plain
    /// list.
    ///
    /// # Arguments
    ///
    /// * `block` - The statements of a round.
    /// * `name` - The name of the variable.
    /// * `place` - Its slot.
    /// * `span` - The loop.
    ///
    /// # Returns
    ///
    /// The statement.
    fn own_before(
        &mut self, block: &st::BlockStmt, name: Name, place: Place, span: Span,
    ) -> Option<Stmt>
    {
        let kind = self.reached_kind(place);
        let cell = kind.as_seq().filter(|_| !kind.is_text()).map(|s| s.cell)?;
        let mut walk = Written(name, false);
        walk.visit_block(block);
        ret_if!(!walk.1, None);
        let held = node_expr(kind, self.read_name(&name, span), span);
        let list = Box::new(own(held));
        let index = Box::new(self.usize_const(0, span));
        #[rustfmt::skip]
        let node = ExprNode::CellAddr { list, index, len: None, first: 0, checked: true };
        let first = self.hidden(cell.raw_pointer_to(), span);
        Some(Stmt::Set {
            place: first,
            value: node_expr(cell.raw_pointer_to(), node, span),
            span,
        })
    }

    /// The function of a round: it takes the number of the round, what the
    /// loop gives each round, and each variable from outside, and runs the
    /// statements of the round inside a loop of one round, so a continue ends
    /// the round.
    ///
    /// # Arguments
    ///
    /// * `head` - What a round takes from its loop.
    /// * `args` - The arguments after the number of the round.
    /// * `free` - The variables from outside, after the arguments of the head.
    /// * `block` - The statements of a round.
    /// * `span` - The loop.
    ///
    /// # Returns
    ///
    /// The index of the function among the functions of the module.
    fn round_fn(
        &mut self, head: RoundHead<'_>, args: &[Expr], free: &[(Name, Place)],
        block: &st::BlockStmt, span: Span,
    ) -> u32
    {
        let index = self.top_count + self.lifted.len() as u32;
        self.lifted.push(None);
        let boxes: Vec<bool> = free.iter().map(|(_, p)| self.is_box(*p)).collect();
        let heads = args.len() - free.len();
        let saved = self.set_aside();
        let depth = self.scopes.len();
        let boxed = nested::boxed_in_stmts(&block.stmts);
        #[rustfmt::skip]
        let outer = self.frame.replace(Frame { locals: Vec::new(), ret: None, fault: None, depth, edits: HashSet::new(), boxes: HashSet::new(), boxed, write_backs: Vec::new(), owned_ret: false });
        let mut stmts = Vec::new();
        self.scoped(|l| {
            let number =
                l.declare(&hidden_ident("%round", span), DataKind::USIZE, Access::Var);
            let heads: Vec<Place> = (0..heads)
                .map(|i| {
                    l.declare(
                        &hidden_ident(&format!("%head{i}"), span),
                        args[i].kind,
                        Access::Var,
                    )
                })
                .collect();
            for ((name, _), (arg, boxed)) in
                free.iter().zip(args[heads.len()..].iter().zip(&boxes))
            {
                let ident = ast::Ident::builder().name(*name).span(span).build();
                let place = l.declare(&ident, arg.kind, Access::Var);
                let Place::Local(n) = place
                else
                {
                    continue;
                };
                let frame = l.frame.as_mut().expect("the frame of the round is open");
                do_if!(*boxed, {
                    frame.boxes.insert(n);
                });
                do_if!(!*boxed, {
                    frame.edits.insert(n);
                });
            }
            stmts = l.round_stmts(&head, number, &heads, block, span);
            stmts.extend(l.exit_drops(depth, span));
            stmts.push(Stmt::Return { value: None, span });
        });
        let frame =
            std::mem::replace(&mut self.frame, outer).expect("the frame is set");
        self.restore(saved);
        #[rustfmt::skip]
        let func = Func::builder().name(self.round_name(index))
            .params(args.len() as u32 + 1).locals(frame.locals).block(stmts).span(span).build();
        self.lifted[(index - self.top_count) as usize] = Some(func);
        index
    }

    /// The statements of the function of a round: a loop of one round, whose
    /// block sets the variables of the loop and runs the statements.
    ///
    /// # Arguments
    ///
    /// * `head` - What a round takes from its loop.
    /// * `number` - The parameter of the number of the round.
    /// * `heads` - The parameters of what the loop gives each round.
    /// * `block` - The statements of a round.
    /// * `span` - The loop.
    ///
    /// # Returns
    ///
    /// The statements.
    fn round_stmts(
        &mut self, head: &RoundHead<'_>, number: Place, heads: &[Place],
        block: &st::BlockStmt, span: Span,
    ) -> Vec<Stmt>
    {
        let first = self.hidden(DataKind::Bool, span);
        let on = node_expr(DataKind::Bool, ExprNode::Const(Value::Bool(true)), span);
        let no = node_expr(DataKind::Bool, ExprNode::Const(Value::Bool(false)), span);
        let mut inner = vec![Stmt::Set { place: first, value: no, span }];
        match head
        {
            RoundHead::Counted(n, kind) =>
            {
                inner.extend(self.counted_round(n, *kind, number, heads, block));
            }
            RoundHead::Each(n, _, kind) =>
            {
                inner.extend(self.each_round_of(n, *kind, number, heads[0], span));
            }
            RoundHead::Chars(n, kind, starts_kind) =>
            {
                let parts = (*kind, *starts_kind);
                inner.extend(self.char_round_of(n, parts, number, heads, span));
            }
            RoundHead::Rows(n, kind) =>
            {
                let table = self.lent_through(heads[0], *kind, span, &mut inner);
                inner.extend(self.row_round(n, table, number));
            }
        }
        vec![Stmt::Loop {
            init: vec![Stmt::Set { place: first, value: on, span }],
            cond: read(first, DataKind::Bool, span),
            update: Vec::new(),
            block: inner,
            span,
            par: None,
        }]
    }

    /// The block of a round of a counted loop: the variable of the loop, the
    /// first value plus the number of the round times the step, or minus it
    /// counting down, then the statements.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `kind` - The kind of its variable.
    /// * `number` - The parameter of the number of the round.
    /// * `heads` - The parameters of the first value, of the direction and of
    ///   the step.
    /// * `block` - The statements of a round.
    ///
    /// # Returns
    ///
    /// The statements.
    fn counted_round(
        &mut self, n: &st::CountedStmt, kind: DataKind, number: Place, heads: &[Place],
        block: &st::BlockStmt,
    ) -> Vec<Stmt>
    {
        let span = n.var.span;
        let mut out = Vec::new();
        self.loops.push(self.scopes.len());
        self.scoped(|l| {
            let var = l.declare(&n.var, kind, Access::LoopVar);
            // The value comes to be in a word, as the count does; it lies
            // between the bounds, so it fits the counter.
            let word = word_of(kind);
            let widen = |e: Expr| node_expr(word, convert_node(e), span);
            let r = widen(read(number, DataKind::USIZE, span));
            let stride = widen(read(heads[2], kind, span));
            let step = node_expr(word, infix_node(InfixOp::Mul, r, stride), span);
            let first = widen(read(heads[0], kind, span));
            let near = |op: InfixOp| {
                let value =
                    node_expr(word, infix_node(op, first.clone(), step.clone()), span);
                node_expr(kind, convert_node(value), span)
            };
            let (up, down) = (near(InfixOp::Add), near(InfixOp::Sub));
            let up_test = read(heads[1], DataKind::Bool, span);
            out.push(Stmt::If {
                cond: up_test,
                then: vec![Stmt::Set { place: var, value: up, span }],
                els: vec![Stmt::Set { place: var, value: down, span }],
                span,
            });
            do_for!(s in &block.stmts, l.stmt(s, &mut out));
            let leaves = out.last().is_some_and(leaves_scope);
            let end = l.drops_from(l.drops.len() - 1, block.span);
            do_if!(!leaves, out.extend(end));
        });
        self.loops.pop();
        out
    }

    /// The block of a round of a loop over the cells of a list: the list, lent
    /// through its address, and the cell of the round, then the statements.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `kind` - The kind of the list.
    /// * `number` - The parameter of the number of the round.
    /// * `at` - The parameter of the address of the list.
    /// * `span` - The source of the loop.
    ///
    /// # Returns
    ///
    /// The statements.
    fn each_round_of(
        &mut self, n: &st::EachStmt, kind: DataKind, number: Place, at: Place,
        span: Span,
    ) -> Vec<Stmt>
    {
        let list = self.hidden(kind, span);
        let addr = Box::new(read(at, kind.pointer_to().as_borrowed(), span));
        let load = node_expr(kind, ExprNode::Load { addr }, span);
        let lent = ExprNode::Dup { value: Box::new(load), lent: true, deep: false };
        let mut out =
            vec![Stmt::Set { place: list, value: node_expr(kind, lent, span), span }];
        let cell = self.hidden(DataKind::USIZE, span);
        out.push(Stmt::Set {
            place: cell,
            value: read(number, DataKind::USIZE, span),
            span,
        });
        let (round, _) = self.each_round(n, list, cell, None);
        out.extend(round);
        out
    }

    /// A local that holds a lent view of the value at an address the loop
    /// gives the round, which no drop lets go of.
    ///
    /// # Arguments
    ///
    /// * `at` - The parameter of the address.
    /// * `kind` - The kind of the value.
    /// * `span` - The source of the loop.
    /// * `out` - The statements so far, which get the set.
    ///
    /// # Returns
    ///
    /// The slot of the view.
    fn lent_through(
        &mut self, at: Place, kind: DataKind, span: Span, out: &mut Vec<Stmt>,
    ) -> Place
    {
        let view = self.hidden(kind, span);
        let addr = Box::new(read(at, kind.pointer_to().as_borrowed(), span));
        let load = node_expr(kind, ExprNode::Load { addr }, span);
        let lent = ExprNode::Dup { value: Box::new(load), lent: true, deep: false };
        out.push(Stmt::Set { place: view, value: node_expr(kind, lent, span), span });
        view
    }

    /// The block of a round of a loop over the characters of text: the text
    /// and the list of starts, lent through their addresses, where the
    /// character of the round starts, then the round as an each loop runs it.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `(kind, starts_kind)` - The kinds of the text and of the list of
    ///   starts.
    /// * `number` - The parameter of the number of the round.
    /// * `heads` - The parameters of the addresses of the text and the list.
    /// * `span` - The source of the loop.
    ///
    /// # Returns
    ///
    /// The statements.
    fn char_round_of(
        &mut self, n: &st::EachStmt, (kind, starts_kind): (DataKind, DataKind),
        number: Place, heads: &[Place], span: Span,
    ) -> Vec<Stmt>
    {
        let mut out = Vec::new();
        let text = self.lent_through(heads[0], kind, span, &mut out);
        let starts = self.lent_through(heads[1], starts_kind, span, &mut out);
        let (start, _) = each_cell(starts, starts_kind, number, span);
        let at = self.hidden(DataKind::USIZE, span);
        out.push(Stmt::Set { place: at, value: start, span });
        let nth = self.hidden(DataKind::USIZE, span);
        out.push(Stmt::Set {
            place: nth,
            value: read(number, DataKind::USIZE, span),
            span,
        });
        let (round, _) = self.each_round(n, text, at, Some(nth));
        out.extend(round);
        out
    }

    /// The name of the function of a round, as a call names it.
    ///
    /// # Arguments
    ///
    /// * `func` - Its index.
    ///
    /// # Returns
    ///
    /// The name.
    fn round_name(&self, func: u32) -> String
    {
        // A trap shows the name before `#`: the function the loop is in.
        format!("{}#round{func}{}", self.current, self.suffix)
    }
}

/// The word of the sign of a counter: `isize` for a signed one, `usize` for
/// an unsigned one.
///
/// # Arguments
///
/// * `kind` - The kind of the counter.
///
/// # Returns
///
/// The word.
fn word_of(kind: DataKind) -> DataKind
{
    let signed = kind.int_kind().is_none_or(|i| i.signed);
    tern!(signed, DataKind::ISIZE, DataKind::USIZE)
}

/// A hidden name for a slot of the function of a round.
///
/// # Arguments
///
/// * `name` - The name, which starts with `%`.
/// * `span` - The loop.
///
/// # Returns
///
/// The name.
fn hidden_ident(name: &str, span: Span) -> ast::Ident
{
    ast::Ident::builder().name(name).span(span).build()
}

/// A conversion of a number to the kind of the expression that holds it.
///
/// # Arguments
///
/// * `operand` - The number.
///
/// # Returns
///
/// The conversion.
fn convert_node(operand: Expr) -> ExprNode
{
    ExprNode::Convert { operand: Box::new(operand), implicit: false }
}

/// The walker that gathers each name the statements of a round use, as a
/// value or as the function of a call.
struct Named(Vec<Name>);

impl Visitor for Named
{
    fn visit_name(&mut self, n: &zakac_ast::expr::NameExpr)
    {
        self.0.push(n.name);
        walk_name(self, n);
    }

    fn visit_call_expr(&mut self, n: &zakac_ast::expr::CallExpr)
    {
        self.0.push(n.callee.name);
        walk_call_expr(self, n);
    }

    fn visit_call_stmt(&mut self, n: &st::CallStmt)
    {
        self.0.push(n.callee.name);
        zakac_ast::visit::walk_call_stmt(self, n);
    }
}

/// The walker that finds whether the statements of a round write through a
/// variable: an assignment, an add or a remove whose place starts at it.
struct Written(Name, bool);

impl Written
{
    /// Note a place written.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    fn note(&mut self, place: &ast::Expr)
    {
        self.1 |= place_root(place).is_some_and(|n| n == self.0.as_str());
    }
}

impl Visitor for Written
{
    fn visit_assign(&mut self, n: &st::AssignStmt)
    {
        self.note(&n.target);
        zakac_ast::visit::walk_assign(self, n);
    }
}
