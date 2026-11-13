//! Generic functions. A function with kind parameters is a template: the
//! check leaves it as it is. Each call of it with known kinds asks for an
//! instance, a duplicate in which those kinds stand for its kind parameters,
//! named so no program can write the name, such as `f<isize>`; the instance
//! joins the program as a plain function, and the next run of the check
//! checks it as one.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NodeId;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::ResultParts;
use zakac_ast::kind::SigParam;
use zakac_ast::kind::TableParts;
use zakac_ast::kind::Wrap;
use zakac_ast::name::Name;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::PRELUDE;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StructDecl;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_expr;
use zakac_ast::visit::walk_expr_mut;
use zakac_diagnostics::Label;
use zakac_diagnostics::resolve as rd;
use zakac_i18n::t;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::fn_kind::FnKind;
use crate::fn_kind::FnParam;
use crate::kind::DataKind;
use crate::kind::Width;
use crate::scope::FnSig;
use crate::scope::FnSigs;
use crate::semcheck::Checker;

/// The generic functions and structs of a program, under their names.
#[derive(Debug, Clone, Default)]
pub(crate) struct Templates
{
    /// The generic functions.
    pub fns: HashMap<Name, FnDecl>,
    /// The generic structs.
    pub structs: HashMap<Name, StructDecl>,
    /// The file of each generic function and struct.
    pub units: HashMap<Name, u32>,
}

/// The instances a run of the check asks for: each instance name with its
/// template and its kinds.
pub(crate) type Wanted = BTreeMap<Name, (Name, Vec<DataKind>)>;

/// The first use of each instance: the name of a call, a value or a kind
/// that asked for it, where a report about the instance points.
pub(crate) type Uses = HashMap<Name, Span>;

/// Whether a statement declares a generic function or struct.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// `true` for a function or a struct with kind parameters.
pub(crate) fn is_template(stmt: &Stmt) -> bool
{
    match stmt
    {
        Stmt::Fn(d) => !d.kind_params.is_empty(),
        Stmt::Struct(d) => !d.kind_params.is_empty(),
        _ => false,
    }
}

/// The generic functions at the top level of a program.
///
/// # Arguments
///
/// * `program` - The program.
///
/// # Returns
///
/// Each template under its name; the first of a name wins, and a template
/// of a file hides one of the prelude.
pub(crate) fn templates(program: &Program) -> Templates
{
    let mut found = Templates::default();
    let all = program.stmts.iter().enumerate().filter(|(_, s)| is_template(s));
    for (i, stmt) in all
    {
        let Some(name) = declared_name(stmt)
        else
        {
            continue;
        };
        let unit = program.unit_of(i);
        // The first of a name wins, but a generic of a file hides one of the
        // prelude.
        let held = found.units.get(&name).copied();
        let taken =
            held.is_some_and(|u| !in_prelude(program, u) || in_prelude(program, unit));
        do_if!(taken, continue);
        found.units.insert(name, unit);
        match stmt
        {
            Stmt::Fn(d) => _ = found.fns.insert(name, d.clone()),
            Stmt::Struct(d) => _ = found.structs.insert(name, d.clone()),
            _ => (),
        }
    }
    found
}

/// Whether a unit is a module of the stdlib, which the compiler carries.
///
/// # Arguments
///
/// * `program` - The program.
/// * `unit` - The unit.
///
/// # Returns
///
/// `true` for a module whose name starts with `စံကား/`.
pub(crate) fn in_stdlib(program: &Program, unit: u32) -> bool
{
    program.units.get(unit as usize).is_some_and(|u| u.name.starts_with("စံကား/"))
}

/// Whether a unit is the prelude.
///
/// # Arguments
///
/// * `program` - The program.
/// * `unit` - The unit.
///
/// # Returns
///
/// `true` for the prelude.
pub(crate) fn in_prelude(program: &Program, unit: u32) -> bool
{
    program.units.get(unit as usize).is_some_and(|u| u.name == PRELUDE)
}

