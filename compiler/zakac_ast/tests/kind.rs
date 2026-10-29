use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::verb::VerbOp;
use zakac_ast::verb::verb_for;
use zakac_lexer::token::Kw;
use zakac_span::span::Span;

/// A kind over `base` with the given wraps and borrow marks.
fn kind(base: BaseKind, wraps: Vec<Wrap>, borrowed: bool, extra: u8) -> Kind
{
    Kind::builder()
        .base(base)
        .wraps(wraps)
        .borrowed(borrowed)
        .extra_borrows(extra)
        .span(Span::PLACEHOLDER)
        .build()
}

#[test]
fn borrowed_and_raw_kinds_print_their_marks()
{
    let int = BaseKind::Int;
    assert_eq!(kind(int, vec![Wrap::Ptr], true, 0).to_sexpr(), "&*isize");
    assert_eq!(
        kind(int, vec![Wrap::RawPtr, Wrap::Ptr], false, 0).to_sexpr(),
        "*~isize"
    );
    assert_eq!(kind(int, vec![Wrap::RawPtr], false, 0).to_sexpr(), "~isize");
    assert_eq!(
        kind(BaseKind::Opaque, vec![Wrap::RawPtr], false, 0).to_sexpr(),
        "rawptr"
    );
    assert_eq!(kind(BaseKind::Opaque, vec![Wrap::Ptr], true, 0).to_sexpr(), "&ptr");
    assert_eq!(kind(int, vec![Wrap::Ptr], true, 1).to_sexpr(), "&&*isize");
}

#[test]
fn pointer_helpers_read_the_wraps()
{
    let raw = kind(BaseKind::Int, vec![Wrap::RawPtr, Wrap::Ptr], false, 0);
    assert!(raw.has_raw());
    assert_eq!(raw.outer_ptr_wrap(), Some(Wrap::Ptr));
    let plain = kind(BaseKind::Int, vec![], false, 0);
    assert!(!plain.has_raw());
    assert_eq!(plain.outer_ptr_wrap(), None);
}

#[test]
fn a_plain_kind_serializes_as_before()
{
    let text = serde_saphyr::to_string(&kind(BaseKind::Int, vec![], false, 0)).unwrap();
    assert!(!text.contains("borrowed"), "{text}");
    assert!(!text.contains("extra_borrows"), "{text}");
    let text = serde_saphyr::to_string(&kind(BaseKind::Int, vec![], true, 1)).unwrap();
    assert!(text.contains("borrowed: true"), "{text}");
    assert!(text.contains("extra_borrows: 1"), "{text}");
}

#[test]
fn kinds_round_trip_through_text_and_raw_artifacts()
{
    let plain = kind(BaseKind::Int, vec![Wrap::Ptr], false, 0);
    let borrowed = kind(BaseKind::Int, vec![Wrap::RawPtr, Wrap::Ptr], true, 2);
    for k in [plain, borrowed]
    {
        let raw = postcard::to_allocvec(&k).unwrap();
        assert_eq!(postcard::from_bytes::<Kind>(&raw).unwrap(), k);
        let text = serde_saphyr::to_string(&k).unwrap();
        assert_eq!(serde_saphyr::from_str::<Kind>(&text).unwrap(), k);
    }
}

#[test]
fn lend_is_a_verb()
{
    assert_eq!(verb_for(Kw::Lend).unwrap().op, VerbOp::Lend);
    assert_eq!(VerbOp::Lend.as_ref(), "lend");
}
