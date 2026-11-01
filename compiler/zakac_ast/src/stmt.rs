use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::AsRefStr;
use zakac_lexer::token::StrLit;
use zakac_span::span::Span;

use crate::expr::Expr;
use crate::expr::Ident;
use crate::kind::Kind;
use crate::verb::Slot;

/// A whole program: the top-level statements of each of its files in order.
///
/// A program read from one file has no units. A program read with its
/// imports lists one unit per file, each imported file ahead of the files
/// that import it, so the file that is run comes last.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Program
{
    /// The top-level statements.
    #[builder(default)]
    pub stmts: Vec<Stmt>,
    /// The files, in the order their statements appear in `stmts`.
    #[builder(default)]
    pub units: Vec<Unit>,
    /// Source region of the file that is run.
    pub span: Span,
}

/// One file of a program read with its imports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Unit
{
    /// The file name, as diagnostics print it.
    pub name: String,
    /// The index in `Program::stmts` of the first statement of the file.
    pub first_stmt: u32,
}

impl Program
{
    /// The unit that holds a top-level statement.
    ///
    /// # Arguments
    ///
    /// * `stmt` - The index of the statement in `stmts`.
    ///
    /// # Returns
    ///
    /// The index of its unit; `0` in a program with no units.
    pub fn unit_of(&self, stmt: usize) -> u32
    {
        let after = self.units.partition_point(|u| u.first_stmt as usize <= stmt);
        after.saturating_sub(1) as u32
    }

    /// Whether a unit is the file that is run.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit index.
    ///
    /// # Returns
    ///
    /// `true` for the last unit, and for each statement of a program with no
    /// units.
    pub fn is_root(&self, unit: u32) -> bool { unit as usize + 1 >= self.units.len() }
}

/// A statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stmt
{
    /// `ကိန်းရှင် x သည် kind v ဖြစ်ပါစေ။`
    Decl(DeclStmt),
    /// `x ကို v ဟု ထားပါ။`
    Assign(AssignStmt),
    /// `v ကို ပြသပါ။`
    Print(PrintStmt),
    /// `အောက်ပါတို့ကို လုပ်ဆောင်ပါ။ ... လုပ်ဆောင်စရာများ ပြီးပါပြီ။`
    Block(BlockStmt),
    /// `အကယ်၍ ... လျှင် ...`
    If(IfStmt),
    /// `cond နေသရွေ့ ...`
    While(WhileStmt),
    /// `a မှ b အထိ i အနေဖြင့် ရေတွက်၍ ...`
    Counted(CountedStmt),
    /// `ကိန်းရှင် i သည် kind ဖြစ်၍ တန်ဖိုးသည် v ဖြစ်ကာ cond နေသရွေ့ ...`
    Generic(GenericStmt),
    /// `ကွင်းဆက် ကို ရပ်ပါ။`
    Break(BreakStmt),
    /// `နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။`
    Continue(ContinueStmt),
    /// `လုပ်ဆောင်ချက် f သည် ...`
    Fn(FnDecl),
    /// `v ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။` or `ပြန်ပေးပါ။`
    Return(ReturnStmt),
    /// `လုပ်ဆောင်ချက် f ကို ... လုပ်ဆောင်ပါ။`
    Call(CallStmt),
    /// Heap allocation into a place.
    Alloc(AllocStmt),
    /// Heap reallocation of the block a place points at.
    Resize(ResizeStmt),
    /// Release of the heap block a place points at.
    Free(FreeStmt),
    /// `ဒေတာအမျိုးအစား <name> တွင် ... တို့ ပါဝင်သည်။`
    Struct(StructDecl),
    /// `"<file>" မှ <name>၊ ... တို့ကို ထည့်သွင်းပါ။`
    Import(ImportStmt),
}

/// Variable or constant declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct DeclStmt
{
    /// The declared name.
    pub name: Ident,
    /// The declared kind.
    pub kind: Kind,
    /// `true` for `ကိန်းသေ`.
    pub is_const: bool,
    /// The initial value, when written.
    pub init: Option<Expr>,
    /// `true` when `အများသုံး` marks it, so other files can import it.
    #[builder(default)]
    pub exported: bool,
    /// Source region, the full stop included.
    pub span: Span,
}

/// Assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct AssignStmt
{
    /// The assigned place: a variable or a dereference.
    pub target: Expr,
    /// The new value.
    pub value: Expr,
    /// Source region.
    pub span: Span,
}

/// The file path of an import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct StrArg
{
    /// The decoded literal.
    pub lit: StrLit,
    /// Source region, quotes included.
    pub span: Span,
}

/// Print statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct PrintStmt
{
    /// The printed operand.
    pub value: Expr,
    /// The particle written after the operand: `ကို` or `ဟု`.
    pub slot: Slot,
    /// Source region.
    pub span: Span,
}

/// A block, or the single statement that stands in for one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct BlockStmt
{
    /// The statements inside.
    #[builder(default)]
    pub stmts: Vec<Stmt>,
    /// `true` when the block is a single statement written without the
    /// block phrases.
    #[builder(default)]
    pub single: bool,
    /// Source region.
    pub span: Span,
}

/// One condition and its block in a conditional statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct IfArm
{
    /// The condition.
    pub cond: Expr,
    /// The block run when the condition holds.
    pub block: BlockStmt,
    /// Source region, from the opening word to the end of the block.
    pub span: Span,
}