/// The name of the instance of a template for some kinds, `f<isize၊ text>`;
/// no written name holds a `<`.
///
/// # Arguments
///
/// * `template` - The name of the template.
/// * `kinds` - The kind of each kind parameter.
///
/// # Returns
///
/// The name.
pub(crate) fn instance_name(template: Name, kinds: &[DataKind]) -> Name
{
    let parts: Vec<String> = kinds.iter().map(|k| k.name()).collect();
    Name::from(format!("{template}<{}>", parts.join("၊ ")))
}

/// Work out the kind of each kind parameter of a template from pairs of a
/// kind as written in it and the kind of the value that fills it: each
/// parameter of a function with its argument, or each field of a struct
/// with its value.
///
/// # Arguments
///
/// * `names` - The kind parameters of the template.
/// * `pairs` - Each kind as written, with the kind of its value when known.
/// * `known` - The instances of generic structs so far.
///
/// # Returns
///
/// The kinds in the order of the kind parameters, or `None` when one remains
/// unknown or two values give one parameter two kinds.
pub(crate) fn infer_kinds<'k>(
    names: &[Name], pairs: impl IntoIterator<Item = (&'k Kind, Option<DataKind>)>,
    known: &Wanted,
) -> Option<Vec<DataKind>>
{
    let mut binder = Binder { names, known, found: HashMap::new() };
    for (pattern, actual) in pairs
    {
        let Some(actual) = actual
        else
        {
            continue;
        };
        binder.bind(pattern, actual)?;
    }
    names.iter().map(|n| binder.found.get(n).copied()).collect()
}

/// The matcher of kinds as written against data kinds.
struct Binder<'a>
{
    /// The kind parameters of the template.
    names: &'a [Name],
    /// The instances of generic structs so far.
    known: &'a Wanted,
    /// The kind each kind parameter stands for, found so far.
    found: HashMap<Name, DataKind>,
}

impl Binder<'_>
{
    /// Match a kind as written against the kind of its value, and record the
    /// kind each kind parameter in it stands for.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The kind as written.
    /// * `actual` - The kind of the value.
    ///
    /// # Returns
    ///
    /// `Some` when the kinds fit, `None` when a kind parameter gets two kinds.
    fn bind(&mut self, pattern: &Kind, actual: DataKind) -> Option<()>
    {
        // The levels of the pattern wrap its base, the innermost first: take
        // them off the value from the outside in. A level the value lacks
        // binds nothing.
        let mut actual = actual;
        for wrap in pattern.wraps.iter().rev()
        {
            let inner = match wrap
            {
                Wrap::Ptr => actual.pointee(),
                Wrap::List(Cells::Inferred) =>
                {
                    actual.as_seq().filter(|s| !s.text).map(|s| s.cell)
                }
                Wrap::List(_) => actual.as_list().map(|l| l.cell),
            };
            let Some(inner) = inner
            else
            {
                return Some(());
            };
            actual = inner;
        }
        if let Some(table) = pattern.table.as_deref()
        {
            let Some((index, value)) = actual.table_parts()
            else
            {
                return Some(());
            };
            self.bind(&table.index, index)?;
            return self.bind(&table.value, value);
        }
        ret_if!(let Some(parts) = pattern.parts.as_deref(), self.bind_parts(parts, actual));
        ret_if!(!pattern.args.is_empty(), self.bind_args(pattern, actual));
        let named = pattern.base == BaseKind::Named;
        let name = pattern.name.as_ref().map(|n| n.name);
        let Some(name) = name.filter(|n| named && self.names.contains(n))
        else
        {
            return Some(());
        };
        match self.found.get(&name)
        {
            Some(known) => tern!(*known == actual, Some(()), None),
            None =>
            {
                self.found.insert(name, actual);
                Some(())
            }
        }
    }

    /// [`Binder::bind`] for the sides of an optional or a result kind.
    ///
    /// # Arguments
    ///
    /// * `parts` - The sides, as written.
    /// * `actual` - The kind of the value.
    ///
    /// # Returns
    ///
    /// `Some` when the kinds fit.
    fn bind_parts(&mut self, parts: &ResultParts, actual: DataKind) -> Option<()>
    {
        if parts.optional
        {
            let (Some(ok), Some(data)) = (&parts.ok, actual.optional_data())
            else
            {
                return Some(());
            };
            return self.bind(ok, data);
        }
        let Some((ok, err)) = actual.fallible_parts()
        else
        {
            return Some(());
        };
        if let (Some(pattern), Some(kind)) = (&parts.ok, ok)
        {
            self.bind(pattern, kind)?;
        }
        if let (Some(pattern), Some(kind)) = (&parts.err, err)
        {
            self.bind(pattern, kind)?;
        }
        Some(())
    }

