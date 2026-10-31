use zakac_i18n::t;
use zakac_span::span::Span;

use crate::code::Code;
use crate::diagnostic::Diagnostic;

/// A name is used without a declaration in scope.
///
/// # Arguments
///
/// * `span` - The name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0301` diagnostic.
pub fn undeclared_name(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::UndeclaredName, t!("codes.ZK0301.message", n = name), span)
}

/// A name is declared twice in one scope.
///
/// # Arguments
///
/// * `span` - The second declared name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0302` diagnostic, with a hint.
pub fn redeclaration(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::Redeclaration, t!("codes.ZK0302.message", n = name), span)
        .with_help(t!("codes.ZK0302.help"))
}

/// A call names a function that is not declared.
///
/// # Arguments
///
/// * `span` - The function name in the call.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0303` diagnostic.
pub fn undeclared_function(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(
        Code::UndeclaredFunction,
        t!("codes.ZK0303.message", n = name),
        span,
    )
}

/// Two functions share a name.
///
/// # Arguments
///
/// * `span` - The name in the second declaration.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0304` diagnostic.
pub fn duplicate_function(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::DuplicateFunction, t!("codes.ZK0304.message", n = name), span)
}

/// Two parameters of one function share a name.
///
/// # Arguments
///
/// * `span` - The second parameter name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0305` diagnostic.
pub fn duplicate_param(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::DuplicateParam, t!("codes.ZK0305.message", n = name), span)
}

/// A constant is assigned.
///
/// # Arguments
///
/// * `span` - The assigned name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0306` diagnostic.
pub fn assign_to_const(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::AssignToConst, t!("codes.ZK0306.message", n = name), span)
}

/// A counted-loop variable is assigned.
///
/// # Arguments
///
/// * `span` - The assigned name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0307` diagnostic.
pub fn assign_to_loop_var(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::AssignToLoopVar, t!("codes.ZK0307.message", n = name), span)
}

/// A constant is declared without an initial value.
///
/// # Arguments
///
/// * `span` - The declaration.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0308` diagnostic.
pub fn const_without_init(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::ConstWithoutInit, t!("codes.ZK0308.message", n = name), span)
}

/// A break stands outside a loop.
///
/// # Arguments
///
/// * `span` - The statement.
///
/// # Returns
///
/// The `ZK0309` diagnostic.
pub fn break_outside_loop(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::BreakOutsideLoop, t!("codes.ZK0309.message"), span)
}

/// A continue stands outside a loop.
///
/// # Arguments
///
/// * `span` - The statement.
///
/// # Returns
///
/// The `ZK0310` diagnostic.
pub fn continue_outside_loop(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::ContinueOutsideLoop, t!("codes.ZK0310.message"), span)
}

/// A return stands outside a function.
///
/// # Arguments
///
/// * `span` - The statement.
///
/// # Returns
///
/// The `ZK0311` diagnostic.
pub fn return_outside_function(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::ReturnOutsideFunction, t!("codes.ZK0311.message"), span)
}

/// A function without a result returns a value.
///
/// # Arguments
///
/// * `span` - The return statement.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0312` diagnostic.
pub fn value_returned_from_void(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(
        Code::ValueReturnedFromVoid,
        t!("codes.ZK0312.message", n = name),
        span,
    )
}

/// A function with a result returns no value.
///
/// # Arguments
///
/// * `span` - The return statement.
/// * `name` - The function name.
/// * `kind` - The result kind, as text.
///
/// # Returns
///
/// The `ZK0313` diagnostic.
pub fn missing_return_value(span: Span, name: &str, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0313.message", n = name, k = kind);
    Diagnostic::new(Code::MissingReturnValue, msg, span)
}

/// A call passes the wrong number of arguments.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function name.
/// * `want` - The number of parameters.
/// * `got` - The number of arguments.
///
/// # Returns
///
/// The `ZK0314` diagnostic.
pub fn arg_count_mismatch(span: Span, name: &str, want: usize, got: usize)
-> Diagnostic
{
    let (want, got) = (want.to_string(), got.to_string());
    let msg = t!("codes.ZK0314.message", n = name, want = want, got = got);
    Diagnostic::new(Code::ArgCountMismatch, msg, span)
}

