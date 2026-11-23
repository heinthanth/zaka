//! Structs based on other structs, `<struct> ကို အခြေခံသော`. The parser holds
//! each base as a part: a field whose name is the name of the base after
//! [`BASE_FIELD`], which no program can write. A value reaches the fields and
//! the functions of its parts as its own, the nearest first: its own, then
//! those of its parts, then those of their parts; two parts at one depth with
//! the name are ZK0364. The check writes each such read as the read of the
//! part, so the later stages see plain fields: `x ၏ f` becomes
//! `x ၏ %%B ၏ f`, `x ၏ g()` becomes `g(x ၏ %%B)`, and the read of a part,
//! `x ကို B အဖြစ် ပြောင်းလဲသော ရလဒ်`, becomes `x ၏ %%B`.

use std::collections::HashSet;

use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::expr::FieldExpr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::BASE_FIELD;
use zakac_ast::name::Name;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::StructDecl;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::kind::DataKind;
use crate::scope::Written;
use crate::semcheck::Checker;
use crate::traits::Wanted;

/// The deepest a search for a name goes through the parts of parts.
const MAX_DEPTH: usize = 32;

/// The parts of a value of a kind: each field that holds a struct it is based
/// on, with its kind.
///
/// # Arguments
///
/// * `kind` - The kind: a struct or a shared object.
///
/// # Returns
///
/// The parts, in order.
fn parts_of(kind: DataKind) -> Vec<(String, DataKind)>
{
    let fields = kind.object_struct().map(|s| s.fields().to_vec()).unwrap_or_default();
    fields.into_iter().filter(|(name, _)| name.starts_with(BASE_FIELD)).collect()
}

/// The name of a part as a program writes it: the name of its struct.
///
/// # Arguments
///
/// * `field` - The field of the part.
///
/// # Returns
///
/// The name.
fn part_name(field: &str) -> &str { field.strip_prefix(BASE_FIELD).unwrap_or(field) }

impl Checker
{
    /// A node number no node of the program has, for a node the check adds.
    ///
    /// # Returns
    ///
    /// The number.
    pub(crate) fn fresh_node(&mut self) -> NodeId
    {
        let id = NodeId(self.next_node);
        self.next_node += 1;
        id
    }

    /// The fields of the parts, nearest first, down to the first depth where
    /// one part holds what `hit` looks for.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind whose parts to search.
    /// * `hit` - Whether a part holds what the search looks for.
    ///
    /// # Returns
    ///
    /// The fields from the value to that part, `None` when no part holds it,
    /// or the names of two parts at one depth that both hold it.
    pub(crate) fn search_parts(
        &self, kind: DataKind, hit: impl Fn(&Self, DataKind) -> bool,
    ) -> Result<Option<Vec<String>>, (String, String)>
    {
        let mut level = vec![(Vec::<String>::new(), kind)];
        let mut seen = HashSet::new();
        for _ in 0..MAX_DEPTH
        {
            let mut next = Vec::new();
            let mut hits = Vec::new();
            for (path, k) in &level
            {
                for (field, part) in parts_of(*k)
                {
                    let mut path = path.clone();
                    path.push(field.clone());
                    do_if_hit(hit(self, part), &mut hits, &path);
                    next.push((path, part));
                }
            }
            match hits.as_slice()
            {
                [one] => return Ok(Some(one.clone())),
                [a, b, ..] =>
                {
                    let name = |p: &Vec<String>| {
                        part_name(p.last().map_or("", String::as_str)).to_string()
                    };
                    return Err((name(a), name(b)));
                }
                [] =>
                {}
            }
            next.retain(|(_, k)| seen.insert(*k));
            ret_if!(next.is_empty(), Ok(None));
            level = next;
        }
        Ok(None)
    }

    /// The read of the parts along a path, each node new.
    ///
    /// # Arguments
    ///
    /// * `base` - The value.
    /// * `path` - The fields of the parts.
    /// * `span` - The span each read takes.
    ///
    /// # Returns
    ///
    /// The read.
    fn path_expr(&mut self, base: Expr, path: &[String], span: Span) -> Expr
    {
        let mut out = base;
        for name in path
        {
            let field = Ident::builder().name(name.as_str()).span(span).build();
            let id = self.fresh_node();
            #[rustfmt::skip]
            let read = FieldExpr::builder().id(id).base(out).field(field).braced(true).span(span).build();
            out = Expr::Field(read);
        }
        out
    }

