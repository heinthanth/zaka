use std::collections::HashSet;

use zakac_ast::expr::BlankListLit;
use zakac_ast::expr::Expr;
use zakac_ast::expr::FieldExpr;
use zakac_ast::expr::IndexExpr;
use zakac_ast::expr::ListLit;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::RangeExpr;
use zakac_ast::expr::StructLit;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::ownership as od;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;

use crate::kind::DataKind;
use crate::place::place_of;
use crate::semcheck::Checker;
use crate::semcheck::mismatch_report;
use crate::struct_check::unknown_name;

/// Where a written kind stands, which decides whether its list counts can
/// be left out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KindUse
{
    /// A declaration; the value can give the counts.
    Decl,
    /// A function result, which cannot be a list of borrows.
    Result,
    /// Each other place: a parameter, a size, a heap unit or a conversion.
    Other,
}

/// The kind a cell operand gives its cells from: the list or slice itself,
/// or the list a pointer points at.
///
/// # Arguments
///
/// * `kind` - The kind of the operand.
///
/// # Returns
///
/// The list or slice kind, or `None` for a kind that is none of these.
pub(crate) fn list_through(kind: DataKind) -> Option<DataKind>
{
    ret_if!(kind.is_list() || kind.is_slice(), Some(kind));
    kind.pointee().filter(|k| k.is_list())
}

/// Whether a kind holds a slice whose cells hold borrows, at once or in the
/// cells of a list or a slice.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for such a kind.
fn has_slice_of_borrows(kind: DataKind) -> bool
{
    ret_if!(let Some(s) = kind.as_slice(), s.cell.holds_borrow());
    kind.as_list().is_some_and(|l| has_slice_of_borrows(l.cell))
}

/// Fill the open counts of `kind` from the shape of a list literal.
///
/// # Arguments
///
/// * `kind` - The written kind, with counts of 0 still open.
/// * `e` - The value.
///
/// # Returns
///
/// The kind with each count filled, or `None` when `e` is no literal that
/// gives them.
fn shape_of(kind: DataKind, e: &Expr) -> Option<DataKind>
{
    ret_if!(!kind.has_open_count(), Some(kind));
    let lit = match e
    {
        Expr::Group(n) => return shape_of(kind, &n.inner),
        Expr::List(lit) => lit,
        _ => return None,
    };
    let l = kind.as_list()?;
    let count = u32::try_from(lit.cells.len()).ok()?;
    let len = tern!(l.len == 0, count, l.len);
    Some(DataKind::list_of(shape_of(l.cell, lit.cells.first()?)?, len))
}

/// Whether a value is a blank list, perhaps in parentheses.
///
/// # Arguments
///
/// * `e` - The value.
///
/// # Returns
///
/// `true` for `စာရင်းအလွတ်`.
fn is_blank(e: &Expr) -> bool
{
    match e
    {
        Expr::Group(n) => is_blank(&n.inner),
        Expr::BlankList(_) => true,
        _ => false,
    }
}

/// Whether the borrow markers of a written kind sit where none is allowed:
/// more than one, or one whose pointer level is raw or missing. The marker
/// marks the outer pointer level, so a list of borrows keeps it.
///
/// # Arguments
///
/// * `written` - The kind phrase.
///
/// # Returns
///
/// `true` for a misplaced marker.
fn bad_marker(written: &Kind) -> bool
{
    ret_true!(written.extra_borrows > 0);
    let is_level = |w: &&Wrap| matches!(w, Wrap::Ptr | Wrap::RawPtr);
    let marked = written.wraps.iter().rev().find(is_level);
    written.borrowed && marked != Some(&Wrap::Ptr)
}

/// Whether a written kind leaves a list count out behind an address level,
/// as `ကိန်းပြည့် စာရင်း လိပ်စာ` does: the value of a declaration fills
/// the counts of its own list levels alone, never those of a list it
/// points at.
///
/// # Arguments
///
/// * `written` - The kind phrase.
///
/// # Returns
///
/// `true` when an open list level has an address level outside it.
fn open_behind_address(written: &Kind) -> bool
{
    let open = written.wraps.iter().position(|w| *w == Wrap::List(Cells::Inferred));
    let is_level = |w: &Wrap| matches!(w, Wrap::Ptr | Wrap::RawPtr);
    let address = written.wraps.iter().rposition(is_level);
    matches!((open, address), (Some(o), Some(a)) if o < a)
}

