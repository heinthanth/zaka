use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;
use zakac_semcheck::kind::Width;
use zakac_semcheck::structs::StructTable;
use zakac_span::span::Span;

/// A kind parsed from its short name; panics on a bad name.
fn k(name: &str) -> DataKind
{
    DataKind::parse(name).unwrap_or_else(|| panic!("{name}"))
}

/// Whether `from` widens to `to` with a 64-bit platform.
fn widens(from: &str, to: &str) -> bool { k(from).widens_to(k(to), 64) }

/// A kind phrase as the parser builds it.
fn ast(base: BaseKind, bits: Option<u16>, signed: Option<bool>) -> Kind
{
    #[rustfmt::skip]
    return Kind::builder().base(base).maybe_bits(bits).maybe_signed(signed).span(Span::PLACEHOLDER).build();
}

#[test]
fn ast_kinds_get_their_defaults()
{
    assert_eq!(
        DataKind::from_ast(&ast(BaseKind::Int, None, None), &StructTable::new()),
        k("isize")
    );
    assert_eq!(
        DataKind::from_ast(&ast(BaseKind::Int, None, Some(false)), &StructTable::new()),
        k("usize")
    );
    assert_eq!(
        DataKind::from_ast(&ast(BaseKind::Int, Some(8), None), &StructTable::new()),
        k("i8")
    );
    assert_eq!(
        DataKind::from_ast(
            &ast(BaseKind::Int, Some(16), Some(false)),
            &StructTable::new()
        ),
        k("u16")
    );
    assert_eq!(
        DataKind::from_ast(&ast(BaseKind::Float, None, None), &StructTable::new()),
        k("f64")
    );
    assert_eq!(
        DataKind::from_ast(&ast(BaseKind::Float, Some(32), None), &StructTable::new()),
        k("f32")
    );
    assert_eq!(
        DataKind::from_ast(&ast(BaseKind::Char, None, None), &StructTable::new()),
        DataKind::Char
    );
    assert_eq!(
        DataKind::from_ast(&ast(BaseKind::Bool, None, None), &StructTable::new()),
        DataKind::Bool
    );
}

#[test]
fn names_round_trip()
{
    let names = [
        "i8",
        "i16",
        "i32",
        "i64",
        "isize",
        "u8",
        "u16",
        "u32",
        "u64",
        "usize",
        "f32",
        "f64",
        "char",
        "bool",
        "i32*",
        "isize**",
        "ptr",
        "ptr*",
        "null",
        "isize&",
        "isize*&",
        "isize^*",
        "isize*^",
        "isize^",
        "rawptr",
        "rawptr*",
        "ptr&",
        "isize&[3]",
        "isize[3]&",
    ];
    for name in names
    {
        assert_eq!(k(name).name(), name);
    }
    assert_eq!(DataKind::parse("i7"), None);
    assert_eq!(DataKind::parse("f8"), None);
    let isize = DataKind::Int(IntKind { signed: true, width: Width::Platform });
    assert_eq!(k("isize"), isize);
}

#[test]
fn integers_widen_within_their_sign()
{
    assert!(widens("i8", "i8"));
    assert!(widens("i8", "i16"));
    assert!(widens("u8", "u64"));
    assert!(!widens("i16", "i8"));
    assert!(!widens("i8", "u16"));
    assert!(widens("u8", "i16"));
    assert!(!widens("u8", "i8"));
}

#[test]
fn platform_kinds_take_the_platform_width()
{
    assert!(widens("i32", "isize"));
    assert!(!widens("i64", "isize"));
    assert!(!widens("isize", "i64"));
    assert!(widens("u32", "usize"));
    assert!(!k("i32").widens_to(k("isize"), 32));
    assert!(k("i16").widens_to(k("isize"), 32));
    assert_eq!(k("isize").bits(32), 32);
    assert_eq!(k("usize").bits(64), 64);
}

