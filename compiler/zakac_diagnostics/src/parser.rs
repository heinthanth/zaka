use zakac_i18n::t;
use zakac_span::span::Span;

use crate::code::Code;
use crate::diagnostic::Diagnostic;

/// Add the hint for a name written attached to a particle.
///
/// # Arguments
///
/// * `diag` - The diagnostic to extend.
/// * `stem` - The name part of the word.
/// * `particle` - The particle at the end of the word.
///
/// # Returns
///
/// The diagnostic with the hint.
pub fn with_split_hint(diag: Diagnostic, stem: &str, particle: &str) -> Diagnostic
{
    diag.with_help(t!("parser.help_split", s = stem, p = particle))
}

/// A token does not fit the grammar at this point.
///
/// # Arguments
///
/// * `span` - The token found.
/// * `expected` - What the grammar allows here.
/// * `found` - The token found.
///
/// # Returns
///
/// The `ZK0201` diagnostic.
pub fn expected_token(span: Span, expected: &str, found: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0201.message", e = expected, f = found);
    Diagnostic::new(Code::ExpectedToken, msg, span)
}

/// An operand is missing.
///
/// # Arguments
///
/// * `span` - The token found in its place.
/// * `found` - That token.
///
/// # Returns
///
/// The `ZK0202` diagnostic.
pub fn expected_operand(span: Span, found: &str) -> Diagnostic
{
    Diagnostic::new(Code::ExpectedOperand, t!("codes.ZK0202.message", f = found), span)
}

/// A statement does not end in a statement verb.
///
/// # Arguments
///
/// * `span` - The token found after the clause.
/// * `found` - That token.
///
/// # Returns
///
/// The `ZK0203` diagnostic, with the list of statement endings.
pub fn expected_stmt_verb(span: Span, found: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0203.message", f = found);
    Diagnostic::new(Code::ExpectedStmtVerb, msg, span)
        .with_help(t!("codes.ZK0203.help"))
}

/// A statement verb appears inside an expression.
///
/// # Arguments
///
/// * `span` - The verb.
/// * `verb` - The verb text.
///
/// # Returns
///
/// The `ZK0204` diagnostic.
pub fn stmt_verb_in_expr(span: Span, verb: &str) -> Diagnostic
{
    Diagnostic::new(Code::StmtVerbInExpr, t!("codes.ZK0204.message", v = verb), span)
}

/// A verb has fewer operands than it takes.
///
/// # Arguments
///
/// * `span` - The verb.
/// * `verb` - The verb text.
/// * `need` - Operands the verb takes.
/// * `found` - Operands available.
///
/// # Returns
///
/// The `ZK0205` diagnostic.
pub fn missing_operand(span: Span, verb: &str, need: usize, found: usize)
-> Diagnostic
{
    let (n, m) = (need.to_string(), found.to_string());
    let msg = t!("codes.ZK0205.message", v = verb, n = n, m = m);
    Diagnostic::new(Code::MissingOperand, msg, span)
}

/// An operand carries the wrong particle for its verb.
///
/// # Arguments
///
/// * `span` - The operand.
/// * `verb` - The verb text.
/// * `index` - One-based position of the operand.
/// * `expected` - The particle the verb wants.
/// * `found` - The particle found, or the words for no particle.
///
/// # Returns
///
/// The `ZK0206` diagnostic.
pub fn slot_mismatch(
    span: Span, verb: &str, index: usize, expected: &str, found: &str,
) -> Diagnostic
{
    let i = index.to_string();
    let msg = t!("codes.ZK0206.message", v = verb, i = i, e = expected, f = found);
    Diagnostic::new(Code::SlotMismatch, msg, span)
}

/// An operand is followed with two particles.
///
/// # Arguments
///
/// * `span` - The second particle.
/// * `first` - The first particle text.
///
/// # Returns
///
/// The `ZK0207` diagnostic.
pub fn double_particle(span: Span, first: &str) -> Diagnostic
{
    Diagnostic::new(Code::DoubleParticle, t!("codes.ZK0207.message", p = first), span)
}

