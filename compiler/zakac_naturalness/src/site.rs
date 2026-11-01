use zakac_ast::expr::Expr;
use zakac_ast::expr::InfixOp;
use zakac_ast::expr::NextCtx;
use zakac_ast::verb::Slot;
use zakac_ast::verb::VERBS;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_lexer::reserved::RESERVED;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Particle;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

/// A place where a particle rule applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Site
{
    /// A clause-final word with its sentence-final particle.
    Final(FinalSite),
    /// A printed operand with the particle written after it.
    Print(PrintSite),
    /// A parenthesized expression and the place it fills.
    Group(GroupSite),
    /// A clause that gives a value: an initial value, a returned value, a
    /// call argument or a loop bound.
    Value(ValueSite),
}

/// A clause used as a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueSite
{
    /// The word that closes the clause.
    pub word: FinalWord,
    /// The sentence-final particle on that word.
    pub tail: Particle,
    /// The whole value.
    pub span: Span,
}

impl ValueSite
{
    /// The site of an expression used as a value, when it is a clause.
    ///
    /// # Arguments
    ///
    /// * `expr` - The value.
    ///
    /// # Returns
    ///
    /// The site, or `None` when the value does not end in a verb, a boolean
    /// or `မဟုတ်`.
    pub fn of(expr: &Expr) -> Option<ValueSite>
    {
        let (word, tail) = clause_end(expr)?;
        Some(ValueSite { word, tail, span: expr.span() })
    }
}

/// A parenthesized expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupSite
{
    /// The place the group fills.
    pub place: Place,
    /// What the group holds, nested parentheses looked through.
    pub inner: Inner,
    /// The group, parentheses included.
    pub span: Span,
}

/// Where a parenthesized expression stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place
{
    /// The whole operand of a statement, an initial value, a loop bound or
    /// a call argument. Such an operand is read as far as it goes, so the
    /// group matters for a logical expression alone.
    Operand,
    /// The whole condition of a branch or a loop, which takes a logical
    /// expression as it is.
    Condition,
    /// Straight inside another group.
    Nested,
}

/// What a group holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inner
{
    /// A logical expression: `ဖြစ်၍`, `သို့မဟုတ်` or postfix `မဟုတ်`.
    Logical,
    /// A list or struct literal, whose `၊` would join the values around it
    /// without the parentheses.
    List,
    /// An operand that stands on its own: a value, a notation or a clause.
    Plain,
}

impl Inner
{
    /// What an expression is, nested parentheses looked through.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression inside the group.
    ///
    /// # Returns
    ///
    /// The shape.
    pub fn of(expr: &Expr) -> Inner
    {
        match expr
        {
            Expr::Group(n) => Inner::of(&n.inner),
            Expr::Logic(_) | Expr::Not(_) => Inner::Logical,
            Expr::List(_) | Expr::Struct(_) => Inner::List,
            _ => Inner::Plain,
        }
    }
}

/// A clause-final word: a verb, a boolean literal or `မဟုတ်`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinalSite
{
    /// The word.
    pub word: FinalWord,
    /// The sentence-final particle written after it.
    pub particle: Particle,
    /// What follows the word.
    pub next: NextCtx,
    /// The word with its particle.
    pub span: Span,
}

/// The kind of a clause-final word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalWord
{
    /// A verb or verb-noun.
    Verb(VerbOp),
    /// `မှန်` or `မမှန်`.
    Bool(bool),
    /// `မဟုတ်`.
    Not,
}

impl FinalWord
{
    /// Whether the word is a negation: `မညီ`, `မကြီး`, `မငယ်`, `မမှန်`,
    /// `မဟုတ်`.
    ///
    /// # Returns
    ///
    /// `true` for a negative word.
    pub fn negative(self) -> bool
    {
        match self
        {
            FinalWord::Verb(op) => matches!(op, VerbOp::Ne | VerbOp::Le | VerbOp::Ge),
            FinalWord::Bool(value) => !value,
            FinalWord::Not => true,
        }
    }

