//! Calls of comptime functions in spots that need a value at compile time:
//! the sites a run of the check meets, the small program that runs them, the
//! runner that runs it on the VM, and the values written back in place of the
//! calls.
//!
//! The check cannot reach the VM (the VM's crates depend on this one), so the
//! VM installs a [`ComptimeRunner`] at start, and the check hands it each
//! small program.

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

use bon::Builder;
use zakac_ast::expr::CallExpr;
use zakac_ast::expr::ComptimeExpr;
use zakac_ast::expr::ConstTree;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NodeId;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::name::Name;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::Unit;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_expr;
use zakac_ast::visit::walk_expr_mut;
use zakac_diagnostics::Diagnostic;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::consts::Folded;
use crate::kind::DataKind;
use crate::kind::Width;
use crate::semcheck::RefTable;
use crate::semcheck::SemcheckOptions;
use crate::value::Value;

/// The start of the name of each global a small program sets to the value
/// of one site; no written name holds `%`.
pub const SITE_GLOBAL: &str = "%ct";

/// A call of a comptime function in a spot that needs its value at compile
/// time, with each argument known.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Site
{
    /// The call.
    pub id: NodeId,
    /// Source region of the call.
    pub span: Span,
    /// The function as the call writes it.
    pub callee: Ident,
    /// The name of the function the call picked among those of its name.
    pub inner: Name,
    /// The name of the function at its declaration.
    pub decl: Span,
    /// The arguments.
    pub args: Vec<ConstTree>,
    /// The result kind.
    pub ret: DataKind,
}

/// A small program that sets one global to the value of each site.
#[derive(Debug, Clone)]
pub struct SmallProgram
{
    /// The program: the declarations the functions reach, and the globals.
    pub program: Program,
    /// The name of the global of each site, in the order of the sites.
    pub globals: Vec<Name>,
    /// The sites.
    pub sites: Vec<Site>,
    /// The folder of the file of the whole program that is run, whose files
    /// a run can read.
    pub folder: Option<std::path::PathBuf>,
}

/// What one comptime call gave: its value or the diagnostic of a run that
/// stopped, and the note of what it printed.
#[derive(Debug, Clone)]
pub struct Ran
{
    /// The value, or why the run stopped.
    pub value: Result<ConstTree, Diagnostic>,
    /// What the call printed, as a note at the call.
    pub note: Option<Diagnostic>,
}

/// What runs comptime calls: the VM, through a small program.
pub trait ComptimeRunner: Send + Sync
{
    /// Run a small program and read the value of each site.
    ///
    /// # Arguments
    ///
    /// * `small` - The program.
    /// * `options` - The check options of the build.
    ///
    /// # Returns
    ///
    /// What each site gave, in order.
    fn run(&self, small: &SmallProgram, options: &SemcheckOptions) -> Vec<Ran>;
}

/// The rights of comptime runs, as the command grants them.
static GRANTS: std::sync::RwLock<Option<zaka_host::Grants>> =
    std::sync::RwLock::new(None);

/// Set the rights of the comptime runs of the process: the
/// `--comptime-allow-*` flags of the command.
///
/// # Arguments
///
/// * `grants` - The rights.
pub fn set_grants(grants: zaka_host::Grants)
{
    do_if!(let Ok(mut held) = GRANTS.write(), *held = Some(grants));
}

/// The rights of the comptime runs of the process: none unless the command
/// set them.
///
/// # Returns
///
/// The rights.
pub fn grants() -> zaka_host::Grants
{
    GRANTS.read().ok().and_then(|g| g.clone()).unwrap_or_default()
}

/// The runner the build installed.
static RUNNER: OnceLock<Box<dyn ComptimeRunner>> = OnceLock::new();

/// Install the runner of comptime calls, once for the process; a later call
/// keeps the first.
///
/// # Arguments
///
/// * `runner` - The runner.
pub fn install(runner: Box<dyn ComptimeRunner>) { _ = RUNNER.set(runner); }

/// The runner the build installed.
///
/// # Returns
///
/// The runner, or `None` when none is installed.
pub(crate) fn runner() -> Option<&'static dyn ComptimeRunner>
{
    RUNNER.get().map(AsRef::as_ref)
}