    /// Infer the read of a field that a part holds, `x ၏ f` for
    /// `x ၏ %%B ၏ f`, when the kind of `x` has no field of the name.
    ///
    /// # Arguments
    ///
    /// * `n` - The read.
    /// * `base` - The kind of `x`.
    ///
    /// # Returns
    ///
    /// The kind of the field, or `None` when no part holds it, or after a
    /// report.
    pub(crate) fn promoted_field(
        &mut self, n: &FieldExpr, base: DataKind,
    ) -> Option<Option<DataKind>>
    {
        let name = n.field.name.as_str();
        let own = |k: DataKind| {
            k.object_struct().is_some_and(|s| {
                !name.starts_with(BASE_FIELD) && s.field(name).is_some()
            })
        };
        let path = match self.search_parts(base, |_, k| own(k))
        {
            Ok(path) => path?,
            Err((a, b)) =>
            {
                self.report(rd::base_name_twice(
                    n.field.span,
                    name,
                    &base.name(),
                    (&a, &b),
                ));
                return Some(None);
            }
        };
        let through = self.path_expr((*n.base).clone(), &path, n.span);
        #[rustfmt::skip]
        let read = FieldExpr::builder().id(n.id).base(through).field(n.field.clone()).braced(true)
            .span(n.span).build();
        let kind = self.infer_field(&read);
        self.op_calls.fields.insert(n.id, Expr::Field(read));
        Some(kind)
    }

    /// Infer a call whose first argument has no function of the name but a
    /// part does: `g(x)` for `g(x ၏ %%B)`, the nearest part first.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    ///
    /// # Returns
    ///
    /// The result kind, `None` when the call runs as written, or `Some(None)`
    /// after a report.
    pub(crate) fn promoted_call(&mut self, n: &CallExpr) -> Option<Option<DataKind>>
    {
        ret_if!(n.place.is_some() || n.parallel, None);
        let written = Written::of(&n.prepositions, &n.phrases);
        let Some(path) = self.call_part_path(&n.callee, &n.args, written)?
        else
        {
            return Some(None);
        };
        let mut through = n.clone();
        through.args[0] = self.path_expr(n.args[0].clone(), &path, n.args[0].span());
        through.field_id = None;
        let kind = self.infer_call(&through);
        self.op_calls.methods.insert(n.id, through);
        Some(kind)
    }

    /// Check a call statement whose first argument has no function of the
    /// name but a part does, as the call on that part:
    /// `g ဟူသော လုပ်ဆောင်ချက်ကို x ဖြင့် လုပ်ဆောင်မည်။` for the call with
    /// `x ၏ %%B`.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// `true` when the statement went to a part, or after a report.
    pub(crate) fn promoted_stmt(&mut self, n: &CallStmt) -> bool
    {
        ret_false!(n.place.is_some() || n.parallel);
        let written = Written::of(&n.prepositions, &n.phrases);
        let Some(path) = self.call_part_path(&n.callee, &n.args, written)
        else
        {
            return false;
        };
        let Some(path) = path
        else
        {
            return true;
        };
        let mut through = n.clone();
        through.args[0] = self.path_expr(n.args[0].clone(), &path, n.args[0].span());
        self.visit_call_stmt(&through);
        self.op_calls.verbs.insert(n.span, through);
        true
    }

    /// The path to the nearest part that a function of the name takes, for a
    /// call whose first argument no function of the name takes.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name of the function.
    /// * `args` - The arguments.
    /// * `written` - The prepositions and phrases of the call.
    ///
    /// # Returns
    ///
    /// The fields of the parts down to that part, `None` when the call runs as
    /// written, or `Some(None)` after a report.
    fn call_part_path(
        &mut self, callee: &Ident, args: &[Expr], written: Written,
    ) -> Option<Option<Vec<String>>>
    {
        let first = args.first()?;
        ret_if!(!self.has_parts, None);
        let name = callee.name;
        ret_if!(self.scopes.lookup(&name).is_some(), None);
        ret_if!(!self.templates_named(&name).is_empty(), None);
        let sigs = self.fns.get(&name).cloned()?;
        ret_if!(sigs.iter().any(|s| self.fits_call(s, args, written)), None);
        let before = self.diags.len();
        let base = self.infer(first, None);
        self.diags.truncate(before);
        let base = base?;
        ret_if!(self.runs_through(base, &name), None);
        let platform = self.platform;
        let takes = |k: DataKind| {
            sigs.iter()
                .any(|s| s.params.first().is_some_and(|p| k.widens_to(*p, platform)))
        };
        match self.search_parts(base, |_, k| takes(k))
        {
            Ok(path) => Some(Some(path?)),
            Err((a, b)) =>
            {
                let clash =
                    rd::base_name_twice(callee.span, &name, &base.name(), (&a, &b));
                self.report(clash);
                Some(None)
            }
        }
    }

    /// Infer a read of a field with no field of the name as the call of a
    /// function a part has, `x ၏ g` for `g(x ၏ %%B)`.
    ///
    /// # Arguments
    ///
    /// * `n` - The read.
    ///
    /// # Returns
    ///
    /// The result kind, or `None` when no part has such a function.
    pub(crate) fn promoted_method(&mut self, n: &FieldExpr)
    -> Option<Option<DataKind>>
    {
        #[rustfmt::skip]
        let call = CallExpr::builder().id(n.id).callee(n.field.clone())
            .args(vec![(*n.base).clone()]).span(n.span).build();
        // The call the search writes takes the node of the read.
        self.promoted_call(&call)
    }

