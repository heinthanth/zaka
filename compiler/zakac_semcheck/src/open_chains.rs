//! Compile-time conditionals whose conditions call comptime functions. The
//! loader leaves such a chain open; the check works out each open condition
//! on a duplicate of the program, where the condition stands as a constant in
//! place of its chain, and a new read of the program settles the chain with
//! the truth the run gave. A condition that names a function an open arm of
//! another chain declares waits for the next read.

use std::collections::HashSet;

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::name::Name;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::IfStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::target::Target;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_expr;
use zakac_ast::visit::walk_stmt_mut;
use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::fold as fd;
use zakac_diagnostics::has_errors;
use zakac_parser::comptime::CondPlace;
use zakac_parser::comptime::Resolved;
use zakac_parser::comptime::truth;
use zakac_parser::load::Loaded;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::semcheck::SemcheckOptions;
use crate::semcheck::semcheck;

/// The start of the name of the constant that holds an open condition in
/// the duplicate of the program; no written name holds `%`.
const COND_CONST: &str = "%cond";

/// Read a program until no compile-time conditional is open: each read
/// settles the chains whose conditions the read before worked out.
///
/// # Arguments
///
/// * `load` - Read the program with the conditions worked out so far.
/// * `options` - The check options.
///
/// # Returns
///
/// The program with each chain settled, or with the faults of a condition
/// that could not be worked out; or the error of a read.
pub fn settled<E>(
    mut load: impl FnMut(&Resolved) -> Result<Loaded, E>, options: &SemcheckOptions,
) -> Result<Loaded, E>
{
    let mut resolved = Resolved::new();
    loop
    {
        let mut loaded = load(&resolved)?;
        // The levels hold as the program is read: a lint code allowed through a
        // name stops no pass.
        loaded.diagnostics =
            options.levels().relevel(std::mem::take(&mut loaded.diagnostics));
        ret_if!(
            loaded.open_conds.is_empty() || has_errors(&loaded.diagnostics),
            Ok(loaded)
        );
        match work_out(&loaded, options)
        {
            Ok(values) => resolved.extend(values),
            Err(diags) =>
            {
                loaded.diagnostics.extend(diags);
                loaded.open_conds.clear();
                return Ok(loaded);
            }
        }
    }
}

/// Read a program and its imports from disk with the passes of [`settled`].
///
/// # Arguments
///
/// * `root` - The file that is run.
/// * `source_map` - The source map, which gets each file of each read.
/// * `lexer` - The lexer options.
/// * `target` - The target.
/// * `options` - The check options.
///
/// # Returns
///
/// The program, each chain settled, or the error of a file that could not
/// be read.
pub fn load_settled(
    root: &std::sync::Arc<zakac_span::source_file::SourceFile>,
    source_map: &mut zakac_span::source_map::SourceMap,
    lexer: &zakac_lexer::lexer::LexerOptions, target: &Target,
    options: &SemcheckOptions,
) -> zakac_host_errors::Result<Loaded>
{
    let read = |r: &Resolved| {
        zakac_parser::load::load_resolved(root, source_map, lexer, target, r)
    };
    settled(read, options)
}

/// Read one file with the passes of [`settled`], for the host.
///
/// # Arguments
///
/// * `source` - The file.
/// * `lexer` - The lexer options.
/// * `options` - The check options.
///
/// # Returns
///
/// The program, each chain settled.
pub fn single(
    source: &zakac_span::source_file::SourceFile,
    lexer: &zakac_lexer::lexer::LexerOptions, options: &SemcheckOptions,
) -> Loaded
{
    single_for(source, lexer, &Target::host(), options)
}

