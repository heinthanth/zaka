use zakac_ast::expr::Expr;
use zakac_ast::expr::GroupExpr;
use zakac_ast::expr::IntLit;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::NullLit;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::expr::SizeOfExpr;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::AllocStmt;
use zakac_ast::stmt::AssignStmt;
use zakac_ast::stmt::FreeStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::ResizeStmt;
use zakac_ast::stmt::SizeArg;
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

/// A variable node.
fn name(id: u32, text: &str, lo: u32) -> Expr
{
    let n = NameExpr::builder().id(NodeId(id)).name(text).span(sp(lo, lo + 1)).build();
    Expr::Name(n)
}

/// A pointer kind: `depth` pointer levels over `base`.
fn ptr_kind(base: BaseKind, bits: Option<u16>, depth: u8) -> Kind
{
    let wraps = vec![Wrap::Ptr; usize::from(depth)];
    Kind::builder().base(base).maybe_bits(bits).wraps(wraps).span(sp(0, 1)).build()
}

/// `p ၏ တန်ဖိုး` over the variable `p`.
fn deref(id: u32, inner: Expr) -> Expr
{
    let span = inner.span();
    let verb = VerbExpr::builder()
        .id(NodeId(id))
        .op(VerbOp::Deref)
        .args(vec![inner])
        .particle(Particle::None)
        .is_cmp(false)
        .span(span)
        .build();
    Expr::Verb(verb)
}

/// The size `10 isize`.
fn ten_isize() -> SizeArg
{
    let unit = Kind::builder().base(BaseKind::Int).span(sp(3, 4)).build();
    SizeArg::builder().count(int(2, 10, 0)).unit(unit).span(sp(0, 4)).build()
}

/// `x ကို 1 နှင့် 2 ၏ ပေါင်းလဒ် ဟု ထားပါ။` written out in full.
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
        .target(name(4, "x", 0))
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
        "+program", "+assign", "+name", "-name", "+verb", "+int", "-int", "+int",
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
    assert_eq!(assign.target.span(), sp(100, 101));
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
    assert_eq!(sample().to_sexpr(), "(program (set x (add 1 2)))");
    let kind = Kind::builder().base(BaseKind::Int).bits(8).signed(false).span(sp(0, 1));
    assert_eq!(kind.build().to_sexpr(), "u8");
    let size = Kind::builder().base(BaseKind::Int).span(sp(0, 1)).build();
    assert_eq!(size.to_sexpr(), "isize");
    let float = Kind::builder().base(BaseKind::Float).span(sp(0, 1)).build();
    assert_eq!(float.to_sexpr(), "f64");
}

#[test]
fn heap_nodes_are_walked()
{
    let free = FreeStmt::builder().target(deref(2, name(1, "p", 0))).span(sp(0, 9));
    let free = Stmt::Free(free.build());
    let alloc = AllocStmt::builder().size(ten_isize()).target(name(1, "p", 6));
    let alloc = Stmt::Alloc(alloc.span(sp(0, 9)).build());
    let mut rec = Recorder::default();
    rec.visit_stmt(&free);
    rec.visit_stmt(&alloc);
    let want = [
        "+free", "+verb", "+name", "-name", "-verb", "-free", "+alloc", "+int", "-int",
        "+kind", "-kind", "+name", "-name", "-alloc",
    ];
    assert_eq!(rec.0, want);
    assert_eq!(free.to_sexpr(), "(free (deref p))");
    assert_eq!(alloc.to_sexpr(), "(alloc p 10 isize)");
}

#[test]
fn resize_walks_the_target_first()
{
    let resize = ResizeStmt::builder().target(name(1, "p", 0)).size(ten_isize());
    let resize = Stmt::Resize(resize.span(sp(0, 9)).build());
    let mut rec = Recorder::default();
    rec.visit_stmt(&resize);
    let want =
        ["+resize", "+name", "-name", "+int", "-int", "+kind", "-kind", "-resize"];
    assert_eq!(rec.0, want);
    assert_eq!(resize.to_sexpr(), "(resize p 10 isize)");
}

#[test]
fn pointer_kinds_print_with_stars()
{
    assert_eq!(ptr_kind(BaseKind::Int, Some(32), 1).to_sexpr(), "*i32");
    assert_eq!(ptr_kind(BaseKind::Int, None, 2).to_sexpr(), "**isize");
    assert_eq!(ptr_kind(BaseKind::Opaque, None, 1).to_sexpr(), "ptr");
    assert_eq!(ptr_kind(BaseKind::Opaque, None, 2).to_sexpr(), "*ptr");
}

#[test]
fn null_and_sizeof_print_and_walk()
{
    let null = Expr::Null(NullLit::builder().id(NodeId(1)).span(sp(0, 1)).build());
    let kind = ptr_kind(BaseKind::Int, Some(32), 0);
    let size = SizeOfExpr::builder().id(NodeId(2)).kind(kind).span(sp(0, 3)).build();
    let size = Expr::SizeOf(size);
    assert_eq!(null.to_sexpr(), "null");
    assert_eq!(size.to_sexpr(), "(sizeof i32)");
    let mut rec = Recorder::default();
    rec.visit_expr(&null);
    rec.visit_expr(&size);
    assert_eq!(rec.0, ["+null", "-null", "+sizeof", "+kind", "-kind", "-sizeof"]);
}

#[test]
fn places_are_names_and_derefs()
{
    let star = PrefixExpr::builder().id(NodeId(3)).op(PrefixOp::Deref);
    let star = Expr::Prefix(star.rhs(name(1, "p", 0)).span(sp(0, 2)).build());
    let group = GroupExpr::builder().id(NodeId(4)).inner(star.clone()).span(sp(0, 4));
    let amp = PrefixExpr::builder().id(NodeId(5)).op(PrefixOp::AddrOf);
    let amp = Expr::Prefix(amp.rhs(name(1, "p", 0)).span(sp(0, 2)).build());
    assert!(name(1, "p", 0).is_place());
    assert!(deref(2, name(1, "p", 0)).is_place());
    assert!(star.is_place());
    assert!(Expr::Group(group.build()).is_place());
    assert!(!amp.is_place());
    assert!(!int(1, 7, 0).is_place());
    assert_eq!(star.to_sexpr(), "(deref p)");
    assert_eq!(amp.to_sexpr(), "(addr p)");
}
