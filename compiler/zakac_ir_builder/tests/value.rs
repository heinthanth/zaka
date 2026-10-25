use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::LogicOp;
use zakac_ir_builder::ir::PrefixOp;
use zakac_ir_builder::value::EvalError;
use zakac_ir_builder::value::Value;
use zakac_ir_builder::value::compare;
use zakac_ir_builder::value::convert;
use zakac_ir_builder::value::float_pattern_text;
use zakac_ir_builder::value::infix;
use zakac_ir_builder::value::infix_wrapping;
use zakac_ir_builder::value::logic;
use zakac_ir_builder::value::prefix;
use zakac_ir_builder::value::prefix_wrapping;
use zakac_semcheck::kind::DataKind;

const P: u16 = 64;

fn v(text: &str) -> Value { text.parse().unwrap_or_else(|e| panic!("{e}")) }

fn kind(name: &str) -> DataKind { DataKind::parse(name).unwrap() }

fn op(o: InfixOp, l: &str, r: &str) -> Result<String, EvalError>
{
    infix(o, v(l), v(r), P).map(|x| x.to_string())
}

#[test]
fn text_round_trips()
{
    for text in [
        "-5:i8",
        "255:u8",
        "-128:i8",
        "-170141183460469231731687303715884105728:i128",
        "340282366920938463463374607431768211455:u128",
        "-1:isize",
        "1.5:f32",
        "0.1:f64",
        "1e300:f64",
        "inf:f64",
        "-inf:f32",
        "NaN:f64",
        "'က':char",
        "'\\n':char",
        "':':char",
        "true:bool",
    ]
    {
        assert_eq!(v(text).to_string(), text);
    }
    for text in ["0.1:f16", "0.1:f128", "-2.5:f16", "inf:f128"]
    {
        let once = v(text);
        assert_eq!(v(&once.to_string()), once, "{text}");
    }
}

#[test]
fn bad_text_is_rejected()
{
    for text in ["5", "5:i7", "300:x8", "abc:i8", "true:i8", "'ab':char", "1.5:i8"]
    {
        assert!(text.parse::<Value>().is_err(), "{text}");
    }
}

#[test]
fn integer_arithmetic_checks_the_range()
{
    assert_eq!(op(InfixOp::Add, "100:i8", "27:i8"), Ok("127:i8".into()));
    assert_eq!(op(InfixOp::Add, "127:i8", "1:i8"), Err(EvalError::Overflow));
    assert_eq!(op(InfixOp::Sub, "0:u8", "1:u8"), Err(EvalError::Overflow));
    assert_eq!(op(InfixOp::Mul, "16:i8", "8:i8"), Err(EvalError::Overflow));
    assert_eq!(op(InfixOp::Mul, "-16:i8", "8:i8"), Ok("-128:i8".into()));
    assert_eq!(op(InfixOp::Add, "255:u8", "0:u8"), Ok("255:u8".into()));
    let max = "340282366920938463463374607431768211455:u128";
    assert_eq!(op(InfixOp::Add, max, "1:u128"), Err(EvalError::Overflow));
    let min = "-170141183460469231731687303715884105728:i128";
    assert_eq!(op(InfixOp::Sub, min, "1:i128"), Err(EvalError::Overflow));
    assert_eq!(
        op(InfixOp::Add, "9223372036854775807:isize", "1:isize"),
        Err(EvalError::Overflow)
    );
    let small = infix(InfixOp::Add, v("127:isize"), v("1:isize"), 8);
    assert_eq!(small, Err(EvalError::Overflow));
}