    /// [`Binder::bind`] for a generic struct with its kinds, such as
    /// `အတွဲ<T၊ U>`, against an instance of that struct.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The kind as written.
    /// * `actual` - The kind of the value.
    ///
    /// # Returns
    ///
    /// `Some` when the kinds fit.
    fn bind_args(&mut self, pattern: &Kind, actual: DataKind) -> Option<()>
    {
        let instance = actual.as_struct().map(|s| Name::new(&s.name));
        let Some((template, kinds)) = instance.and_then(|n| self.known.get(&n))
        else
        {
            return Some(());
        };
        let same = pattern.name.as_ref().is_some_and(|n| n.name == *template);
        ret_if!(!same, Some(()));
        let pairs: Vec<(&Kind, DataKind)> =
            pattern.args.iter().zip(kinds.iter().copied()).collect();
        pairs.into_iter().try_for_each(|(p, k)| self.bind(p, k))
    }
}

/// The kind phrase that names a data kind, to stand in an instance where a
/// kind parameter stood.
///
/// # Arguments
///
/// * `kind` - The data kind.
/// * `span` - The span the phrase takes, the kind parameter's.
///
/// # Returns
///
/// The phrase, or `None` for a kind no phrase names: a borrow and a raw
/// address, which no program writes either.
pub(crate) fn kind_ast(kind: DataKind, span: Span) -> Option<Kind>
{
    let plain = |base: BaseKind| Kind::builder().base(base).span(span).build();
    let wrapped = |inner: DataKind, wrap: Wrap| {
        let mut phrase = kind_ast(inner, span)?;
        phrase.wraps.push(wrap);
        Some(phrase)
    };
    match kind
    {
        DataKind::Int(n) =>
        {
            let bits = tern!(let Width::Bits(b) = n.width, Some(b), None);
            let signed = tern!(n.signed, None, Some(false));
            let int = Kind::builder()
                .base(BaseKind::Int)
                .maybe_bits(bits)
                .maybe_signed(signed);
            Some(int.span(span).build())
        }
        DataKind::Float(bits) =>
        {
            let bits = (bits != 64).then_some(bits);
            Some(
                Kind::builder()
                    .base(BaseKind::Float)
                    .maybe_bits(bits)
                    .span(span)
                    .build(),
            )
        }
        DataKind::Char => Some(plain(BaseKind::Char)),
        DataKind::Bool => Some(plain(BaseKind::Bool)),
        DataKind::Seq(s) if s.text => Some(plain(BaseKind::Text)),
        DataKind::Seq(s) if !s.boxed => wrapped(s.cell, Wrap::List(Cells::Inferred)),
        DataKind::List(l) => wrapped(l.cell, Wrap::List(Cells::Fixed(l.len))),
        DataKind::Ptr(_) if kind.is_owner_ptr() && !kind.is_opaque() =>
        {
            wrapped(kind.pointee()?, Wrap::Ptr)
        }
        DataKind::Struct(_) => struct_ast(kind, span),
        DataKind::Enum(e) => Some(named(&e.name, span)),
        DataKind::Fn(f) => fn_ast(f, span),
        _ => None,
    }
}

/// [`kind_ast`] for a function kind: each parameter with its marks, and the
/// result.
///
/// # Arguments
///
/// * `f` - The function kind.
/// * `span` - The span the phrase takes.
///
/// # Returns
///
/// The phrase, or `None` when a part has none.
fn fn_ast(f: &FnKind, span: Span) -> Option<Kind>
{
    let param = |p: &FnParam| -> Option<SigParam> {
        let mark = p.mark.map(|m| Ident::builder().name(m).span(span).build());
        let kind = kind_ast(p.kind, span)?;
        Some(
            SigParam::builder()
                .edit(p.edit)
                .kind(kind)
                .maybe_mark(mark)
                .span(span)
                .build(),
        )
    };
    let params: Option<Vec<SigParam>> = f.params.iter().map(param).collect();
    let ret = match f.ret
    {
        Some(k) => Some(kind_ast(k, span)?),
        None => None,
    };
    let sig = zakac_ast::kind::FnSig::builder().params(params?).maybe_ret(ret).build();
    Some(Kind::builder().base(BaseKind::Fn).sig(Box::new(sig)).span(span).build())
}

