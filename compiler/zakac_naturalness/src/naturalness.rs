use bon::Builder;
use zakac_ast::expr::BoolLit;
use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::expr::GroupExpr;
use zakac_ast::expr::NextCtx;
use zakac_ast::expr::NotExpr;
use zakac_ast::expr::VerbExpr;
use zakac_ast::stmt::AssignStmt;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::CountedStmt;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::GenericStmt;
use zakac_ast::stmt::IfArm;
use zakac_ast::stmt::PrintArg;
use zakac_ast::stmt::PrintStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::ReturnStmt;
use zakac_ast::stmt::WhileStmt;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_assign;
use zakac_ast::visit::walk_call_expr;
use zakac_ast::visit::walk_call_stmt;
use zakac_ast::visit::walk_counted;
use zakac_ast::visit::walk_decl;
use zakac_ast::visit::walk_generic;
use zakac_ast::visit::walk_group;
use zakac_ast::visit::walk_if_arm;
use zakac_ast::visit::walk_not;
use zakac_ast::visit::walk_print;
use zakac_ast::visit::walk_return;
use zakac_ast::visit::walk_verb;
use zakac_ast::visit::walk_while;
use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_diagnostics::naturalness as diag;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::classifiers::check_classifiers;
use crate::rules::first_match;
use crate::site::FinalSite;
use crate::site::FinalWord;
use crate::site::GroupSite;
use crate::site::Inner;
use crate::site::Place;
use crate::site::PrintSite;
use crate::site::Shape;
use crate::site::Site;
use crate::site::ValueSite;
use crate::site::kw_text;
use crate::site::particle_text;

/// Options of the check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Builder)]
pub struct NaturalnessOptions
{
    /// Report the rules as errors instead of warnings.
    #[builder(default)]
    pub strict: bool,
}

/// Check the particles and the classifiers of a program.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `options` - The check options.
///
/// # Returns
///
/// The diagnostics, in source order; errors when `strict` is set.
pub fn check(program: &Program, options: &NaturalnessOptions) -> Vec<Diagnostic>
{
    let mut checker = Checker::default();
    checker.visit_program(program);
    checker.diags.extend(check_classifiers(program));
    checker.diags.sort_by_key(|d| d.span.lo);
    let level = tern!(options.strict, Level::Error, Level::Warning);
    checker.diags.into_iter().map(|d| d.with_level(level)).collect()
}

/// The walker that builds a site for each word and runs the rules on it.
#[derive(Default)]
struct Checker
{
    /// The diagnostics found so far.
    diags: Vec<Diagnostic>,
}

impl Checker
{
    /// Run the rules on a site and record the first that fires.
    ///
    /// # Arguments
    ///
    /// * `site` - The site.
    fn report(&mut self, site: Site)
    {
        let Some(code) = first_match(&site)
        else
        {
            return;
        };
        self.diags.push(build(code, &site));
    }

    /// Run the rules on a clause-final word.
    ///
    /// # Arguments
    ///
    /// * `site` - The word with its particle and what follows it.
    fn report_final(&mut self, site: FinalSite) { self.report(Site::Final(site)); }

    /// Run the rules on an expression that fills a whole place, when it is
    /// a group.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression.
    /// * `place` - The place it fills.
    fn report_group(&mut self, expr: &Expr, place: Place)
    {
        let Expr::Group(n) = expr
        else
        {
            return;
        };
        let site = GroupSite { place, inner: Inner::of(&n.inner), span: n.span };
        self.report(Site::Group(site));
    }

    /// Run the rules on each argument of a call.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments.
    fn report_args(&mut self, args: &[Expr])
    {
        do_for!(arg in args, self.report_value(arg));
    }

    /// Run the rules on an expression that gives a value: its parentheses
    /// and, when it is a clause, its closing particle.
    ///
    /// # Arguments
    ///
    /// * `expr` - The value.
    fn report_value(&mut self, expr: &Expr)
    {
        self.report_group(expr, Place::Operand);
        do_if!(let Some(site) = ValueSite::of(expr), self.report(Site::Value(site)));
    }
}

impl Visitor for Checker
{
    fn visit_verb(&mut self, n: &VerbExpr)
    {
        #[rustfmt::skip]
        let site = FinalSite { word: FinalWord::Verb(n.op), particle: n.particle, next: n.next, span: n.span };
        self.report_final(site);
        walk_verb(self, n);
    }

