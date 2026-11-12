use zakac_i18n::t;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::code::Code;
use crate::diagnostic::Diagnostic;

/// A value does not convert to the kind its place expects.
///
/// # Arguments
///
/// * `span` - The value.
/// * `want` - The expected kind, as text.
/// * `got` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0401` diagnostic, with a hint about the conversion verb.
pub fn kind_mismatch(span: Span, want: &str, got: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0401.message", want = want, got = got);
    Diagnostic::new(Code::KindMismatch, msg, span).with_help(t!("codes.ZK0401.help"))
}

/// A value does not convert to the kind its place expects, and no
/// conversion can bridge the two: a list never converts, and none turns a
/// borrow into an owner or back.
///
/// # Arguments
///
/// * `span` - The value.
/// * `want` - The expected kind, as text.
/// * `got` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0401` diagnostic, with no help.
pub fn kind_mismatch_unbridged(span: Span, want: &str, got: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0401.message", want = want, got = got);
    Diagnostic::new(Code::KindMismatch, msg, span)
}

/// A condition is not a boolean.
///
/// # Arguments
///
/// * `span` - The condition.
/// * `got` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0402` diagnostic, with a hint to compare a pointer with null.
pub fn condition_not_bool(span: Span, got: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0402.message", got = got);
    let diag = Diagnostic::new(Code::ConditionNotBool, msg, span);
    let pointer = got.ends_with('*') || got == "ptr";
    tern!(pointer, diag.with_help(t!("codes.ZK0402.help")), diag)
}

/// An operand has a kind its operator does not take.
///
/// # Arguments
///
/// * `span` - The operand.
/// * `op` - The operator spelling.
/// * `want` - What the operator takes, in words.
/// * `got` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0403` diagnostic.
pub fn operand_kind(span: Span, op: &str, want: &str, got: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0403.message", op = op, want = want, got = got);
    Diagnostic::new(Code::OperandKind, msg, span)
}

/// Two operands have no common kind.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `lhs` - The kind of the left operand, as text.
/// * `rhs` - The kind of the right operand, as text.
///
/// # Returns
///
/// The `ZK0404` diagnostic.
pub fn incompatible_operands(span: Span, lhs: &str, rhs: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0404.message", l = lhs, r = rhs);
    Diagnostic::new(Code::IncompatibleOperands, msg, span)
}

/// The sameness test `X နှင့် Y သည် တစ်ခုတည်း` on kinds that are not shared
/// objects or function values.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `lhs` - Name of the left kind.
/// * `rhs` - Name of the right kind.
///
/// # Returns
///
/// The `ZK0404` diagnostic, with a help line.
pub fn sameness_of_values(span: Span, lhs: &str, rhs: &str) -> Diagnostic
{
    incompatible_operands(span, lhs, rhs).with_help(t!("codes.ZK0404.help_same"))
}

/// A counted-loop bound or step is not an integer.
///
/// # Arguments
///
/// * `span` - The bound or step.
/// * `got` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0405` diagnostic.
pub fn loop_bound_not_int(span: Span, got: &str) -> Diagnostic
{
    Diagnostic::new(Code::LoopBoundNotInt, t!("codes.ZK0405.message", got = got), span)
}

/// An each loop walks a value it cannot walk: an each-cell loop over no
/// list or slice, or over cells that hold an owner pointer, or an
/// each-character loop over no text.
///
/// # Arguments
///
/// * `span` - The value the loop walks.
/// * `got` - Its kind, as text.
///
/// # Returns
///
/// The `ZK0434` diagnostic.
pub fn each_source_wrong(span: Span, got: &str) -> Diagnostic
{
    Diagnostic::new(Code::EachSourceWrong, t!("codes.ZK0434.message", got = got), span)
}

/// `[]` or `စာရင်းအလွတ်` is given to a counted list.
///
/// # Arguments
///
/// * `span` - The `[]`.
/// * `kind` - The counted list kind, as text.
///
/// # Returns
///
/// The `ZK0436` diagnostic.
pub fn brackets_on_counted_list(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0436.message", k = kind);
    Diagnostic::new(Code::BracketsOnCountedList, msg, span)
        .with_help(t!("codes.ZK0436.help"))
}

/// An add or a remove statement changes a value that cannot grow or shrink.
///
/// # Arguments
///
/// * `span` - The value.
/// * `kind` - Its kind, as text.
///
/// # Returns
///
/// The `ZK0437` diagnostic.
pub fn list_cannot_grow(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(Code::ListCannotGrow, t!("codes.ZK0437.message", k = kind), span)
}