#[test]
fn division_truncates_and_rejects_zero()
{
    assert_eq!(op(InfixOp::Div, "-7:i8", "2:i8"), Ok("-3:i8".into()));
    assert_eq!(op(InfixOp::Rem, "-7:i8", "2:i8"), Ok("-1:i8".into()));
    assert_eq!(op(InfixOp::Rem, "7:i8", "-2:i8"), Ok("1:i8".into()));
    assert_eq!(op(InfixOp::Div, "7:u8", "0:u8"), Err(EvalError::ZeroDivisor));
    assert_eq!(op(InfixOp::Rem, "7:i32", "0:i32"), Err(EvalError::ZeroDivisor));
    assert_eq!(op(InfixOp::Div, "-128:i8", "-1:i8"), Err(EvalError::Overflow));
    assert_eq!(op(InfixOp::Rem, "-128:i8", "-1:i8"), Err(EvalError::Overflow));
    let min = "-170141183460469231731687303715884105728:i128";
    assert_eq!(op(InfixOp::Div, min, "-1:i128"), Err(EvalError::Overflow));
}

#[test]
fn bitwise_and_shifts()
{
    assert_eq!(op(InfixOp::BitAnd, "-1:i8", "15:i8"), Ok("15:i8".into()));
    assert_eq!(op(InfixOp::BitOr, "-128:i8", "1:i8"), Ok("-127:i8".into()));
    assert_eq!(op(InfixOp::BitXor, "255:u8", "15:u8"), Ok("240:u8".into()));
    assert_eq!(op(InfixOp::Shl, "1:i8", "7:usize"), Ok("-128:i8".into()));
    assert_eq!(op(InfixOp::Shl, "255:u8", "4:u8"), Ok("240:u8".into()));
    assert_eq!(op(InfixOp::Shr, "-128:i8", "7:u8"), Ok("-1:i8".into()));
    assert_eq!(op(InfixOp::Shr, "128:u8", "7:u8"), Ok("1:u8".into()));
    let far = EvalError::ShiftTooFar { distance: 8, bits: 8 };
    assert_eq!(op(InfixOp::Shl, "1:i8", "8:usize"), Err(far));
    assert_eq!(
        op(InfixOp::Shl, "1:i128", "127:u8"),
        Ok(i128::MIN.to_string() + ":i128")
    );
    assert_eq!(prefix(PrefixOp::BitNot, v("0:u8"), P), Ok(v("255:u8")));
    assert_eq!(prefix(PrefixOp::BitNot, v("0:i8"), P), Ok(v("-1:i8")));
}

#[test]
fn negation()
{
    assert_eq!(prefix(PrefixOp::Neg, v("5:i8"), P), Ok(v("-5:i8")));
    assert_eq!(prefix(PrefixOp::Neg, v("-128:i8"), P), Err(EvalError::Overflow));
    assert_eq!(prefix(PrefixOp::Neg, v("1.5:f16"), P), Ok(v("-1.5:f16")));
    assert_eq!(prefix(PrefixOp::Not, v("true:bool"), P), Ok(v("false:bool")));
    assert_eq!(prefix(PrefixOp::Not, v("1:i8"), P), Err(EvalError::Mismatch));
}

#[test]
fn floats_round_at_their_own_width()
{
    let third =
        |k: &str| op(InfixOp::Div, &format!("1:{k}"), &format!("3:{k}")).unwrap();
    assert_eq!(third("f64"), "0.3333333333333333:f64");
    assert_eq!(third("f32"), "0.33333334:f32");
    let half_third = v(&third("f16"));
    let Value::Float { raw, .. } = half_third
    else
    {
        panic!("not a float")
    };
    assert_eq!(raw, 0x3555);
    let quad = v(&third("f128"));
    let wide = convert(quad, kind("f64"), P).unwrap();
    assert_eq!(wide, v("0.3333333333333333:f64"));
    assert_ne!(convert(wide, kind("f128"), P).unwrap(), quad);
    assert_eq!(op(InfixOp::Div, "1:f64", "0:f64"), Ok("inf:f64".into()));
    assert_eq!(op(InfixOp::Rem, "-7.5:f64", "2:f64"), Ok("-1.5:f64".into()));
    assert_eq!(op(InfixOp::Add, "65504:f16", "65504:f16"), Ok("inf:f16".into()));
}