/// Whether a value of a kind can stand as a literal of the program: a number,
/// a character, true or false, a member of an enumeration, text, and lists
/// and structs of these.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` when it can.
pub(crate) fn holds_as_constant(kind: DataKind) -> bool
{
    let kind = kind.repr();
    let wrapped = kind.fallible_parts().is_some() || kind.is_optional();
    ret_if!(wrapped || kind.is_table(), false);
    match kind
    {
        DataKind::Int(_) | DataKind::Float(_) | DataKind::Char | DataKind::Bool => true,
        DataKind::List(_) | DataKind::Seq(_) =>
        {
            kind.is_text() || kind.cell_kind().is_some_and(holds_as_constant)
        }
        DataKind::Struct(s) => s.fields().iter().all(|(_, k)| holds_as_constant(*k)),
        _ => false,
    }
}

/// The tree of a folded value.
///
/// # Arguments
///
/// * `folded` - The value.
///
/// # Returns
///
/// The tree, or `None` for a value the folder knows as a whole alone, or an
/// address.
pub(crate) fn tree_of(folded: &Folded) -> Option<ConstTree>
{
    match folded
    {
        Folded::Value(v) => scalar_raw(*v).map(ConstTree::Scalar),
        Folded::Text(t) => Some(ConstTree::Text(t.clone())),
        Folded::Items(items) =>
        {
            items.iter().map(tree_of).collect::<Option<_>>().map(ConstTree::Items)
        }
        Folded::Whole => None,
    }
}

/// The raw bits of a scalar, as a tree holds them.
///
/// # Arguments
///
/// * `v` - The value.
///
/// # Returns
///
/// The bits, or `None` for an address.
pub fn scalar_raw(v: Value) -> Option<u128>
{
    match v
    {
        Value::Int { raw, .. } => Some(raw),
        Value::Float { raw, .. } => Some(u128::from(raw)),
        Value::Char(c) => Some(u128::from(u32::from(c))),
        Value::Bool(b) => Some(u128::from(b)),
        Value::Ptr(_) => None,
    }
}

/// The folded value of a tree of a kind.
///
/// # Arguments
///
/// * `tree` - The tree.
/// * `kind` - Its kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The value, or `None` when the tree does not fit the kind.
pub(crate) fn folded_of(
    tree: &ConstTree, kind: DataKind, platform: u16,
) -> Option<Folded>
{
    let kind = kind.repr();
    match tree
    {
        ConstTree::Scalar(raw) => scalar_value(kind, *raw, platform).map(Folded::Value),
        ConstTree::Text(t) => Some(Folded::Text(t.clone())),
        ConstTree::Items(items) =>
        {
            let kinds = part_kinds(kind, items.len())?;
            let parts = items.iter().zip(kinds).map(|(t, k)| folded_of(t, k, platform));
            parts.collect::<Option<_>>().map(Folded::Items)
        }
    }
}

/// The kind of each part of a list or a struct.
///
/// # Arguments
///
/// * `kind` - The list or struct kind.
/// * `count` - The count of parts.
///
/// # Returns
///
/// The kinds, or `None` for a kind with no parts.
pub fn part_kinds(kind: DataKind, count: usize) -> Option<Vec<DataKind>>
{
    let kind = kind.repr();
    if let Some(s) = kind.as_struct()
    {
        return Some(s.fields().iter().map(|(_, k)| *k).collect());
    }
    kind.cell_kind().map(|cell| vec![cell; count])
}

/// The value of raw bits of a scalar kind.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `raw` - The bits.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The value, or `None` for a kind that is no scalar.
pub fn scalar_value(kind: DataKind, raw: u128, platform: u16) -> Option<Value>
{
    match kind.repr()
    {
        DataKind::Int(k) => Some(Value::int(k, raw, platform)),
        DataKind::Float(bits) => Some(Value::Float { bits, raw: raw as u64 }),
        DataKind::Char => char::from_u32(raw as u32).map(Value::Char),
        DataKind::Bool => Some(Value::Bool(raw != 0)),
        _ => None,
    }
}

/// The comptime function a site calls, as declared: the instance of a
/// generic, or the function of the name.
///
/// # Arguments
///
/// * `program` - The program.
/// * `site` - The site.
///
/// # Returns
///
/// The index of the statement that declares it, or `None`.
fn callee_at(program: &Program, site: &Site) -> Option<usize>
{
    let fns = || {
        program.stmts.iter().enumerate().filter_map(|(i, s)| match s
        {
            Stmt::Fn(f) if f.name.span == site.decl => Some((i, f)),
            _ => None,
        })
    };
    let instance = fns().find(|(_, f)| f.name.name == site.inner);
    instance.or_else(|| fns().next()).map(|(i, _)| i)
}

