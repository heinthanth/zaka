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
    /// `ကိန်းရှင် x သည် kind တစ်ခု ဖြစ်၍ တန်ဖိုးသည် v ဖြစ်ပါစေ။`
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
    /// `l ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ ...`
    Each(EachStmt),
    /// `ကိန်းရှင် i သည် kind တစ်ခု ဖြစ်၍ တန်ဖိုးသည် v ဖြစ်ကာ cond နေသရွေ့ ...`
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
    /// A blit of cells from one address to another.
    Blit(BlitStmt),
    /// A fill of cells at an address with one value.
    Fill(FillStmt),
    /// `l ထဲသို့ v ကို ထည့်ပါ။`
    Add(AddStmt),
    /// `l ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။`
    Remove(RemoveStmt),
    /// `t ၏ အကွက် a မှ b ထိကို v ဖြင့် အစားထိုးပါ။`
    Replace(ReplaceStmt),
    /// `ဒေတာအမျိုးအစား <name> တွင် ... တို့ ပါဝင်သည်။`
    Struct(StructDecl),
    /// `"<file>" မှ <name>၊ ... တို့ကို ထည့်သွင်းပါ။`
    Import(ImportStmt),
    /// `အမျိုးအစား <name> ဆိုသည်မှာ <member>၊ ... တို့ ဖြစ်သည်။`
    Enum(EnumDecl),
    /// `<subject> သည် <case> ဖြစ်လျှင် ... တခြား <kind> ဖြစ်လျှင် ...`
    Switch(SwitchStmt),
}

