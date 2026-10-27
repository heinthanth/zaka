use std::borrow::Cow;

use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use wasm_bindgen::JsValue;
use zakac_i18n::t;

use crate::debug::octets;

/// The most heap a run can take in the browser: the wasm32 address space.
pub const HEAP_CEILING: u64 = 4 << 30;

/// The cap on live heap octets of a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Display, EnumString)]
pub enum HeapCap
{
    /// The RAM the browser reports, up to [`HEAP_CEILING`].
    #[default]
    #[strum(serialize = "auto")]
    Auto,
    /// 16 MiB.
    #[strum(serialize = "16")]
    Mib16,
    /// 64 MiB.
    #[strum(serialize = "64")]
    Mib64,
    /// 256 MiB.
    #[strum(serialize = "256")]
    Mib256,
    /// 1 GiB.
    #[strum(serialize = "1024")]
    Gib1,
}

impl HeapCap
{
    /// The cap in octets.
    ///
    /// # Returns
    ///
    /// The octets; for `Auto`, the RAM the browser reports, or the ceiling
    /// when it reports none.
    pub fn octets(self) -> u64
    {
        let mib = self.to_string().parse::<u64>().ok();
        mib.map_or_else(auto_heap, |m| m << 20)
    }

    /// Label in the picker; `Auto` names the cap it comes to.
    ///
    /// # Returns
    ///
    /// The label.
    pub fn label(self) -> Cow<'static, str>
    {
        let size = octets(self.octets());
        match self
        {
            HeapCap::Auto => t!("explorer.settings.heap_auto", size = size),
            _ => Cow::Owned(size),
        }
    }
}

/// The heap a run can take on this device: the RAM the browser reports, as
/// `navigator.deviceMemory` in GiB, up to the wasm32 address space. A
/// browser that reports none gets the ceiling; an allocation the host then
/// refuses traps as out of heap.
///
/// # Returns
///
/// The octets.
fn auto_heap() -> u64
{
    let Some(nav) = web_sys::window().map(|w| w.navigator())
    else
    {
        return HEAP_CEILING;
    };
    let gib = js_sys::Reflect::get(&nav, &JsValue::from_str("deviceMemory"));
    let gib = gib.ok().and_then(|v| v.as_f64()).filter(|g| *g > 0.0);
    gib.map_or(HEAP_CEILING, |g| ((g * (1u64 << 30) as f64) as u64).min(HEAP_CEILING))
}

/// How the AST view shows the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum AstMode
{
    /// The tree, with nodes that open and shut.
    #[default]
    Tree,
    /// The S-expression form, indented as Lisp code is.
    Sexpr,
    /// The tree drawn as boxes and lines.
    Graph,
}

impl AstMode
{
    /// Label of the switch.
    ///
    /// # Returns
    ///
    /// The label.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            AstMode::Tree => t!("explorer.ast.tree"),
            AstMode::Sexpr => t!("explorer.ast.sexpr"),
            AstMode::Graph => t!("explorer.ast.graph"),
        }
    }
}

/// Which IR the lowering view shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum LowerMode
{
    /// The IR as lowered, before folding.
    Plain,
    /// The IR after folding.
    #[default]
    Folded,
}

impl LowerMode
{
    /// Label of the switch.
    ///
    /// # Returns
    ///
    /// The label.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            LowerMode::Plain => t!("explorer.lower.plain"),
            LowerMode::Folded => t!("explorer.lower.folded"),
        }
    }
}