/// The function a statement declares.
///
/// # Arguments
///
/// * `program` - The program.
/// * `at` - The statement.
///
/// # Returns
///
/// The function.
fn fn_at(program: &Program, at: usize) -> Option<&FnDecl>
{
    tern!(let Some(Stmt::Fn(f)) = program.stmts.get(at), Some(f), None)
}

/// The top-level statements of a program, on where each starts, to find
/// the statement that holds a place.
struct Places
{
    /// The span and index of each statement, in order of start.
    spans: Vec<(Span, usize)>,
}

impl Places
{
    /// The places of the statements of a program up to an end.
    ///
    /// # Arguments
    ///
    /// * `program` - The program.
    /// * `end` - The end of the statements.
    ///
    /// # Returns
    ///
    /// The places.
    fn of(program: &Program, end: usize) -> Self
    {
        let mut spans: Vec<_> = program.stmts[..end]
            .iter()
            .enumerate()
            .map(|(i, s)| (s.span(), i))
            .collect();
        spans.sort_by_key(|(s, i)| (s.lo, *i));
        Places { spans }
    }

    /// The statements that hold a place: one, or a template and its
    /// instances, which share their spans.
    ///
    /// # Arguments
    ///
    /// * `at` - The place.
    ///
    /// # Returns
    ///
    /// The indexes.
    fn holding(&self, at: Span) -> Vec<usize>
    {
        let after = self.spans.partition_point(|(s, _)| s.lo <= at.lo);
        let before = self.spans[..after].iter().rev();
        let holds = |(s, _): &&(Span, usize)| s.contains(&at);
        let first = before.clone().find(holds).map(|(s, _)| *s);
        let Some(span) = first
        else
        {
            return Vec::new();
        };
        before.filter(|(s, _)| *s == span).map(|(_, i)| *i).collect()
    }
}

/// The uses between the top-level statements of a program: the statements
/// each one names, through the names the check resolved.
pub(crate) struct Reach
{
    /// The statements each statement names.
    edges: Vec<Vec<usize>>,
}

impl Reach
{
    /// The uses of a program.
    ///
    /// # Arguments
    ///
    /// * `program` - The program.
    /// * `refs` - Each use of a name with its declaration, in order.
    ///
    /// # Returns
    ///
    /// The uses.
    pub(crate) fn of(program: &Program, refs: &RefTable) -> Self
    {
        let places = Places::of(program, program.stmts.len());
        let named = |stmt: &Stmt| -> Vec<usize> {
            let span = stmt.span();
            let from = refs.partition_point(|(at, _)| at.lo < span.lo);
            let inside = refs[from..].iter().take_while(|(at, _)| at.lo < span.hi);
            let inside = inside.filter(|(at, _)| span.contains(at));
            inside.flat_map(|(_, decl)| places.holding(*decl)).collect()
        };
        Reach { edges: program.stmts.iter().map(named).collect() }
    }

    /// The statements some statements reach, those statements among them.
    ///
    /// # Arguments
    ///
    /// * `roots` - The statements to start from.
    ///
    /// # Returns
    ///
    /// Whether each statement is reached.
    fn forward(&self, roots: &[usize]) -> Vec<bool>
    {
        let mut seen = vec![false; self.edges.len()];
        let mut todo = roots.to_vec();
        while let Some(i) = todo.pop()
        {
            if seen[i]
            {
                continue;
            }
            seen[i] = true;
            todo.extend(self.edges[i].iter().copied().filter(|j| !seen[*j]));
        }
        seen
    }

    /// The statements that reach some statements, those statements among them.
    ///
    /// # Arguments
    ///
    /// * `bad` - The statements.
    ///
    /// # Returns
    ///
    /// Whether each statement reaches one.
    fn backward(&self, bad: &[bool]) -> Vec<bool>
    {
        let mut back: Vec<Vec<usize>> = vec![Vec::new(); self.edges.len()];
        for (i, to) in self.edges.iter().enumerate()
        {
            do_for!(j in to, back[*j].push(i));
        }
        let mut seen = vec![false; self.edges.len()];
        let mut todo: Vec<usize> = (0..bad.len()).filter(|i| bad[*i]).collect();
        while let Some(i) = todo.pop()
        {
            if seen[i]
            {
                continue;
            }
            seen[i] = true;
            todo.extend(back[i].iter().copied().filter(|j| !seen[*j]));
        }
        seen
    }