    /// Infer the read of a part, `x ကို B အဖြစ် ပြောင်းလဲသော ရလဒ်`, as
    /// `x ၏ %%B`, the nearest part of that kind first.
    ///
    /// # Arguments
    ///
    /// * `n` - The conversion.
    /// * `value` - The value.
    /// * `from` - Its kind.
    /// * `target` - The kind converted to.
    ///
    /// # Returns
    ///
    /// The kind of the part, `None` when no part has the kind, or `Some(None)`
    /// after a report.
    pub(crate) fn part_read(
        &mut self, n: &VerbExpr, value: &Expr, from: DataKind, target: DataKind,
    ) -> Option<Option<DataKind>>
    {
        let path = match self.search_parts(from, |_, k| k == target)
        {
            Ok(path) => path?,
            Err((a, b)) =>
            {
                let name = target.name();
                self.report(rd::base_name_twice(n.span, &name, &from.name(), (&a, &b)));
                return Some(None);
            }
        };
        let (last, inner) = path.split_last()?;
        let through = self.path_expr(value.clone(), inner, n.span);
        let field = Ident::builder().name(last.as_str()).span(n.span).build();
        #[rustfmt::skip]
        let read = FieldExpr::builder().id(n.id).base(through).field(field).braced(true)
            .span(n.span).build();
        let kind = self.infer_field(&read);
        self.op_calls.fields.insert(n.id, Expr::Field(read));
        Some(kind)
    }

    /// The path to the nearest part whose own function gives a kind a
    /// function of a trait, for a function whose result is not the kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `trait_name` - The trait.
    /// * `want` - The function of the trait, with the kind in place.
    ///
    /// # Returns
    ///
    /// The fields of the parts down to that part and its kind, or `None`.
    pub(crate) fn part_gives(
        &self, kind: DataKind, trait_name: &str, want: &Wanted,
    ) -> Option<(Vec<String>, DataKind)>
    {
        ret_if!(!self.has_parts || want.ret == Some(kind), None);
        let for_part = |part: DataKind| {
            let mut params = want.params.clone();
            do_if!(!params.is_empty(), params[0] = part);
            Wanted { name: want.name.clone(), params, ret: want.ret }
        };
        let gives = |s: &Self, part: DataKind| {
            s.fitting(&for_part(part), s.places(part, trait_name)) == 1
        };
        let path = self.search_parts(kind, gives).ok().flatten()?;
        let part = path_kind(kind, &path)?;
        Some((path, part))
    }

    /// Whether a part of a kind takes the default of a function of a trait.
    /// The kind then takes the default too: it runs on the whole value, so a
    /// function the kind gives itself wins in it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `trait_name` - The trait.
    /// * `name` - The function.
    ///
    /// # Returns
    ///
    /// `true` when a part takes it.
    pub(crate) fn part_takes_default(
        &self, kind: DataKind, trait_name: &str, name: &Name,
    ) -> bool
    {
        ret_false!(!self.has_parts);
        let takes = |s: &Self, part: DataKind| s.takes_default(part, trait_name, name);
        self.search_parts(kind, takes).ok().flatten().is_some()
    }

    /// Check the parts of a struct: each is a struct or a shared object
    /// (ZK0363).
    ///
    /// # Arguments
    ///
    /// * `n` - The struct.
    pub(crate) fn check_parts(&mut self, n: &StructDecl)
    {
        let parts = n.fields.iter().filter(|f| f.name.name.starts_with(BASE_FIELD));
        let kinds =
            parts.map(|f| (f.kind.span, DataKind::from_ast(&f.kind, &self.structs)));
        let wrong: Vec<_> = kinds
            .filter(|(_, k)| k.object_struct().is_none())
            .map(|(span, k)| rd::base_not_a_struct(span, &k.name()))
            .collect();
        do_for!(d in wrong, self.report(d));
    }
}

/// The kind of the part at the end of a path of parts.
///
/// # Arguments
///
/// * `kind` - The kind the path starts from.
/// * `path` - The fields of the parts.
///
/// # Returns
///
/// The kind, or `None` for a path that does not read.
fn path_kind(kind: DataKind, path: &[String]) -> Option<DataKind>
{
    let mut at = kind;
    for name in path
    {
        at = at.object_struct()?.field(name)?.1;
    }
    Some(at)
}

/// Add a path to the hits of a search when its part holds what the search
/// looks for.
///
/// # Arguments
///
/// * `hit` - Whether the part holds it.
/// * `hits` - The hits so far.
/// * `path` - The path to the part.
fn do_if_hit(hit: bool, hits: &mut Vec<Vec<String>>, path: &[String])
{
    ret_if!(!hit);
    hits.push(path.to_vec());
}
