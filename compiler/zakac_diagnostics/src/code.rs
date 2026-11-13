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
/// `06` constant folding, `07` ownership, `08` the targets of `build`.
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
/// A statement verb ends in `ပါ`; statements end in `မည်`.
OldEnding = ("ZK0110", Level::Error),

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
/// A generic loop is declared with the constant word.
ConstLoop = ("ZK0225", Level::Error),
/// A block closer has no open block.
UnmatchedBlockClose = ("ZK0226", Level::Error),
/// The cell count of a list kind is not a whole number above zero.
BadListCount = ("ZK0227", Level::Error),
/// A struct is declared inside a block.
NestedStruct = ("ZK0228", Level::Error),
/// The export mark stands before something other than a top-level
/// declaration.
ExportNotOnDecl = ("ZK0229", Level::Error),
/// An import comes after a declaration or statement, or inside a block.
ImportNotFirst = ("ZK0230", Level::Error),
/// A file imports itself through a chain of imports.
CircularImport = ("ZK0231", Level::Error),
/// A program that was not read from a file has an import.
ImportNeedsFile = ("ZK0232", Level::Error),
/// A declared kind has no `တစ်<classifier>` after it.
DeclNeedsClassifier = ("ZK0234", Level::Error),
/// An enumeration is declared inside a block.
NestedEnum = ("ZK0235", Level::Error),
/// A switch arm follows the default arm.
ArmAfterDefault = ("ZK0236", Level::Error),
/// Some parameters of a header have a mark and some do not.
MixedMarks = ("ZK0237", Level::Error),
/// An import names a module the stdlib does not have.
UnknownStdModule = ("ZK0238", Level::Error),
/// A function kind with parameters stands bare in a parameter or field list.
FnKindNeedsParens = ("ZK0239", Level::Error),
/// A kind alias is declared inside a block.
NestedAlias = ("ZK0240", Level::Error),
/// The mark string of a parameter is not one word a mark can be.
BadMark = ("ZK0241", Level::Error),
/// `ဒေတာ ထည့်ပေးရန်` on a parameter of a function kind, where a call through
/// the value cannot know the place is filled.
FillInFnKind = ("ZK0243", Level::Error),
/// Expressions or blocks nest deeper than the parser follows.
NestingTooDeep = ("ZK0244", Level::Error),
/// A list of two or more items in words does not join its last item with
/// `နှင့်`, or has a `၊` after the `နှင့်`.
ListNeedsAnd = ("ZK0245", Level::Error),
/// The word that closes a list does not fit the count of its items: one item
/// takes the word with no `တို့`, two or more the word with `တို့`.
ListWordCount = ("ZK0246", Level::Error),
/// A trait or a trait block stands inside a block.
NestedTrait = ("ZK0247", Level::Error),
/// A constraint in a notes block names no kind parameter of its declaration.
NoteNamesNoParam = ("ZK0248", Level::Error),