/// `၏ အရှေ့တွင်` follows a value that is no cell of a list and no character
/// of text.
///
/// # Arguments
///
/// * `span` - The value.
///
/// # Returns
///
/// The `ZK0439` diagnostic, with help naming the forms.
pub fn insert_not_at_cell(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0439.message");
    Diagnostic::new(Code::InsertNotAtCell, msg, span).with_help(t!("codes.ZK0439.help"))
}

/// A parameter of the start block is no text list, is a place to change, or
/// comes after the second.
///
/// # Arguments
///
/// * `span` - The parameter.
///
/// # Returns
///
/// The `ZK0440` diagnostic, with help naming the kind.
pub fn start_param_kind(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0440.message");
    Diagnostic::new(Code::StartParamKind, msg, span).with_help(t!("codes.ZK0440.help"))
}

/// The result of the start block is no integer.
///
/// # Arguments
///
/// * `span` - The result kind.
/// * `kind` - Its spelling.
///
/// # Returns
///
/// The `ZK0441` diagnostic, with help naming the kind.
pub fn start_result_kind(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0441.message", k = kind);
    Diagnostic::new(Code::StartResultKind, msg, span).with_help(t!("codes.ZK0441.help"))
}

/// A call to a function that can fail does not pass the failure on.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function name.
///
/// # Returns
///
/// The `ZK0443` diagnostic.
pub fn failure_not_passed(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0443.message", f = name);
    let help = t!("codes.ZK0443.help", f = name);
    Diagnostic::new(Code::FailureNotPassed, msg, span).with_help(help)
}

/// A call passes a failure on, but the function cannot fail.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function name.
///
/// # Returns
///
/// The `ZK0444` diagnostic.
pub fn pass_without_failure(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0444.message", f = name);
    let help = t!("codes.ZK0444.help", f = name);
    Diagnostic::new(Code::PassWithoutFailure, msg, span).with_help(help)
}

/// A failure passes on from a function that cannot fail, or from the
/// top-level statements.
///
/// # Arguments
///
/// * `span` - The call.
///
/// # Returns
///
/// The `ZK0445` diagnostic.
pub fn pass_outside_failing(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0445.message");
    Diagnostic::new(Code::PassOutsideFailing, msg, span)
        .with_help(t!("codes.ZK0445.help"))
}

/// A failure passes on, but its error kind differs from the error kind of
/// the function around the call.
///
/// # Arguments
///
/// * `span` - The call.
/// * `found` - The error kind of the called function.
/// * `want` - The error kind of the function around the call.
///
/// # Returns
///
/// The `ZK0446` diagnostic.
pub fn fault_kind_differs(span: Span, found: &str, want: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0446.message", found = found, want = want);
    Diagnostic::new(Code::FaultKindDiffers, msg, span)
        .with_help(t!("codes.ZK0446.help"))
}

/// An error value is given back from a function that cannot fail.
///
/// # Arguments
///
/// * `span` - The return statement.
/// * `name` - The function name.
///
/// # Returns
///
/// The `ZK0447` diagnostic.
pub fn fault_from_sure(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0447.message", f = name);
    Diagnostic::new(Code::FaultFromSure, msg, span).with_help(t!("codes.ZK0447.help"))
}

/// A test or a conversion of a result reads a value that is no result of a
/// function that can fail.
///
/// # Arguments
///
/// * `span` - The value.
/// * `kind` - Its kind, as text.
///
/// # Returns
///
/// The `ZK0448` diagnostic.
pub fn not_a_result(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0448.message", k = kind);
    Diagnostic::new(Code::NotAResult, msg, span).with_help(t!("codes.ZK0448.help"))
}

/// A conversion to the good result reads a result with no good value.
///
/// # Arguments
///
/// * `span` - The result.
///
/// # Returns
///
/// The `ZK0449` diagnostic.
pub fn no_good_value(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0449.message");
    Diagnostic::new(Code::NoGoodValue, msg, span).with_help(t!("codes.ZK0449.help"))
}

/// A conversion to the error value reads a result whose error side holds no
/// value.
///
/// # Arguments
///
/// * `span` - The result.
///
/// # Returns
///
/// The `ZK0450` diagnostic.
pub fn no_fault_value(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0450.message");
    Diagnostic::new(Code::NoFaultValue, msg, span).with_help(t!("codes.ZK0450.help"))
}

