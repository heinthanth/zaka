use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use bon::Builder;
use zakac_rust_sugars::ret_if;

use crate::compile::Options;
use crate::compile::PLATFORM_BITS;
use crate::compile::Phase;
use crate::state::settings::AstMode;

/// Version of the link format.
const FORMAT: &str = "3";

/// Version of the older format, without the stage and the AST view.
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
}

/// Encode a link for the URL hash.
///
/// The format is `3.<platform bits>.<stage>.<AST view>.<base64url of the
/// UTF-8 source>`, with the stage and the view in lowercase, as in
/// `3.64.parse.graph.…`.
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
    format!("{FORMAT}.{bits}.{stage}.{}.{text}", link.ast_mode)
}

/// Decode a URL hash made with [`encode`]; a link of the older `2.` format
/// opens with the default stage and view.
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
        FORMAT => decode_view(rest),
        FORMAT_2 =>
        {
            rest.split_once('.').and_then(|(bits, text)| decode_base(bits, text))
        }
        _ => None,
    }
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
