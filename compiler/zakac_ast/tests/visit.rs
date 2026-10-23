use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::IntLit;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::AssignStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::Slot;
use zakac_ast::verb::VERBS;
use zakac_ast::verb::VerbOp;
use zakac_ast::verb::verb_for;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_expr_mut;
use zakac_lexer::token::DigitScript;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Particle;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

/// A span from `lo` to `hi`.
fn sp(lo: u32, hi: u32) -> Span { Span::new(Pos::new(lo), Pos::new(hi)) }

/// An integer literal node.
fn int(id: u32, value: u128, lo: u32) -> Expr
{
    let lit = IntLit::builder()
        .id(NodeId(id))
        .value(value)
        .base(10)
        .script(DigitScript::Western)
        .span(sp(lo, lo + 1))
        .build();
    Expr::Int(lit)
}

/// `x ကို 1 နှင့် 2 ၏ ပေါင်းလဒ် ဟု ထားပါ။` written out in full.
fn sample() -> Program
{
    let sum = VerbExpr::builder()
        .id(NodeId(3))
        .op(VerbOp::Add)
        .args(vec![int(1, 1, 10), int(2, 2, 20)])
        .particle(Particle::None)
        .is_cmp(false)
        .span(sp(10, 30))
        .build();
    let assign = AssignStmt::builder()
        .target(Ident::builder().name("x").span(sp(0, 1)).build())
        .value(Expr::Verb(sum))
        .span(sp(0, 40))
        .build();
    Program::builder().stmts(vec![Stmt::Assign(assign)]).span(sp(0, 41)).build()
}

/// Records `enter` / `leave` calls as `+label` / `-label`.
#[derive(Default)]
struct Recorder(Vec<String>);

impl Visitor for Recorder
{
    fn enter(&mut self, n: &dyn Node) { self.0.push(format!("+{}", n.label())) }

    fn leave(&mut self, n: &dyn Node) { self.0.push(format!("-{}", n.label())) }
}

#[test]
fn walk_visits_children_in_source_order()
{
    let mut rec = Recorder::default();
    rec.visit_program(&sample());
    let want = [
        "+program", "+assign", "+ident", "-ident", "+verb", "+int", "-int", "+int",
        "-int", "-verb", "-assign", "-program",
    ];
    assert_eq!(rec.0, want);
}

#[test]
fn accept_calls_the_matching_visit_method()
{
    let mut rec = Recorder::default();
    int(1, 7, 0).accept(&mut rec);
    assert_eq!(rec.0, ["+int", "-int"]);
}

/// Replaces each integer literal with a name.
struct IntToName;

impl VisitorMut for IntToName
{
    fn visit_expr_mut(&mut self, n: &mut Expr)
    {
        let Expr::Int(lit) = n
        else
        {
            return walk_expr_mut(self, n);
        };
        let name = NameExpr::builder().id(lit.id).name("n").span(lit.span).build();
        *n = Expr::Name(name);
    }
}

/// Moves each span right with a fixed offset.
struct Shift(u32);

impl VisitorMut for Shift
{
    fn visit_span_mut(&mut self, span: &mut Span)
    {
        *span = sp(span.lo.as_u32() + self.0, span.hi.as_u32() + self.0);
    }
}

#[test]
fn visitor_mut_replaces_nodes_in_place()
{
    let mut program = sample();
    IntToName.visit_program_mut(&mut program);
    let Stmt::Assign(assign) = &program.stmts[0]
    else
    {
        panic!("not an assignment")
    };
    let Expr::Verb(sum) = &assign.value
    else
    {
        panic!("not a verb")
    };
    assert!(sum.args.iter().all(|a| matches!(a, Expr::Name(_))));
}

#[test]
fn visitor_mut_reaches_each_span()
{
    let mut program = sample();
    Shift(100).visit_program_mut(&mut program);
    assert_eq!(program.span, sp(100, 141));
    let Stmt::Assign(assign) = &program.stmts[0]
    else
    {
        panic!("not an assignment")
    };
    assert_eq!(assign.target.span, sp(100, 101));
    assert_eq!(assign.value.span(), sp(110, 130));
}

#[test]
fn valence_table_has_one_row_per_verb()
{
    for verb in VERBS
    {
        assert_eq!(verb_for(verb.kw), Some(verb));
        assert!(!verb.slots.is_empty());
        assert!(!(verb.is_stmt && verb.is_cmp));
    }
    assert!(verb_for(Kw::Not).is_none());
    assert_eq!(verb_for(Kw::Assign).unwrap().slots, &[Slot::Ko, Slot::Hu]);
}

#[test]
fn shift_slots_match_either_direction()
{
    use zakac_ast::verb::Dir;
    assert!(Slot::Bit(Dir::Right).fills(Slot::Bit(Dir::Left)));
    assert!(Slot::Ko.fills(Slot::Ko));
    assert!(!Slot::Ko.fills(Slot::Hu));
}

#[test]
fn kind_builder_keeps_optional_fields()
{
    let kind = Kind::builder().base(BaseKind::Int).bits(8).span(sp(0, 3)).build();
    assert_eq!(kind.bits, Some(8));
    assert_eq!(kind.signed, None);
}

#[test]
fn sexpr_prints_the_tree_compactly()
{
    use zakac_ast::s_expr::ToSexpr;
    assert_eq!(sample().to_sexpr(), "{(set x (add 1 2))}");
    let kind = Kind::builder().base(BaseKind::Int).bits(8).signed(false).span(sp(0, 1));
    assert_eq!(kind.build().to_sexpr(), "u8");
    let size = Kind::builder().base(BaseKind::Int).span(sp(0, 1)).build();
    assert_eq!(size.to_sexpr(), "isize");
    let float = Kind::builder().base(BaseKind::Float).span(sp(0, 1)).build();
    assert_eq!(float.to_sexpr(), "f64");
}
