use zakac_i18n::t;
use zakac_rust_sugars::tern;
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
    let diag = Diagnostic::new(
        Code::UndeclaredName,
        t!("codes.ZK0301.message", n = name),
        span,
    );
    // The implicit result variable of an earlier version is gone.
    tern!(name == "ရလဒ်", diag.with_help(t!("codes.ZK0301.help_result")), diag)
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

/// A function with a result can reach the end of its block with no return.
///
/// # Arguments
///
/// * `span` - The block.
/// * `name` - The function name.
///
/// # Returns
///
/// The `ZK0339` diagnostic.
pub fn end_without_return(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0339.message", n = name);
    Diagnostic::new(Code::EndWithoutReturn, msg, span)
        .with_help(t!("codes.ZK0339.help"))
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

/// A program has a second start block.
///
/// # Arguments
///
/// * `span` - The header word of the second block.
///
/// # Returns
///
/// The `ZK0325` diagnostic.
pub fn second_start(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0325.message");
    Diagnostic::new(Code::SecondStart, msg, span).with_help(t!("codes.ZK0325.help"))
}

/// A statement that would run stands at the top level of the file that is run.
///
/// # Arguments
///
/// * `span` - The statement.
///
/// # Returns
///
/// The `ZK0326` diagnostic.
pub fn top_level_stmt(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0326.message");
    Diagnostic::new(Code::TopLevelStmt, msg, span).with_help(t!("codes.ZK0326.help"))
}

/// The file that is run has no start block.
///
/// # Arguments
///
/// * `span` - The start of the file.
///
/// # Returns
///
/// The `ZK0327` diagnostic.
pub fn no_start(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0327.message");
    Diagnostic::new(Code::NoStart, msg, span).with_help(t!("codes.ZK0327.help"))
}

/// Two members of one enumeration share a name.
///
/// # Arguments
///
/// * `span` - The second member name.
/// * `owner` - The enumeration name.
/// * `name` - The member name.
///
/// # Returns
///
/// The `ZK0328` diagnostic.
pub fn duplicate_member(span: Span, owner: &str, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0328.message", e = owner, n = name);
    Diagnostic::new(Code::DuplicateMember, msg, span)
}

/// A member name that the enumeration does not have.
///
/// # Arguments
///
/// * `span` - The member name.
/// * `owner` - The enumeration name.
/// * `name` - The member name.
///
/// # Returns
///
/// The `ZK0329` diagnostic.
pub fn unknown_member(span: Span, owner: &str, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0329.message", e = owner, n = name);
    Diagnostic::new(Code::UnknownMember, msg, span)
}

/// The name before `ထဲမှ` is no enumeration written with `များ`.
///
/// # Arguments
///
/// * `span` - The name.
/// * `name` - The name as written.
/// * `fix` - The spelling with `များ`, when the name is an enumeration written
///   without it.
///
/// # Returns
///
/// The `ZK0330` diagnostic, with help when a fix is known.
pub fn unknown_enum(span: Span, name: &str, fix: Option<&str>) -> Diagnostic
{
    let diag =
        Diagnostic::new(Code::UnknownEnum, t!("codes.ZK0330.message", n = name), span);
    let Some(fix) = fix
    else
    {
        return diag;
    };
    diag.with_help(t!("codes.ZK0330.help", f = fix))
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

/// An argument of a preposition call carries a preposition other than its
/// parameter's.
///
/// # Arguments
///
/// * `span` - The preposition written.
/// * `name` - The function.
/// * `index` - The argument, counted from 1.
/// * `found` - The preposition written.
/// * `want` - The preposition of the parameter.
///
/// # Returns
///
/// The `ZK0331` diagnostic, with the right preposition as a hint.
pub fn wrong_preposition(
    span: Span, name: &str, index: usize, found: &str, want: &str,
) -> Diagnostic
{
    let msg = t!("codes.ZK0331.message", i = index, n = name, f = found, w = want);
    Diagnostic::new(Code::WrongPreposition, msg, span)
        .with_help(t!("codes.ZK0331.help", w = want))
}

/// No function of a name takes the arguments of a call.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function name.
/// * `list` - The parameters of each function of the name.
///
/// # Returns
///
/// The `ZK0332` diagnostic, with the parameters of each function as a hint.
pub fn no_overload_fits(span: Span, name: &str, list: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0332.message", n = name);
    Diagnostic::new(Code::NoOverloadFits, msg, span)
        .with_help(t!("codes.ZK0332.help", l = list))
}

/// More than one function of a name takes the arguments of a call.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function name.
///
/// # Returns
///
/// The `ZK0333` diagnostic.
pub fn overload_unclear(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0333.message", n = name);
    Diagnostic::new(Code::OverloadUnclear, msg, span).with_help(t!("codes.ZK0333.help"))
}

/// An external function is none of the functions the host serves.
///
/// # Arguments
///
/// * `span` - The name in the declaration.
/// * `name` - The name.
/// * `list` - The functions the host serves.
///
/// # Returns
///
/// The `ZK0334` diagnostic, with the served functions as a hint.
pub fn unknown_host_fn(span: Span, name: &str, list: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0334.message", n = name);
    Diagnostic::new(Code::UnknownHostFn, msg, span)
        .with_help(t!("codes.ZK0334.help", l = list))
}

/// A call names both a function and a variable of a function kind.
///
/// # Arguments
///
/// * `span` - The name at the call.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0335` diagnostic.
pub fn callee_twice(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::CalleeTwice, t!("codes.ZK0335.message", n = name), span)
        .with_help(t!("codes.ZK0335.help"))
}

/// A kind alias names itself, at once or through other aliases.
///
/// # Arguments
///
/// * `span` - The name in the alias declaration.
/// * `name` - The alias.
///
/// # Returns
///
/// The `ZK0336` diagnostic.
pub fn alias_loop(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::AliasLoop, t!("codes.ZK0336.message", n = name), span)
}