/// A function without a result is used as a value.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0315` diagnostic.
pub fn value_from_void_function(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(
        Code::ValueFromVoidFunction,
        t!("codes.ZK0315.message", n = name),
        span,
    )
}

/// Two structs share a name.
///
/// # Arguments
///
/// * `span` - The name in the second declaration.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0316` diagnostic.
pub fn duplicate_struct(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::DuplicateStruct, t!("codes.ZK0316.message", n = name), span)
}

/// Two fields of one struct share a name.
///
/// # Arguments
///
/// * `span` - The second field name.
/// * `owner` - The struct name.
/// * `name` - The field name.
///
/// # Returns
///
/// The `ZK0317` diagnostic.
pub fn duplicate_field(span: Span, owner: &str, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0317.message", s = owner, n = name);
    Diagnostic::new(Code::DuplicateField, msg, span)
}

/// A kind names a struct that is not declared.
///
/// # Arguments
///
/// * `span` - The name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0318` diagnostic.
pub fn unknown_kind_name(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::UnknownKindName, t!("codes.ZK0318.message", n = name), span)
}

/// A variable, parameter, field or function takes the name of a struct.
///
/// # Arguments
///
/// * `span` - The name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0319` diagnostic.
pub fn struct_name_clash(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::StructNameClash, t!("codes.ZK0319.message", n = name), span)
}

/// A field access names a field the struct does not have.
///
/// # Arguments
///
/// * `span` - The field name.
/// * `owner` - The struct name.
/// * `name` - The field name.
///
/// # Returns
///
/// The `ZK0320` diagnostic.
pub fn unknown_field(span: Span, owner: &str, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0320.message", s = owner, n = name);
    Diagnostic::new(Code::UnknownField, msg, span)
}

/// A struct literal sets one field twice.
///
/// # Arguments
///
/// * `span` - The second field name.
/// * `owner` - The struct name.
/// * `name` - The field name.
///
/// # Returns
///
/// The `ZK0321` diagnostic.
pub fn field_set_twice(span: Span, owner: &str, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0321.message", s = owner, n = name);
    Diagnostic::new(Code::FieldSetTwice, msg, span)
}

/// An import names something the imported file does not export.
///
/// # Arguments
///
/// * `span` - The imported name.
/// * `file` - The imported file, as written.
/// * `name` - The name.
/// * `private` - Whether the file declares the name without exporting it.
///
/// # Returns
///
/// The `ZK0322` diagnostic, with help when the name is private.
pub fn import_not_found(span: Span, file: &str, name: &str, private: bool)
-> Diagnostic
{
    let msg = t!("codes.ZK0322.message", f = file, n = name);
    let diag = Diagnostic::new(Code::ImportNotFound, msg, span);
    match private
    {
        true => diag.with_help(t!("codes.ZK0322.help_private", n = name)),
        false => diag,
    }
}

/// The help for a name that another file declares but this file cannot
/// see.
///
/// # Arguments
///
/// * `name` - The name.
/// * `file` - The file that declares it.
///
/// # Returns
///
/// The help text.
pub fn hidden_help(name: &str, file: &str) -> String
{
    t!("codes.ZK0322.help_hidden", n = name, f = file).to_string()
}

/// An imported file holds a statement that would run.
///
/// # Arguments
///
/// * `span` - The statement.
/// * `file` - The imported file.
///
/// # Returns
///
/// The `ZK0323` diagnostic.
pub fn stmt_in_imported_file(span: Span, file: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0323.message", f = file);
    Diagnostic::new(Code::StmtInImportedFile, msg, span)
}

/// An exported declaration names a private struct of its file in a kind.
///
/// # Arguments
///
/// * `span` - The kind.
/// * `name` - The exported declaration.
/// * `private` - The private struct.
///
/// # Returns
///
/// The `ZK0324` diagnostic, with help to export the struct.
pub fn private_in_export(span: Span, name: &str, private: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0324.message", n = name, s = private);
    let help = t!("codes.ZK0324.help", n = name, s = private);
    Diagnostic::new(Code::PrivateInExport, msg, span).with_help(help)
}