#[test]
fn integers_widen_to_floats_without_loss()
{
    assert!(widens("i8", "f32"));
    assert!(widens("i16", "f32"));
    assert!(!widens("i32", "f32"));
    assert!(widens("i32", "f64"));
    assert!(!widens("i64", "f64"));
    assert!(!widens("isize", "f64"));
    assert!(!widens("f32", "i64"));
}

#[test]
fn floats_widen_to_wider_floats()
{
    assert!(widens("f32", "f64"));
    assert!(!widens("f64", "f32"));
}

#[test]
fn chars_and_bools_never_widen()
{
    assert!(widens("char", "char"));
    assert!(widens("bool", "bool"));
    assert!(!widens("char", "u32"));
    assert!(!widens("bool", "u8"));
    assert!(!widens("u8", "char"));
}

#[test]
fn literals_fit_their_range()
{
    assert!(k("i8").fits_int(127, false, 64));
    assert!(!k("i8").fits_int(128, false, 64));
    assert!(k("i8").fits_int(128, true, 64));
    assert!(!k("i8").fits_int(129, true, 64));
    assert!(k("u8").fits_int(255, false, 64));
    assert!(!k("u8").fits_int(256, false, 64));
    assert!(!k("u8").fits_int(1, true, 64));
    assert!(k("u8").fits_int(0, true, 64));
    assert!(k("u64").fits_int(u128::from(u64::MAX), false, 64));
    assert!(!k("u64").fits_int(1 << 64, false, 64));
    assert!(k("i64").fits_int(1 << 63, true, 64));
    assert!(!k("i64").fits_int(1 << 63, false, 64));
    assert!(!k("isize").fits_int(1 << 40, false, 32));
    assert!(k("isize").fits_int(1 << 40, false, 64));
    assert!(!k("f64").fits_int(1, false, 64));
}

#[test]
fn casts_follow_the_conversion_table()
{
    assert!(k("f64").cast_allowed(k("u8")));
    assert!(k("i64").cast_allowed(k("f32")));
    assert!(k("char").cast_allowed(k("u32")));
    assert!(k("u32").cast_allowed(k("char")));
    assert!(k("bool").cast_allowed(k("u8")));
    assert!(k("bool").cast_allowed(k("bool")));
    assert!(!k("u8").cast_allowed(k("bool")));
    assert!(!k("f64").cast_allowed(k("char")));
    assert!(!k("char").cast_allowed(k("f64")));
    assert!(!k("bool").cast_allowed(k("f32")));
}

#[test]
fn flavour_helpers()
{
    assert_eq!(k("isize&").pointee(), Some(k("isize")));
    assert_eq!(k("isize*&").pointee(), Some(k("isize*")));
    assert_eq!(k("isize^*").pointee(), Some(k("isize^")));
    assert_eq!(k("isize*^").pointee(), Some(k("isize*")));
    assert_eq!(k("isize&").pointer_to(), k("isize**"));
    assert_eq!(k("isize^").pointer_to(), k("isize^*"));
    assert_eq!(k("isize*").as_borrowed(), k("isize&"));
    assert_eq!(k("isize&").as_owner(), k("isize*"));
    assert_eq!(k("isize^").as_borrowed(), k("isize^"));
    assert_eq!(k("isize").as_borrowed(), k("isize"));
    assert_eq!(k("isize").raw_pointer_to(), k("isize^"));
    assert_eq!(k("isize*").raw_pointer_to(), k("isize*^"));
    assert!(k("isize&").same_shape(k("isize*")));
    assert!(!k("isize^").same_shape(k("isize*")));
    assert!(k("ptr&").is_opaque());
    assert!(k("rawptr").is_opaque());
    assert!(!k("rawptr*").is_opaque());
    assert_eq!(k("ptr&").pointee(), None);
    assert!(k("isize*").is_owner_ptr() && k("ptr").is_owner_ptr());
    assert!(k("isize&").is_borrow() && !k("isize*").is_borrow());
    assert!(k("isize^").is_raw_ptr() && k("rawptr").is_raw_ptr());
    assert!(k("isize*^").is_raw_ptr());
    assert!(!k("isize^*").is_raw_ptr());
    assert!(k("isize^*").is_owner_ptr() && !k("isize^*").is_raw_ptr());
    assert_eq!(k("isize&").ptr_kind().map(|p| p.depth), Some(1));
    assert_eq!(k("isize").ptr_kind(), None);
    assert_eq!(DataKind::parse("isize&*"), None);
    assert_eq!(DataKind::parse("&isize"), None);
    assert_eq!(DataKind::parse("&null"), None);
}

