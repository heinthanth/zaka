use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use bon::Builder;
use zakac_borrow_checker::options::MemCheck;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::options::RunChecks;
use zakac_codegen_target_vm::options::ResultMode;
use zakac_codegen_target_vm::options::ScratchMode;
use zakac_diagnostics::Code;
use zakac_diagnostics::lint::LINT_CODES;
use zakac_diagnostics::lint::LintLevel;
use zakac_diagnostics::lint::LintLevels;
use zakac_ir_builder::ir::FoldOptions;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::Options;
use crate::compile::PLATFORM_BITS;
use crate::compile::Phase;
use crate::compile::Target;
use crate::state::settings::AstMode;
use crate::state::settings::LowerMode;

/// Version of the link format.
const FORMAT: &str = "9";

/// Version of the format before each C check, the OS, the CPU and the lint.
const FORMAT_8: &str = "8";

/// Version of the format before the code generation target.
const FORMAT_7: &str = "7";

/// Version of the format before the scratch mode.
const FORMAT_6: &str = "6";

/// Version of the format before the result mode.
const FORMAT_5: &str = "5";

/// Version of the format before the checkers.
const FORMAT_4: &str = "4";

/// Version of the older format, without the foldings and the lowering view.
const FORMAT_3: &str = "3";

/// Version of the oldest format, without the stage and the AST view.
const FORMAT_2: &str = "2";

/// What a link holds: the source, the options, and the view it opens.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Link
{
    /// The source text.
    #[builder(into)]
    pub source: String,
    /// The compile options.
    pub options: Options,
    /// The open stage of the pipeline.
    #[builder(default)]
    pub stage: Phase,
    /// How the AST view shows the tree.
    #[builder(default)]
    pub ast_mode: AstMode,
    /// Which IR the lowering view shows.
    #[builder(default)]
    pub lower_mode: LowerMode,
}

/// Encode a link for the URL hash.
///
/// The format is `9.<platform bits>.<foldings>.<checker>.<mem
/// flags>.<result mode>.<scratch mode>.<target>.<C checks>.<OS>.<CPU>.<lint>.
/// <stage>.<AST view>.<lowering view>.<base64url of the UTF-8 source>`, with
/// the check, the modes, the target, the OS, the CPU, the stage and the views
/// in lowercase, as in
/// `9.64.7.error.0.in-place.apart.vm.63.linux.x86_64.000….lower.tree.folded.…`.
/// The foldings are a digit from 0 to 7: 1 for expressions, 2 for constants, 4
/// for branches. The mem flags are a digit from 0 to 7: 1 for the strict
/// check, 2 to forbid unsafe code, 4 to stop on overflow; formats before 7 hold
/// 0 to 3. The C checks are the number of their bits (`RunChecks::bits`); the
/// lint is a digit for each lint code in the order of `LINT_CODES`: 0 for its
/// own level, then 1 off, 2 warn, 3 error. Format 8 held a level of the C
/// checks in their place and no OS, CPU or lint.
///
/// # Arguments
///
/// * `link` - The link.
///
/// # Returns
///
/// The hash, without the leading `#`.
pub fn encode(link: &Link) -> String
{
    let text = URL_SAFE_NO_PAD.encode(link.source.as_bytes());
    let stage = <&'static str>::from(link.stage).to_lowercase();
    let bits = link.options.platform_bits;
    let fold = fold_digit(link.options.fold);
    let views = format!("{}.{}", link.ast_mode, link.lower_mode);
    let check: &'static str = link.options.check.into();
    let mem = mem_digit(&link.options);
    let results: &'static str = link.options.results.into();
    let scratch: &'static str = link.options.scratch.into();
    let target: &'static str = link.options.target.into();
    let c_checks = link.options.c_checks.bits();
    let os: &'static str = link.options.os.into();
    let arch: &'static str = link.options.arch.into();
    let lint = lint_digits(link.options.lint);
    let modes = format!("{results}.{scratch}.{target}.{c_checks}.{os}.{arch}.{lint}");
    format!("{FORMAT}.{bits}.{fold}.{check}.{mem}.{modes}.{stage}.{views}.{text}")
}