#[test]
fn f128_literals_keep_more_than_53_bits()
{
    let exact = Value::float("9007199254740993", 128).unwrap();
    let lossy = Value::float("9007199254740993", 64).unwrap();
    let back = convert(exact, kind("u64"), P).unwrap();
    assert_eq!(back, v("9007199254740993:u64"));
    assert_eq!(convert(lossy, kind("u64"), P).unwrap(), v("9007199254740992:u64"));
}

#[test]
fn comparisons()
{
    assert_eq!(compare(CmpOp::Lt, v("-1:i8"), v("1:i8")), Ok(true));
    assert_eq!(compare(CmpOp::Lt, v("255:u8"), v("1:u8")), Ok(false));
    assert_eq!(compare(CmpOp::Ge, v("'ခ':char"), v("'က':char")), Ok(true));
    assert_eq!(compare(CmpOp::Eq, v("NaN:f64"), v("NaN:f64")), Ok(false));
    assert_eq!(compare(CmpOp::Ne, v("NaN:f64"), v("NaN:f64")), Ok(true));
    assert_eq!(compare(CmpOp::Le, v("-0.0:f64"), v("0.0:f64")), Ok(true));
    assert_eq!(compare(CmpOp::Eq, v("1:i8"), v("1:u8")), Err(EvalError::Mismatch));
    assert_eq!(logic(LogicOp::And, v("true:bool"), v("false:bool")), Ok(false));
    assert_eq!(logic(LogicOp::Or, v("true:bool"), v("false:bool")), Ok(true));
}

#[test]
fn conversions()
{
    let to = |text: &str, k: &str| convert(v(text), kind(k), P).unwrap().to_string();
    assert_eq!(to("-1:i8", "u16"), "65535:u16");
    assert_eq!(to("300:i32", "u8"), "44:u8");
    assert_eq!(to("200:u8", "i8"), "-56:i8");
    assert_eq!(to("-2:i8", "i64"), "-2:i64");
    assert_eq!(to("255:u8", "f64"), "255.0:f64");
    assert_eq!(to("16777217:i32", "f32"), "16777216.0:f32");
    assert_eq!(to("11.99:f64", "isize"), "11:isize");
    assert_eq!(to("-11.99:f64", "isize"), "-11:isize");
    assert_eq!(to("300.5:f64", "u8"), "255:u8");
    assert_eq!(to("-5.0:f64", "u8"), "0:u8");
    assert_eq!(to("-1000.0:f32", "i8"), "-128:i8");
    assert_eq!(to("NaN:f64", "i32"), "0:i32");
    assert_eq!(to("inf:f64", "i16"), "32767:i16");
    assert_eq!(to("0.1:f64", "f32"), "0.1:f32");
    assert_eq!(to("'A':char", "u8"), "65:u8");
    assert_eq!(to("'က':char", "u8"), "0:u8");
    assert_eq!(to("4096:u32", "char"), "'က':char");
    assert_eq!(to("-1:i32", "char"), "'\u{fffd}':char");
    assert_eq!(to("55296:u32", "char"), "'\u{fffd}':char");
    assert_eq!(to("true:bool", "u8"), "1:u8");
    assert_eq!(convert(v("1:i8"), DataKind::Bool, P), Err(EvalError::Mismatch));
}

#[test]
fn defaults_and_literals()
{
    assert_eq!(Value::default_of(kind("i8")).to_string(), "0:i8");
    assert_eq!(Value::default_of(kind("f32")).to_string(), "0.0:f32");
    assert_eq!(Value::default_of(DataKind::Char).to_string(), "'\\0':char");
    assert_eq!(Value::default_of(DataKind::Bool).to_string(), "false:bool");
    let DataKind::Int(i8k) = kind("i8")
    else
    {
        panic!()
    };
    assert_eq!(Value::int(i8k, 0b1111_1110, P).to_string(), "-2:i8");
    assert_eq!(Value::int(i8k, 128u128.wrapping_neg(), P).to_string(), "-128:i8");
    assert_eq!(Value::float("100e+10", 64), Some(v("1000000000000.0:f64")));
    assert_eq!(Value::float("1.x", 64), None);
}