/// [`kind_ast`] for a struct kind: an optional, a result, a ဇယား or a
/// declared struct.
///
/// # Arguments
///
/// * `kind` - The struct kind.
/// * `span` - The span the phrase takes.
///
/// # Returns
///
/// The phrase, or `None` when a part has none.
fn struct_ast(kind: DataKind, span: Span) -> Option<Kind>
{
    let side = |k: Option<DataKind>| -> Option<Option<Kind>> {
        match k
        {
            Some(k) => Some(Some(kind_ast(k, span)?)),
            None => Some(None),
        }
    };
    if let Some(data) = kind.optional_data()
    {
        let parts = ResultParts::builder().maybe_ok(side(Some(data))?).optional(true);
        return Some(result_kind(parts.build(), span));
    }
    if let Some((index, value)) = kind.table_parts()
    {
        let parts = TableParts::builder()
            .index(kind_ast(index, span)?)
            .value(kind_ast(value, span)?);
        let table =
            Kind::builder().base(BaseKind::Table).table(Box::new(parts.build()));
        return Some(table.span(span).build());
    }
    if let Some((ok, err)) = kind.fallible_parts()
    {
        let parts = ResultParts::builder().maybe_ok(side(ok)?).maybe_err(side(err)?);
        return Some(result_kind(parts.build(), span));
    }
    Some(named(&kind.as_struct()?.name, span))
}

/// A result or optional kind phrase with its sides.
///
/// # Arguments
///
/// * `parts` - The sides.
/// * `span` - The span the phrase takes.
///
/// # Returns
///
/// The phrase.
fn result_kind(parts: ResultParts, span: Span) -> Kind
{
    Kind::builder().base(BaseKind::Result).parts(Box::new(parts)).span(span).build()
}

/// The phrase of a struct or an enumeration name.
///
/// # Arguments
///
/// * `name` - The name.
/// * `span` - The span the phrase takes.
///
/// # Returns
///
/// The phrase.
fn named(name: &str, span: Span) -> Kind
{
    let ident = Ident::builder().name(name).span(span).build();
    Kind::builder().base(BaseKind::Named).name(ident).span(span).build()
}

/// The most runs of the check that can add instances: each run adds the
/// instances the instances of the run before ask for, so this is the
/// deepest a chain of instances goes. Past it a call that asks for an
/// instance gets none and reports ZK0342.
pub(crate) const INSTANCE_DEPTH: u32 = 64;

impl Checker
{
    /// The function a call of a generic function runs: its instance for the
    /// kinds written at the call, or else worked out from the arguments, and
    /// then from the kind the place of the call expects. The call asks for the
    /// instance; a run before the instance exists checks the call against
    /// nothing, and the next run checks it again.
    ///
    /// # Arguments
    ///
    /// * `template` - The generic function.
    /// * `callee` - The name at the call.
    /// * `kind_args` - The kinds written at the call.
    /// * `args` - The arguments.
    /// * `expect` - The kind the place of the call expects, when known.
    ///
    /// # Returns
    ///
    /// The signature of the instance, or `None` while it does not exist or
    /// after a report.
    pub(crate) fn generic_callee(
        &mut self, template: &FnDecl, callee: &Ident, kind_args: &[Kind],
        args: &[Expr], expect: Option<DataKind>,
    ) -> Option<FnSig>
    {
        let kinds = match kind_args.is_empty()
        {
            true => self.call_kinds(template, args, expect),
            false => Some(self.written_kinds(
                template.kind_params.len(),
                callee,
                kind_args,
            )?),
        };
        self.instance_sig(template, callee, kinds)
    }