/// Operands wait for a verb at the end of a clause.
///
/// # Arguments
///
/// * `span` - The clause.
/// * `waiting` - The waiting operands with their particles.
///
/// # Returns
///
/// The `ZK0208` diagnostic.
pub fn unclosed_clause(span: Span, waiting: &str) -> Diagnostic
{
    Diagnostic::new(Code::UnclosedClause, t!("codes.ZK0208.message", w = waiting), span)
}

/// Operands are left over before a statement verb.
///
/// # Arguments
///
/// * `span` - The left-over operands.
/// * `verb` - The statement verb text.
/// * `waiting` - The operands with their particles.
///
/// # Returns
///
/// The `ZK0209` diagnostic.
pub fn dangling_operands(span: Span, verb: &str, waiting: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0209.message", v = verb, w = waiting);
    Diagnostic::new(Code::DanglingOperands, msg, span)
}

/// A comparison is the direct operand of another comparison.
///
/// # Arguments
///
/// * `span` - The outer comparison.
///
/// # Returns
///
/// The `ZK0210` diagnostic, with a hint about `ဖြစ်၍`.
pub fn chained_comparison(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::ChainedComparison, t!("codes.ZK0210.message"), span)
        .with_help(t!("codes.ZK0210.help"))
}

/// An else or else-if word starts a statement.
///
/// # Arguments
///
/// * `span` - The word.
/// * `word` - The word text.
///
/// # Returns
///
/// The `ZK0211` diagnostic.
pub fn else_without_if(span: Span, word: &str) -> Diagnostic
{
    Diagnostic::new(Code::ElseWithoutIf, t!("codes.ZK0211.message", w = word), span)
}

/// A generic loop update stands outside a generic loop.
///
/// # Arguments
///
/// * `span` - The `ပြီးလျှင်` word.
///
/// # Returns
///
/// The `ZK0212` diagnostic.
pub fn update_outside_loop(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::UpdateOutsideLoop, t!("codes.ZK0212.message"), span)
}

/// A generic loop update is not the last statement of its loop.
///
/// # Arguments
///
/// * `span` - The update statement.
///
/// # Returns
///
/// The `ZK0213` diagnostic.
pub fn update_not_last(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::UpdateNotLast, t!("codes.ZK0213.message"), span)
}

/// A function is declared inside a block.
///
/// # Arguments
///
/// * `span` - The function header.
///
/// # Returns
///
/// The `ZK0214` diagnostic.
pub fn nested_function(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::NestedFunction, t!("codes.ZK0214.message"), span)
}

/// A struct is declared inside a block.
///
/// # Arguments
///
/// * `span` - The declaration.
///
/// # Returns
///
/// The `ZK0228` diagnostic.
pub fn nested_struct(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::NestedStruct, t!("codes.ZK0228.message"), span)
}

/// A single-statement block is a conditional.
///
/// # Arguments
///
/// * `span` - The inner `အကယ်၍`.
///
/// # Returns
///
/// The `ZK0215` diagnostic.
pub fn nested_single_line_if(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::NestedSingleLineIf, t!("codes.ZK0215.message"), span)
}

/// A kind has a bit count it cannot have.
///
/// # Arguments
///
/// * `span` - The bit count.
/// * `count` - The bit count text.
/// * `allowed` - The allowed counts.
///
/// # Returns
///
/// The `ZK0216` diagnostic.
pub fn invalid_bit_count(span: Span, count: &str, allowed: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0216.message", n = count, a = allowed);
    Diagnostic::new(Code::InvalidBitCount, msg, span)
}

/// A sign word stands before a kind other than an integer.
///
/// # Arguments
///
/// * `span` - The kind phrase.
///
/// # Returns
///
/// The `ZK0217` diagnostic.
pub fn signed_float(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::SignedFloat, t!("codes.ZK0217.message"), span)
}

/// A base prefix is not an integer from 2 to 36.
///
/// # Arguments
///
/// * `span` - The base number.
/// * `base` - Its text.
///
/// # Returns
///
/// The `ZK0218` diagnostic.
pub fn invalid_base(span: Span, base: &str) -> Diagnostic
{
    Diagnostic::new(Code::InvalidBase, t!("codes.ZK0218.message", b = base), span)
}