/// Variable or constant declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct DeclStmt
{
    /// The declared name.
    pub name: Ident,
    /// The declared kind, `<kind> တစ်<classifier>`; `None` when the value
    /// alone gives it.
    pub kind: Option<Kind>,
    /// The classifier after `တစ်`, written with the kind.
    #[serde(default)]
    pub classifier: Option<Ident>,
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

/// A loop over each cell of a list or a slice, or each character of text:
/// `<list> ၏ အကွက်တိုင်း ကို <name> အနေဖြင့် ယူ၍ ...` or
/// `<text> ၏ စာလုံးတိုင်း ကို <name> အနေဖြင့် ယူ၍ ...`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct EachStmt
{
    /// The list, the slice or the text walked.
    pub source: Expr,
    /// `true` for each character, `false` for each cell.
    pub chars: bool,
    /// The variable that holds a duplicate of each cell or character.
    pub var: Ident,
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
    /// Its kind, `<kind> တစ်<classifier>`; `None` when the initial value
    /// alone gives it.
    pub kind: Option<Kind>,
    /// The classifier after `တစ်`, written with the kind.
    #[serde(default)]
    pub classifier: Option<Ident>,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Param
{
    /// Whether `လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော` marks it: the function changes
    /// the caller's place, not a duplicate.
    #[builder(default)]
    #[serde(default)]
    pub edit: bool,
    /// The parameter kind.
    pub kind: Kind,
    /// The parameter name.
    pub name: Ident,
    /// Source region.
    pub span: Span,
}

/// The name the start block carries: the word of its header.
pub const START_NAME: &str = "ပရိုဂရမ်";

/// Function declaration, or the start block (`ပရိုဂရမ်သည် ...`), which has the
/// same parts and runs once, after the top-level statements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct FnDecl
{
    /// The function name.
    pub name: Ident,
    /// The parameters.
    #[builder(default)]
    pub params: Vec<Param>,
    /// The result kind, when the function returns a value; a result kind
    /// ([`crate::kind::BaseKind::Result`]) for a function that can fail.
    pub ret: Option<Kind>,
    /// The function block.
    pub block: BlockStmt,
    /// `true` when `အများသုံး` marks it, so other files can import it.
    #[builder(default)]
    pub exported: bool,
    /// `true` for the start block, where the program starts; no call can
    /// name it.
    #[builder(default)]
    pub start: bool,
    /// Source region.
    pub span: Span,
}

/// Return statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct ReturnStmt
{
    /// The returned value; `None` for a bare `ပြန်ပေးပါ။`.
    pub value: Option<Expr>,
    /// `true` when the value is an error value: `... ကို အမှားအယွင်းအဖြစ်
    /// ပြန်ပေးပါ။`
    #[builder(default)]
    #[serde(default)]
    pub fault: bool,
    /// `true` when the value is the good result of a function that can fail:
    /// `... ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးပါ။`
    #[builder(default)]
    #[serde(default)]
    pub good: bool,
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
    /// Whether `(လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)` marks each argument, one
    /// flag for each.
    #[builder(default)]
    #[serde(default)]
    pub edits: Vec<bool>,
    /// `true` when `မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးပါ` follows: a failure
    /// of the call leaves the calling function with the same error value.
    #[builder(default)]
    #[serde(default)]
    pub pass: bool,
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

/// A blit of the cells at `src` to `dst`, `memcpy` when the two ranges
/// must stand apart and `memmove` when the two can overlap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct BlitStmt
{
    /// The address the cells come from.
    pub src: Expr,
    /// The count of cells, or of octets when no kind is written.
    pub size: SizeArg,
    /// The address the cells go to.
    pub dst: Expr,
    /// Whether the two ranges can overlap.
    pub overlap: bool,
    /// Source region.
    pub span: Span,
}

/// A fill of the cells at `dst` with one value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct FillStmt
{
    /// The address of the first cell.
    pub dst: Expr,
    /// The value each cell gets.
    pub value: Expr,
    /// The count of cells, or of octets when no kind is written.
    pub size: SizeArg,
    /// Source region.
    pub span: Span,
}

/// `<list> ထဲသို့ <value> ကို ထည့်ပါ။`: the value becomes a new last cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct AddStmt
{
    /// The list that grows, a place; or, with `before`, the cell or the
    /// character of it that the value goes in front of.
    pub list: Expr,
    /// The new cell.
    pub value: Expr,
    /// Whether the value goes in front of the cell or the character `list`
    /// names: `<cell> ၏ အရှေ့တွင် <value> ကို ထည့်ပါ။`.
    #[serde(default)]
    #[builder(default)]
    pub before: bool,
    /// Source region.
    pub span: Span,
}

/// `<text> ၏ အကွက် <start> မှ <end> ထိကို <value> ဖြင့် အစားထိုးပါ။`, or
/// `<text> ၏ <start> လုံးမြောက်စာလုံးမှ <end> လုံးမြောက်စာလုံးထိကို <value> ဖြင့်
/// အစားထိုးပါ။`: the cells or the characters from `start` to `end`, both
/// included, give their place to the value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct ReplaceStmt
{
    /// The text that changes, a place.
    pub text: Expr,
    /// The first cell or character that goes.
    pub start: Expr,
    /// The last cell or character that goes.
    pub end: Expr,
    /// Whether `start` and `end` count characters from 1, not u8 cells
    /// from 0.
    #[serde(default)]
    #[builder(default)]
    pub chars: bool,
    /// The text, the view of text or the character that takes their place.
    pub value: Expr,
    /// Source region.
    pub span: Span,
}

/// `<list> ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။`: the list loses its last cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct RemoveStmt
{
    /// The list or the text that shrinks, a place.
    pub list: Expr,
    /// Whether the last character of text goes, not the last cell:
    /// `<text> ၏ နောက်ဆုံးစာလုံးကို ဖျက်ပါ။`.
    #[serde(default)]
    #[builder(default)]
    pub chars: bool,
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

/// One member of an enumeration: its name, and the value in parentheses
/// when one is written.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct EnumMember
{
    /// The member name.
    pub name: Ident,
    /// The value, `(<value>)`; with none, the member takes the value of the
    /// member before it plus one, and the first takes 0.
    pub value: Option<Expr>,
    /// Source region, from the name to the closing parenthesis.
    pub span: Span,
}

/// Enumeration declaration:
/// `အမျိုးအစား <name> ဆိုသည်မှာ <member> (<value>)?၊ ... တို့ ဖြစ်သည်။`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct EnumDecl
{
    /// The enumeration name.
    pub name: Ident,
    /// The members in order.
    pub members: Vec<EnumMember>,
    /// `true` when `အများသုံး` marks it, so other files can import it.
    #[builder(default)]
    pub exported: bool,
    /// Source region, the full stop included.
    pub span: Span,
}

/// One arm of a switch: `<case> ဖြစ်လျှင် <block>`, or with several cases,
/// `<case>၊ <case> တို့ ဖြစ်လျှင် <block>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct SwitchArm
{
    /// The values the subject is compared with; the arm runs when the subject
    /// equals one of them.
    pub cases: Vec<Expr>,
    /// The block run when the subject equals the case.
    pub block: BlockStmt,
    /// Source region, from the first case to the end of the block.
    pub span: Span,
}

/// The default arm of a switch: `တခြား <kind> ဖြစ်လျှင် <block>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct SwitchDefault
{
    /// The kind of the subject, as written.
    pub kind: Kind,
    /// The block run when no case equals the subject.
    pub block: BlockStmt,
    /// Source region, from `တခြား` to the end of the block.
    pub span: Span,
}

/// A switch over one subject:
/// `<subject> သည် <case> ဖြစ်လျှင် ... <case> ဖြစ်လျှင် ... တခြား <kind> ဖြစ်လျှင် ...`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct SwitchStmt
{
    /// The value each case is compared with.
    pub subject: Expr,
    /// The arms, in order; the first whose case equals the subject runs.
    #[builder(default)]
    pub arms: Vec<SwitchArm>,
    /// The default arm.
    pub default: Option<SwitchDefault>,
    /// Source region.
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