/// A C function has a name that is no C name.
///
/// # Arguments
///
/// * `span` - The name in the declaration.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0337` diagnostic.
pub fn not_a_c_name(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::NotACName, t!("codes.ZK0337.message", n = name), span)
        .with_help(t!("codes.ZK0337.help"))
}

/// The C name of a function that code outside can call is no C name.
///
/// # Arguments
///
/// * `span` - The C name.
/// * `name` - The C name.
///
/// # Returns
///
/// The `ZK0337` diagnostic.
pub fn not_a_c_export_name(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0337.message_export", n = name);
    Diagnostic::new(Code::NotACName, msg, span)
        .with_help(t!("codes.ZK0337.help_export"))
}

/// The C name of a struct a lib gives to C is no C name.
///
/// # Arguments
///
/// * `span` - The C name.
/// * `name` - The C name.
///
/// # Returns
///
/// The `ZK0337` diagnostic.
pub fn not_a_c_struct_name(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0337.message_struct", n = name);
    Diagnostic::new(Code::NotACName, msg, span)
        .with_help(t!("codes.ZK0337.help_export"))
}

/// A C function of a name declared before has other parameter or result
/// kinds.
///
/// # Arguments
///
/// * `span` - The name in the second declaration.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0338` diagnostic.
pub fn c_name_twice(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::CNameTwice, t!("codes.ZK0338.message", n = name), span)
        .with_help(t!("codes.ZK0338.help"))
}

/// A call or a kind gives a generic another number of kind arguments than it
/// declares.
///
/// # Arguments
///
/// * `span` - The call or the kind.
/// * `name` - The generic.
/// * `want` - The number of kind parameters it declares.
/// * `got` - The number of kind arguments written.
///
/// # Returns
///
/// The `ZK0340` diagnostic.
pub fn kind_arg_count(span: Span, name: &str, want: usize, got: usize) -> Diagnostic
{
    let msg = t!("codes.ZK0340.message", n = name, w = want, g = got);
    Diagnostic::new(Code::KindArgCount, msg, span).with_help(t!("codes.ZK0340.help"))
}

/// The kinds of a call of a generic function or a value of a generic struct
/// cannot be worked out.
///
/// # Arguments
///
/// * `span` - The call or the value.
/// * `name` - The generic function or struct.
///
/// # Returns
///
/// The `ZK0341` diagnostic.
pub fn kinds_unknown(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0341.message", n = name);
    Diagnostic::new(Code::KindsUnknown, msg, span)
        .with_help(t!("codes.ZK0341.help", n = name))
}