#[test]
fn wide_and_narrow_floats_print_like_f64()
{
    assert_eq!(v("-3:f16").to_string(), "-3.0:f16");
    assert_eq!(v("1e30:f128").to_string(), "1e30:f128");
    assert_eq!(v("0.5:f128").to_string(), "0.5:f128");
}

fn wrapping(o: InfixOp, l: &str, r: &str) -> Result<String, EvalError>
{
    infix_wrapping(o, v(l), v(r), P).map(|x| x.to_string())
}

#[test]
fn wrapping_ops_wrap_to_the_width()
{
    let w = |o, l, r| wrapping(o, l, r).unwrap();
    assert_eq!(w(InfixOp::Add, "255:u8", "1:u8"), "0:u8");
    assert_eq!(w(InfixOp::Add, "127:i8", "1:i8"), "-128:i8");
    assert_eq!(w(InfixOp::Sub, "0:u16", "1:u16"), "65535:u16");
    assert_eq!(w(InfixOp::Mul, "200:u8", "2:u8"), "144:u8");
    assert_eq!(w(InfixOp::Div, "-128:i8", "-1:i8"), "-128:i8");
    assert_eq!(w(InfixOp::Rem, "-128:i8", "-1:i8"), "0:i8");
    let max = "340282366920938463463374607431768211455:u128";
    assert_eq!(w(InfixOp::Add, max, "1:u128"), "0:u128");
    let min = "-170141183460469231731687303715884105728:i128";
    assert_eq!(w(InfixOp::Div, min, "-1:i128"), min.to_string());
    assert_eq!(w(InfixOp::Sub, min, "1:i128"), format!("{}:i128", i128::MAX));
}

#[test]
fn wrapping_ops_keep_zero_divisor_and_shift_errors()
{
    assert_eq!(wrapping(InfixOp::Div, "1:i32", "0:i32"), Err(EvalError::ZeroDivisor));
    assert_eq!(wrapping(InfixOp::Rem, "1:u8", "0:u8"), Err(EvalError::ZeroDivisor));
    let far = Err(EvalError::ShiftTooFar { distance: 8, bits: 8 });
    assert_eq!(wrapping(InfixOp::Shl, "1:u8", "8:u8"), far);
}

#[test]
fn wrapping_ops_match_checked_ops_when_in_range()
{
    let cases = [
        (InfixOp::Add, "3:i32", "4:i32"),
        (InfixOp::Div, "-7:i32", "2:i32"),
        (InfixOp::Rem, "-7:i32", "2:i32"),
        (InfixOp::Shr, "-8:i8", "1:u8"),
        (InfixOp::Shl, "3:u8", "2:u8"),
        (InfixOp::BitXor, "12:u8", "10:u8"),
        (InfixOp::Add, "1.5:f64", "2.25:f64"),
        (InfixOp::Div, "1:f32", "0:f32"),
        (InfixOp::Rem, "7.5:f64", "2:f64"),
    ];
    for (o, l, r) in cases
    {
        assert_eq!(wrapping(o, l, r), op(o, l, r), "{o:?} {l} {r}");
    }
}

#[test]
fn wrapping_negation()
{
    let neg = |t| prefix_wrapping(PrefixOp::Neg, v(t), P).unwrap().to_string();
    assert_eq!(neg("-128:i8"), "-128:i8");
    assert_eq!(neg("5:i8"), "-5:i8");
    assert_eq!(neg("1:u8"), "255:u8");
    assert_eq!(neg("1.5:f64"), "-1.5:f64");
    let not = prefix_wrapping(PrefixOp::Not, v("true:bool"), P).unwrap();
    assert_eq!(not, v("false:bool"));
    let bit_not = prefix_wrapping(PrefixOp::BitNot, v("0:u8"), P).unwrap();
    assert_eq!(bit_not, v("255:u8"));
}

#[test]
fn float_text_is_public()
{
    assert_eq!(float_pattern_text(64, 0.1f64.to_bits() as u128), "0.1");
    assert_eq!(float_pattern_text(32, 3.0f32.to_bits() as u128), "3.0");
}
