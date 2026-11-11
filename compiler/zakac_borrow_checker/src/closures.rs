//! The closures of a routine. A closure is the function value of a function
//! declared inside a function with captures: its thunk calls that function
//! with the box of each variable it captures, which the record holds. The
//! routine fills the record in a slot, and the closure moves it to the heap.
//! The flow sees what the function does through the boxes at each call that
//! can run it: a call that gets a closure as an argument, which can run it or
//! not, and a call through a slot that holds one. A box is counted, so the
//! closure and the routine share it, and a free through it reaches the block
//! the captured variable holds.
//!
//! This pass finds, once for the routine, the function each closure runs and
//! the boxes it passes; which closures a slot holds at each node is a fact of
//! the flow ([`State::closures`]), which each write of the slot sets.

use std::collections::BTreeSet;
use std::collections::HashMap;

use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_ir_builder::value::Value;

use crate::state::State;
use crate::walk::each_expr;
use crate::walk::each_stmt;
use crate::walk::stmt_exprs;

/// The closures of a routine.
#[derive(Debug, Default)]
pub(crate) struct Closures
{
    /// The borrows each such function takes after its declared
    /// parameters, as the routine makes its closures.
    borrows: HashMap<u32, Vec<Expr>>,
    /// The fields of the record of each such function, after the code and
    /// info words, as the routine fills them.
    fields: HashMap<u32, Vec<Expr>>,
    /// The function each thunk of the routine's closures calls.
    thunks: HashMap<u32, u32>,
}

impl Closures
{
    /// Find the closures of a routine.
    ///
    /// # Arguments
    ///
    /// * `module` - The module.
    /// * `block` - The statements of the routine.
    ///
    /// # Returns
    ///
    /// The closures.
    pub(crate) fn of(module: &Module, block: &[Stmt]) -> Closures
    {
        let mut records: HashMap<Place, Vec<Expr>> = HashMap::new();
        each_stmt(block, &mut |s| {
            let Stmt::Set { place, value, .. } = s
            else
            {
                return;
            };
            let ExprNode::List(cells) = &value.node
            else
            {
                return;
            };
            records.insert(*place, cells.iter().skip(2).cloned().collect());
        });
        let mut found = Closures::default();
        each_stmt(block, &mut |s| {
            let mut note = |e: &Expr| found.note_record(module, &records, e);
            stmt_exprs(s).into_iter().for_each(|e| each_expr(e, &mut note));
        });
        found
    }

    /// Keep the borrows of the function a closure runs.
    ///
    /// # Arguments
    ///
    /// * `module` - The module.
    /// * `records` - The cells each slot is set to, after the first word.
    /// * `e` - An expression; a closure is noted.
    fn note_record(
        &mut self, module: &Module, records: &HashMap<Place, Vec<Expr>>, e: &Expr,
    )
    {
        let ExprNode::Closure { func, record, .. } = &e.node
        else
        {
            return;
        };
        let (Some(target), ExprNode::AddrOf(slot)) =
            (lifted_of(module, *func), &record.node)
        else
        {
            return;
        };
        let cells = records.get(slot).cloned().unwrap_or_default();
        let borrows = lent_fields(module, target, &cells);
        self.borrows.entry(target).or_insert(borrows);
        self.fields.entry(target).or_insert(cells);
        self.thunks.insert(*func, target);
    }

    /// The functions declared inside a function whose closures a value can
    /// be.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    /// * `state` - The facts where it is read.
    ///
    /// # Returns
    ///
    /// The function numbers; none for a value that holds no closure.
    pub(crate) fn held_in(&self, e: &Expr, state: &State) -> BTreeSet<u32>
    {
        match &e.node
        {
            ExprNode::Closure { func, .. } =>
            {
                self.thunks.get(func).copied().into_iter().collect()
            }
            ExprNode::Read(place) =>
            {
                state.closures.get(place).cloned().unwrap_or_default()
            }
            ExprNode::Convert { operand: value, .. } | ExprNode::Dup { value, .. } =>
            {
                self.held_in(value, state)
            }
            ExprNode::Take { addr } => match addr.node
            {
                ExprNode::AddrOf(place) =>
                {
                    state.closures.get(&place).cloned().unwrap_or_default()
                }
                _ => BTreeSet::new(),
            },
            _ => BTreeSet::new(),
        }
    }

    /// The closures the record of a function declared inside a function can
    /// hold: a duplicate, a move or a borrow of a slot that holds one.
    ///
    /// # Arguments
    ///
    /// * `func` - The function.
    /// * `state` - The facts where the closure runs.
    ///
    /// # Returns
    ///
    /// The function numbers.
    pub(crate) fn inner(&self, func: u32, state: &State) -> BTreeSet<u32>
    {
        let fields = self.fields.get(&func).map(Vec::as_slice).unwrap_or_default();
        let through = |e: &Expr| match &e.node
        {
            ExprNode::AddrOf(p) => state.closures.get(p).cloned().unwrap_or_default(),
            _ => self.held_in(e, state),
        };
        fields.iter().flat_map(through).collect()
    }

    /// The borrows a function declared inside a function takes after its
    /// declared parameters, as the routine makes its closures.
    ///
    /// # Arguments
    ///
    /// * `func` - The function.
    ///
    /// # Returns
    ///
    /// The borrows, or `None` when the routine makes no closure of it.
    pub(crate) fn borrows(&self, func: u32) -> Option<&[Expr]>
    {
        self.borrows.get(&func).map(Vec::as_slice)
    }
}

/// The function a thunk calls: the one call of its block.
///
/// # Arguments
///
/// * `module` - The module.
/// * `thunk` - The thunk.
///
/// # Returns
///
/// The function number, or `None` when the thunk calls none.
fn lifted_of(module: &Module, thunk: u32) -> Option<u32>
{
    let block = &module.funcs.get(thunk as usize)?.block;
    block.iter().find_map(|s| match s
    {
        Stmt::Call { func, .. } => Some(*func),
        Stmt::Set { value: Expr { node: ExprNode::Call { func, .. }, .. }, .. } =>
        {
            Some(*func)
        }
        _ => None,
    })
}

/// The borrow each field of a closure record passes to the hidden
/// parameter of its capture: a field of the parameter's kind is a borrow of a
/// local, and each other field holds a value of the record, which no local
/// reaches, so it passes null.
///
/// # Arguments
///
/// * `module` - The module.
/// * `func` - The function the closure runs.
/// * `cells` - The fields of the record after the code and info words.
///
/// # Returns
///
/// The borrows, one for each capture.
fn lent_fields(module: &Module, func: u32, cells: &[Expr]) -> Vec<Expr>
{
    let Some(f) = module.funcs.get(func as usize)
    else
    {
        return Vec::new();
    };
    let first = (f.params as usize).saturating_sub(cells.len());
    let want = |i: usize| f.locals.get(first + i).map(|s| s.kind);
    let field = |(i, c): (usize, &Expr)| match want(i)
    {
        Some(k) if k == c.kind.repr() => c.clone(),
        k =>
        {
            let kind = k.unwrap_or(c.kind);
            Expr::builder()
                .kind(kind)
                .node(ExprNode::Const(Value::NULL))
                .span(c.span)
                .build()
        }
    };
    cells.iter().enumerate().map(field).collect()
}