/// A digit is not valid in the base of its literal.
///
/// # Arguments
///
/// * `span` - The literal digits.
/// * `digit` - The first invalid digit.
/// * `base` - The base.
///
/// # Returns
///
/// The `ZK0219` diagnostic.
pub fn invalid_digit(span: Span, digit: char, base: u32) -> Diagnostic
{
    let (d, b) = (digit.to_string(), base.to_string());
    Diagnostic::new(Code::InvalidDigit, t!("codes.ZK0219.message", d = d, b = b), span)
}

/// An integer literal does not fit its kind.
///
/// # Arguments
///
/// * `span` - The literal.
/// * `text` - The literal digits.
///
/// # Returns
///
/// The `ZK0220` diagnostic.
pub fn literal_out_of_range(span: Span, text: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0220.message", l = text);
    Diagnostic::new(Code::LiteralOutOfRange, msg, span)
}

/// An integer kind prefix stands before a literal with a fraction.
///
/// # Arguments
///
/// * `span` - The literal.
///
/// # Returns
///
/// The `ZK0221` diagnostic.
pub fn float_for_int_kind(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::FloatForIntKind, t!("codes.ZK0221.message"), span)
}

/// A character or boolean kind prefixes a number.
///
/// # Arguments
///
/// * `span` - The kind phrase.
///
/// # Returns
///
/// The `ZK0222` diagnostic.
pub fn bad_literal_kind(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::BadLiteralKind, t!("codes.ZK0222.message"), span)
}

/// A generic loop is declared with the constant word.
///
/// # Arguments
///
/// * `span` - The `ကိန်းသေ` word.
///
/// # Returns
///
/// The `ZK0225` diagnostic.
pub fn const_loop(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::ConstLoop, t!("codes.ZK0225.message"), span)
}

/// A block closer has no open block.
///
/// # Arguments
///
/// * `span` - The closer.
///
/// # Returns
///
/// The `ZK0226` diagnostic.
pub fn unmatched_block_close(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::UnmatchedBlockClose, t!("codes.ZK0226.message"), span)
}

/// The cell count of a list kind is not a whole number above zero.
///
/// # Arguments
///
/// * `span` - The count.
/// * `count` - The count as written.
///
/// # Returns
///
/// The `ZK0227` diagnostic.
pub fn bad_list_count(span: Span, count: &str) -> Diagnostic
{
    Diagnostic::new(Code::BadListCount, t!("codes.ZK0227.message", n = count), span)
}

/// A count prefix pairs with the `စာရင်း` of a slice kind.
///
/// # Arguments
///
/// * `span` - The kind.
///
/// # Returns
///
/// The `ZK0233` diagnostic.
pub fn slice_count(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::SliceCount, t!("codes.ZK0233.message"), span)
}

/// The export mark stands before something other than a top-level
/// declaration.
///
/// # Arguments
///
/// * `span` - The export mark.
///
/// # Returns
///
/// The `ZK0229` diagnostic.
pub fn export_not_on_decl(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::ExportNotOnDecl, t!("codes.ZK0229.message"), span)
}

/// An import comes after a declaration or statement, or inside a block.
///
/// # Arguments
///
/// * `span` - The import.
///
/// # Returns
///
/// The `ZK0230` diagnostic, with help to move it up.
pub fn import_not_first(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0230.message");
    Diagnostic::new(Code::ImportNotFirst, msg, span).with_help(t!("codes.ZK0230.help"))
}

/// A file imports itself through a chain of imports.
///
/// # Arguments
///
/// * `span` - The import that closes the circle.
/// * `chain` - The file names along the circle, joined with arrows.
///
/// # Returns
///
/// The `ZK0231` diagnostic.
pub fn circular_import(span: Span, chain: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0231.message", chain = chain);
    Diagnostic::new(Code::CircularImport, msg, span)
}

/// A program that was not read from a file has an import.
///
/// # Arguments
///
/// * `span` - The import.
/// * `file` - The imported file, as written.
///
/// # Returns
///
/// The `ZK0232` diagnostic.
pub fn import_needs_file(span: Span, file: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0232.message", f = file);
    Diagnostic::new(Code::ImportNeedsFile, msg, span)
}