/// A failure with no error value in a function whose failure holds one.
///
/// # Arguments
///
/// * `span` - The statement.
/// * `kind` - The error kind, as text.
///
/// # Returns
///
/// The `ZK0451` diagnostic.
pub fn fault_value_missing(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0451.message", k = kind);
    Diagnostic::new(Code::FaultValueMissing, msg, span)
        .with_help(t!("codes.ZK0451.help"))
}

/// A conversion reads the part of a result that the code before it proves
/// the result does not hold.
///
/// # Arguments
///
/// * `span` - The conversion.
/// * `name` - The variable that holds the result.
/// * `good` - Whether the result is proven good, so the error part is read.
/// * `optional` - Whether it is an optional, proven to hold no data.
///
/// # Returns
///
/// The `ZK0454` diagnostic.
pub fn wrong_part_proven(
    span: Span, name: &str, good: bool, optional: bool,
) -> Diagnostic
{
    let msg = match (good, optional)
    {
        (_, true) => t!("codes.ZK0454.message_no_data", n = name),
        (true, false) => t!("codes.ZK0454.message_good", n = name),
        (false, false) => t!("codes.ZK0454.message_fault", n = name),
    };
    Diagnostic::new(Code::WrongPartProven, msg, span).with_help(t!("codes.ZK0454.help"))
}

/// An integer literal does not fit the kind its place expects.
///
/// # Arguments
///
/// * `span` - The literal.
/// * `value` - The literal value, as text.
/// * `kind` - The expected kind, as text.
///
/// # Returns
///
/// The `ZK0406` diagnostic.
pub fn literal_does_not_fit(span: Span, value: &str, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0406.message", v = value, k = kind);
    Diagnostic::new(Code::LiteralDoesNotFit, msg, span)
}

/// A constant shift distance is not below the bit width.
///
/// # Arguments
///
/// * `span` - The distance.
/// * `distance` - The distance, as text.
/// * `width` - The bit width of the shifted value.
///
/// # Returns
///
/// The `ZK0407` diagnostic.
pub fn shift_too_far(span: Span, distance: &str, width: u16) -> Diagnostic
{
    let width = width.to_string();
    let msg = t!("codes.ZK0407.message", n = distance, w = width);
    Diagnostic::new(Code::ShiftTooFar, msg, span)
}

/// A conversion between two kinds is not allowed.
///
/// # Arguments
///
/// * `span` - The conversion.
/// * `from` - The source kind, as text.
/// * `to` - The target kind, as text.
///
/// # Returns
///
/// The `ZK0408` diagnostic.
pub fn invalid_cast(span: Span, from: &str, to: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0408.message", from = from, to = to);
    Diagnostic::new(Code::InvalidCast, msg, span)
}

/// A counted-loop step is not positive.
///
/// # Arguments
///
/// * `span` - The step.
///
/// # Returns
///
/// The `ZK0409` diagnostic.
pub fn step_not_positive(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::StepNotPositive, t!("codes.ZK0409.message"), span)
}

/// An assignment target is neither a variable nor a dereference.
///
/// # Arguments
///
/// * `span` - The target operand.
///
/// # Returns
///
/// The `ZK0410` diagnostic.
pub fn not_a_place(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::NotAPlace, t!("codes.ZK0410.message"), span)
}

/// The address of a value that has no address is taken.
///
/// # Arguments
///
/// * `span` - The operand of the address operation.
/// * `what` - What the operand is, in words.
///
/// # Returns
///
/// The `ZK0411` diagnostic.
pub fn no_address(span: Span, what: &str) -> Diagnostic
{
    Diagnostic::new(Code::NoAddress, t!("codes.ZK0411.message", what = what), span)
}

/// The address of a constant or a counted-loop variable is taken.
///
/// # Arguments
///
/// * `span` - The operand of the address operation.
/// * `name` - The fixed name.
///
/// # Returns
///
/// The `ZK0412` diagnostic.
pub fn fixed_address(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::FixedAddress, t!("codes.ZK0412.message", name = name), span)
}

/// A value that is not a pointer is dereferenced.
///
/// # Arguments
///
/// * `span` - The dereferenced operand.
/// * `kind` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0413` diagnostic.
pub fn not_a_pointer(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(Code::NotAPointer, t!("codes.ZK0413.message", kind = kind), span)
}