    /// The statements a small program keeps: each one except a statement
    /// that holds an open site, a global the functions do not reach (its
    /// starting value would run), the start block, and each statement that
    /// reaches one of these. Each other function and kind remains, so a use
    /// the check records no name for still finds its declaration.
    ///
    /// # Arguments
    ///
    /// * `program` - The program.
    /// * `open` - The statements that hold an open site.
    /// * `roots` - The statements of the functions the small program runs.
    ///
    /// # Returns
    ///
    /// Whether each statement remains.
    pub(crate) fn kept(
        &self, program: &Program, open: &HashSet<usize>, roots: &[usize],
    ) -> Vec<bool>
    {
        let reached = self.forward(roots);
        let bad: Vec<bool> = program
            .stmts
            .iter()
            .enumerate()
            .map(|(i, s)| match s
            {
                _ if open.contains(&i) => true,
                Stmt::Decl(_) => !reached[i],
                Stmt::Fn(f) => f.start,
                _ => false,
            })
            .collect();
        self.backward(&bad).into_iter().map(|b| !b).collect()
    }
}

/// The top-level statement of a program that holds each site.
///
/// # Arguments
///
/// * `program` - The program.
/// * `sites` - The sites.
///
/// # Returns
///
/// The statements, as a set.
fn holders(program: &Program, sites: &[Site]) -> HashSet<usize>
{
    let places = Places::of(program, program.stmts.len());
    sites.iter().flat_map(|s| places.holding(s.span)).collect()
}

/// Split the sites a run met into those that can run now and those that
/// wait: a site waits while a statement its function reaches holds another
/// open site.
///
/// # Arguments
///
/// * `program` - The program.
/// * `refs` - Each use of a name with its declaration.
/// * `sites` - The open sites.
///
/// # Returns
///
/// The sites that can run, and those that wait.
pub(crate) fn runnable(
    program: &Program, refs: &RefTable, sites: &[Site],
) -> (Vec<Site>, Vec<Site>)
{
    let open = holders(program, sites);
    let reach = Reach::of(program, refs);
    sites.iter().cloned().partition(|site| {
        let Some(at) = callee_at(program, site)
        else
        {
            return false;
        };
        reach.kept(program, &open, &[at])[at]
    })
}

/// Build the small programs of the sites that can run: one for each file
/// that declares one of their functions, made of that file as the file that
/// is run, the files before it, the statements the functions reach, and one
/// global for each site.
///
/// # Arguments
///
/// * `program` - The program.
/// * `refs` - Each use of a name with its declaration.
/// * `sites` - The sites to run.
/// * `open` - Each open site of the run, those to run among them.
///
/// # Returns
///
/// The small programs, and each site whose function is not found.
pub(crate) fn small_programs(
    program: &Program, refs: &RefTable, sites: &[Site], open: &[Site],
) -> (Vec<SmallProgram>, Vec<Site>)
{
    let mut of_unit: HashMap<u32, Vec<(Site, usize)>> = HashMap::new();
    let mut lost = Vec::new();
    for site in sites
    {
        match callee_at(program, site)
        {
            Some(at) =>
            {
                of_unit.entry(program.unit_of(at)).or_default().push((site.clone(), at))
            }
            None => lost.push(site.clone()),
        }
    }
    let mut units: Vec<_> = of_unit.into_iter().collect();
    units.sort_by_key(|(u, _)| *u);
    let smalls = units
        .into_iter()
        .map(|(unit, group)| small_program(program, refs, open, unit, &group));
    (smalls.collect(), lost)
}

/// Build the small program of the sites whose functions one file declares.
///
/// # Arguments
///
/// * `program` - The program.
/// * `refs` - Each use of a name with its declaration.
/// * `sites` - Each open site of the run.
/// * `unit` - The file.
/// * `group` - Each site, with the statement of its function.
///
/// # Returns
///
/// The small program.
fn small_program(
    program: &Program, refs: &RefTable, sites: &[Site], unit: u32,
    group: &[(Site, usize)],
) -> SmallProgram
{
    let next = unit as usize + 1;
    let end =
        program.units.get(next).map_or(program.stmts.len(), |u| u.first_stmt as usize);
    let roots: Vec<usize> = group.iter().map(|(_, at)| *at).collect();
    let open = holders(program, sites);
    let mut keep = Reach::of(program, refs).kept(program, &open, &roots);
    keep.truncate(end);
    let mut small = pruned(program, &keep, next);
    let mut id = crate::generic::next_id(program);
    let mut globals = Vec::new();
    for (n, (site, at)) in group.iter().enumerate()
    {
        let name = Name::new(&format!("{SITE_GLOBAL}{n}"));
        let call = site_call(program, site, *at, &mut id);
        let ident = Ident::builder().name(name).span(site.span).build();
        #[rustfmt::skip]
        let decl = DeclStmt::builder().name(ident).is_const(false).init(call).span(site.span)
            .build();
        small.stmts.push(Stmt::Decl(decl));
        globals.push(name);
    }
    let sites = group.iter().map(|(s, _)| s.clone()).collect();
    let folder = program.folder.clone();
    SmallProgram { program: small, globals, sites, folder }
}