    /// The spelling of the word, for messages.
    ///
    /// # Returns
    ///
    /// The reserved word text.
    pub fn text(self) -> &'static str
    {
        let kw = match self
        {
            FinalWord::Verb(op) => VERBS.iter().find(|v| v.op == op).map(|v| v.kw),
            FinalWord::Bool(true) => Some(Kw::True),
            FinalWord::Bool(false) => Some(Kw::False),
            FinalWord::Not => Some(Kw::Not),
        };
        kw.and_then(kw_text).unwrap_or_default()
    }
}

/// A printed operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrintSite
{
    /// What the operand is.
    pub shape: Shape,
    /// The particle written after it: `ကို` or `ဟု`.
    pub written: Slot,
    /// The operand.
    pub span: Span,
}

/// What a printed operand is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape
{
    /// A clause whose final word carries a sentence-final particle.
    Sentence,
    /// A string literal.
    Str,
    /// A name, a number, an arithmetic result or a bare clause.
    Value,
}

impl Shape
{
    /// Tell what a printed operand is.
    ///
    /// # Arguments
    ///
    /// * `expr` - The operand.
    ///
    /// # Returns
    ///
    /// The shape.
    pub fn of(expr: &Expr) -> Shape
    {
        ret_if!(is_text(expr), Shape::Str);
        let tail = clause_tail(expr);
        tail.is_some_and(|p| p != Particle::None)
            .then_some(Shape::Sentence)
            .unwrap_or(Shape::Value)
    }
}

/// The sentence-final particle of the word that closes an expression.
///
/// Looks through a group to its inner expression and through a logical
/// chain to its right side.
///
/// # Arguments
///
/// * `expr` - The expression.
///
/// # Returns
///
/// The particle of the closing verb, boolean or `မဟုတ်`, or `None` when the
/// expression does not end in one.
pub fn clause_tail(expr: &Expr) -> Option<Particle>
{
    clause_end(expr).map(|(_, particle)| particle)
}

/// The word that closes an expression and its sentence-final particle.
///
/// Looks through a group to its inner expression and through a logical
/// chain to its right side.
///
/// # Arguments
///
/// * `expr` - The expression.
///
/// # Returns
///
/// The closing word and its particle, or `None` when the expression does
/// not end in a verb, a boolean or `မဟုတ်`.
pub fn clause_end(expr: &Expr) -> Option<(FinalWord, Particle)>
{
    match expr
    {
        Expr::Group(n) => clause_end(&n.inner),
        Expr::Logic(n) => clause_end(&n.rhs),
        Expr::Verb(n) => Some((FinalWord::Verb(n.op), n.particle)),
        Expr::Bool(n) => Some((FinalWord::Bool(n.value), n.particle)),
        Expr::Not(n) => Some((FinalWord::Not, n.particle)),
        _ => None,
    }
}

/// The spelling of a reserved word.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The first spelling in the reserved table.
pub fn kw_text(kw: Kw) -> Option<&'static str>
{
    RESERVED.iter().find(|(_, k)| *k == kw).map(|(text, _)| *text)
}

/// The spelling of a sentence-final particle.
///
/// # Arguments
///
/// * `particle` - The particle.
///
/// # Returns
///
/// The text; blank for none.
pub fn particle_text(particle: Particle) -> &'static str
{
    match particle
    {
        Particle::None => "",
        Particle::Thi => "သည်",
        Particle::Pa => "ပါ",
        Particle::PaThi => "ပါသည်",
    }
}

/// Whether an expression reads as text: a string literal, or a join with
/// `+` that has one on a side.
///
/// # Arguments
///
/// * `expr` - The expression.
///
/// # Returns
///
/// `true` for such an expression.
fn is_text(expr: &Expr) -> bool
{
    match expr
    {
        Expr::Str(_) => true,
        Expr::Group(n) => is_text(&n.inner),
        Expr::Infix(n) if n.op == InfixOp::Add => is_text(&n.lhs) || is_text(&n.rhs),
        _ => false,
    }
}