/// An opaque pointer is dereferenced or offset.
///
/// # Arguments
///
/// * `span` - The opaque pointer operand.
///
/// # Returns
///
/// The `ZK0414` diagnostic, with a hint about the conversion verb.
pub fn opaque_pointer(span: Span) -> Diagnostic
{
    let diag = Diagnostic::new(Code::OpaquePointer, t!("codes.ZK0414.message"), span);
    diag.with_help(t!("codes.ZK0414.help"))
}

/// An operator that pointers do not take is used on a pointer.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `op` - The operator spelling.
///
/// # Returns
///
/// The `ZK0415` diagnostic.
pub fn pointer_operator(span: Span, op: &str) -> Diagnostic
{
    Diagnostic::new(Code::PointerOperator, t!("codes.ZK0415.message", op = op), span)
}

/// Pointers to different kinds are compared or subtracted.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `a` - The kind of the left pointer, as text.
/// * `b` - The kind of the right pointer, as text.
///
/// # Returns
///
/// The `ZK0416` diagnostic.
pub fn pointer_kinds_differ(span: Span, a: &str, b: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0416.message", a = a, b = b);
    Diagnostic::new(Code::PointerKindsDiffer, msg, span)
}

/// A heap statement is given a value that is not a pointer.
///
/// # Arguments
///
/// * `span` - The operand of the heap statement.
/// * `kind` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0417` diagnostic.
pub fn heap_needs_pointer(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(
        Code::HeapNeedsPointer,
        t!("codes.ZK0417.message", kind = kind),
        span,
    )
}

/// A raw duplicate or fill writes cells that own a block or borrow one.
///
/// # Arguments
///
/// * `span` - The size of the statement.
/// * `kind` - The kind of the cells, as text.
///
/// # Returns
///
/// The `ZK0442` diagnostic, with help.
pub fn blit_of_held_cells(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0442.message", kind = kind);
    Diagnostic::new(Code::BlitOfHeldCells, msg, span).with_help(t!("codes.ZK0442.help"))
}

/// A list literal has more or fewer cells than its kind.
///
/// # Arguments
///
/// * `span` - The literal.
/// * `want` - The cell count of the kind.
/// * `found` - The cell count of the literal.
///
/// # Returns
///
/// The `ZK0419` diagnostic.
pub fn list_count_mismatch(span: Span, want: u32, found: usize) -> Diagnostic
{
    let msg = t!("codes.ZK0419.message", want = want, found = found);
    Diagnostic::new(Code::ListCountMismatch, msg, span)
}

/// A blank list stands where no list kind is known.
///
/// # Arguments
///
/// * `span` - The blank list.
///
/// # Returns
///
/// The `ZK0420` diagnostic.
pub fn blank_list_kind(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::BlankListKind, t!("codes.ZK0420.message"), span)
        .with_help(t!("codes.ZK0420.help"))
}

/// Cells are read from a value that is no list.
///
/// # Arguments
///
/// * `span` - The value.
/// * `kind` - Its kind, as text.
///
/// # Returns
///
/// The `ZK0422` diagnostic.
pub fn not_a_list(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(Code::NotAList, t!("codes.ZK0422.message", k = kind), span)
}

/// A cell number is no integer.
///
/// # Arguments
///
/// * `span` - The cell number.
/// * `kind` - Its kind, as text.
///
/// # Returns
///
/// The `ZK0423` diagnostic.
pub fn cell_number_not_int(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0423.message", k = kind);
    Diagnostic::new(Code::CellNumberNotInt, msg, span)
}

/// A list kind is too large for the machine.
///
/// # Arguments
///
/// * `span` - The kind.
/// * `kind` - The kind, as text.
///
/// # Returns
///
/// The `ZK0425` diagnostic.
pub fn list_too_large(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(Code::ListTooLarge, t!("codes.ZK0425.message", k = kind), span)
}

/// A field is read from a value that is no struct.
///
/// # Arguments
///
/// * `span` - The field name.
/// * `name` - The field name.
/// * `kind` - The kind of the value before `၏`, as text.
///
/// # Returns
///
/// The `ZK0426` diagnostic.
pub fn not_a_struct(span: Span, name: &str, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0426.message", n = name, k = kind);
    Diagnostic::new(Code::NotAStruct, msg, span)
}

/// A struct holds itself, other than through an address.
///
/// # Arguments
///
/// * `span` - The struct name in its declaration.
/// * `name` - The struct name.
///
/// # Returns
///
/// The `ZK0427` diagnostic.
pub fn struct_holds_itself(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::StructHoldsItself, t!("codes.ZK0427.message", n = name), span)
        .with_help(t!("codes.ZK0427.help", n = name))
}