/// A call of a generic function asks for an instance past the limit of
/// instances, as when each instance calls the function with a deeper kind.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The generic function.
///
/// # Returns
///
/// The `ZK0342` diagnostic.
pub fn instances_endless(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0342.message", n = name);
    Diagnostic::new(Code::InstancesEndless, msg, span)
        .with_help(t!("codes.ZK0342.help"))
}

/// A kind given to a generic does not have a trait its kind parameter needs.
///
/// # Arguments
///
/// * `span` - The use of the generic.
/// * `kind` - The kind.
/// * `bound` - The trait.
/// * `missing` - The function of the trait the kind lacks, or blank for a
///   built-in trait.
///
/// # Returns
///
/// The `ZK0343` diagnostic.
pub fn bound_not_met(span: Span, kind: &str, bound: &str, missing: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0343.message", k = kind, t = bound);
    let help = tern!(
        missing.is_empty(),
        t!("codes.ZK0343.help_builtin", k = kind, t = bound),
        t!("codes.ZK0343.help", k = kind, f = missing)
    );
    Diagnostic::new(Code::BoundNotMet, msg, span).with_help(help)
}

/// A constraint or a trait block names no trait.
///
/// # Arguments
///
/// * `span` - The name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0344` diagnostic.
pub fn unknown_trait(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0344.message", n = name);
    Diagnostic::new(Code::UnknownTrait, msg, span)
        .with_help(t!("codes.ZK0344.help", n = name))
}

/// A trait block lacks a function of its trait.
///
/// # Arguments
///
/// * `span` - The head of the block.
/// * `trait_name` - The trait.
/// * `name` - The function.
///
/// # Returns
///
/// The `ZK0345` diagnostic.
pub fn trait_fn_missing(span: Span, trait_name: &str, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0345.message", t = trait_name, f = name);
    Diagnostic::new(Code::TraitFnMissing, msg, span).with_help(t!("codes.ZK0345.help"))
}

/// A trait block leaves out a function of its trait that has a default, and
/// does not take the defaults.
///
/// # Arguments
///
/// * `span` - The trait block.
/// * `trait_name` - The trait.
/// * `name` - The function.
///
/// # Returns
///
/// The `ZK0345` diagnostic, whose help names the line that takes defaults.
pub fn trait_fn_missing_default(span: Span, trait_name: &str, name: &str)
-> Diagnostic
{
    let msg = t!("codes.ZK0345.message", t = trait_name, f = name);
    Diagnostic::new(Code::TraitFnMissing, msg, span)
        .with_help(t!("codes.ZK0345.inherit", t = trait_name))
}

/// A trait block takes the defaults of its trait, and none is left for it
/// to take.
///
/// # Arguments
///
/// * `span` - The trait block.
/// * `trait_name` - The trait.
///
/// # Returns
///
/// The `ZK0351` diagnostic.
pub fn inherits_nothing(span: Span, trait_name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0351.message", t = trait_name);
    Diagnostic::new(Code::InheritsNothing, msg, span).with_help(t!("codes.ZK0351.help"))
}

/// A function of a trait block is not one of its trait, or has other
/// parameter or result kinds than the trait gives it.
///
/// # Arguments
///
/// * `span` - The name of the function.
/// * `trait_name` - The trait.
/// * `name` - The function.
///
/// # Returns
///
/// The `ZK0346` diagnostic.
pub fn trait_fn_foreign(span: Span, trait_name: &str, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0346.message", t = trait_name, f = name);
    Diagnostic::new(Code::TraitFnForeign, msg, span).with_help(t!("codes.ZK0346.help"))
}

/// Two functions give a kind the same function of a trait.
///
/// # Arguments
///
/// * `span` - The use that needs the trait.
/// * `kind` - The kind.
/// * `name` - The function.
///
/// # Returns
///
/// The `ZK0347` diagnostic.
pub fn trait_fn_twice(span: Span, kind: &str, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0347.message", k = kind, f = name);
    Diagnostic::new(Code::TraitFnTwice, msg, span).with_help(t!("codes.ZK0347.help"))
}