/// The statements a mask keeps, in the files before an end, with the imports
/// that name a declaration left out dropped.
///
/// # Arguments
///
/// * `program` - The program.
/// * `keep` - Whether each statement before the end remains.
/// * `units` - The count of files that remain.
///
/// # Returns
///
/// The program.
fn pruned(program: &Program, keep: &[bool], units: usize) -> Program
{
    let declared: HashSet<(u32, Name)> = keep
        .iter()
        .enumerate()
        .filter(|(_, k)| **k)
        .flat_map(|(i, _)| {
            declared_names(&program.stmts[i])
                .into_iter()
                .map(move |n| (program.unit_of(i), n))
        })
        .collect();
    let mut stmts = Vec::new();
    // The index each kept statement had, to place the start of each file.
    let mut was = Vec::new();
    for (i, stmt) in program.stmts[..keep.len()].iter().enumerate()
    {
        let kept = match stmt
        {
            Stmt::Import(n) => kept_import(n, &declared, keep, program, i),
            _ => tern!(keep[i], Some(stmt.clone()), None),
        };
        do_if!(kept.is_some(), was.push(i as u32));
        stmts.extend(kept);
    }
    // A program of one file with no imports lists no files.
    let units: Vec<Unit> = program.units[..units.min(program.units.len())]
        .iter()
        .map(|u| {
            let first = was.partition_point(|i| *i < u.first_stmt) as u32;
            Unit { first_stmt: first, ..u.clone() }
        })
        .collect();
    let mut small = program.clone();
    small.stmts = stmts;
    small.units = units;
    small
}

/// An import of a small program: the names whose declarations remain, or
/// nothing when none does or its file keeps no statement.
///
/// # Arguments
///
/// * `n` - The import.
/// * `declared` - The names each file keeps, on the file.
/// * `keep` - Whether each statement remains.
/// * `program` - The program.
/// * `at` - The index of the import.
///
/// # Returns
///
/// The import, or `None`.
fn kept_import(
    n: &zakac_ast::stmt::ImportStmt, declared: &HashSet<(u32, Name)>, keep: &[bool],
    program: &Program, at: usize,
) -> Option<Stmt>
{
    let unit = program.unit_of(at);
    let used = keep.iter().enumerate().any(|(i, k)| *k && program.unit_of(i) == unit);
    let from = n.unit?;
    let names: Vec<Ident> = n
        .names
        .iter()
        .filter(|i| declared.contains(&(from, i.name)))
        .cloned()
        .collect();
    ret_if!(!used || names.is_empty(), None);
    Some(Stmt::Import(zakac_ast::stmt::ImportStmt { names, ..n.clone() }))
}

/// The names a top-level statement declares.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The names.
fn declared_names(stmt: &Stmt) -> Vec<Name>
{
    match stmt
    {
        Stmt::Fn(n) => vec![n.name.name],
        Stmt::Decl(n) => vec![n.name.name],
        Stmt::Struct(n) => vec![n.name.name],
        Stmt::Enum(n) => vec![n.name.name],
        Stmt::Alias(n) => vec![n.name.name],
        Stmt::Trait(n) => vec![n.name.name],
        _ => Vec::new(),
    }
}

/// The call of a site for its global: the call as written, each argument
/// put in as its value.
///
/// # Arguments
///
/// * `program` - The program.
/// * `site` - The site.
/// * `at` - The statement of its function.
/// * `id` - The next free node number.
///
/// # Returns
///
/// The call.
fn site_call(program: &Program, site: &Site, at: usize, id: &mut u32) -> Expr
{
    let params: Vec<Kind> = fn_at(program, at)
        .map(|f| f.params.iter().map(|p| p.kind.clone()).collect())
        .unwrap_or_default();
    let written = find_call(program, site.id);
    let mut call = written.unwrap_or_else(|| {
        CallExpr::builder()
            .id(site.id)
            .callee(site.callee.clone())
            .span(site.span)
            .build()
    });
    call.args = site
        .args
        .iter()
        .zip(params)
        .map(|(value, kind)| {
            *id += 1;
            #[rustfmt::skip]
            let arg = ComptimeExpr::builder().id(NodeId(*id - 1)).callee(site.callee.clone())
                .value(value.clone()).kind(kind).span(site.span).build();
            Expr::Comptime(arg)
        })
        .collect();
    Expr::Call(call)
}