/// Read one file with the passes of [`settled`], for a target.
///
/// # Arguments
///
/// * `source` - The file.
/// * `lexer` - The lexer options.
/// * `target` - The target.
/// * `options` - The check options.
///
/// # Returns
///
/// The program, each chain settled.
pub fn single_for(
    source: &zakac_span::source_file::SourceFile,
    lexer: &zakac_lexer::lexer::LexerOptions, target: &Target,
    options: &SemcheckOptions,
) -> Loaded
{
    let read = |r: &Resolved| {
        Ok::<_, std::convert::Infallible>(Loaded::single_resolved(
            source, lexer, target, r,
        ))
    };
    match settled(read, options)
    {
        Ok(loaded) => loaded,
        Err(never) => match never {},
    }
}

/// One open condition in the duplicate of the program.
struct Open
{
    /// The name of its constant.
    name: Name,
    /// Its place.
    place: CondPlace,
    /// The condition.
    span: Span,
    /// The names of the functions its chain's arms declare.
    arm_fns: HashSet<Name>,
}

/// Work out each open condition of a read: put each in a constant in place
/// of its chain, check the duplicate, which runs the comptime calls, and settle
/// what is left.
///
/// # Arguments
///
/// * `loaded` - The read.
/// * `options` - The check options.
///
/// # Returns
///
/// The truth of each condition worked out, at least one; or the faults.
fn work_out(
    loaded: &Loaded, options: &SemcheckOptions,
) -> Result<Vec<(CondPlace, bool)>, Vec<Diagnostic>>
{
    let mut dup = loaded.program.clone();
    let mut lift = Lift { open: &loaded.open_conds, found: Vec::new() };
    dup.stmts = lift.stmts(std::mem::take(&mut dup.stmts));
    let found = lift.found;
    let out = semcheck(&mut dup, options);
    let target = dup.target.unwrap_or_else(Target::host);
    let inits = consts_of(&dup);
    let mut values = Vec::new();
    let mut waiting = Vec::new();
    let mut faults = Vec::new();
    for open in &found
    {
        let inside: Vec<Diagnostic> = out
            .diagnostics
            .iter()
            .filter(|d| open.span.contains(&d.span))
            .cloned()
            .collect();
        let unknown = |d: &Diagnostic| {
            matches!(d.code, Code::UndeclaredName | Code::UndeclaredFunction)
        };
        let undeclared = !inside.is_empty() && inside.iter().all(unknown);
        let init = inits.iter().find(|(n, _)| *n == open.name).map(|(_, e)| e);
        let worked = init.filter(|_| !has_errors(&inside)).map(|e| {
            let text = e.to_sexpr();
            truth(e, &target, &|_| text.clone())
        });
        match worked
        {
            Some(Ok(b)) => values.push((open.place.clone(), b)),
            _ if undeclared => waiting.push((open, inside)),
            Some(Err(d)) => faults.push(d),
            None => faults.extend(inside),
        }
    }
    ret_if!(!faults.is_empty(), Err(faults));
    ret_if!(!values.is_empty(), Ok(values));
    Err(waiting
        .into_iter()
        .flat_map(|(open, inside)| still_open(open, inside))
        .collect())
}

/// The faults of a condition still open after a read settled nothing: a
/// call of a function its own chain's arms declare waits on itself; else the
/// names it could not find.
///
/// # Arguments
///
/// * `open` - The condition.
/// * `inside` - The diagnostics of the check inside it.
///
/// # Returns
///
/// The faults.
fn still_open(open: &Open, inside: Vec<Diagnostic>) -> Vec<Diagnostic>
{
    let own = inside.iter().find_map(|d| {
        open.arm_fns
            .iter()
            .find(|n| d.message.contains(n.as_str()))
            .map(|n| (d.span, *n))
    });
    match own
    {
        Some((span, name)) => vec![fd::comptime_waits_on_itself(span, name.as_str())],
        None => inside,
    }
}

