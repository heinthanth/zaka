use zakac_diagnostics::ownership::borrow_stored;
use zakac_diagnostics::ownership::freed_borrow;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::value::Value;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::checks::report;
use crate::flow::Flow;
use crate::roots::field_kind;
use crate::roots::list_slot;

/// Report a free, a resize or an allocation whose destination holds a
/// borrow: an owner alone gives its block back or takes a new one.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `dest` - The destination of the heap statement.
/// * `span` - The statement.
///
/// # Returns
///
/// `true` when the destination holds a borrow, so the statement is
/// rejected.
pub(crate) fn check_free_dest(flow: &mut Flow, dest: &Dest, span: Span) -> bool
{
    let kind = match dest
    {
        Dest::Slot(place) => flow.ctx.kind_of(*place),
        Dest::Mem(addr) => value_kind_at(addr),
    };
    let rejected = kind.is_some_and(DataKind::is_borrow);
    do_if!(rejected, report(flow, freed_borrow(span)));
    rejected
}

/// Report a borrow stored through an address: in mem it could outlast
/// the block it points into. A cell of a list slot is that slot, so it
/// follows the rule of a set of the slot.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `addr` - The address written to.
/// * `value` - The value stored.
/// * `span` - The statement.
pub(crate) fn check_stored_borrow(
    flow: &mut Flow, addr: &Expr, value: &Expr, span: Span,
)
{
    ret_if!(!keeps_a_borrow(value));
    let Some(place) = list_slot(addr)
    else
    {
        return report(flow, borrow_stored(span));
    };
    check_global_set(flow, place, value, span);
}

/// Report a borrow a function stores in a global: the global outlasts
/// the call, and the places the borrow points into can die with it. The
/// main program can keep borrows in its globals, since those last for the
/// whole run. Setting it to `ဗလာ` clears it, which cannot leave it
/// pointing at a dead place.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The slot set.
/// * `value` - The value it gets.
/// * `span` - The statement.
pub(crate) fn check_global_set(flow: &mut Flow, place: Place, value: &Expr, span: Span)
{
    ret_if!(flow.routine.is_main || matches!(place, Place::Local(_)));
    do_if!(keeps_a_borrow(value), report(flow, borrow_stored(span)));
}

/// Whether a stored value can keep a borrow: a value of a kind that
/// holds one, save the constant `ဗလာ`, which points nowhere.
///
/// # Arguments
///
/// * `value` - The value.
///
/// # Returns
///
/// `true` when storing the value can keep a borrow.
fn keeps_a_borrow(value: &Expr) -> bool
{
    let cleared = matches!(value.node, ExprNode::Const(Value::NULL));
    kind_holds_borrow(value.kind) && !cleared
}

/// Whether a value of a kind carries a borrow: a borrow, or a list whose
/// cells hold one at some depth.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` when the value cannot go where a borrow cannot live.
pub(crate) fn kind_holds_borrow(kind: DataKind) -> bool { kind.holds_borrow() }

/// The kind of the value an address names. The kind of a cell address
/// drops the borrow mark of the cell, so a cell gets the cell kind of its
/// list; a field address gets the field kind of its struct.
///
/// # Arguments
///
/// * `addr` - The address.
///
/// # Returns
///
/// The kind, or `None` for an address of an opaque value.
fn value_kind_at(addr: &Expr) -> Option<DataKind>
{
    ret_if!(let ExprNode::FieldAddr { base, field, .. } = &addr.node, field_kind(base.kind, *field));
    let ExprNode::CellAddr { list, .. } = &addr.node
    else
    {
        return addr.kind.pointee();
    };
    let list_kind = tern!(list.kind.is_list(), Some(list.kind), list.kind.pointee());
    list_kind?.as_list().map(|l| l.cell)
}

#[cfg(test)]
mod tests
{
    use zakac_diagnostics::Code;
    use zakac_ir_builder::ir::Stmt;

    use crate::flow::run_flow;
    use crate::options::BorrowCheckOptions;
    use crate::roots::global_roots;
    use crate::routine::Routine;
    use crate::testing::lower;

    #[test]
    fn a_borrow_stored_through_a_heap_address_is_zk0703()
    {
        let mut module = lower(
            "ကိန်းရှင် ရ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ပပ သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် လိပ်စာ ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပပ တွင် သိမ်းထားမည်။
ပပ ၏ တန်ဖိုး ကို ရ ဟု ထားပါ။
",
        );
        let Some(Stmt::Store { value, .. }) = module.main.last_mut()
        else
        {
            panic!("the last statement is a store");
        };
        value.kind = value.kind.as_borrowed();
        let roots = global_roots(&module, &[]);
        let options = BorrowCheckOptions::default();
        let found =
            run_flow(&module, Routine::main(&module), &[], &roots, &options, 64);
        let codes: Vec<Code> = found.diags.iter().map(|d| d.code).collect();
        assert_eq!(codes, [Code::BorrowStored]);
    }
}