/// A cell of text is the target of an assignment.
///
/// # Arguments
///
/// * `span` - The cell.
/// * `kind` - The name of the text kind.
///
/// # Returns
///
/// The `ZK0429` diagnostic.
pub fn text_cell_write(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(Code::TextCellWrite, t!("codes.ZK0429.message", k = kind), span)
}

/// A declaration with no kind has a value that gives none.
///
/// # Arguments
///
/// * `span` - The value.
/// * `name` - The declared name.
///
/// # Returns
///
/// The `ZK0430` diagnostic.
pub fn kind_not_inferred(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::KindNotInferred, t!("codes.ZK0430.message", n = name), span)
        .with_help(t!("codes.ZK0430.help"))
}

/// A place a call changes through one argument appears in another.
///
/// # Arguments
///
/// * `span` - The other argument.
/// * `name` - The variable of the place.
///
/// # Returns
///
/// The `ZK0433` diagnostic.
pub fn edit_shared(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::EditShared, t!("codes.ZK0433.message", n = name), span)
}

/// The value of an enumeration member does not work out when the program
/// compiles.
///
/// # Arguments
///
/// * `span` - The value.
/// * `name` - The member name.
///
/// # Returns
///
/// The `ZK0456` diagnostic.
pub fn member_value(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::MemberValue, t!("codes.ZK0456.message", n = name), span)
}

/// Two members of one enumeration share a value.
///
/// # Arguments
///
/// * `span` - The second member.
/// * `name` - The second member name.
/// * `first` - The first member with that value.
///
/// # Returns
///
/// The `ZK0457` diagnostic.
pub fn duplicate_member_value(span: Span, name: &str, first: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0457.message", n = name, f = first);
    Diagnostic::new(Code::DuplicateMemberValue, msg, span)
}

/// A switch over an enumeration names neither each member nor a default
/// arm.
///
/// # Arguments
///
/// * `span` - The subject.
/// * `missing` - The members no arm names, joined for the reader.
/// * `stem` - The stem of the enumeration name, for the default arm.
/// * `word` - The classifier of the enumeration.
///
/// # Returns
///
/// The `ZK0458` diagnostic, with help to add a default arm.
pub fn switch_missing(span: Span, missing: &str, stem: &str, word: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0458.message", m = missing);
    let help = t!("codes.ZK0458.help", k = stem, w = word);
    Diagnostic::new(Code::SwitchMissing, msg, span).with_help(help)
}

/// The default arm of a switch names a kind other than its subject's.
///
/// # Arguments
///
/// * `span` - The kind of the default arm.
/// * `written` - The kind it names.
/// * `subject` - The kind of the subject.
///
/// # Returns
///
/// The `ZK0459` diagnostic.
pub fn default_kind(span: Span, written: &str, subject: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0459.message", w = written, s = subject);
    Diagnostic::new(Code::DefaultKind, msg, span)
}

/// Two cases of one switch have one value.
///
/// # Arguments
///
/// * `span` - The later case.
///
/// # Returns
///
/// The `ZK0460` diagnostic.
pub fn duplicate_case(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::DuplicateCase, t!("codes.ZK0460.message"), span)
}

/// An enumeration member with no value written, whose value cannot count on
/// from the member before it: the value kind is no integer, or the value
/// passes the largest of the kind.
///
/// # Arguments
///
/// * `span` - The member name.
/// * `name` - The member name.
/// * `kind` - The value kind.
///
/// # Returns
///
/// The `ZK0461` diagnostic.
pub fn member_needs_value(span: Span, name: &str, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0461.message", n = name, k = kind);
    Diagnostic::new(Code::MemberNeedsValue, msg, span)
}

/// A call goes through a value that is no function value.
///
/// # Arguments
///
/// * `span` - The name or the place at the call.
/// * `name` - Its text.
/// * `kind` - Its kind.
///
/// # Returns
///
/// The `ZK0462` diagnostic.
pub fn not_a_function(span: Span, name: &str, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0462.message", n = name, k = kind);
    Diagnostic::new(Code::NotAFunction, msg, span)
}

/// A leave for C to call a value that is no function value.
///
/// # Arguments
///
/// * `span` - The value.
/// * `kind` - Its kind.
///
/// # Returns
///
/// The `ZK0462` diagnostic.
pub fn leave_needs_function(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0462.leave", k = kind);
    Diagnostic::new(Code::NotAFunction, msg, span)
}

