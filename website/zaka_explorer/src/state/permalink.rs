use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use bon::Builder;
use zakac_borrow_checker::options::MemCheck;
use zakac_ir_builder::ir::FoldOptions;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::Options;
use crate::compile::PLATFORM_BITS;
use crate::compile::Phase;
use crate::state::settings::AstMode;
use crate::state::settings::LowerMode;

/// Version of the link format.
const FORMAT: &str = "5";

/// Version of the format before the ownership checks.
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
/// The format is `5.<platform bits>.<foldings>.<ownership check>.<mem
/// flags>.<stage>.<AST view>.<lowering view>.<base64url of the UTF-8
/// source>`, with the check, the stage and the views in lowercase, as in
/// `5.64.7.error.0.lower.tree.folded.…`. The foldings are a digit from 0 to
/// 7: 1 for expressions, 2 for constants, 4 for branches. The mem flags are
/// a digit from 0 to 3: 1 for the strict check, 2 to forbid raw addresses.
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
    format!("{FORMAT}.{bits}.{fold}.{check}.{mem}.{stage}.{views}.{text}")
}

/// The digit of the flags of the ownership check.
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
    u8::from(options.strict) | u8::from(options.no_raw) << 1
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
        FORMAT => decode_5(rest),
        FORMAT_4 => decode_4(rest),
        FORMAT_3 => decode_view(rest),
        FORMAT_2 =>
        {
            rest.split_once('.').and_then(|(bits, text)| decode_base(bits, text))
        }
        _ => None,
    }
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
    link.options.no_raw = mem & 2 != 0;
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
/// The link with the default ownership checks, or `None` when a part is not
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
    let options = Options::builder().platform_bits(bits).build();
    Some(Link::builder().source(source).options(options).build())
}