impl Checker
{
    /// The data kind of a written kind, reporting a list too large for the
    /// machine, a count left out where nothing gives it, a list of borrows
    /// as a function result, and a slice whose cells hold borrows.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind phrase.
    /// * `at` - Where it stands.
    ///
    /// # Returns
    ///
    /// The kind; it still has open counts for a declaration.
    pub(crate) fn written_kind(&mut self, written: &Kind, at: KindUse) -> DataKind
    {
        let kind = DataKind::from_ast(written, &self.structs);
        let span = written.span;
        let names = self.structs.keys().map(String::as_str).collect();
        do_if!(let Some(d) = unknown_name(written, &names), self.report(d));
        let large = kind.checked_size(self.platform).is_none();
        do_if!(large, self.report(kd::list_too_large(span, &kind.name())));
        let open = at != KindUse::Decl && written.has_inferred_cells();
        do_if!(open, self.report(kd::list_count_missing(span)));
        let result = at == KindUse::Result && kind.is_list() && kind.holds_borrow();
        do_if!(result, self.report(kd::borrow_list_result(span)));
        let nested = has_slice_of_borrows(kind);
        do_if!(nested, self.report(kd::slice_of_borrows(span, &kind.name())));
        do_if!(bad_marker(written), self.report(od::bad_borrow_marker(span)));
        let raw = self.no_raw && written.has_raw();
        do_if!(raw, self.report(od::raw_not_allowed(span)));
        kind
    }

    /// Check the value of a declaration against its written kind, filling
    /// list counts the kind leaves out from the value. A count left out
    /// behind an address level is refused: no value fills it.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind phrase.
    /// * `init` - The value, when one is written.
    ///
    /// # Returns
    ///
    /// The kind of the declared name, or `None` when its counts remain open.
    pub(crate) fn decl_kind(
        &mut self, written: &Kind, init: Option<&Expr>,
    ) -> Option<DataKind>
    {
        let kind = self.written_kind(written, KindUse::Decl);
        if open_behind_address(written)
        {
            self.report(kd::list_count_missing(written.span));
            do_if!(let Some(init) = init, { self.infer(init, None); });
            return None;
        }
        if !kind.has_open_count()
        {
            do_if!(let Some(init) = init, self.expect_kind(init, Some(kind)));
            return Some(kind);
        }
        let Some(init) = init.filter(|e| !is_blank(e))
        else
        {
            self.report(kd::list_count_missing(written.span));
            return None;
        };
        self.fill_counts(kind, init)
    }

    /// Fill the open counts of a declared kind from its value, then check
    /// the value against the filled kind like each other initializer.
    ///
    /// # Arguments
    ///
    /// * `kind` - The written kind, with counts still open.
    /// * `init` - The value.
    ///
    /// # Returns
    ///
    /// The filled kind, or `None` after an error.
    fn fill_counts(&mut self, kind: DataKind, init: &Expr) -> Option<DataKind>
    {
        if let Some(shaped) = shape_of(kind, init)
        {
            self.expect_kind(init, Some(shaped));
            return Some(shaped);
        }
        let found = self.infer(init, None)?;
        let filled = kind.fill_counts(found);
        let mismatch = mismatch_report(init.span(), kind, found);
        do_if!(filled.is_none(), self.report(mismatch));
        do_if!(let Some(filled) = filled, self.expect_fit(init, found, filled));
        filled
    }

    /// Infer a list literal. With a list kind expected, each cell takes the
    /// cell kind; else the cells share the kind each widens to.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The list kind, or `None` after an error.
    pub(crate) fn infer_list(
        &mut self, n: &ListLit, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let count = u32::try_from(n.cells.len()).ok()?;
        let Some(want) = expect.and_then(DataKind::as_list)
        else
        {
            let cell = self.common_cell(&n.cells)?;
            return Some(DataKind::list_of(cell, count));
        };
        do_for!(c in &n.cells, self.expect_kind(c, Some(want.cell)));
        let wrong = want.len != 0 && want.len != count;
        let mismatch = kd::list_count_mismatch(n.span, want.len, n.cells.len());
        ret_if!(wrong, self.report(mismatch), None);
        Some(DataKind::list_of(want.cell, count))
    }