/// The address of a function of an overloaded name fits no single one.
///
/// # Arguments
///
/// * `span` - The address.
/// * `name` - The function name.
///
/// # Returns
///
/// The `ZK0463` diagnostic.
pub fn fn_addr_unclear(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::FnAddrUnclear, t!("codes.ZK0463.message", n = name), span)
        .with_help(t!("codes.ZK0463.help"))
}

/// The address of an external function is taken.
///
/// # Arguments
///
/// * `span` - The address.
/// * `name` - The function name.
///
/// # Returns
///
/// The `ZK0464` diagnostic.
pub fn external_fn_addr(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::ExternalFnAddr, t!("codes.ZK0464.message", n = name), span)
        .with_help(t!("codes.ZK0464.help"))
}

/// A parameter or the result of a C function has a kind C cannot take.
///
/// # Arguments
///
/// * `span` - The parameter, or the name of the function for its result.
/// * `kind` - The kind, as text.
///
/// # Returns
///
/// The `ZK0465` diagnostic.
pub fn kind_not_across_c(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(Code::KindNotAcrossC, t!("codes.ZK0465.message", k = kind), span)
        .with_help(t!("codes.ZK0465.help"))
}

/// A C function gives back text with no failure side, which a NULL from C
/// could not fill.
///
/// # Arguments
///
/// * `span` - The name of the function.
/// * `kind` - The result kind written.
///
/// # Returns
///
/// The `ZK0465` diagnostic, with the result kind to write.
pub fn text_result_across_c(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(Code::KindNotAcrossC, t!("codes.ZK0465.message", k = kind), span)
        .with_help(t!("codes.ZK0465.help_text"))
}

/// `အပိုင်အနေဖြင့်` stands on the result of a function that is no C function
/// giving back text or a failure.
///
/// # Arguments
///
/// * `span` - The name of the function.
/// * `name` - The function name.
///
/// # Returns
///
/// The `ZK0467` diagnostic.
pub fn owned_not_c_text(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::OwnedNotCText, t!("codes.ZK0467.message", n = name), span)
        .with_help(t!("codes.ZK0467.help"))
}

/// A parameter of a C function is marked `ဒေတာ ပြောင်းလဲရန်`.
///
/// # Arguments
///
/// * `span` - The parameter.
/// * `name` - The parameter name.
///
/// # Returns
///
/// The `ZK0466` diagnostic.
pub fn edit_across_c(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::EditAcrossC, t!("codes.ZK0466.message", n = name), span)
        .with_help(t!("codes.ZK0466.help"))
}

/// An operation that works with pointers stands outside marked code.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `what` - The words of the operation, as text.
///
/// # Returns
///
/// The `ZK0468` diagnostic.
pub fn unsafe_outside(span: Span, what: &str) -> Diagnostic
{
    Diagnostic::new(Code::UnsafeOutside, t!("codes.ZK0468.message", w = what), span)
        .with_help(t!("codes.ZK0468.help"))
}

/// A function marked `စိတ်မချရသော` is taken as a value.
///
/// # Arguments
///
/// * `span` - The value.
/// * `name` - The function.
///
/// # Returns
///
/// The `ZK0469` diagnostic.
pub fn unsafe_fn_value(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::UnsafeFnValue, t!("codes.ZK0469.message", n = name), span)
        .with_help(t!("codes.ZK0469.help"))
}

/// `အပိုင်အနေဖြင့်` stands on a parameter or result that is no pointer.
///
/// # Arguments
///
/// * `span` - The parameter or the function name.
/// * `kind` - The kind, as text.
///
/// # Returns
///
/// The `ZK0470` diagnostic.
pub fn owned_not_ptr(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(Code::OwnedNotPtr, t!("codes.ZK0470.message", k = kind), span)
}

/// Marked code where `--no-unsafe` refuses it.
///
/// # Arguments
///
/// * `span` - The mark.
///
/// # Returns
///
/// The `ZK0471` diagnostic.
pub fn unsafe_refused(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::UnsafeRefused, t!("codes.ZK0471.message"), span)
}

/// A value of a kind from outside the program is made, copied or printed.
///
/// # Arguments
///
/// * `span` - The value.
/// * `kind` - The kind name.
/// * `use_` - What the program does with it.
///
/// # Returns
///
/// The `ZK0473` diagnostic.
pub fn handle_in_zaka(span: Span, kind: &str, use_: HandleUse) -> Diagnostic
{
    let cue = match use_
    {
        HandleUse::Make => t!("codes.ZK0473.make"),
        HandleUse::Duplicate => t!("codes.ZK0473.duplicate"),
        HandleUse::Print => t!("codes.ZK0473.print"),
    };
    let message = t!("codes.ZK0473.message", k = kind, w = cue);
    Diagnostic::new(Code::HandleInZaka, message, span)
        .with_help(t!("codes.ZK0473.help"))
}

