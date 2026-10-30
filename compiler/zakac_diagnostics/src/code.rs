use std::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use zakac_i18n::t;

use crate::diagnostic::Level;

/// Define [`Code`] from one row per code, so the enum, the id table and the
/// level table cannot drift apart.
#[rustfmt::skip]
macro_rules! define_codes {
($($(#[$meta:meta])* $name:ident = ($id:literal, $level:expr)),+ $(,)?) => {
/// A stable diagnostic code, printed as `error[ZK0101]` and stored in
/// artifacts as its id.
///
/// Ids are `ZK` plus four digits; the first two give the phase:
/// `01` lexer, `02` parser, `03` resolve, `04` kinds, `05` naturalness,
/// `06` constant folding, `07` ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code { $($(#[$meta])* $name,)+ }

impl Code
{
    /// All codes, in id order.
    pub const ALL: &'static [Code] = &[$(Code::$name),+];

    /// The stable id, for example `ZK0101`.
    ///
    /// # Returns
    ///
    /// The id.
    pub const fn id(self) -> &'static str
    {
        match self { $(Code::$name => $id,)+ }
    }

    /// The level this code is reported at.
    ///
    /// # Returns
    ///
    /// The level.
    pub const fn level(self) -> Level
    {
        match self { $(Code::$name => $level,)+ }
    }
}
};}

define_codes! {
// lexer: ZK01xx
/// A word holds a character that cannot start or continue a token.
InvalidCharacter = ("ZK0101", Level::Error),
/// A word starts with a digit but is not a number.
InvalidNumber = ("ZK0102", Level::Error),
/// A number mixes Burmese and Western digits.
MixedDigitScripts = ("ZK0103", Level::Error),
/// A string literal reaches the end of the line.
UnterminatedString = ("ZK0104", Level::Error),
/// A character literal reaches the end of the line.
UnterminatedChar = ("ZK0105", Level::Error),
/// An escape sequence is not known.
UnknownEscape = ("ZK0106", Level::Error),
/// A character literal does not hold one character.
BadCharLength = ("ZK0107", Level::Error),
/// A raw identifier reaches the end of the line.
UnterminatedRawIdentifier = ("ZK0108", Level::Error),
/// A raw identifier has nothing between its backticks.
BlankRawIdentifier = ("ZK0109", Level::Error),

// parser: ZK02xx
/// A token does not fit the grammar at this point.
ExpectedToken = ("ZK0201", Level::Error),
/// An operand is missing.
ExpectedOperand = ("ZK0202", Level::Error),
/// A statement does not end in a statement verb.
ExpectedStmtVerb = ("ZK0203", Level::Error),
/// A statement verb appears inside an expression.
StmtVerbInExpr = ("ZK0204", Level::Error),
/// A verb has fewer operands than it takes.
MissingOperand = ("ZK0205", Level::Error),
/// An operand carries the wrong particle for its verb.
SlotMismatch = ("ZK0206", Level::Error),
/// An operand is followed with two particles.
DoubleParticle = ("ZK0207", Level::Error),
/// Operands wait for a verb at the end of a clause.
UnclosedClause = ("ZK0208", Level::Error),
/// Operands are left over before a statement verb.
DanglingOperands = ("ZK0209", Level::Error),
/// A comparison is the direct operand of another comparison.
ChainedComparison = ("ZK0210", Level::Error),
/// An else or else-if word starts a statement.
ElseWithoutIf = ("ZK0211", Level::Error),
/// A generic loop update stands outside a generic loop.
UpdateOutsideLoop = ("ZK0212", Level::Error),
/// A generic loop update is not the last statement of its loop.
UpdateNotLast = ("ZK0213", Level::Error),
/// A function is declared inside a block.
NestedFunction = ("ZK0214", Level::Error),
/// A single-statement block is a conditional.
NestedSingleLineIf = ("ZK0215", Level::Error),
/// A kind has a bit count it cannot have.
InvalidBitCount = ("ZK0216", Level::Error),
/// A sign word stands before a kind other than an integer.
SignedFloat = ("ZK0217", Level::Error),
/// A base prefix is not an integer from 2 to 36.
InvalidBase = ("ZK0218", Level::Error),
/// A digit is not valid in the base of its literal.
InvalidDigit = ("ZK0219", Level::Error),
/// An integer literal does not fit its kind.
LiteralOutOfRange = ("ZK0220", Level::Error),
/// An integer kind prefix stands before a literal with a fraction.
FloatForIntKind = ("ZK0221", Level::Error),
/// A character or boolean kind prefixes a number.
BadLiteralKind = ("ZK0222", Level::Error),
/// A string literal is used as a value.
StrOutsidePrint = ("ZK0223", Level::Error),
/// A generic loop is declared with the constant word.
ConstLoop = ("ZK0225", Level::Error),
/// A block closer has no open block.
UnmatchedBlockClose = ("ZK0226", Level::Error),
/// The cell count of a list kind is not a whole number above zero.
BadListCount = ("ZK0227", Level::Error),
/// A struct is declared inside a block.
NestedStruct = ("ZK0228", Level::Error),

// resolve: ZK03xx
/// A name is used without a declaration in scope.
UndeclaredName = ("ZK0301", Level::Error),
/// A name is declared twice in one scope.
Redeclaration = ("ZK0302", Level::Error),
/// A call names a function that is not declared.
UndeclaredFunction = ("ZK0303", Level::Error),
/// Two functions share a name.
DuplicateFunction = ("ZK0304", Level::Error),
/// Two parameters of one function share a name.
DuplicateParam = ("ZK0305", Level::Error),
/// A constant is assigned.
AssignToConst = ("ZK0306", Level::Error),
/// A counted-loop variable is assigned.
AssignToLoopVar = ("ZK0307", Level::Error),
/// A constant is declared without an initial value.
ConstWithoutInit = ("ZK0308", Level::Error),
/// A break stands outside a loop.
BreakOutsideLoop = ("ZK0309", Level::Error),
/// A continue stands outside a loop.
ContinueOutsideLoop = ("ZK0310", Level::Error),
/// A return stands outside a function.
ReturnOutsideFunction = ("ZK0311", Level::Error),
/// A function without a result returns a value.
ValueReturnedFromVoid = ("ZK0312", Level::Error),
/// A function with a result returns no value.
MissingReturnValue = ("ZK0313", Level::Error),
/// A call passes the wrong number of arguments.
ArgCountMismatch = ("ZK0314", Level::Error),
/// A function without a result is used as a value.
ValueFromVoidFunction = ("ZK0315", Level::Error),
/// Two structs share a name.
DuplicateStruct = ("ZK0316", Level::Error),
/// Two fields of one struct share a name.
DuplicateField = ("ZK0317", Level::Error),
/// A kind names a struct that is not declared.
UnknownKindName = ("ZK0318", Level::Error),
/// A variable, parameter, field or function takes the name of a struct.
StructNameClash = ("ZK0319", Level::Error),
/// A field access names a field the struct does not have.
UnknownField = ("ZK0320", Level::Error),

// kinds: ZK04xx
/// A value does not convert to the kind its place expects.
KindMismatch = ("ZK0401", Level::Error),
/// A condition is not a boolean.
ConditionNotBool = ("ZK0402", Level::Error),
/// An operand has a kind its operator does not take.
OperandKind = ("ZK0403", Level::Error),
/// Two operands have no common kind.
IncompatibleOperands = ("ZK0404", Level::Error),
/// A counted-loop bound or step is not an integer.
LoopBoundNotInt = ("ZK0405", Level::Error),
/// An integer literal does not fit the kind its place expects.
LiteralDoesNotFit = ("ZK0406", Level::Error),
/// A constant shift distance is not below the bit width.
ShiftTooFar = ("ZK0407", Level::Error),
/// A conversion between two kinds is not allowed.
InvalidCast = ("ZK0408", Level::Error),
/// A counted-loop step is not positive.
StepNotPositive = ("ZK0409", Level::Error),
/// An assignment target is neither a variable nor a dereference.
NotAPlace = ("ZK0410", Level::Error),
/// The address of a value that has no address is taken.
NoAddress = ("ZK0411", Level::Error),
/// The address of a constant or a counted-loop variable is taken.
FixedAddress = ("ZK0412", Level::Error),
/// A value that is not a pointer is dereferenced.
NotAPointer = ("ZK0413", Level::Error),
/// An opaque pointer is dereferenced or offset.
OpaquePointer = ("ZK0414", Level::Error),
/// An operator that pointers do not take is used on a pointer.
PointerOperator = ("ZK0415", Level::Error),
/// Pointers to different kinds are compared or subtracted.
PointerKindsDiffer = ("ZK0416", Level::Error),
/// A heap statement is given a value that is not a pointer.
HeapNeedsPointer = ("ZK0417", Level::Error),
/// A list literal has more or fewer cells than its kind.
ListCountMismatch = ("ZK0419", Level::Error),
/// A blank list stands where no list kind is known.
BlankListKind = ("ZK0420", Level::Error),
/// A list kind has no cell count and nothing gives one.
ListCountMissing = ("ZK0421", Level::Error),
/// Cells are read from a value that is no list.
NotAList = ("ZK0422", Level::Error),
/// A cell number is no integer.
CellNumberNotInt = ("ZK0423", Level::Error),
/// A function gives back a list as its result.
ListResult = ("ZK0424", Level::Error),
/// A list kind is too large for the machine.
ListTooLarge = ("ZK0425", Level::Error),
/// A field is read from a value that is no struct.
NotAStruct = ("ZK0426", Level::Error),
/// A struct holds itself, other than through an address.
StructHoldsItself = ("ZK0427", Level::Error),
/// A function gives back a struct as its result.
StructResult = ("ZK0428", Level::Error),
/// A negative verb before `သို့မဟုတ်` carries `သည်`.
NegativeBeforeOr = ("ZK0501", Level::Warning),
/// A verb before an attached `လျှင်` or `နေသရွေ့` carries a particle.
ParticleBeforeAttached = ("ZK0502", Level::Warning),
/// `မဟုတ်` carries `သည်`.
NotWithThi = ("ZK0503", Level::Warning),
/// A verb before `ဖြစ်၍` carries `ပါ`.
PaBeforeAnd = ("ZK0504", Level::Warning),
/// A sentence or a string is printed with `ကို`.
PrintSentenceWithKo = ("ZK0505", Level::Warning),
/// A plain value is printed with `ဟု`.
PrintValueWithHu = ("ZK0506", Level::Warning),
/// Parentheses around an expression that stands alone in its place.
RedundantGroup = ("ZK0507", Level::Warning),
/// A clause used as a value ends in `ပါ`.
PaInValue = ("ZK0508", Level::Warning),
/// A count word uses a classifier other than the kind's own.
ClassifierMismatch = ("ZK0509", Level::Warning),
// constant folding: ZK06xx
/// An operation on constants gives a value outside its kind.
ConstantOverflow = ("ZK0601", Level::Error),
/// An integer division or remainder has the constant zero as divisor.
ConstantZeroDivisor = ("ZK0602", Level::Error),
/// A constant cell number is outside its list.
CellOutOfRange = ("ZK0603", Level::Error),
// ownership: ZK07xx
/// A name is used after its block moved to another owner.
MovedOut = ("ZK0701", Level::Error),
/// A borrowed address is freed or resized.
FreedBorrow = ("ZK0702", Level::Error),
/// A borrowed address is stored where it could outlast its owner.
BorrowStored = ("ZK0703", Level::Error),
/// An owner changes its block while a borrow of it is still used.
InvalidatedLoan = ("ZK0704", Level::Error),
/// The address of a borrowed address is taken.
BorrowOfBorrow = ("ZK0705", Level::Error),
/// A function returns a borrow of one of its locals.
LocalBorrowReturned = ("ZK0706", Level::Error),
/// A call frees through one argument while another reaches the same block.
AliasedArgument = ("ZK0707", Level::Error),
/// An owner goes out of scope without its block freed or moved.
BlockLeaked = ("ZK0708", Level::Error),
/// A name known to hold ဗလာ is dereferenced.
NullDeref = ("ZK0709", Level::Error),
/// A constant offset is past the end of its block.
OffsetPastEnd = ("ZK0710", Level::Error),
/// A name that can hold ဗလာ is dereferenced without a check.
NullDerefPossible = ("ZK0711", Level::Warning),
/// Pointers into different blocks are subtracted.
MixedBlocks = ("ZK0712", Level::Error),
/// A borrow marker sits where none is allowed.
BadBorrowMarker = ("ZK0713", Level::Error),
/// A borrowed address is given where an owner is wanted.
BorrowForOwner = ("ZK0714", Level::Error),
/// An owner is given where a borrow is wanted and would move.
OwnerForBorrow = ("ZK0715", Level::Error),
/// A raw address is lent.
LendOfRaw = ("ZK0716", Level::Error),
/// A raw address is used though raw addresses are turned off.
RawNotAllowed = ("ZK0717", Level::Error),
/// A struct field holds a borrow.
BorrowField = ("ZK0718", Level::Error) }

impl Code
{
    /// Find a code from its id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id, for example `ZK0101`.
    ///
    /// # Returns
    ///
    /// The code, or `None` for an unknown id.
    pub fn parse(id: &str) -> Option<Code>
    {
        Self::ALL.iter().copied().find(|c| c.id() == id)
    }

    /// Short localized title from the catalogue.
    ///
    /// # Returns
    ///
    /// The title.
    pub fn title(self) -> String { self.catalogue_text("title") }

    /// Long localized explanation from the catalogue.
    ///
    /// # Returns
    ///
    /// The explanation.
    pub fn explain(self) -> String { self.catalogue_text("explain") }

    /// Look up a catalogue text for this code.
    ///
    /// # Arguments
    ///
    /// * `field` - The text name: `title` or `explain`.
    ///
    /// # Returns
    ///
    /// The localized text.
    fn catalogue_text(self, field: &str) -> String
    {
        let name = format!("codes.{}.{field}", self.id());
        t!(name).to_string()
    }
}

impl fmt::Display for Code
{
    /// Write the id.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The result of the write.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.id()) }
}

impl Serialize for Code
{
    /// Serialize as the id string.
    ///
    /// # Arguments
    ///
    /// * `serializer` - The serializer.
    ///
    /// # Returns
    ///
    /// The result of the serialization.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for Code
{
    /// Deserialize from the id string.
    ///
    /// # Arguments
    ///
    /// * `deserializer` - The deserializer.
    ///
    /// # Returns
    ///
    /// The code, or an error for an unknown id.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>
    {
        let id = String::deserialize(deserializer)?;
        Code::parse(&id).ok_or_else(|| {
            serde::de::Error::custom(format!("unknown diagnostic code {id}"))
        })
    }
}