    fn visit_bool(&mut self, n: &BoolLit)
    {
        #[rustfmt::skip]
        let site = FinalSite { word: FinalWord::Bool(n.value), particle: n.particle, next: n.next, span: n.span };
        self.report_final(site);
    }

    fn visit_not(&mut self, n: &NotExpr)
    {
        #[rustfmt::skip]
        let site = FinalSite { word: FinalWord::Not, particle: n.particle, next: n.next, span: n.span };
        self.report_final(site);
        walk_not(self, n);
    }

    fn visit_print(&mut self, n: &PrintStmt)
    {
        let shape = Shape::of(&n.value);
        let span = n.value.span();
        self.report(Site::Print(PrintSite { shape, written: n.slot, span }));
        do_if!(let PrintArg::Expr(e) = &n.value, self.report_group(e, Place::Operand));
        walk_print(self, n);
    }

    fn visit_assign(&mut self, n: &AssignStmt)
    {
        self.report_group(&n.value, Place::Operand);
        walk_assign(self, n);
    }

    fn visit_return(&mut self, n: &ReturnStmt)
    {
        do_if!(let Some(e) = &n.value, self.report_value(e));
        walk_return(self, n);
    }

    fn visit_decl(&mut self, n: &DeclStmt)
    {
        do_if!(let Some(e) = &n.init, self.report_value(e));
        walk_decl(self, n);
    }

    fn visit_generic(&mut self, n: &GenericStmt)
    {
        do_if!(let Some(e) = &n.init, self.report_value(e));
        self.report_group(&n.cond, Place::Condition);
        walk_generic(self, n);
    }

    fn visit_counted(&mut self, n: &CountedStmt)
    {
        self.report_value(&n.from);
        self.report_value(&n.to);
        do_if!(let Some(e) = &n.step, self.report_value(e));
        walk_counted(self, n);
    }

    fn visit_if_arm(&mut self, n: &IfArm)
    {
        self.report_group(&n.cond, Place::Condition);
        walk_if_arm(self, n);
    }

    fn visit_while(&mut self, n: &WhileStmt)
    {
        self.report_group(&n.cond, Place::Condition);
        walk_while(self, n);
    }

    fn visit_call_stmt(&mut self, n: &CallStmt)
    {
        self.report_args(&n.args);
        walk_call_stmt(self, n);
    }

    fn visit_call_expr(&mut self, n: &CallExpr)
    {
        self.report_args(&n.args);
        walk_call_expr(self, n);
    }

    fn visit_group(&mut self, n: &GroupExpr)
    {
        self.report_group(&n.inner, Place::Nested);
        walk_group(self, n);
    }
}

/// Build the diagnostic of a rule at a site.
///
/// # Arguments
///
/// * `code` - The code of the rule that fired.
/// * `site` - The site.
///
/// # Returns
///
/// The diagnostic.
fn build(code: Code, site: &Site) -> Diagnostic
{
    match (code, site)
    {
        (Code::NegativeBeforeOr, Site::Final(n)) =>
        {
            diag::negative_before_or(n.span, n.word.text())
        }
        (Code::ParticleBeforeAttached, Site::Final(n)) =>
        {
            let particle = particle_text(n.particle);
            let next =
                tern!(n.next == NextCtx::Then, kw_text(Kw::Then), kw_text(Kw::While));
            let next = next.unwrap_or_default();
            diag::particle_before_attached(n.span, n.word.text(), particle, next)
        }
        (Code::NotWithThi, Site::Final(n)) =>
        {
            diag::not_with_thi(n.span, particle_text(n.particle))
        }
        (Code::PaBeforeAnd, Site::Final(n)) =>
        {
            diag::pa_before_and(n.span, n.word.text(), particle_text(n.particle))
        }
        (Code::PrintSentenceWithKo, Site::Print(n)) =>
        {
            diag::print_sentence_with_ko(n.span)
        }
        (Code::PrintValueWithHu, Site::Print(n)) => diag::print_value_with_hu(n.span),
        (Code::RedundantGroup, Site::Group(n)) => diag::redundant_group(n.span),
        (Code::PaInValue, Site::Value(n)) =>
        {
            diag::pa_in_value(n.span, n.word.text(), particle_text(n.tail))
        }
        _ => unreachable!("each rule builds the diagnostic of its own site kind"),
    }
}