/// Conditional statement with else-if arms and an else block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct IfStmt
{
    /// The `အကယ်၍` arm, then each `ထိုသို့မဟုတ်ဘဲ` arm.
    #[builder(default)]
    pub arms: Vec<IfArm>,
    /// The `သို့မဟုတ်ပါက` block.
    pub else_block: Option<BlockStmt>,
    /// Source region.
    pub span: Span,
}

/// While loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct WhileStmt
{
    /// The condition.
    pub cond: Expr,
    /// The loop block.
    pub block: BlockStmt,
    /// Source region.
    pub span: Span,
}

/// Direction word of a counted loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr)]
#[serde(rename_all = "snake_case")]
pub enum CountMode
{
    /// `ရေတွက်၍`: toward the end bound.
    #[strum(serialize = "count")]
    Count,
    /// `တိုး၍`: adds the step.
    #[strum(serialize = "up")]
    Increase,
    /// `လျှော့၍`: subtracts the step.
    #[strum(serialize = "down")]
    Decrease,
}

/// Counted loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct CountedStmt
{
    /// The loop variable.
    pub var: Ident,
    /// The first bound.
    pub from: Expr,
    /// The last bound, inclusive.
    pub to: Expr,
    /// The step, when written.
    pub step: Option<Expr>,
    /// The direction word.
    pub mode: CountMode,
    /// The loop block.
    pub block: BlockStmt,
    /// Source region.
    pub span: Span,
}

/// Generic loop, like a C `for`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct GenericStmt
{
    /// The loop variable.
    pub var: Ident,
    /// Its kind.
    pub kind: Kind,
    /// Its initial value, when written.
    pub init: Option<Expr>,
    /// The condition.
    pub cond: Expr,
    /// The `ပြီးလျှင်` statement, run after each iteration.
    #[builder(into)]
    pub update: Option<Box<Stmt>>,
    /// The loop block, without the update.
    pub block: BlockStmt,
    /// Source region.
    pub span: Span,
}

/// `ကွင်းဆက် ကို ရပ်ပါ။`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct BreakStmt
{
    /// Source region.
    pub span: Span,
}

/// `နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct ContinueStmt
{
    /// Source region.
    pub span: Span,
}

/// A function parameter: `ကိန်းပြည့် က`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Param
{
    /// The parameter kind.
    pub kind: Kind,
    /// The parameter name.
    pub name: Ident,
    /// Source region.
    pub span: Span,
}

/// Function declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct FnDecl
{
    /// The function name.
    pub name: Ident,
    /// The parameters.
    #[builder(default)]
    pub params: Vec<Param>,
    /// The result kind, when the function returns a value.
    pub ret: Option<Kind>,
    /// The function block.
    pub block: BlockStmt,
    /// `true` when `အများသုံး` marks it, so other files can import it.
    #[builder(default)]
    pub exported: bool,
    /// Source region.
    pub span: Span,
}

/// Return statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct ReturnStmt
{
    /// The returned value; `None` for a bare `ပြန်ပေးပါ။`.
    pub value: Option<Expr>,
    /// Source region.
    pub span: Span,
}

/// Call statement; the result is dropped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct CallStmt
{
    /// The function name.
    pub callee: Ident,
    /// The arguments.
    #[builder(default)]
    pub args: Vec<Expr>,
    /// Source region.
    pub span: Span,
}

/// The size operand of a heap statement: a count, and the kind of each
/// element when one is written.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct SizeArg
{
    /// The element count, or the size in octets when no kind is written.
    pub count: Expr,
    /// The element kind, when written.
    pub unit: Option<Kind>,
    /// The classifier of the count word, the text before its `စာ`: `ခု` in
    /// `ခုစာ`, `ယောက်` in `ယောက်စာ`. Written with each unit kind.
    pub classifier: Option<Ident>,
    /// Source region.
    pub span: Span,
}

/// Heap allocation: the new address is stored in `target`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct AllocStmt
{
    /// The requested size.
    pub size: SizeArg,
    /// The place that receives the address.
    pub target: Expr,
    /// Source region.
    pub span: Span,
}

/// Heap reallocation: the block at `target` gets a new size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct ResizeStmt
{
    /// The place that holds the address, updated in place.
    pub target: Expr,
    /// The new size.
    pub size: SizeArg,
    /// Source region.
    pub span: Span,
}

/// Heap release of the block at `target`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct FreeStmt
{
    /// The address to release.
    pub target: Expr,
    /// Source region.
    pub span: Span,
}

/// Struct declaration:
/// `ဒေတာအမျိုးအစား <name> တွင် <kind> <field>၊ ... တို့ ပါဝင်သည်။`, with an
/// optional classifier clause `ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct StructDecl
{
    /// The struct name.
    pub name: Ident,
    /// The fields in order, each a kind and a name.
    pub fields: Vec<Param>,
    /// The classifier that counts values of the struct, when written.
    pub classifier: Option<Ident>,
    /// `true` when `အများသုံး` marks it, so other files can import it.
    #[builder(default)]
    pub exported: bool,
    /// Source region, the full stop included.
    pub span: Span,
}

/// Import: `"<file>" မှ <name>၊ ... တို့ကို ထည့်သွင်းပါ။`, or with one name,
/// `"<file>" မှ <name> ကို ထည့်သွင်းပါ။`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct ImportStmt
{
    /// The file path, relative to the importing file.
    pub path: StrArg,
    /// The imported names.
    pub names: Vec<Ident>,
    /// The unit of the imported file, once the loader has read it.
    pub unit: Option<u32>,
    /// Source region, the full stop included.
    pub span: Span,
}