/// A trait block stands in a file that declares neither its kind nor its
/// trait.
///
/// # Arguments
///
/// * `span` - The head of the block.
/// * `kind` - The kind.
/// * `trait_name` - The trait.
///
/// # Returns
///
/// The `ZK0348` diagnostic.
pub fn trait_block_place(span: Span, kind: &str, trait_name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0348.message", k = kind, t = trait_name);
    Diagnostic::new(Code::TraitBlockPlace, msg, span).with_help(t!("codes.ZK0348.help"))
}

/// The mark of a declaration the compiler gives,
/// `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော`, stands outside the modules of the
/// stdlib.
///
/// # Arguments
///
/// * `span` - The declaration.
///
/// # Returns
///
/// The `ZK0349` diagnostic.
pub fn builtin_outside_core(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0349.message");
    Diagnostic::new(Code::BuiltinOutsideCore, msg, span)
        .with_help(t!("codes.ZK0349.help"))
}

/// A function of a file has the name, the kinds and the prepositions of a
/// function the compiler gives in the core, so it takes each sentence of
/// that function in the file.
///
/// # Arguments
///
/// * `span` - The name of the function.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0350` diagnostic.
pub fn same_as_core(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0350.message", n = name);
    Diagnostic::new(Code::SameAsCore, msg, span).with_help(t!("codes.ZK0350.help"))
}

/// A call written as a sentence gives other phrases than the head of its
/// function: another starting or ending phrase, or none where the head has
/// one.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function name.
/// * `want` - The call as the head spells it, the arguments as `…`.
///
/// # Returns
///
/// The `ZK0352` diagnostic.
pub fn wrong_phrase(span: Span, name: &str, want: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0352.message", n = name);
    Diagnostic::new(Code::WrongPhrase, msg, span)
        .with_help(t!("codes.ZK0352.help", w = want))
}

/// An enumeration gives its members values and its variants data.
///
/// # Arguments
///
/// * `span` - The first member with a value.
/// * `owner` - The enumeration name.
///
/// # Returns
///
/// The `ZK0353` diagnostic.
pub fn value_and_data(span: Span, owner: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0353.message", e = owner);
    Diagnostic::new(Code::ValueAndData, msg, span).with_help(t!("codes.ZK0353.help"))
}

/// A variant that holds data is written with none.
///
/// # Arguments
///
/// * `span` - The variant.
/// * `owner` - The enumeration name.
/// * `variant` - The variant name.
/// * `kind` - The kind of its data.
/// * `written` - The variant as written, `<stem>များထဲမှ <variant>`.
///
/// # Returns
///
/// The `ZK0354` diagnostic.
pub fn variant_needs_data(
    span: Span, owner: &str, variant: &str, kind: &str, written: &str,
) -> Diagnostic
{
    let msg = t!("codes.ZK0354.message", v = variant, e = owner, k = kind);
    let help = t!("codes.ZK0354.help", w = written);
    Diagnostic::new(Code::VariantNeedsData, msg, span).with_help(help)
}

/// Data is given to, read from or named for a variant that holds none.
///
/// # Arguments
///
/// * `span` - The data, the read or the name.
/// * `owner` - The enumeration name.
/// * `variant` - The variant name.
/// * `written` - The variant as written with no data.
///
/// # Returns
///
/// The `ZK0355` diagnostic.
pub fn variant_has_no_data(
    span: Span, owner: &str, variant: &str, written: &str,
) -> Diagnostic
{
    let msg = t!("codes.ZK0355.message", v = variant, e = owner);
    let help = t!("codes.ZK0355.help", w = written);
    Diagnostic::new(Code::VariantHasNoData, msg, span).with_help(help)
}

/// A test or a read of a variant looks at a value that is no enumeration.
///
/// # Arguments
///
/// * `span` - The value.
/// * `kind` - Its kind.
///
/// # Returns
///
/// The `ZK0356` diagnostic.
pub fn not_an_enum_value(span: Span, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0356.message", k = kind);
    Diagnostic::new(Code::NotAnEnumValue, msg, span).with_help(t!("codes.ZK0356.help"))
}

/// A name for the data of a variant stands outside the condition of a
/// conditional or an arm of a switch.
///
/// # Arguments
///
/// * `span` - The name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0357` diagnostic.
pub fn name_outside_test(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0357.message", n = name);
    Diagnostic::new(Code::NameOutsideTest, msg, span).with_help(t!("codes.ZK0357.help"))
}

