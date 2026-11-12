use std::collections::BTreeMap;

use zakac_ast::expr::StructLit;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::AllocStmt;
use zakac_ast::stmt::BlitStmt;
use zakac_ast::stmt::FillStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::ResizeStmt;
use zakac_ast::stmt::SizeArg;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_alloc;
use zakac_ast::visit::walk_blit;
use zakac_ast::visit::walk_fill;
use zakac_ast::visit::walk_resize;
use zakac_ast::visit::walk_struct_lit;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::naturalness as diag;
use zakac_lexer::reserved::PLURAL;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

/// The classifier of each kind that declares none: the general `ခု`.
pub const GENERAL_CLASSIFIER: &str = "ခု";

/// The classifier of `စာလုံး`.
pub const CHARACTER_CLASSIFIER: &str = "လုံး";

/// The most aliases one alias can lead through to the kind it counts with.
const ALIAS_DEPTH: usize = 16;

/// Report each measure word whose classifier is not the one of the kind it
/// counts: in each slot of a kind, after the kind of a size, and at the end of
/// a struct literal. A struct, an enumeration and an alias count with the
/// word each declares, an alias with none with the word of its kind,
/// `စာလုံး` with `လုံး`, and each other kind with `ခု`.
///
/// # Arguments
///
/// * `program` - The tree.
///
/// # Returns
///
/// The `ZK0509` diagnostics.
pub(crate) fn check_classifiers(program: &Program) -> Vec<Diagnostic>
{
    let mut words = BTreeMap::new();
    let mut aliases = BTreeMap::new();
    for stmt in &program.stmts
    {
        match stmt
        {
            Stmt::Struct(d) =>
            {
                _ = words.insert(d.name.name.as_str(), word(&d.classifier))
            }
            Stmt::Enum(d) =>
            {
                let name = d.name.name.as_str();
                let stem = name.strip_suffix(PLURAL).filter(|s| !s.is_empty());
                do_for!(n in std::iter::once(name).chain(stem), {
                    words.insert(n, word(&d.classifier));
                });
            }
            Stmt::Alias(d) =>
            {
                let own = d.classifier.as_ref().map(|c| c.name.as_str());
                aliases.insert(d.name.name.as_str(), (own, &d.kind));
            }
            _ => (),
        }
    }
    let mut sizes = Sizes { words, aliases, unit: None, diags: Vec::new() };
    sizes.visit_program(program);
    sizes.diags
}

/// The word a declaration gives, or `ခု` when it gives none.
///
/// # Arguments
///
/// * `classifier` - The classifier clause of the declaration.
///
/// # Returns
///
/// The word.
fn word(classifier: &Option<zakac_ast::expr::Ident>) -> &str
{
    classifier.as_ref().map_or(GENERAL_CLASSIFIER, |c| c.name.as_str())
}

/// The walker over the kinds, the sizes and the struct literals of a program.
struct Sizes<'p>
{
    /// The classifier of each struct and enumeration.
    words: BTreeMap<&'p str, &'p str>,
    /// The word each alias declares, when it declares one, and its kind.
    aliases: BTreeMap<&'p str, (Option<&'p str>, &'p Kind)>,
    /// The kind of the size being walked, whose word the size reports.
    unit: Option<Span>,
    /// The diagnostics found so far.
    diags: Vec<Diagnostic>,
}

impl Sizes<'_>
{
    /// The classifier a kind counts with.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `depth` - The aliases followed so far.
    ///
    /// # Returns
    ///
    /// The classifier.
    fn want(&self, kind: &Kind, depth: usize) -> &str
    {
        let alias = kind.alias.as_ref().filter(|_| depth < ALIAS_DEPTH);
        let found = alias.and_then(|a| self.aliases.get(a.name.as_str()));
        if let Some((own, target)) = found
        {
            return own.unwrap_or_else(|| self.want(target, depth + 1));
        }
        ret_if!(!kind.wraps.is_empty(), GENERAL_CLASSIFIER);
        ret_if!(kind.base == BaseKind::Char, CHARACTER_CLASSIFIER);
        let named = kind.name.as_ref().filter(|_| kind.base == BaseKind::Named);
        let own = named.and_then(|n| self.words.get(n.name.as_str()));
        own.copied().unwrap_or(GENERAL_CLASSIFIER)
    }

    /// Report the measure word of a kind in a slot, and of each kind in the
    /// slots of a function kind inside it: its parameters and its result.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    fn check_kind(&mut self, kind: &Kind)
    {
        ret_if!(self.unit == Some(kind.span));
        if let Some(found) = &kind.classifier
        {
            let want = self.want(kind, 0).to_string();
            let text = kind.to_sexpr();
            do_if!(
                found.name.as_str() != want,
                self.diags.push(diag::decl_classifier_mismatch(
                    found.span,
                    &text,
                    &want,
                    found.name.as_str(),
                ))
            );
        }
        if let Some(sig) = &kind.sig
        {
            do_for!(p in &sig.params, self.check_kind(&p.kind));
            do_if!(let Some(ret) = &sig.ret, self.check_kind(ret));
        }
        let parts = kind.parts.as_deref();
        let sides = parts.into_iter().flat_map(|p| p.ok.iter().chain(&p.err));
        do_for!(side in sides, self.check_kind(side));
        let table = kind.table.as_deref().into_iter();
        do_for!(k in table.flat_map(|t| [&t.index, &t.value]), self.check_kind(k));
    }

    /// Report a size whose count word has another classifier than its kind.
    ///
    /// # Arguments
    ///
    /// * `size` - The size.
    fn check_size(&mut self, size: &SizeArg)
    {
        self.unit = size.unit.as_ref().map(|u| u.span);
        let Some(unit) = &size.unit
        else
        {
            return;
        };
        let Some(found) = &unit.classifier
        else
        {
            return;
        };
        let want = self.want(unit, 0).to_string();
        ret_if!(found.name.as_str() == want);
        let kind = unit.to_sexpr();
        self.diags.push(diag::classifier_mismatch(
            found.span,
            &kind,
            &want,
            found.name.as_str(),
        ));
    }
}

impl Visitor for Sizes<'_>
{
    fn visit_kind(&mut self, n: &Kind) { self.check_kind(n); }

    fn visit_alloc(&mut self, n: &AllocStmt)
    {
        self.check_size(&n.size);
        walk_alloc(self, n);
    }

    fn visit_resize(&mut self, n: &ResizeStmt)
    {
        self.check_size(&n.size);
        walk_resize(self, n);
    }

    fn visit_blit(&mut self, n: &BlitStmt)
    {
        self.check_size(&n.size);
        walk_blit(self, n);
    }

    fn visit_fill(&mut self, n: &FillStmt)
    {
        self.check_size(&n.size);
        walk_fill(self, n);
    }

    fn visit_struct_lit(&mut self, n: &StructLit)
    {
        // The checker reports a name that no struct declaration gives.
        // The notation form `<struct>(…)` writes no classifier to check.
        let name = n.name.name.as_str();
        let Some(found) = &n.classifier
        else
        {
            return walk_struct_lit(self, n);
        };
        let want = self.words.get(name).copied().filter(|w| found.name != *w);
        self.diags.extend(want.map(|want| {
            diag::literal_classifier_mismatch(found.span, name, want, &found.name)
        }));
        walk_struct_lit(self, n);
    }
}