    /// The function a value of a generic function names: its instance for
    /// the kinds written after `::`, or else for the function kind its place
    /// expects.
    ///
    /// # Arguments
    ///
    /// * `template` - The generic function.
    /// * `name` - The name in the value.
    /// * `kind_args` - The kinds written.
    /// * `expect` - The kind the place of the value expects, when known.
    ///
    /// # Returns
    ///
    /// The signature of the instance, or `None` while it does not exist or
    /// after a report.
    pub(crate) fn generic_value(
        &mut self, template: &FnDecl, name: &Ident, kind_args: &[Kind],
        expect: Option<DataKind>,
    ) -> Option<FnSig>
    {
        let kinds = match kind_args.is_empty()
        {
            true =>
            {
                let f = expect.and_then(DataKind::as_fn);
                let params = f.map(|f| f.param_kinds()).unwrap_or_default();
                let found = params.into_iter().map(Some).chain([f.and_then(|f| f.ret)]);
                let written =
                    template.params.iter().map(|p| &p.kind).chain(&template.ret);
                infer_kinds(
                    &params_of(template),
                    written.zip(found),
                    &self.known_structs,
                )
            }
            false =>
            {
                Some(self.written_kinds(template.kind_params.len(), name, kind_args)?)
            }
        };
        self.instance_sig(template, name, kinds)
    }

    /// The instance of a generic function for kinds, asked for: ZK0341 when
    /// the kinds are not known or a kind cannot stand in a phrase, the report
    /// of each constraint a kind does not meet, and ZK0342 while the instance
    /// does not exist.
    ///
    /// # Arguments
    ///
    /// * `template` - The generic function.
    /// * `ident` - The name at the use.
    /// * `kinds` - The kinds, when known.
    ///
    /// # Returns
    ///
    /// The signature of the instance, or `None`.
    fn instance_sig(
        &mut self, template: &FnDecl, ident: &Ident, kinds: Option<Vec<DataKind>>,
    ) -> Option<FnSig>
    {
        let kinds = self.known_kinds(ident, kinds)?;
        ret_if!(!self.meets_bounds(&template.kind_params, &kinds, ident.span), None);
        let name = instance_name(template.name.name, &kinds);
        self.wanted.insert(name, (template.name.name, kinds));
        self.uses.entry(name).or_insert(ident.span);
        self.chosen.insert((self.top, ident.span), name);
        let sig = self.fns.get(&name).and_then(|s| s.first().cloned());
        let endless = rd::instances_endless(ident.span, &ident.name);
        ret_if!(sig.is_none(), self.report(endless), None);
        sig
    }

    /// Kinds that each stand in a phrase, or ZK0341.
    ///
    /// # Arguments
    ///
    /// * `ident` - The name at the use.
    /// * `kinds` - The kinds, when known.
    ///
    /// # Returns
    ///
    /// The kinds, or `None` after a report.
    pub(crate) fn known_kinds(
        &mut self, ident: &Ident, kinds: Option<Vec<DataKind>>,
    ) -> Option<Vec<DataKind>>
    {
        let named =
            |ks: &Vec<DataKind>| ks.iter().all(|k| kind_ast(*k, ident.span).is_some());
        let unknown = rd::kinds_unknown(ident.span, &ident.name);
        ret_if!(!kinds.as_ref().is_some_and(named), self.report(unknown), None);
        kinds
    }

    /// The functions a call of a function that is not generic can run; kinds
    /// written at the call are ZK0340, as the function takes none.
    ///
    /// # Arguments
    ///
    /// * `callee` - The name at the call.
    /// * `kind_args` - The kinds written at the call.
    /// * `place` - The place that holds the function value, when written.
    ///
    /// # Returns
    ///
    /// The functions, or `None` after a report.
    pub(crate) fn plain_callee(
        &mut self, callee: &Ident, kind_args: &[Kind], place: Option<&Expr>,
    ) -> Option<Vec<FnSig>>
    {
        let count = rd::kind_arg_count(callee.span, &callee.name, 0, kind_args.len());
        ret_if!(!kind_args.is_empty(), self.report(count), None);
        self.call_target(callee, place)
    }