/// A field of the C struct a handle points at is written.
///
/// # Arguments
///
/// * `span` - The field.
/// * `kind` - The kind from outside.
///
/// # Returns
///
/// The `ZK0474` diagnostic.
pub fn c_field_write(span: Span, kind: &str) -> Diagnostic
{
    let message = t!("codes.ZK0474.message", k = kind);
    Diagnostic::new(Code::CFieldWrite, message, span).with_help(t!("codes.ZK0474.help"))
}

/// A test word that does not fit the kind: `ထဲတွင် ဒေတာ ရှိ` on a result, or
/// `ထဲတွင် ရလဒ်ကောင်း ရှိ` on an optional.
///
/// # Arguments
///
/// * `span` - The value tested.
/// * `kind` - Its kind.
/// * `optional` - Whether the value is an optional.
///
/// # Returns
///
/// The `ZK0475` diagnostic.
pub fn test_word_mismatch(span: Span, kind: &str, optional: bool) -> Diagnostic
{
    let msg = tern!(
        optional,
        t!("codes.ZK0475.optional", k = kind),
        t!("codes.ZK0475.result", k = kind)
    );
    Diagnostic::new(Code::TestWordMismatch, msg, span)
}

/// An error value where the place wants no result kind with an error side.
///
/// # Arguments
///
/// * `span` - The value.
///
/// # Returns
///
/// The `ZK0476` diagnostic.
pub fn fault_needs_result(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::FaultNeedsResult, t!("codes.ZK0476.message"), span)
        .with_help(t!("codes.ZK0476.help"))
}

/// `အလွတ်` where the place wants no optional kind.
///
/// # Arguments
///
/// * `span` - The value.
///
/// # Returns
///
/// The `ZK0477` diagnostic.
pub fn no_data_needs_optional(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::NoDataNeedsOptional, t!("codes.ZK0477.message"), span)
        .with_help(t!("codes.ZK0477.help"))
}

/// `ကနဦးတန်ဖိုး` where no kind can be worked out from its place.
///
/// # Arguments
///
/// * `span` - The value.
///
/// # Returns
///
/// The `ZK0478` diagnostic.
pub fn initial_needs_kind(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::InitialNeedsKind, t!("codes.ZK0478.message"), span)
        .with_help(t!("codes.ZK0478.help"))
}

