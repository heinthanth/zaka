use std::collections::BTreeMap;

use zakac_ast::expr::StructLit;
use zakac_ast::kind::BaseKind;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::AllocStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::ResizeStmt;
use zakac_ast::stmt::SizeArg;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_alloc;
use zakac_ast::visit::walk_resize;
use zakac_ast::visit::walk_struct_lit;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::naturalness as diag;
use zakac_rust_sugars::ret_if;

/// The classifier of each kind that declares none: the general `ခု`.
pub const GENERAL_CLASSIFIER: &str = "ခု";

/// Report each count word whose classifier is not the one of the kind it
/// counts: the classifier a struct declares, `ခု` for each other kind.
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
    let words = program
        .stmts
        .iter()
        .filter_map(|s| match s
        {
            Stmt::Struct(decl) =>
            {
                let word =
                    decl.classifier.as_ref().map_or(GENERAL_CLASSIFIER, |c| &c.name);
                Some((decl.name.name.as_str(), word))
            }
            _ => None,
        })
        .collect();
    let mut sizes = Sizes { words, diags: Vec::new() };
    sizes.visit_program(program);
    sizes.diags
}

/// The walker over the sizes of heap statements and the counts of struct
/// literals.
struct Sizes<'p>
{
    /// The classifier of each struct.
    words: BTreeMap<&'p str, &'p str>,
    /// The diagnostics found so far.
    diags: Vec<Diagnostic>,
}

impl Sizes<'_>
{
    /// Report a size whose count word has another classifier than its kind.
    ///
    /// # Arguments
    ///
    /// * `size` - The size.
    fn check(&mut self, size: &SizeArg)
    {
        let (Some(unit), Some(found)) = (&size.unit, &size.classifier)
        else
        {
            return;
        };
        let named = unit.wraps.is_empty() && unit.base == BaseKind::Named;
        let own = unit.name.as_ref().and_then(|n| self.words.get(n.name.as_str()));
        let want = own.filter(|_| named).copied().unwrap_or(GENERAL_CLASSIFIER);
        ret_if!(found.name == want);
        let kind = unit.to_sexpr();
        self.diags.push(diag::classifier_mismatch(
            found.span,
            &kind,
            want,
            &found.name,
        ));
    }
}

impl Visitor for Sizes<'_>
{
    fn visit_alloc(&mut self, n: &AllocStmt)
    {
        self.check(&n.size);
        walk_alloc(self, n);
    }

    fn visit_resize(&mut self, n: &ResizeStmt)
    {
        self.check(&n.size);
        walk_resize(self, n);
    }

    fn visit_struct_lit(&mut self, n: &StructLit)
    {
        // The checker reports a name that no struct declaration gives.
        let name = n.name.name.as_str();
        let found = &n.classifier;
        let want = self.words.get(name).copied().filter(|w| *w != found.name);
        self.diags.extend(want.map(|want| {
            diag::literal_classifier_mismatch(found.span, name, want, &found.name)
        }));
        walk_struct_lit(self, n);
    }
}