#[test]
fn borrows_are_held_through_lists()
{
    assert!(k("isize&[3]").holds_borrow());
    assert!(k("isize&").holds_borrow());
    assert!(!k("isize*[3]").holds_borrow());
    assert!(!k("isize*").holds_borrow());
}

#[test]
fn flavours_widen_within_themselves()
{
    assert!(widens("isize&", "isize&"));
    assert!(widens("isize^", "isize^"));
    assert!(widens("null", "isize&"));
    assert!(widens("isize&", "ptr&"));
    assert!(widens("isize*", "ptr"));
    assert!(widens("isize^", "rawptr"));
    assert!(widens("rawptr", "isize^"));
    assert!(!widens("isize*", "isize&"));
    assert!(!widens("isize&", "isize*"));
    assert!(!widens("isize^", "isize*"));
    assert!(!widens("isize*", "isize^"));
    assert!(!widens("isize&", "ptr"));
    assert!(!widens("isize*", "rawptr"));
    assert!(!widens("isize^*", "isize**"));
}

#[test]
fn casts_cross_flavours()
{
    assert!(k("isize*").cast_allowed(k("isize^")));
    assert!(k("isize^").cast_allowed(k("isize*")));
    assert!(k("isize&").cast_allowed(k("isize^")));
    assert!(k("rawptr").cast_allowed(k("usize")));
    assert!(k("usize").cast_allowed(k("isize^")));
    assert!(k("usize").cast_allowed(k("rawptr")));
    assert!(k("usize").cast_allowed(k("isize*")));
    assert!(k("usize").cast_allowed(k("ptr")));
    assert!(k("isize*").cast_allowed(k("usize")));
}

#[test]
fn ast_kinds_hold_their_marks()
{
    let ptr = || Wrap::Ptr;
    let list = || Wrap::List(Cells::Fixed(3));
    let from = |base, wraps| {
        let base = ast(base, None, None);
        DataKind::from_ast(&Kind { wraps, ..base }, &StructTable::new())
    };
    assert_eq!(from(BaseKind::Int, vec![ptr()]), k("isize*"));
    assert_eq!(from(BaseKind::Int, vec![ptr(), ptr()]), k("isize**"));
    assert_eq!(from(BaseKind::Int, vec![ptr(), list()]), k("isize*[3]"));
    assert_eq!(from(BaseKind::Int, vec![list(), ptr()]), k("isize[3]*"));
    assert_eq!(from(BaseKind::Opaque, vec![ptr()]), k("ptr"));
    assert_eq!(from(BaseKind::Opaque, vec![ptr(), ptr()]), k("ptr*"));
}

#[test]
fn opaque_pointers_pass_address_free_blocks_alone()
{
    let refused = [
        ("isize*&", "ptr&"),
        ("ptr&", "isize*&"),
        ("isize**", "ptr"),
        ("ptr", "isize**"),
        ("isize*[2]&", "ptr&"),
        ("ptr&", "isize&[2]&"),
        ("isize^*", "ptr"),
        ("ptr*", "ptr"),
        ("ptr", "ptr*"),
    ];
    for (from, to) in refused
    {
        assert!(!widens(from, to), "{from} to {to}");
    }
    let allowed = [
        ("isize&", "ptr&"),
        ("ptr&", "isize&"),
        ("isize*", "ptr"),
        ("ptr", "isize[3]*"),
        ("isize[1]&", "ptr&"),
        ("isize**", "isize**"),
        ("isize*^", "rawptr"),
        ("rawptr", "isize*^"),
    ];
    for (from, to) in allowed
    {
        assert!(widens(from, to), "{from} to {to}");
    }
}