/// The value of each constant of the duplicate that holds an open condition.
///
/// # Arguments
///
/// * `program` - The duplicate.
///
/// # Returns
///
/// Each constant's name with its value.
fn consts_of(program: &Program) -> Vec<(Name, Expr)>
{
    /// The walker that finds the constants.
    struct Consts(Vec<(Name, Expr)>);
    impl Visitor for Consts
    {
        fn visit_decl(&mut self, n: &DeclStmt)
        {
            let ours = n.name.name.as_str().starts_with(COND_CONST);
            if let Some(init) = n.init.as_ref().filter(|_| ours)
            {
                self.0.push((n.name.name, init.clone()));
            }
        }
    }
    let mut consts = Consts(Vec::new());
    consts.visit_program(program);
    consts.0
}

/// The walker that puts each open chain's open conditions in constants in
/// place of the chain.
struct Lift<'a>
{
    /// Each open condition of the read, with its place.
    open: &'a [(Span, CondPlace)],
    /// The conditions lifted so far.
    found: Vec<Open>,
}

impl Lift<'_>
{
    /// The statements with each open chain replaced with the constants of
    /// its open conditions.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    ///
    /// # Returns
    ///
    /// The statements.
    fn stmts(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt>
    {
        let mut out = Vec::with_capacity(stmts.len());
        for mut stmt in stmts
        {
            match stmt
            {
                Stmt::If(n) if n.comptime && self.is_open(&n) =>
                {
                    out.extend(self.lift(&n))
                }
                _ =>
                {
                    walk_stmt_mut(self, &mut stmt);
                    out.push(stmt);
                }
            }
        }
        out
    }

    /// Whether a chain holds an open condition.
    ///
    /// # Arguments
    ///
    /// * `n` - The chain.
    ///
    /// # Returns
    ///
    /// `true` when it does.
    fn is_open(&self, n: &IfStmt) -> bool
    {
        n.arms.iter().any(|a| self.place_of(&a.cond).is_some())
    }

    /// The place of an open condition.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    ///
    /// # Returns
    ///
    /// The place, or `None` for a condition that is not open.
    fn place_of(&self, cond: &Expr) -> Option<CondPlace>
    {
        self.open.iter().find(|(s, _)| *s == cond.span()).map(|(_, k)| k.clone())
    }

    /// The constants of the open conditions of a chain.
    ///
    /// # Arguments
    ///
    /// * `n` - The chain.
    ///
    /// # Returns
    ///
    /// One constant for each open condition.
    fn lift(&mut self, n: &IfStmt) -> Vec<Stmt>
    {
        let arm_fns = arm_fns(n);
        let mut out = Vec::new();
        for arm in &n.arms
        {
            let Some(place) = self.place_of(&arm.cond)
            else
            {
                continue;
            };
            let span = arm.cond.span();
            let name = Name::new(&format!("{COND_CONST}{}", self.found.len()));
            let ident = Ident::builder().name(name).span(span).build();
            #[rustfmt::skip]
            let decl = DeclStmt::builder().name(ident).is_const(true).init(arm.cond.clone())
                .span(span).build();
            out.push(Stmt::Decl(decl));
            self.found.push(Open { name, place, span, arm_fns: arm_fns.clone() });
        }
        out
    }
}

impl VisitorMut for Lift<'_>
{
    fn visit_block_mut(&mut self, n: &mut zakac_ast::stmt::BlockStmt)
    {
        n.stmts = self.stmts(std::mem::take(&mut n.stmts));
    }
}

/// The names of the functions the arms of a chain declare, at each depth.
///
/// # Arguments
///
/// * `n` - The chain.
///
/// # Returns
///
/// The names.
fn arm_fns(n: &IfStmt) -> HashSet<Name>
{
    /// The walker that finds the functions.
    struct Fns(HashSet<Name>);
    impl Visitor for Fns
    {
        fn visit_fn(&mut self, n: &zakac_ast::stmt::FnDecl)
        {
            _ = self.0.insert(n.name.name);
        }

        fn visit_expr(&mut self, n: &Expr) { walk_expr(self, n) }
    }
    let mut fns = Fns(HashSet::new());
    let blocks = n.arms.iter().map(|a| &a.block).chain(n.else_block.as_ref());
    blocks.for_each(|b| fns.visit_block(b));
    fns.0
}