/// The reason a kind has no initial value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoInitial<'a>
{
    /// A function value.
    Function,
    /// A handle, which C alone makes.
    Handle,
    /// An enumeration with no member marked; the enumeration name.
    Member(&'a str),
    /// A result, which must hold its good value or its failure.
    Result,
    /// A part inside the kind has none; the name of the part.
    Part(&'a str),
}

/// `ကနဦးတန်ဖိုး` for a kind that has none.
///
/// # Arguments
///
/// * `span` - The value.
/// * `kind` - The name of the kind.
/// * `why` - The reason the kind has none.
///
/// # Returns
///
/// The `ZK0479` diagnostic.
pub fn no_initial_value(span: Span, kind: &str, why: NoInitial<'_>) -> Diagnostic
{
    let reason = match why
    {
        NoInitial::Function => t!("codes.ZK0479.function"),
        NoInitial::Handle => t!("codes.ZK0479.handle"),
        NoInitial::Member(e) => t!("codes.ZK0479.member", e = e),
        NoInitial::Result => t!("codes.ZK0479.result"),
        NoInitial::Part(p) => t!("codes.ZK0479.part", p = p),
    };
    let msg = t!("codes.ZK0479.message", k = kind, r = reason);
    Diagnostic::new(Code::NoInitialValue, msg, span).with_help(t!("codes.ZK0479.help"))
}

/// A second member of an enumeration marked `ကနဦးတန်ဖိုး`.
///
/// # Arguments
///
/// * `span` - The second marked member.
/// * `owner` - The enumeration.
/// * `first` - The member marked first.
///
/// # Returns
///
/// The `ZK0480` diagnostic.
pub fn two_initial_members(span: Span, owner: &str, first: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0480.message", e = owner);
    let help = t!("codes.ZK0480.help", m = first);
    Diagnostic::new(Code::TwoInitialMembers, msg, span).with_help(help)
}

/// A kind no program can build, since its fields lead back to it.
///
/// # Arguments
///
/// * `span` - The declaration of the kind.
/// * `kind` - The kind.
/// * `chain` - The loop, as the kinds in order: `က → ခ → က`.
///
/// # Returns
///
/// The `ZK0481` diagnostic.
pub fn kind_never_built(span: Span, kind: &str, chain: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0481.message", k = kind, c = chain);
    Diagnostic::new(Code::KindNeverBuilt, msg, span).with_help(t!("codes.ZK0481.help"))
}

/// A struct literal that leaves out fields.
///
/// # Arguments
///
/// * `span` - The literal.
/// * `kind` - The struct.
/// * `fields` - The fields left out, each in quotes.
///
/// # Returns
///
/// The `ZK0482` diagnostic.
pub fn field_left_out(span: Span, kind: &str, fields: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0482.message", k = kind, f = fields);
    Diagnostic::new(Code::FieldLeftOut, msg, span).with_help(t!("codes.ZK0482.help"))
}

/// A global of a module declared with no value.
///
/// # Arguments
///
/// * `span` - The name.
/// * `name` - The global.
///
/// # Returns
///
/// The `ZK0483` diagnostic.
pub fn module_global_needs_value(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0483.message", n = name);
    Diagnostic::new(Code::ModuleGlobalNeedsValue, msg, span)
        .with_help(t!("codes.ZK0483.help"))
}

/// What a program does with a handle that C alone can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleUse
{
    /// A literal makes one.
    Make,
    /// A duplicate makes a second one.
    Duplicate,
    /// A print writes one.
    Print,
}

/// A C function with a pointer in its kinds, or a result that can be NULL,
/// is not marked `စိတ်မချရသော`, so it cannot be called from safe code.
///
/// # Arguments
///
/// * `span` - The name of the function.
/// * `name` - The function.
///
/// # Returns
///
/// The `ZK0472` diagnostic.
pub fn foreign_needs_mark(span: Span, name: &str) -> Diagnostic
{
    let message = t!("codes.ZK0472.message", n = name, w = t!("codes.ZK0472.pointer"));
    Diagnostic::new(Code::ForeignNeedsMark, message, span)
        .with_help(t!("codes.ZK0472.help"))
}

/// A row read, a row count or a row loop on a value that is no ဇယား.
///
/// # Arguments
///
/// * `span` - The value.
/// * `kind` - Its kind, as text.
///
/// # Returns
///
/// The `ZK0484` diagnostic.
pub fn not_a_table(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0484.message", k = kind);
    Diagnostic::new(Code::NotATable, msg, span).with_help(t!("codes.ZK0484.help"))
}

/// A ဇယား kind whose index kind holds a part that cannot be hashed and
/// compared as a whole value.
///
/// # Arguments
///
/// * `span` - The kind.
/// * `kind` - The index kind, as text.
/// * `part` - The part that fails, as text.
///
/// # Returns
///
/// The `ZK0485` diagnostic.
pub fn bad_index_kind(span: Span, kind: &str, part: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0485.message", k = kind, part = part);
    Diagnostic::new(Code::BadIndexKind, msg, span).with_help(t!("codes.ZK0485.help"))
}

/// `ဇယားအလွတ်` or `[:]` where no ဇယား kind can be worked out from its place.
///
/// # Arguments
///
/// * `span` - The blank ဇယား.
///
/// # Returns
///
/// The `ZK0486` diagnostic.
pub fn table_needs_kind(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0486.message");
    Diagnostic::new(Code::TableNeedsKind, msg, span).with_help(t!("codes.ZK0486.help"))
}

/// A ဇယား kind whose index kind holds a decimal number.
///
/// # Arguments
///
/// * `span` - The kind, or the literal it comes from.
/// * `kind` - The index kind, as text.
///
/// # Returns
///
/// The `ZK0487` warning.
pub fn decimal_index(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0487.message", k = kind);
    Diagnostic::new(Code::DecimalIndex, msg, span).with_help(t!("codes.ZK0487.help"))
}

/// `ကို ဖျက်မည်` after a value that is no row of a ဇယား.
///
/// # Arguments
///
/// * `span` - The value.
///
/// # Returns
///
/// The `ZK0488` diagnostic.
pub fn remove_needs_row(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0488.message");
    Diagnostic::new(Code::RemoveNeedsRow, msg, span).with_help(t!("codes.ZK0488.help"))
}