// resolve: ZK03xx
/// A name is used without a declaration in scope.
UndeclaredName = ("ZK0301", Level::Error),
/// A name is declared twice in one scope.
Redeclaration = ("ZK0302", Level::Error),
/// A call names a function that is not declared.
UndeclaredFunction = ("ZK0303", Level::Error),
/// Two functions of one name take the same parameters and marks.
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
/// A struct literal sets one field twice.
FieldSetTwice = ("ZK0321", Level::Error),
/// An import names something the imported file does not export.
ImportNotFound = ("ZK0322", Level::Error),
/// An imported file holds a statement that would run.
StmtInImportedFile = ("ZK0323", Level::Error),
/// An exported declaration names a private struct of its file in a kind.
PrivateInExport = ("ZK0324", Level::Error),
/// A program has a second start block.
SecondStart = ("ZK0325", Level::Error),
/// A statement that would run stands at the top level of the file that is run.
TopLevelStmt = ("ZK0326", Level::Error),
/// The file that is run has no start block.
NoStart = ("ZK0327", Level::Error),
/// Two members of one enumeration share a name.
DuplicateMember = ("ZK0328", Level::Error),
/// `<name>များထဲမှ <member>` names a member the enumeration does not have.
UnknownMember = ("ZK0329", Level::Error),
/// The name before `ထဲမှ` is no enumeration written with `များ`.
UnknownEnum = ("ZK0330", Level::Error),
/// An argument of a marked call carries a mark other than its parameter's.
WrongMark = ("ZK0331", Level::Error),
/// No function of a name takes the arguments of a call.
NoOverloadFits = ("ZK0332", Level::Error),
/// More than one function of a name takes the arguments of a call.
OverloadUnclear = ("ZK0333", Level::Error),
/// An external function is none of the functions the host serves.
UnknownHostFn = ("ZK0334", Level::Error),
/// A call names both a function and a variable that holds a function value.
CalleeTwice = ("ZK0335", Level::Error),
/// A kind alias names itself, at once or through other aliases.
AliasLoop = ("ZK0336", Level::Error),
/// A C function has a name that is no C name.
NotACName = ("ZK0337", Level::Error),
/// Two C functions of one name have different parameter or result kinds.
CNameTwice = ("ZK0338", Level::Error),
/// A function with a result can reach the end of its block with no return.
EndWithoutReturn = ("ZK0339", Level::Error),
/// A call or a kind gives a generic another number of kind arguments than
/// it declares.
KindArgCount = ("ZK0340", Level::Error),
/// The kinds of a call of a generic function or a value of a generic struct
/// cannot be worked out, or two values give one kind parameter two kinds.
KindsUnknown = ("ZK0341", Level::Error),
/// A generic function makes instances that make further instances without
/// end, past the limit of instances.
InstancesEndless = ("ZK0342", Level::Error),
/// A kind given to a generic does not have a trait its kind parameter needs.
BoundNotMet = ("ZK0343", Level::Error),
/// A constraint or a trait block names no trait.
UnknownTrait = ("ZK0344", Level::Error),
/// A trait block lacks a function of its trait.
TraitFnMissing = ("ZK0345", Level::Error),
/// A function of a trait block is not one of its trait, or has other
/// parameter or result kinds.
TraitFnForeign = ("ZK0346", Level::Error),
/// Two functions give a kind the same function of a trait.
TraitFnTwice = ("ZK0347", Level::Error),
/// A trait block stands in a file that declares neither its kind nor its
/// trait.
TraitBlockPlace = ("ZK0348", Level::Error),
/// The mark of a declaration the compiler gives stands outside the stdlib.
BuiltinOutsidePrelude = ("ZK0349", Level::Error),

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
/// Cells are read from a value that is no list.
NotAList = ("ZK0422", Level::Error),
/// A cell number is no integer.
CellNumberNotInt = ("ZK0423", Level::Error),
/// A list kind is too large for the machine.
ListTooLarge = ("ZK0425", Level::Error),
/// A field is read from a value that is no struct.
NotAStruct = ("ZK0426", Level::Error),
/// A struct holds itself, other than through an address.
StructHoldsItself = ("ZK0427", Level::Error),
/// A cell of text is written.
TextCellWrite = ("ZK0429", Level::Error),
/// A declaration with no kind has a value that gives none.
KindNotInferred = ("ZK0430", Level::Error),
/// A place a call changes through one argument appears in another.
EditShared = ("ZK0433", Level::Error),
/// An each loop walks a value it cannot walk.
EachSourceWrong = ("ZK0434", Level::Error),
/// `[]` is given to a counted list outside its declaration.
BracketsOnCountedList = ("ZK0436", Level::Error),
/// An add or a remove statement changes a value that cannot grow or shrink.
ListCannotGrow = ("ZK0437", Level::Error),
/// `၏ အရှေ့တွင်` follows a value that is no cell of a list and no character
/// of text.
InsertNotAtCell = ("ZK0439", Level::Error),
/// A parameter of the start block is no text list, or there are more than two.
StartParamKind = ("ZK0440", Level::Error),
/// The result of the start block is no integer.
StartResultKind = ("ZK0441", Level::Error),
/// A raw duplicate or fill writes cells that own a block or borrow one.
BlitOfHeldCells = ("ZK0442", Level::Error),
/// A call to a function that can fail does not pass the failure on.
FailureNotPassed = ("ZK0443", Level::Error),
/// A call passes a failure on, but the function cannot fail.
PassWithoutFailure = ("ZK0444", Level::Error),
/// A failure passes on from a function that cannot fail, or from the
/// top-level statements.
PassOutsideFailing = ("ZK0445", Level::Error),
/// A failure passes on, but its error kind differs from the error kind of
/// the function around the call.
FaultKindDiffers = ("ZK0446", Level::Error),
/// An error value is given back from a function that cannot fail.
FaultFromSure = ("ZK0447", Level::Error),
/// A test or a conversion of a result reads a value that is no result of a
/// function that can fail.
NotAResult = ("ZK0448", Level::Error),
/// A conversion to the good result reads a result with no good value: the
/// function gives back nothing but a failure.
NoGoodValue = ("ZK0449", Level::Error),
/// A conversion to the error value reads a result whose error side holds
/// no value.
NoFaultValue = ("ZK0450", Level::Error),
/// `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်` in a function whose failure holds
/// an error value.
FaultValueMissing = ("ZK0451", Level::Error),
/// A conversion reads the part of a result that the code before it proves
/// the result does not hold.
WrongPartProven = ("ZK0454", Level::Error),
/// The value of an enumeration member does not work out when the program
/// compiles.
MemberValue = ("ZK0456", Level::Error),
/// Two members of one enumeration share a value.
DuplicateMemberValue = ("ZK0457", Level::Error),
/// A switch over an enumeration names neither each member nor a default arm.
SwitchMissing = ("ZK0458", Level::Error),
/// The default arm of a switch names a kind other than its subject's.
DefaultKind = ("ZK0459", Level::Error),
/// Two cases of one switch have one value.
DuplicateCase = ("ZK0460", Level::Error),
/// An enumeration member with no value whose value cannot count on.
MemberNeedsValue = ("ZK0461", Level::Error),
/// A call goes through a value that is no function value.
NotAFunction = ("ZK0462", Level::Error),
/// The address of a function of an overloaded name fits no single one.
FnAddrUnclear = ("ZK0463", Level::Error),
/// The address of an external function is taken.
ExternalFnAddr = ("ZK0464", Level::Error),
/// A parameter or the result of a C function has a kind C cannot take.
KindNotAcrossC = ("ZK0465", Level::Error),
/// A parameter of a C function is marked `ဒေတာ ပြောင်းလဲရန်`.
EditAcrossC = ("ZK0466", Level::Error),
/// `အပိုင်အနေဖြင့်` on a result that is no text from a C function.
OwnedNotCText = ("ZK0467", Level::Error),
/// An operation that works with pointers stands outside unsafe code.
UnsafeOutside = ("ZK0468", Level::Error),
/// An unsafe function, `စိတ်မချရသော`, is taken as a value.
UnsafeFnValue = ("ZK0469", Level::Error),
/// `အပိုင်အနေဖြင့်` stands on a parameter or result that is no pointer.
OwnedNotPtr = ("ZK0470", Level::Error),
/// Unsafe code where `--no-unsafe` refuses it.
UnsafeRefused = ("ZK0471", Level::Error),
/// A C function with a pointer in its kinds, or a result that can be NULL,
/// is not declared `စိတ်မချရသော`.
ForeignNeedsUnsafe = ("ZK0472", Level::Error),
/// A value of a kind from outside the program is made, copied or printed
/// in Zaka.
HandleInZaka = ("ZK0473", Level::Error),
/// A field of a struct a handle points at is written; C owns it.
CFieldWrite = ("ZK0474", Level::Error),
/// A test for data on a result, or for a good side on an optional.
TestWordMismatch = ("ZK0475", Level::Error),
/// An error value where no result kind with an error side is wanted.
FaultNeedsResult = ("ZK0476", Level::Error),
NoDataNeedsOptional = ("ZK0477", Level::Error),
/// `ကနဦးတန်ဖိုး` where no kind can be worked out from its place.
InitialNeedsKind = ("ZK0478", Level::Error),
/// `ကနဦးတန်ဖိုး` for a kind that has none: a function value, a handle, an
/// enumeration with no member marked, or a kind that holds one of them.
NoInitialValue = ("ZK0479", Level::Error),
/// Two members of one enumeration marked `ကနဦးတန်ဖိုး`.
TwoInitialMembers = ("ZK0480", Level::Error),
/// A kind no program can build: its fields hold it again, with no optional
/// field or plain list on the loop to end the chain.
KindNeverBuilt = ("ZK0481", Level::Error),
/// A struct literal that leaves out a field: no field takes a hidden default.
FieldLeftOut = ("ZK0482", Level::Error),
/// A global of a module declared with no value: a module has no start block
/// to set it in.
ModuleGlobalNeedsValue = ("ZK0483", Level::Error),
/// A row read, a row count or a row loop on a value that is no ဇယား.
NotATable = ("ZK0484", Level::Error),
/// A ဇယား kind whose index kind holds a part that cannot be hashed and
/// compared as a whole value: an object, a function value, a pointer, an
/// optional, a result or a ဇယား.
BadIndexKind = ("ZK0485", Level::Error),
/// `ဇယားအလွတ်` or `[:]` where no ဇယား kind can be worked out from its place.
TableNeedsKind = ("ZK0486", Level::Error),
/// A ဇယား kind whose index kind holds a decimal number, which rounding can
/// split into two rows.
DecimalIndex = ("ZK0487", Level::Warning),
/// `ကို ဖျက်မည်` after a value that is no row of a ဇယား.
RemoveNeedsRow = ("ZK0488", Level::Error),
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
/// A variable or constant of an imported file starts with a value that is
/// not known at compile time.
GlobalInitNotConstant = ("ZK0604", Level::Error),
/// A constant range passes the end of a list of known count.
RangeOutOfList = ("ZK0605", Level::Error),
/// A constant that holds text or a plain list holds a value that needs the
/// program to run.
ConstNotKnown = ("ZK0606", Level::Error),
/// A range of known text starts or ends inside a character.
RangeCutsChar = ("ZK0607", Level::Error),
// ownership: ZK07xx
/// A name is used after its block moved to another owner.
MovedOut = ("ZK0701", Level::Error),
/// A pointer to a variable, not to a heap block, is freed or resized.
BadFree = ("ZK0702", Level::Error),
/// A borrowed address is stored where it could outlast its owner.
BorrowStored = ("ZK0703", Level::Error),
/// An owner changes its block while a borrow of it is still used.
InvalidatedLoan = ("ZK0704", Level::Error),
/// A function returns a borrow of one of its locals.
LocalBorrowReturned = ("ZK0706", Level::Error),
/// A call frees through one argument while another reaches the same block.
AliasedArgument = ("ZK0707", Level::Error),
/// An owner goes out of scope without its block freed or moved.
BlockLeaked = ("ZK0708", Level::Error),
/// A name known to hold the null address is dereferenced.
NullDeref = ("ZK0709", Level::Error),
/// A constant offset is past the end of its block.
OffsetPastEnd = ("ZK0710", Level::Error),
/// A name that can hold the null address is dereferenced without a check.
NullDerefPossible = ("ZK0711", Level::Warning),
/// Pointers into different blocks are subtracted.
MixedBlocks = ("ZK0712", Level::Error),
/// With `--overflow stop`, an integer operation overflows for each value
/// its operands can hold.
OverflowProven = ("ZK0719", Level::Warning),
/// The two ranges of a `ကူးယူမည်` overlap for certain.
BlitRangesOverlap = ("ZK0721", Level::Error),
/// A read of a variable, a field or a cell that can be unset on a path to it.
ReadUnset = ("ZK0722", Level::Error),
/// A read of a cell the check cannot prove set: a cell was written, or is
/// read, at a position the check does not know.
ReadUnsetDoubt = ("ZK0723", Level::Warning),
/// A call of a function that reads a global that can be unset at the call.
GlobalUnsetAtCall = ("ZK0724", Level::Error),
/// A return that can leave a parameter marked `ဒေတာ ထည့်ပေးရန်` unset.
FillUnsetAtReturn = ("ZK0725", Level::Error) }

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
