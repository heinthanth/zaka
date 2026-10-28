use zaka_vm::print::text;
use zakac_ir_builder::value::Value;

fn v(t: &str) -> Value { t.parse().unwrap_or_else(|e| panic!("{e}")) }

#[test]
fn values_print_as_the_spec_tells()
{
    let cases = [
        ("true:bool", "မှန်"),
        ("false:bool", "မမှန်"),
        ("-5:i8", "-၅"),
        ("255:u8", "၂၅၅"),
        (
            "-170141183460469231731687303715884105728:i128",
            "-၁၇၀၁၄၁၁၈၃၄၆၀၄၆၉၂၃၁၇၃၁၆၈၇၃၀၃၇၁၅၈၈၄၁၀၅၇၂၈",
        ),
        (
            "340282366920938463463374607431768211455:u128",
            "၃၄၀၂၈၂၃၆၆၉၂၀၉၃၈၄၆၃၄၆၃၃၇၄၆၀၇၄၃၁၇၆၈၂၁၁၄၅၅",
        ),
        ("3.0:f64", "၃.၀"),
        ("0.1:f64", "၀.၁"),
        ("1.5:f32", "၁.၅"),
        ("1e300:f64", "၁e၃၀၀"),
        ("inf:f64", "inf"),
        ("-inf:f32", "-inf"),
        ("NaN:f64", "NaN"),
        ("'က':char", "က"),
        ("'A':char", "A"),
        ("0x0:ptr", "ဗလာ"),
        ("0x1000:ptr", "0x1000"),
    ];
    for (value, want) in cases
    {
        assert_eq!(text(v(value)), want, "{value}");
    }
}

#[test]
fn lists_print_their_cells_between_brackets()
{
    use zaka_vm::mem::Mem;
    use zaka_vm::mem::Origin;
    use zaka_vm::print::list_text;
    use zakac_semcheck::kind::DataKind;

    let mut m = Mem::new(64, 1 << 20);
    let a = m.alloc(32, Origin::Heap, 0).unwrap();
    m.store(a, v("1:isize")).unwrap();
    m.store(a + 24, v("-4:isize")).unwrap();
    let kind = |n: &str| DataKind::parse(n).unwrap();
    assert_eq!(list_text(&m, a, kind("[4]isize")), Ok("[၁၊ ၀၊ ၀၊ -၄]".to_string()));
    assert_eq!(
        list_text(&m, a, kind("[2][2]isize")),
        Ok("[[၁၊ ၀]၊ [၀၊ -၄]]".to_string())
    );
    assert_eq!(list_text(&m, a, kind("isize")), Ok("၁".to_string()));
}
