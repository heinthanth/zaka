mod common;

use std::fmt::Write;

use common::c_files;
use common::cc;
use common::compile_and_run;
use common::samples;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::emit::CFile;
use zakac_codegen_target_c::emit::MAIN_SOURCE;
use zakac_codegen_target_c::options::Checks;
use zakac_semcheck::kind::IntKind;
use zakac_semcheck::kind::Width;
use zakac_semcheck::value::Value;
use zakac_semcheck::value::value_text;

/// Floats whose text is hard to get right: thresholds of the plain form,
/// powers of two, the ends of the range, signed zeros and the specials.
const TRICKY_F64: &[f64] = &[
    0.0,
    -0.0,
    1.0,
    -1.0,
    0.1,
    0.2,
    0.3,
    1.0 / 3.0,
    2.0 / 3.0,
    100.0,
    4.35,
    0.07,
    1e-4,
    9.999e-5,
    1.5e-5,
    1e15,
    1e16,
    9_999_999_999_999_998.0,
    123_456_789.125,
    1e23,
    1e300,
    1e-300,
    9_007_199_254_740_992.0,
    f64::MAX,
    f64::MIN,
    f64::MIN_POSITIVE,
    5e-324,
    f64::EPSILON,
    f64::NAN,
    f64::INFINITY,
    f64::NEG_INFINITY,
];

/// The next number of a fixed xorshift sequence, so each run checks the
/// same patterns.
///
/// # Arguments
///
/// * `state` - The state, changed.
///
/// # Returns
///
/// The number.
fn next(state: &mut u64) -> u64
{
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// The probe program and the text the VM prints for each line of it.
///
/// # Returns
///
/// The C text of `main.c` and the output it must print.
fn probe() -> (String, String)
{
    let mut lines = String::new();
    let mut want = String::new();
    let mut put = |call: String, v: Value| {
        let _ = writeln!(lines, "    {call}; zk_put_line();");
        let _ = writeln!(want, "{}", value_text(v));
    };
    let mut state = 0x9e37_79b9_7f4a_7c15;
    let doubles = TRICKY_F64.iter().map(|v| v.to_bits());
    let doubles: Vec<u64> =
        doubles.chain((0..3000).map(|_| next(&mut state))).collect();
    for raw in doubles
    {
        let v = Value::Float { bits: 64, raw };
        put(format!("zk_put_f64(zk_d(UINT64_C({raw:#x})))"), v);
    }
    let singles = TRICKY_F64.iter().map(|v| (*v as f32).to_bits());
    let singles: Vec<u32> =
        singles.chain((0..3000).map(|_| next(&mut state) as u32)).collect();
    for raw in singles
    {
        let v = Value::Float { bits: 32, raw: u64::from(raw) };
        put(format!("zk_put_f32(zk_f(UINT32_C({raw:#x})))"), v);
    }
    let i64_kind = IntKind::new(true, Width::Bits(64));
    let u64_kind = IntKind::new(false, Width::Bits(64));
    for raw in [0, 1, u64::MAX, 1 << 63, (1 << 63) - 1, 12_345]
    {
        let int = Value::int(i64_kind, u128::from(raw), 64);
        put(format!("zk_put_i((int64_t)UINT64_C({raw:#x}))"), int);
        let uint = Value::int(u64_kind, u128::from(raw), 64);
        put(format!("zk_put_u(UINT64_C({raw:#x}))"), uint);
    }
    let main = format!(
        "#include \"zaka_kinds.h\"\nstatic double zk_d(uint64_t b) {{ double v; \
         memcpy(&v, &b, 8); return v; }}\nstatic float zk_f(uint32_t b) {{ float v; \
         memcpy(&v, &b, 4); return v; }}\nint main(int argc, char **argv)\n{{\n    \
         zk_start(argc, argv);\n{lines}    return zk_finish(0);\n}}\n"
    );
    (main, want)
}

/// The runtime writes each number as the VM does: floats in the fewest
/// digits that read back, integers of each width, in Burmese digits.
#[test]
fn numbers_print_as_the_vm_prints_them()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let hello = samples().join("basics/hello.zaka");
    let mut files = c_files(&hello, OverflowMode::Stop, Checks::Default).unwrap();
    let (main, want) = probe();
    files.retain(|f| f.name != MAIN_SOURCE);
    files.push(CFile::builder().name(MAIN_SOURCE).text(main).build());
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    let pairs = run.stdout.lines().zip(want.lines()).enumerate();
    let wrong: Vec<String> = pairs
        .filter(|(_, (got, want))| got != want)
        .map(|(n, (got, want))| format!("line {}: got {got}, want {want}", n + 1))
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert_eq!(run.stdout.lines().count(), want.lines().count());
}
