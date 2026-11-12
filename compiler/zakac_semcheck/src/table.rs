//! The checks of ဇယား kinds, literals, rows and the loops over them.

use zakac_ast::expr::Expr;
use zakac_ast::expr::TableLit;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::EachPart;
use zakac_ast::stmt::EachStmt;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::kind::DataKind;
use crate::list::KindUse;
use crate::semcheck::Checker;

impl Checker
{
    /// Check each ဇယား kind in a written kind: its index kind must hash and
    /// compare as a whole value, and one that holds a decimal number gets a
    /// warning.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind phrase.
    pub(crate) fn check_table_kinds(&mut self, written: &Kind)
    {
        if let Some(t) = written.table.as_deref()
        {
            let index = DataKind::from_ast(&t.index, &self.structs);
            self.check_index_kind(index, t.index.span);
            self.check_table_kinds(&t.index);
            self.check_table_kinds(&t.value);
        }
        let parts = written.parts.as_deref();
        let sides: Vec<&Kind> =
            parts.into_iter().flat_map(|p| p.ok.iter().chain(&p.err)).collect();
        do_for!(k in sides, self.check_table_kinds(k));
    }

    /// Check an index kind.
    ///
    /// # Arguments
    ///
    /// * `index` - The index kind.
    /// * `span` - Where it comes from.
    pub(crate) fn check_index_kind(&mut self, index: DataKind, span: Span)
    {
        if let Some(part) = index.index_gap()
        {
            self.report(kd::bad_index_kind(span, &index.name(), &part));
            return;
        }
        do_if!(
            index.holds_float(),
            self.report(kd::decimal_index(span, &index.name()))
        );
    }

    /// Infer a ဇယား literal: its written kind, else the ဇယား kind its place
    /// wants, else the kinds of its first row. Each row converts to the
    /// index kind and the value kind.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The ဇယား kind, or `None` after an error.
    pub(crate) fn infer_table(
        &mut self, n: &TableLit, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let written = n.kind.as_ref().map(|k| self.written_kind(k, KindUse::Other));
        let given = written.or(expect.filter(|k| k.is_table()));
        let mut rows = n.rows.iter();
        let kind = match given
        {
            Some(kind) => kind,
            None =>
            {
                let Some(first) = rows.next()
                else
                {
                    self.report(kd::table_needs_kind(n.span));
                    return None;
                };
                let index = self.infer(&first.index, None);
                let value = self.infer(&first.value, None);
                let (index, value) = (index?, value?);
                self.check_index_kind(index, first.index.span());
                DataKind::table(index, value)
            }
        };
        let (index, value) = kind.table_parts()?;
        for row in rows
        {
            self.expect_kind(&row.index, Some(index));
            self.expect_kind(&row.value, Some(value));
        }
        Some(kind)
    }

    /// Infer `<ဇယား> ထဲမှ အညွှန်း <index>` and the read of its value. A read
    /// gives an optional of the value kind; the value a statement writes
    /// through, and the row a remove takes, have the value kind itself.
    ///
    /// # Arguments
    ///
    /// * `n` - The row or the read.
    /// * `table` - The ဇယား.
    /// * `index` - The index.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after an error.
    pub(crate) fn infer_row(
        &mut self, n: &VerbExpr, table: &Expr, index: &Expr,
    ) -> Option<DataKind>
    {
        let kind = self.infer(table, None);
        let Some((at, value)) = kind.and_then(DataKind::table_parts)
        else
        {
            do_if!(let Some(k) = kind, self.report(kd::not_a_table(table.span(), &k.name())));
            _ = self.infer(index, None);
            return None;
        };
        self.expect_kind(index, Some(at));
        let write = n.op == VerbOp::Row || self.row_writes.contains(&n.id);
        Some(tern!(write, value, DataKind::optional(value)))
    }

    /// Infer `<optional> ထဲမှ ဒေတာ၊ မရှိလျှင် <value>`: the data kind, which the
    /// fallback converts to.
    ///
    /// # Arguments
    ///
    /// * `optional` - The optional.
    /// * `fallback` - The value for an optional with no data.
    ///
    /// # Returns
    ///
    /// The data kind, or `None` after an error.
    pub(crate) fn infer_or_else(
        &mut self, optional: &Expr, fallback: &Expr,
    ) -> Option<DataKind>
    {
        let data = self.infer_result_part(VerbOp::Unwrap, optional);
        self.expect_kind(fallback, data);
        data
    }

    /// Infer `<ဇယား> ၏ အညွှန်းအရေအတွက်`, the number of rows.
    ///
    /// # Arguments
    ///
    /// * `table` - The ဇယား.
    ///
    /// # Returns
    ///
    /// `usize`, or `None` after an error.
    pub(crate) fn infer_row_count(&mut self, table: &Expr) -> Option<DataKind>
    {
        let kind = self.infer(table, None)?;
        ret_if!(kind.is_table(), Some(DataKind::USIZE));
        self.report(kd::not_a_table(table.span(), &kind.name()));
        None
    }

    /// Mark the read of a row at the base of a place that a statement writes
    /// through, so it has the value kind and not an optional.
    ///
    /// # Arguments
    ///
    /// * `target` - The place.
    pub(crate) fn mark_row_write(&mut self, target: &Expr)
    {
        match target
        {
            Expr::Group(n) => self.mark_row_write(&n.inner),
            Expr::Field(n) => self.mark_row_write(&n.base),
            Expr::Index(n) => self.mark_row_write(&n.list),
            // A write into a row of a ဇယား held in a row writes through both.
            Expr::Verb(n) if n.op == VerbOp::RowValue =>
            {
                self.row_writes.insert(n.id);
                do_if!(let Some(table) = n.args.first(), self.mark_row_write(table));
            }
            _ => (),
        }
    }

    /// Check `<row> ကို ဖျက်မည်`: the ဇယား is a place that can be written, and
    /// the index converts to its index kind.
    ///
    /// # Arguments
    ///
    /// * `row` - What the statement removes.
    pub(crate) fn remove_row(&mut self, row: &Expr)
    {
        let Expr::Verb(v) = row
        else
        {
            return self.report(kd::remove_needs_row(row.span()));
        };
        let [table, index] = v.args.as_slice()
        else
        {
            return self.report(kd::remove_needs_row(row.span()));
        };
        ret_if!(v.op != VerbOp::Row, self.report(kd::remove_needs_row(row.span())));
        let kind = self.place_kind(table);
        let Some((at, value)) = kind.and_then(DataKind::table_parts)
        else
        {
            do_if!(let Some(k) = kind, self.report(kd::not_a_table(table.span(), &k.name())));
            return;
        };
        self.expect_kind(index, Some(at));
        self.kinds.insert(v.id, value);
    }

    /// The kinds of the names of an each loop over the rows of a ဇယား: the
    /// first of two names takes the index; one name takes the index for
    /// `အညွှန်းတိုင်း` and the value for `အတန်းတိုင်း`.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value the loop walks.
    /// * `n` - The loop.
    ///
    /// # Returns
    ///
    /// The kind of the first of two names, when there are two, and the kind
    /// of the last name; `None` after a report.
    pub(crate) fn row_var_kinds(
        &mut self, kind: DataKind, n: &EachStmt,
    ) -> Option<(Option<DataKind>, DataKind)>
    {
        let Some((index, value)) = kind.table_parts()
        else
        {
            self.report(kd::not_a_table(n.source.span(), &kind.name()));
            return None;
        };
        ret_if!(n.place.is_some(), Some((Some(index), value)));
        Some((None, tern!(n.part == EachPart::Index, index, value)))
    }
}
