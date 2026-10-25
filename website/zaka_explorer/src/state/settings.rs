use std::borrow::Cow;

use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use zakac_i18n::t;

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
