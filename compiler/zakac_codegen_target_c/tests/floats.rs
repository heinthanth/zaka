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
use zakac_semcheck::value::Value;
use zakac_semcheck::value::value_text;

/// The seed of the xorshift sequence that the probe and the test share.
const SEED: u64 = 0x2545_f491_4f6c_dd1d;

/// Random f64 patterns, spread over each exponent.
const F64_WIDE: u32 = 60_000;
/// Random f64 patterns with an exponent near 1, where the plain form holds.
const F64_NEAR: u32 = 60_000;
/// f32 patterns, one in each stride: each exponent, and a spread of
/// significands.
const F32_STRIDED: u32 = 1 << 19;
/// Random f128 patterns over the whole range; each takes thousands of digits.
const F128_WIDE: u32 = 400;
/// Random f128 patterns with an exponent near 1.
const F128_NEAR: u32 = 4000;

/// The next number of a fixed xorshift sequence; the C probe runs the same
/// steps.
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

/// A random pattern with its exponent moved near that of 1.
///
/// # Arguments
///
/// * `r` - The random bits.
/// * `exp_at` - The bit index of the exponent field.
/// * `exp_mask` - The mask of the exponent field, shifted down.
/// * `bias` - The bias of the exponent.
///
/// # Returns
///
/// The pattern.
fn near_one(r: u64, exp_at: u32, exp_mask: u64, bias: u64) -> u64
{
    let exp = bias - 70 + r % 130;
    (r & !(exp_mask << exp_at)) | (exp << exp_at)
}

/// The patterns at the edges: the powers of two (one in 61 for f128), the
/// subnormals at both ends, the largest finite, and the specials, for a
/// format of `bits` bits.
///
/// # Arguments
///
/// * `bits` - The width.
/// * `frac` - The bits of the fraction field.
///
/// # Returns
///
/// The patterns.
fn edges(bits: u32, frac: u32) -> Vec<u128>
{
    let exp_max = (1u128 << (bits - frac - 1)) - 1;
    let sign = 1u128 << (bits - 1);
    let step = if bits == 128 { 61 } else { 1 };
    let powers = (0..=exp_max).step_by(step).chain([exp_max]).map(|e| e << frac);
    let ends =
        [1, 2, 3, (1u128 << frac) - 1, (1u128 << frac) + 1, (exp_max << frac) - 1];
    let nan = (exp_max << frac) | 1;
    let all: Vec<u128> = powers.chain(ends).chain([nan]).collect();
    all.iter().copied().chain(all.iter().map(|p| p | sign)).collect()
}

/// The C text of a call that writes a pattern through the print of its kind
/// when the C compiler has the kind, and through the bits otherwise; both
/// print the same text.
///
/// # Arguments
///
/// * `bits` - The width.
/// * `raw` - The pattern.
///
/// # Returns
///
/// The C text, ending in a line break.
fn kind_put(bits: u32, raw: u128) -> String
{
    let (hi, lo) = ((raw >> 64) as u64, raw as u64);
    let bare = format!("zk_put_fbits({bits}, UINT64_C({hi:#x}), UINT64_C({lo:#x}))");
    let guarded = match bits
    {
        16 => format!("#ifdef ZK_HAS_F16\n    zk_put_f16(zk_f16_of({lo:#x}));\n#else"),
        128 => format!(
            "#ifdef ZK_HAS_F128\n    zk_put_f128(zk_f128_of(UINT64_C({hi:#x}), \
             UINT64_C({lo:#x})));\n#else"
        ),
        _ => return format!("    {bare}; zk_put_line();\n"),
    };
    format!("{guarded}\n    {bare};\n#endif\n    zk_put_line();\n")
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
    let mut put = |bits: u32, raw: u128| {
        let _ =
            writeln!(want, "{}", value_text(Value::Float { bits: bits as u16, raw }));
    };
    for (bits, frac) in [(16, 10), (32, 23), (64, 52), (128, 112)]
    {
        for raw in edges(bits, frac)
        {
            lines.push_str(&kind_put(bits, raw));
            put(bits, raw);
        }
    }
    let mut state = SEED;
    (0..F64_WIDE).for_each(|_| put(64, u128::from(next(&mut state))));
    for _ in 0..F64_NEAR
    {
        put(64, u128::from(near_one(next(&mut state), 52, 0x7ff, 1023)));
    }
    (0..F32_STRIDED).for_each(|i| put(32, u128::from(i.wrapping_mul(8191) ^ 0x5a5a)));
    (0..1u32 << 16).for_each(|b| put(16, u128::from(b)));
    for _ in 0..F128_WIDE
    {
        let hi = next(&mut state);
        put(128, (u128::from(hi) << 64) | u128::from(next(&mut state)));
    }
    for _ in 0..F128_NEAR
    {
        let hi = near_one(next(&mut state), 48, 0x7fff, 16383);
        put(128, (u128::from(hi) << 64) | u128::from(next(&mut state)));
    }
    let main = format!(
        "#include \"zaka_kinds.h\"\nstatic uint64_t zk_next(uint64_t *s) {{ *s ^= *s \
         << 13; *s ^= *s >> 7; *s ^= *s << 17; return *s; }}\nstatic uint64_t \
         zk_near(uint64_t r, int at, uint64_t mask, uint64_t bias) {{ uint64_t e = \
         bias - 70 + r % 130; return (r & ~(mask << at)) | (e << at); }}\nint \
         main(int argc, char **argv)\n{{\n    zk_start(argc, argv);\n{lines}uint64_t \
         s = UINT64_C({SEED:#x}), hi;\nfor (uint32_t i = 0; i < {F64_WIDE}; i++) {{ \
         zk_put_fbits(64, 0, zk_next(&s)); zk_put_line(); }}\nfor (uint32_t i = 0; i \
         < {F64_NEAR}; i++) {{ zk_put_fbits(64, 0, zk_near(zk_next(&s), 52, 0x7ff, \
         1023)); zk_put_line(); }}\nfor (uint32_t i = 0; i < {F32_STRIDED}; i++) {{ \
         zk_put_fbits(32, 0, (uint32_t)(i * 8191u) ^ 0x5a5a); zk_put_line(); }}\nfor \
         (uint32_t b = 0; b < 65536; b++) {{ zk_put_fbits(16, 0, b); zk_put_line(); \
         }}\nfor (uint32_t i = 0; i < {F128_WIDE}; i++) {{ hi = zk_next(&s); \
         zk_put_fbits(128, hi, zk_next(&s)); zk_put_line(); }}\nfor (uint32_t i = 0; \
         i < {F128_NEAR}; i++) {{ hi = zk_near(zk_next(&s), 48, 0x7fff, 16383); \
         zk_put_fbits(128, hi, zk_next(&s)); zk_put_line(); }}\nreturn \
         zk_finish(0);\n}}\n"
    );
    (main, want)
}

/// The runtime writes each float kind as the VM does, worked out on the bits
/// with no print from libc: f32 and f64 in the fewest digits that read back,
/// f16 and f128 in the digits rustc_apfloat writes. Each f16 pattern is
/// checked, and a wide spread of the others.
#[test]
fn float_bits_print_as_the_vm_prints_them()
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
        .take(20)
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert_eq!(run.stdout.lines().count(), want.lines().count());
}