/// The digits of the lint levels, one for each lint code.
///
/// # Arguments
///
/// * `lint` - The levels.
///
/// # Returns
///
/// The digits.
fn lint_digits(lint: LintLevels) -> String
{
    let digit = |code: &Code| match lint.get(*code)
    {
        None => '0',
        Some(LintLevel::Off) => '1',
        Some(LintLevel::Warn) => '2',
        Some(LintLevel::Error) => '3',
    };
    LINT_CODES.iter().map(digit).collect()
}

/// The lint levels of digits made with [`lint_digits`].
///
/// # Arguments
///
/// * `digits` - The digits.
///
/// # Returns
///
/// The levels, or `None` for other text.
fn read_lint(digits: &str) -> Option<LintLevels>
{
    ret_if!(digits.chars().count() != LINT_CODES.len(), None);
    let level = |c: char| match c
    {
        '0' => Some(None),
        '1' => Some(Some(LintLevel::Off)),
        '2' => Some(Some(LintLevel::Warn)),
        '3' => Some(Some(LintLevel::Error)),
        _ => None,
    };
    let mut pairs = LINT_CODES.iter().zip(digits.chars());
    pairs.try_fold(LintLevels::default(), |lint, (code, c)| {
        Some(lint.set(*code, level(c)?))
    })
}

/// The digit of the flags of the checker.
///
/// # Arguments
///
/// * `options` - The compile options.
///
/// # Returns
///
/// The digit, from 0 to 3.
fn mem_digit(options: &Options) -> u8
{
    let stop = options.overflow == OverflowMode::Stop;
    u8::from(options.strict) | u8::from(options.no_unsafe) << 1 | u8::from(stop) << 2
}

/// The digit of a set of foldings.
///
/// # Arguments
///
/// * `fold` - The foldings.
///
/// # Returns
///
/// The digit, from 0 to 7.
fn fold_digit(fold: FoldOptions) -> u8
{
    let bit = |on: bool, value: u8| tern!(on, value, 0);
    bit(fold.exprs, 1) | bit(fold.consts, 2) | bit(fold.branches, 4)
}

/// The foldings of a digit made with [`fold_digit`].
///
/// # Arguments
///
/// * `digit` - The digit.
///
/// # Returns
///
/// The foldings, or `None` for other text.
fn read_fold(digit: &str) -> Option<FoldOptions>
{
    let n: u8 = digit.parse().ok().filter(|n| *n < 8)?;
    let on = |value: u8| n & value != 0;
    Some(FoldOptions::builder().exprs(on(1)).consts(on(2)).branches(on(4)).build())
}

/// Decode a URL hash made with [`encode`]; a link of an older format opens
/// with the defaults for the parts it lacks.
///
/// # Arguments
///
/// * `hash` - The hash, with or without the leading `#`.
///
/// # Returns
///
/// The link, or `None` when a part is not well formed.
pub fn decode(hash: &str) -> Option<Link>
{
    let (format, rest) = hash.trim_start_matches('#').split_once('.')?;
    match format
    {
        FORMAT => decode_9(rest),
        FORMAT_8 => decode_8(rest),
        FORMAT_7 => decode_7(rest),
        FORMAT_6 => decode_6(rest),
        FORMAT_5 => decode_5(rest),
        FORMAT_4 => decode_4(rest),
        FORMAT_3 => decode_view(rest),
        FORMAT_2 =>
        {
            rest.split_once('.').and_then(|(bits, text)| decode_base(bits, text))
        }
        _ => None,
    }
}

/// Read a link from the URL hash: decode it, or, when a part does not decode,
/// keep its source and its stage with the default options and views. A link
/// of a format this build does not know, such as one a dev build wrote before
/// the format changed, then keeps the program and the open stage rather than
/// falling back to the default sample at the lexer stage.
///
/// # Arguments
///
/// * `hash` - The hash, with or without the leading `#`.
///
/// # Returns
///
/// The link, or `None` when the source does not decode either.
pub fn read(hash: &str) -> Option<Link> { decode(hash).or_else(|| salvage(hash)) }