    /// The kind the cells of a literal with no expected kind share.
    ///
    /// # Arguments
    ///
    /// * `cells` - The cells; never blank.
    ///
    /// # Returns
    ///
    /// The kind each cell widens to, or `None` after an error.
    fn common_cell(&mut self, cells: &[Expr]) -> Option<DataKind>
    {
        let kinds: Vec<Option<DataKind>> =
            cells.iter().map(|c| self.infer(c, None)).collect();
        let kinds: Vec<DataKind> = kinds.into_iter().collect::<Option<_>>()?;
        let mut common = kinds[0];
        for (cell, kind) in cells.iter().zip(&kinds).skip(1)
        {
            common = self.common_or_report(common, *kind, cell.span())?;
        }
        Some(common)
    }

    /// Infer a blank list: it takes the list kind its place expects.
    ///
    /// # Arguments
    ///
    /// * `n` - The blank list.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The expected list kind, or `None` when none is known.
    pub(crate) fn infer_blank(
        &mut self, n: &BlankListLit, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let want = expect.filter(|k| k.is_list() && !k.has_open_count());
        do_if!(want.is_none(), self.report(kd::blank_list_kind(n.span)));
        want
    }

    /// Infer one cell of a list or a slice, or of the list a pointer points
    /// at.
    ///
    /// # Arguments
    ///
    /// * `n` - The cell expression.
    ///
    /// # Returns
    ///
    /// The cell kind, or `None` after an error.
    pub(crate) fn infer_index(&mut self, n: &IndexExpr) -> Option<DataKind>
    {
        let list = self.infer(&n.list, None);
        let index = self.infer(&n.index, Some(DataKind::USIZE));
        let bad = index.filter(|k| !k.is_int());
        let span = n.index.span();
        do_if!(let Some(k) = bad, self.report(kd::cell_number_not_int(span, &k.name())));
        let cell = self.list_of_operand(&n.list, list?)?.cell_kind()?;
        tern!(bad.is_some(), None, Some(cell))
    }

    /// Infer a range of cells: a slice of `len` cells from `start` on, of a
    /// list, a slice, or the block a pointer holds.
    ///
    /// # Arguments
    ///
    /// * `n` - The range.
    ///
    /// # Returns
    ///
    /// The slice kind, or `None` after an error.
    pub(crate) fn infer_range(&mut self, n: &RangeExpr) -> Option<DataKind>
    {
        let source = self.infer(&n.source, None);
        let numbers =
            [&n.start, &n.len].map(|e| (e, self.infer(e, Some(DataKind::USIZE))));
        let bad: Vec<_> =
            numbers.iter().filter(|(_, k)| k.is_some_and(|k| !k.is_int())).collect();
        for (e, k) in &bad
        {
            let found = k.map(DataKind::name).unwrap_or_default();
            self.report(kd::cell_number_not_int(e.span(), &found));
        }
        let cell = self.range_cells(&n.source, source?)?;
        let slice = DataKind::slice_of(cell);
        let span = n.span;
        ret_if!(
            cell.holds_borrow(),
            self.report(kd::slice_of_borrows(span, &slice.name())),
            None
        );
        tern!(
            bad.is_empty() && numbers.iter().all(|(_, k)| k.is_some()),
            Some(slice),
            None
        )
    }

    /// The cell kind a range reads from its source, reporting a source that
    /// is no list, no slice and no pointer to a block. A list must be a
    /// place, since the slice points into it.
    ///
    /// # Arguments
    ///
    /// * `source` - The source.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The cell kind, or `None` after the report.
    fn range_cells(&mut self, source: &Expr, kind: DataKind) -> Option<DataKind>
    {
        if let Some(cells) = list_through(kind)
        {
            ret_if!(kind.is_list() && !self.addressable(source), None);
            return cells.cell_kind();
        }
        let span = source.span();
        ret_if!(kind.is_raw_ptr(), self.report(od::lend_of_raw(span)), None);
        let pointee = kind.pointee().filter(|_| kind.is_checked_ptr());
        do_if!(pointee.is_none(), self.report(kd::not_a_list(span, &kind.name())));
        pointee
    }

    /// Lend each cell of a list as a slice. The list must be a place.
    ///
    /// # Arguments
    ///
    /// * `operand` - The list.
    /// * `cell` - The kind of its cells.
    ///
    /// # Returns
    ///
    /// The slice kind, or `None` after an error.
    pub(crate) fn lend_list(
        &mut self, operand: &Expr, cell: DataKind,
    ) -> Option<DataKind>
    {
        ret_if!(!self.addressable(operand), None);
        let slice = DataKind::slice_of(cell);
        let span = operand.span();
        ret_if!(
            cell.holds_borrow(),
            self.report(kd::slice_of_borrows(span, &slice.name())),
            None
        );
        Some(slice)
    }