/// A read of a variant that a test ruled out.
///
/// # Arguments
///
/// * `span` - The read.
/// * `name` - The variable.
/// * `proven` - The variant the test proved.
/// * `read` - The variant read.
///
/// # Returns
///
/// The `ZK0358` diagnostic.
pub fn wrong_variant_proven(
    span: Span, name: &str, proven: &str, read: &str,
) -> Diagnostic
{
    let msg = t!("codes.ZK0358.message", n = name, p = proven, v = read);
    let help = t!("codes.ZK0358.help", p = proven);
    Diagnostic::new(Code::WrongVariantProven, msg, span).with_help(help)
}

/// A case of a switch over an enumeration whose variants hold data gives
/// the data too.
///
/// # Arguments
///
/// * `span` - The case.
/// * `owner` - The enumeration name.
///
/// # Returns
///
/// The `ZK0359` diagnostic.
pub fn case_with_data(span: Span, owner: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0359.message", e = owner);
    Diagnostic::new(Code::CaseWithData, msg, span).with_help(t!("codes.ZK0359.help"))
}

/// The reason a trait cannot be a kind, `<trait> အရာ`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAKind
{
    /// The compiler gives the trait; it has no functions to run.
    Marked,
    /// A function does not take the trait's kind first.
    First,
    /// A function takes the trait's kind in another parameter too.
    Elsewhere,
    /// A function gives the trait's kind back inside another kind.
    Result,
    /// A function has kind parameters of its own.
    Generic,
}

/// A trait used as a kind has a function a trait value cannot run.
///
/// # Arguments
///
/// * `span` - The trait where it is used as a kind.
/// * `name` - The trait.
/// * `cause` - The reason.
/// * `func` - The function of the trait the reason is about.
///
/// # Returns
///
/// The `ZK0360` diagnostic.
pub fn trait_not_a_kind(
    span: Span, name: &str, cause: NotAKind, func: &str,
) -> Diagnostic
{
    let reason = match cause
    {
        NotAKind::Marked => t!("codes.ZK0360.marked"),
        NotAKind::First => t!("codes.ZK0360.first", f = func),
        NotAKind::Elsewhere => t!("codes.ZK0360.elsewhere", f = func),
        NotAKind::Result => t!("codes.ZK0360.result", f = func),
        NotAKind::Generic => t!("codes.ZK0360.generic", f = func),
    };
    let msg = t!("codes.ZK0360.message", t = name, w = reason);
    Diagnostic::new(Code::TraitNotAKind, msg, span).with_help(t!("codes.ZK0360.help"))
}

/// What a program does with a trait value that trait values do not have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitValueUse
{
    /// `==`, `!=` or `တစ်ခုတည်း`.
    Compare,
    /// A print.
    Print,
    /// `ကနဦးတန်ဖိုး`.
    Initial,
    /// A call through it as a function value.
    Call,
}

/// A trait value is used as trait values are not.
///
/// # Arguments
///
/// * `span` - The use.
/// * `use_` - What the use does.
///
/// # Returns
///
/// The `ZK0361` diagnostic.
pub fn trait_value_use(span: Span, use_: TraitValueUse) -> Diagnostic
{
    let what = match use_
    {
        TraitValueUse::Compare => t!("codes.ZK0361.compare"),
        TraitValueUse::Print => t!("codes.ZK0361.print"),
        TraitValueUse::Initial => t!("codes.ZK0361.initial"),
        TraitValueUse::Call => t!("codes.ZK0361.call"),
    };
    let msg = t!("codes.ZK0361.message", w = what);
    Diagnostic::new(Code::TraitValueUse, msg, span).with_help(t!("codes.ZK0361.help"))
}

/// A plain value goes into a trait value whose trait has a function that
/// changes its first parameter, so that function changes the clone alone.
///
/// # Arguments
///
/// * `span` - The value that goes in.
/// * `kind` - Its kind.
/// * `func` - The function that changes its first parameter.
///
/// # Returns
///
/// The `ZK0362` diagnostic.
pub fn trait_value_clones(span: Span, kind: &str, func: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0362.message", k = kind, f = func);
    Diagnostic::new(Code::TraitValueClones, msg, span)
        .with_help(t!("codes.ZK0362.help", k = kind))
}