/// The source and the stage of a link whose other parts do not decode. The
/// source is the last part, as base64url holds no dot; the stage is the first
/// part that names a phase.
///
/// # Arguments
///
/// * `hash` - The hash, with or without the leading `#`.
///
/// # Returns
///
/// The link with the default options and views, or `None` when the source
/// does not decode.
fn salvage(hash: &str) -> Option<Link>
{
    let (head, text) = hash.trim_start_matches('#').rsplit_once('.')?;
    let raw = URL_SAFE_NO_PAD.decode(text).ok()?;
    let source = String::from_utf8(raw).ok()?;
    let stage = head.split('.').find_map(|part| part.parse().ok()).unwrap_or_default();
    #[rustfmt::skip]
    return Some(Link::builder().source(source).options(Options::default()).stage(stage).build());
}

/// Decode the part of a link after the format: bits, foldings, ownership
/// check, mem flags, result mode, scratch mode, target, C checks, OS, CPU,
/// lint, stage, AST view, lowering view, source.
///
/// # Arguments
///
/// * `rest` - The part.
///
/// # Returns
///
/// The link, or `None` when a part is not well formed.
fn decode_9(rest: &str) -> Option<Link>
{
    let mut parts = rest.splitn(12, '.');
    let mut next = || parts.next();
    let head: Vec<&str> = (0..7).map(|_| next()).collect::<Option<_>>()?;
    let (c_checks, os, arch, lint, rest) =
        (next()?, next()?, next()?, next()?, next()?);
    let mut link = decode_8(&format!("{}.default.{rest}", head.join(".")))?;
    link.options.c_checks = RunChecks::from_bits(c_checks.parse().ok()?)?;
    link.options.os = os.parse().ok()?;
    link.options.arch = arch.parse().ok()?;
    link.options.lint = read_lint(lint)?;
    Some(link)
}

/// Decode the part of a link after the format: bits, foldings, ownership
/// check, mem flags, result mode, scratch mode, target, C checks, stage, AST
/// view, lowering view, source.
///
/// # Arguments
///
/// * `rest` - The part.
///
/// # Returns
///
/// The link, or `None` when a part is not well formed.
fn decode_8(rest: &str) -> Option<Link>
{
    let mut parts = rest.splitn(7, '.');
    let mut next = || parts.next();
    let (bits, fold, check, mem, results, scratch, rest) =
        (next()?, next()?, next()?, next()?, next()?, next()?, next()?);
    let (target, rest) = rest.split_once('.')?;
    let (c_checks, rest) = rest.split_once('.')?;
    let head = format!("{bits}.{fold}.{check}.{mem}.{results}.{scratch}");
    let mut link = decode_7(&format!("{head}.{rest}"))?;
    link.options.target = target.parse::<Target>().ok()?;
    link.options.c_checks = c_checks.parse::<Checks>().ok()?.into();
    Some(link)
}

/// Decode the part of a link after the format: bits, foldings, ownership
/// check, mem flags, result mode, scratch mode, stage, AST view, lowering
/// view, source.
///
/// # Arguments
///
/// * `rest` - The part.
///
/// # Returns
///
/// The link, or `None` when a part is not well formed.
fn decode_7(rest: &str) -> Option<Link>
{
    let mut parts = rest.splitn(6, '.');
    let mut next = || parts.next();
    let (bits, fold, check, mem, results, rest) =
        (next()?, next()?, next()?, next()?, next()?, next()?);
    let (scratch, rest) = rest.split_once('.')?;
    let mem: u8 = mem.parse().ok().filter(|n| *n < 8)?;
    let low = mem & 3;
    let mut link = decode_6(&format!("{bits}.{fold}.{check}.{low}.{results}.{rest}"))?;
    link.options.scratch = scratch.parse::<ScratchMode>().ok()?;
    link.options.overflow = tern!(mem & 4 != 0, OverflowMode::Stop, OverflowMode::Wrap);
    Some(link)
}