    /// Infer a field of a struct.
    ///
    /// # Arguments
    ///
    /// * `n` - The field expression.
    ///
    /// # Returns
    ///
    /// The field kind, or `None` after an error.
    pub(crate) fn infer_field(&mut self, n: &FieldExpr) -> Option<DataKind>
    {
        let base = self.infer(&n.base, None)?;
        let field = &n.field;
        let Some(s) = base.as_struct()
        else
        {
            self.report(kd::not_a_struct(field.span, &field.name, &base.name()));
            return None;
        };
        let found = s.field(&field.name).map(|(_, kind)| kind);
        do_if!(
            found.is_none(),
            self.report(rd::unknown_field(field.span, &s.name, &field.name))
        );
        found
    }

    /// Infer a struct literal: each field it sets takes the kind of that
    /// field.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    ///
    /// # Returns
    ///
    /// The struct kind, or `None` after an error.
    pub(crate) fn infer_struct_lit(&mut self, n: &StructLit) -> Option<DataKind>
    {
        let name = &n.name;
        let kind = self.structs.get(&name.name).copied();
        do_if!(
            kind.is_none(),
            self.report(rd::unknown_kind_name(name.span, &name.name))
        );
        let s = kind.and_then(DataKind::as_struct);
        let mut seen = HashSet::new();
        for init in &n.fields
        {
            let field = &init.field;
            let twice = !seen.insert(field.name.as_str());
            do_if!(
                twice,
                self.report(rd::field_set_twice(field.span, &name.name, &field.name))
            );
            let want = s.and_then(|s| s.field(&field.name)).map(|(_, k)| k);
            let unknown = s.filter(|_| want.is_none());
            let unknown =
                unknown.map(|s| rd::unknown_field(field.span, &s.name, &field.name));
            do_if!(let Some(d) = unknown, self.report(d));
            self.expect_kind(&init.value, want);
        }
        kind
    }

    /// Infer the cell count of a list or a slice.
    ///
    /// # Arguments
    ///
    /// * `value` - The list, a pointer to one, or a slice.
    ///
    /// # Returns
    ///
    /// `usize`, or `None` after an error.
    pub(crate) fn infer_cell_count(&mut self, value: &Expr) -> Option<DataKind>
    {
        let kind = self.infer(value, None)?;
        self.list_of_operand(value, kind)?;
        Some(DataKind::USIZE)
    }

    /// The list or slice a cell operand gives its cells from, reporting an
    /// operand that is no list, no pointer to one and no slice.
    ///
    /// # Arguments
    ///
    /// * `operand` - The operand.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The list or slice kind, or `None` after the report.
    fn list_of_operand(&mut self, operand: &Expr, kind: DataKind) -> Option<DataKind>
    {
        let list = list_through(kind);
        do_if!(
            list.is_none(),
            self.report(kd::not_a_list(operand.span(), &kind.name()))
        );
        list
    }

    /// The variable a place belongs to: the variable itself, or the list
    /// variable a cell sits in. A cell reached through a pointer belongs to
    /// no variable.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    ///
    /// # Returns
    ///
    /// The variable, or `None`.
    pub(crate) fn owner_name<'a>(&self, place: &'a Expr) -> Option<&'a NameExpr>
    {
        match place
        {
            Expr::Name(n) => Some(n),
            Expr::Group(n) => self.owner_name(&n.inner),
            Expr::Index(n) if self.holds_list(&n.list) => self.owner_name(&n.list),
            Expr::Field(n) => self.owner_name(&n.base),
            _ => None,
        }
    }

    /// Whether a cell sits in a list that is no place, such as a literal or a
    /// call result, so it cannot be written or have its address taken.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    ///
    /// # Returns
    ///
    /// `true` for a cell of a list value with no address.
    pub(crate) fn in_temp_list(&self, place: &Expr) -> bool
    {
        match place
        {
            Expr::Group(n) => self.in_temp_list(&n.inner),
            Expr::Index(n) if self.holds_list(&n.list) =>
            {
                place_of(&n.list).is_none_or(|inner| self.in_temp_list(inner))
            }
            Expr::Field(n) =>
            {
                place_of(&n.base).is_none_or(|inner| self.in_temp_list(inner))
            }
            _ => false,
        }
    }

    /// Whether an expression was inferred to be a list, not a pointer to
    /// one.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// `true` for a list.
    fn holds_list(&self, e: &Expr) -> bool
    {
        self.kinds.get(&e.id()).is_some_and(|k| k.is_list())
    }
}
