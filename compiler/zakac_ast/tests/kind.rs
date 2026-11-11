use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::s_expr::ToSexpr;
use zakac_span::span::Span;

/// A kind over `base` with the given wraps.
fn kind(base: BaseKind, wraps: Vec<Wrap>) -> Kind
{
    Kind::builder().base(base).wraps(wraps).span(Span::PLACEHOLDER).build()
}

#[test]
fn pointer_kinds_print_their_marks()
{
    let int = BaseKind::Int;
    assert_eq!(kind(int, vec![Wrap::Ptr]).to_sexpr(), "isize*");
    assert_eq!(kind(int, vec![Wrap::Ptr, Wrap::Ptr]).to_sexpr(), "isize**");
    assert_eq!(kind(BaseKind::Opaque, vec![Wrap::Ptr]).to_sexpr(), "ptr");
    assert_eq!(kind(BaseKind::Opaque, vec![Wrap::Ptr, Wrap::Ptr]).to_sexpr(), "ptr*");
}

#[test]
fn pointer_helpers_read_the_wraps()
{
    let ptr = kind(BaseKind::Int, vec![Wrap::Ptr]);
    assert_eq!(ptr.outer_ptr_wrap(), Some(Wrap::Ptr));
    assert_eq!(kind(BaseKind::Int, vec![]).outer_ptr_wrap(), None);
}

#[test]
fn a_plain_kind_leaves_out_its_blank_fields()
{
    let text = serde_saphyr::to_string(&kind(BaseKind::Int, vec![])).unwrap();
    assert!(!text.contains("name") && !text.contains("parts"), "{text}");
}

#[test]
fn kinds_round_trip_through_text_and_raw_artifacts()
{
    let plain = kind(BaseKind::Int, vec![]);
    let ptr = kind(BaseKind::Int, vec![Wrap::Ptr, Wrap::Ptr]);
    for k in [plain, ptr]
    {
        let raw = postcard::to_allocvec(&k).unwrap();
        assert_eq!(postcard::from_bytes::<Kind>(&raw).unwrap(), k);
        let text = serde_saphyr::to_string(&k).unwrap();
        assert_eq!(serde_saphyr::from_str::<Kind>(&text).unwrap(), k);
    }
}