/// Decode the part of a link after the format: bits, foldings, ownership
/// check, mem flags, result mode, stage, AST view, lowering view, source.
///
/// # Arguments
///
/// * `rest` - The part.
///
/// # Returns
///
/// The link, or `None` when a part is not well formed.
fn decode_6(rest: &str) -> Option<Link>
{
    let mut parts = rest.splitn(5, '.');
    let (bits, fold, check, mem, rest) =
        (parts.next()?, parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    let (results, rest) = rest.split_once('.')?;
    let mut link = decode_5(&format!("{bits}.{fold}.{check}.{mem}.{rest}"))?;
    link.options.results = results.parse::<ResultMode>().ok()?;
    Some(link)
}

/// Decode the part of a link after the format: bits, foldings, ownership
/// check, mem flags, stage, AST view, lowering view, source.
///
/// # Arguments
///
/// * `rest` - The part.
///
/// # Returns
///
/// The link, or `None` when a part is not well formed.
fn decode_5(rest: &str) -> Option<Link>
{
    let (bits, rest) = rest.split_once('.')?;
    let (fold, rest) = rest.split_once('.')?;
    let (check, rest) = rest.split_once('.')?;
    let (mem, rest) = rest.split_once('.')?;
    let mem: u8 = mem.parse().ok().filter(|n| *n < 4)?;
    let mut link = decode_4(&format!("{bits}.{fold}.{rest}"))?;
    link.options.check = check.parse::<MemCheck>().ok()?;
    link.options.strict = mem & 1 != 0;
    link.options.no_unsafe = mem & 2 != 0;
    Some(link)
}

/// Decode the part of a link after the format: bits, foldings, stage, AST
/// view, lowering view, source.
///
/// # Arguments
///
/// * `rest` - The part.
///
/// # Returns
///
/// The link with the default checkers, or `None` when a part is not
/// well formed.
fn decode_4(rest: &str) -> Option<Link>
{
    let (bits, rest) = rest.split_once('.')?;
    let (fold, rest) = rest.split_once('.')?;
    let (stage, rest) = rest.split_once('.')?;
    let (ast, rest) = rest.split_once('.')?;
    let (lower, text) = rest.split_once('.')?;
    let mut link = decode_view(&format!("{bits}.{stage}.{ast}.{text}"))?;
    link.options.fold = read_fold(fold)?;
    link.lower_mode = lower.parse().ok()?;
    Some(link)
}

/// Decode the part of a link after the format: bits, stage, view, source.
///
/// # Arguments
///
/// * `rest` - The part.
///
/// # Returns
///
/// The link, or `None` when a part is not well formed.
fn decode_view(rest: &str) -> Option<Link>
{
    let mut parts = rest.splitn(4, '.');
    let (bits, stage, mode, text) =
        (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    let mut link = decode_base(bits, text)?;
    link.stage = stage.parse().ok()?;
    link.ast_mode = mode.parse().ok()?;
    Some(link)
}

/// Decode the bits and the source of a link.
///
/// # Arguments
///
/// * `bits` - The platform bits.
/// * `text` - The base64url source.
///
/// # Returns
///
/// The link with the default stage and view, or `None` when a part is not
/// well formed.
fn decode_base(bits: &str, text: &str) -> Option<Link>
{
    let bits: u16 = bits.parse().ok()?;
    ret_if!(!PLATFORM_BITS.contains(&bits), None);
    let raw = URL_SAFE_NO_PAD.decode(text).ok()?;
    let source = String::from_utf8(raw).ok()?;
    // Links before format 7 hold no overflow mode: arithmetic had no mode
    // but wrap when those links were made.
    #[rustfmt::skip]
    let options = Options::builder().platform_bits(bits).overflow(OverflowMode::Wrap).build();
    Some(Link::builder().source(source).options(options).build())
}