/// The call with a node number.
///
/// # Arguments
///
/// * `program` - The program.
/// * `id` - The node number.
///
/// # Returns
///
/// The call, or `None`.
fn find_call(program: &Program, id: NodeId) -> Option<CallExpr>
{
    /// The walker that finds the call.
    struct Find(NodeId, Option<CallExpr>);
    impl Visitor for Find
    {
        fn visit_expr(&mut self, n: &Expr)
        {
            if let Expr::Call(c) = n
                && c.id == self.0
            {
                self.1 = Some(c.clone());
                return;
            }
            walk_expr(self, n);
        }
    }
    let mut find = Find(id, None);
    find.visit_program(program);
    find.1
}

/// Write the value of each site that ran in place of its call.
///
/// # Arguments
///
/// * `program` - The program.
/// * `values` - The value of each call, with the kind its function declares, on
///   the node number of the call.
///
/// # Returns
///
/// The count of calls written.
pub(crate) fn write_values(
    program: &mut Program, values: &HashMap<NodeId, (ConstTree, Kind)>,
) -> usize
{
    /// The walker that writes the values.
    struct Writer<'a>(&'a HashMap<NodeId, (ConstTree, Kind)>, usize);
    impl VisitorMut for Writer<'_>
    {
        fn visit_expr_mut(&mut self, n: &mut Expr)
        {
            let found =
                tern!(let Expr::Call(c) = n, self.0.get(&c.id).map(|v| (c, v)), None);
            let Some((c, (value, kind))) = found
            else
            {
                return walk_expr_mut(self, n);
            };
            #[rustfmt::skip]
            let done = ComptimeExpr::builder().id(c.id).callee(c.callee.clone()).value(value.clone())
                .kind(kind.clone()).span(c.span).build();
            *n = Expr::Comptime(done);
            self.1 += 1;
        }
    }
    let mut writer = Writer(values, 0);
    writer.visit_program_mut(program);
    writer.1
}

/// The kind of the value of a site as written: built from the result kind
/// the call gave, so an instance of a generic writes its own kinds.
///
/// # Arguments
///
/// * `site` - The site.
///
/// # Returns
///
/// The kind, or `None` for a kind a literal cannot take.
pub(crate) fn declared_ret(site: &Site) -> Option<Kind>
{
    written_kind(site.ret, site.span)
}

/// A kind as the program writes it.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `span` - The source region to give it.
///
/// # Returns
///
/// The kind, or `None` for one with no written form here.
pub fn written_kind(kind: DataKind, span: Span) -> Option<Kind>
{
    let base = |b: BaseKind| Kind::builder().base(b).span(span);
    let named = |name: &str| {
        let ident = Ident::builder().name(Name::new(name)).span(span).build();
        Kind::builder().base(BaseKind::Named).name(ident).span(span).build()
    };
    let wrapped = |cell: DataKind, cells: Cells| {
        written_kind(cell, span).map(|mut k| {
            k.wraps.push(Wrap::List(cells));
            k
        })
    };
    match kind
    {
        DataKind::Int(k) =>
        {
            let bits = tern!(let Width::Bits(n) = k.width, Some(n), None);
            Some(base(BaseKind::Int).signed(k.signed).maybe_bits(bits).build())
        }
        DataKind::Float(bits) => Some(base(BaseKind::Float).bits(bits).build()),
        DataKind::Char => Some(base(BaseKind::Char).build()),
        DataKind::Bool => Some(base(BaseKind::Bool).build()),
        DataKind::Seq(s) if s.text => Some(base(BaseKind::Text).build()),
        DataKind::Seq(s) => wrapped(s.cell, Cells::Inferred),
        DataKind::List(l) => wrapped(l.cell, Cells::Fixed(l.len)),
        DataKind::Struct(s) => Some(named(&s.name)),
        DataKind::Enum(e) => Some(named(&e.name)),
        _ => None,
    }
}