    /// The kinds written for a generic, one for each kind parameter: ZK0340
    /// for another count.
    ///
    /// # Arguments
    ///
    /// * `want` - The count of kind parameters.
    /// * `ident` - The name at the use.
    /// * `kind_args` - The kinds written.
    ///
    /// # Returns
    ///
    /// The kinds, or `None` after a report.
    pub(crate) fn written_kinds(
        &mut self, want: usize, ident: &Ident, kind_args: &[Kind],
    ) -> Option<Vec<DataKind>>
    {
        let count = rd::kind_arg_count(ident.span, &ident.name, want, kind_args.len());
        ret_if!(kind_args.len() != want, self.report(count), None);
        Some(kind_args.iter().map(|k| DataKind::from_ast(k, &self.structs)).collect())
    }

    /// The kinds the arguments of a call of a generic function show, and the
    /// result the place of the call expects for each kind the arguments leave
    /// open.
    ///
    /// # Arguments
    ///
    /// * `template` - The generic function.
    /// * `args` - The arguments.
    /// * `expect` - The kind the place of the call expects, when known.
    ///
    /// # Returns
    ///
    /// The kinds, or `None` when one remains unknown.
    fn call_kinds(
        &mut self, template: &FnDecl, args: &[Expr], expect: Option<DataKind>,
    ) -> Option<Vec<DataKind>>
    {
        let names = params_of(template);
        let found = self.quiet_kinds(args.iter());
        let written: Vec<&Kind> = template.params.iter().map(|p| &p.kind).collect();
        let pairs = || written.iter().copied().zip(found.iter().copied());
        let known = &self.known_structs;
        let from_args = infer_kinds(&names, pairs(), known);
        ret_if!(from_args.is_some() || expect.is_none(), from_args);
        let result = template.ret.iter().zip([expect]);
        infer_kinds(&names, pairs().chain(result), known)
    }

    /// The kind each value has on its own, with no report: the check of the
    /// call or the literal against its instance reports each fault.
    ///
    /// # Arguments
    ///
    /// * `values` - The values.
    ///
    /// # Returns
    ///
    /// The kind of each, when known.
    pub(crate) fn quiet_kinds<'e>(
        &mut self, values: impl Iterator<Item = &'e Expr>,
    ) -> Vec<Option<DataKind>>
    {
        let mark = self.diags.len();
        let kinds = values.map(|v| self.infer(v, None)).collect();
        self.diags.truncate(mark);
        kinds
    }
}

impl Checker
{
    /// The generic function a name calls: none when the file declares a
    /// function of the name, or a variable or a parameter holds it, as the
    /// file's own name hides a generic, the prelude's ones included.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The generic function, or `None`.
    pub(crate) fn template_named(&self, name: &Name) -> Option<FnDecl>
    {
        ret_if!(
            self.fns_named(name).is_some() || self.scopes.lookup(name).is_some(),
            None
        );
        self.templates.fns.get(name).cloned()
    }

    /// Start the check of an instance of a generic function: it sees the
    /// functions of the file of its generic, of the file of each of its
    /// kinds, and of the file of each trait its kind parameters need, beside
    /// those of the file it stands in.
    ///
    /// # Arguments
    ///
    /// * `stmt` - The top-level statement.
    ///
    /// # Returns
    ///
    /// The functions the file saw before, for [`Checker::close_instance`];
    /// `None` for a statement that is no instance.
    pub(crate) fn open_instance(&mut self, stmt: &Stmt) -> Option<FnSigs>
    {
        let Stmt::Fn(decl) = stmt
        else
        {
            return None;
        };
        let (template, kinds) = self.known_fns.get(&decl.name.name).cloned()?;
        let mut units: Vec<u32> =
            self.templates.units.get(&template).copied().into_iter().collect();
        units.extend(kinds.iter().filter_map(|k| self.kind_home(*k)));
        let params = self.templates.fns.get(&template).map(|t| t.kind_params.clone());
        let bounds = params.iter().flatten().flat_map(|p| p.bounds.iter());
        let names = bounds.filter_map(|b| b.name.as_ref().map(|n| n.name));
        units.extend(names.filter_map(|n| self.traits.get(&n).map(|(u, _)| *u)));
        let saved = self.fns.clone();
        for unit in units.into_iter().filter(|u| *u != self.unit)
        {
            let more = self.unit_fns.get(unit as usize).cloned().unwrap_or_default();
            do_for!((name, sigs) in more, add_sigs(&mut self.fns, name, sigs));
        }
        self.in_instance = true;
        Some(saved)
    }

    /// End the check of an instance of a generic function.
    ///
    /// # Arguments
    ///
    /// * `fns` - The functions the file saw before.
    pub(crate) fn close_instance(&mut self, fns: FnSigs)
    {
        self.fns = fns;
        self.in_instance = false;
    }

    /// Point each report found in an instance at the use that asked for the
    /// instance, as the report stands in the code of its generic.
    ///
    /// # Arguments
    ///
    /// * `stmt` - The instance.
    /// * `from` - The count of reports before its check.
    pub(crate) fn label_instance(&mut self, stmt: &Stmt, from: usize)
    {
        let Some(name) = declared_name(stmt)
        else
        {
            return;
        };
        let Some(at) = self.uses.get(&name).copied()
        else
        {
            return;
        };
        let text = t!("semcheck.instance_use", n = name.as_str()).to_string();
        for d in self.diags.iter_mut().skip(from)
        {
            d.labels.push(Label::builder().span(at).text(text.clone()).build());
        }
    }

    /// Give a call or a function value in an instance that names a function
    /// of another file the name the later stages give that function in
    /// each file: its name and the suffix of its file, `<name>@<file>`.
    ///
    /// # Arguments
    ///
    /// * `sig` - The function.
    /// * `span` - The name at the use.
    pub(crate) fn name_foreign(&mut self, sig: &FnSig, span: Span)
    {
        let from_prelude = |u: &u32| Some(*u) == self.prelude_unit;
        let foreign = sig
            .unit
            .filter(|u| *u != self.unit && (self.in_instance || from_prelude(u)));
        let Some(unit) = foreign
        else
        {
            return;
        };
        let file = self.unit_names.get(unit as usize).cloned().unwrap_or_default();
        let stem = std::path::Path::new(&file).file_stem().and_then(|s| s.to_str());
        let name = Name::from(format!("{}@{}", sig.inner, stem.unwrap_or(&file)));
        self.chosen.insert((self.top, span), name);
    }
}

/// Add functions of a name to the functions a file sees, each once.
///
/// # Arguments
///
/// * `fns` - The functions the file sees.
/// * `name` - The name.
/// * `sigs` - The functions of that name.
fn add_sigs(fns: &mut FnSigs, name: Name, sigs: Vec<FnSig>)
{
    let list = fns.entry(name).or_default();
    let fresh: Vec<FnSig> =
        sigs.into_iter().filter(|s| !list.iter().any(|x| x.decl == s.decl)).collect();
    list.extend(fresh);
}

/// The names of the kind parameters of a generic function.
///
/// # Arguments
///
/// * `template` - The generic function.
///
/// # Returns
///
/// The names, in order.
fn params_of(template: &FnDecl) -> Vec<Name>
{
    template.kind_params.iter().map(|p| p.name.name).collect()
}

/// Add to a program each instance of a generic function or struct that it
/// does not hold so far.
///
/// # Arguments
///
/// * `program` - The program.
/// * `templates` - The templates.
/// * `fns` - The function instances the run asked for.
/// * `structs` - The struct instances known so far.
///
/// # Returns
///
/// `true` when an instance was added, so the check runs again.
pub(crate) fn add_instances(
    program: &mut Program, templates: &Templates, fns: &Wanted, structs: &Wanted,
) -> bool
{
    let have: HashSet<Name> = program.stmts.iter().filter_map(declared_name).collect();
    let fn_of = |t: &Name| templates.fns.get(t).cloned().map(Stmt::Fn);
    let struct_of = |t: &Name| templates.structs.get(t).cloned().map(Stmt::Struct);
    let fresh_fns = fns.iter().filter_map(|(n, (t, k))| Some((*n, fn_of(t)?, k)));
    let fresh_structs =
        structs.iter().filter_map(|(n, (t, k))| Some((*n, struct_of(t)?, k)));
    let fresh = fresh_fns.chain(fresh_structs).filter(|(n, _, _)| !have.contains(n));
    let fresh: Vec<_> = fresh.collect();
    let mut next = next_id(program);
    let mut added = false;
    for (name, template, kinds) in fresh
    {
        let at = matches!(template, Stmt::Fn(_))
            .then(|| template_at(program, &template))
            .flatten();
        let Some(made) = instance(template, kinds, name, &mut next)
        else
        {
            continue;
        };
        place_instance(program, made, at);
        added = true;
    }
    added
}

/// Where the template of a function stands in a program.
///
/// # Arguments
///
/// * `program` - The program.
/// * `template` - The template.
///
/// # Returns
///
/// The index of its statement.
fn template_at(program: &Program, template: &Stmt) -> Option<usize>
{
    let name = declared_name(template)?;
    let at =
        |s: &Stmt| is_template(s) && matches!(s, Stmt::Fn(d) if d.name.name == name);
    program.stmts.iter().position(at)
}

/// Put an instance into a program: an instance of a function right after its
/// template, in the file of the template, so it sees the names of that file;
/// an instance of a struct at the end, as struct names stand for the whole
/// program.
///
/// # Arguments
///
/// * `program` - The program.
/// * `made` - The instance.
/// * `at` - Where the template of a function stands.
fn place_instance(program: &mut Program, made: Stmt, at: Option<usize>)
{
    let Some(at) = at
    else
    {
        return program.stmts.push(made);
    };
    program.stmts.insert(at + 1, made);
    let later = program.units.iter_mut().filter(|u| u.first_stmt as usize > at);
    later.for_each(|u| u.first_stmt += 1);
}

/// The name a function or struct statement declares.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The name, or `None` for another statement.
fn declared_name(stmt: &Stmt) -> Option<Name>
{
    match stmt
    {
        Stmt::Fn(d) => Some(d.name.name),
        Stmt::Struct(d) => Some(d.name.name),
        _ => None,
    }
}

/// The instance of a template for some kinds.
///
/// # Arguments
///
/// * `template` - The template, a function or a struct.
/// * `kinds` - The kind of each kind parameter.
/// * `name` - The name of the instance.
/// * `next` - The first node number no node holds; moved past the numbers the
///   instance takes.
///
/// # Returns
///
/// The instance, or `None` when a kind has no phrase.
fn instance(
    mut template: Stmt, kinds: &[DataKind], name: Name, next: &mut u32,
) -> Option<Stmt>
{
    let (params, ident) = match &mut template
    {
        Stmt::Fn(d) => (&mut d.kind_params, &mut d.name),
        Stmt::Struct(d) => (&mut d.kind_params, &mut d.name),
        _ => return None,
    };
    let span = ident.span;
    let phrases: Option<Vec<Kind>> = kinds.iter().map(|k| kind_ast(*k, span)).collect();
    let names = params.iter().map(|p| p.name.name);
    let bound: HashMap<Name, Kind> = names.zip(phrases?).collect();
    params.clear();
    *ident = Ident::builder().name(name).span(span).build();
    // An instance is the program's own: no file imports it.
    match &mut template
    {
        Stmt::Fn(d) => d.exported = false,
        Stmt::Struct(d) => d.exported = false,
        _ => (),
    }
    let mut fill = Fill { bound: &bound, next: *next };
    fill.visit_stmt_mut(&mut template);
    *next = fill.next;
    Some(template)
}

/// The walker that fills an instance: its kind stands in place of each kind
/// parameter, and each expression takes a node number of its own.
struct Fill<'a>
{
    /// The kind phrase of each kind parameter.
    bound: &'a HashMap<Name, Kind>,
    /// The next node number to give.
    next: u32,
}

impl VisitorMut for Fill<'_>
{
    fn visit_kind_mut(&mut self, n: &mut Kind) { n.substitute(self.bound); }

    fn visit_expr_mut(&mut self, n: &mut Expr)
    {
        *n.id_mut() = NodeId(self.next);
        self.next += 1;
        walk_expr_mut(self, n);
    }
}

/// The first node number no expression of a program holds.
///
/// # Arguments
///
/// * `program` - The program.
///
/// # Returns
///
/// The number.
pub(crate) fn next_id(program: &Program) -> u32
{
    let mut top = Top(0);
    top.visit_program(program);
    top.0
}

/// The walker that finds the first free node number.
struct Top(u32);

impl Visitor for Top
{
    fn visit_expr(&mut self, n: &Expr)
    {
        self.0 = self.0.max(n.id().0 + 1);
        walk_expr(self, n);
    }
}
