use std::error::Error as StdError;

use clap::Command;
use clap::error::ContextKind;
use clap::error::ContextValue;
use clap::error::ErrorKind;
use termcolor::WriteColor;
use zakac_i18n::t;
use zakac_reporter::writer::write_line;
use zakac_rust_sugars::tern;

use crate::clap_utils::CommandInfo;
use crate::clap_utils::create_version_string;
use crate::command::ExitCode;

/// Handle an error from the command line argument parser.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd` - The command to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `stdout` - The stdout writer.
/// * `stderr` - The stderr writer.
///
/// # Returns
///
/// The exit code to return.
pub(crate) fn handle_args_error<W, E>(
    error: clap::Error, cmd: &Command, cmd_info: CommandInfo, stdout: &mut W,
    stderr: &mut E,
) -> ExitCode
where
    W: WriteColor + ?Sized,
    E: WriteColor + ?Sized,
{
    match error.kind()
    {
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand =>
        {
            print_help(error, stdout)
        }
        ErrorKind::DisplayVersion => print_version(&cmd_info, stdout),
        ErrorKind::InvalidValue => invalid_value(error, &cmd_info, stderr),
        ErrorKind::UnknownArgument => unknown_argument(error, &cmd_info, stderr),
        ErrorKind::InvalidSubcommand => unknown_subcommand(error, &cmd_info, stderr),
        ErrorKind::ArgumentConflict => arg_conflict(error, &cmd_info, stderr),
        ErrorKind::MissingSubcommand => missing_subcommand(&cmd, stderr),
        ErrorKind::NoEquals => no_equals(error, &cmd_info, stderr),
        ErrorKind::ValueValidation => value_validation(error, &cmd_info, stderr),
        ErrorKind::TooManyValues => more_val_count(error, &cmd_info, stderr),
        ErrorKind::TooFewValues => less_val_count(error, &cmd_info, stderr),
        ErrorKind::WrongNumberOfValues => wrong_val_count(error, &cmd_info, stderr),
        ErrorKind::MissingRequiredArgument => missing_arg(error, &cmd_info, stderr),
        _ => generic(error, &cmd_info, stderr),
    }
}

/// Write a localized CLI usage error to a writer.
///
/// # Arguments
///
/// * `writer` - The writer to write the error to.
/// * `cmd_info` - The command or subcommand information.
/// * `msg` - The error message.
///
/// # Returns
///
/// `InvalidUsage` on success, `IoError` otherwise.
fn write_cli_error<W>(
    writer: &mut W, cmd_info: &CommandInfo, msg: impl AsRef<str>,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let msg = t!("cli_errors.template", name = cmd_info.name, msg = msg.as_ref());
    let result = write_line(writer, &msg);
    tern!(result.is_ok(), ExitCode::InvalidUsage, ExitCode::IoError)
}

/// Get the context string from an error.
///
/// # Arguments
///
/// * `error` - The error to get the context string from.
/// * `kind` - The context kind to get the string from.
///
/// # Returns
/// The context string.
fn context_string(error: &clap::Error, kind: ContextKind) -> Option<String>
{
    error.get(kind).map(|value| value.to_string())
}

/// Handle an invalid value error from the command line argument parser.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn invalid_value<W>(
    error: clap::Error, cmd_info: &CommandInfo, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let Some(arg) = context_string(&error, ContextKind::InvalidArg)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let Some(value) = context_string(&error, ContextKind::InvalidValue)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let Some(choices) = context_string(&error, ContextKind::ValidValue)
    else
    {
        return generic(error, cmd_info, writer);
    };

    let v = value.escape_debug().to_string();
    let c = choices.escape_debug();
    let message = t!("cli_errors.invalid_value", arg = arg, value = v, choices = c);
    write_cli_error(writer, cmd_info, message)
}

/// Handle an unknown argument error from the command line argument parser.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn unknown_argument<W>(
    error: clap::Error, cmd_info: &CommandInfo, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let Some(arg) = context_string(&error, ContextKind::InvalidArg)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let message = t!("cli_errors.unknown_argument", arg = arg);
    write_cli_error(writer, cmd_info, message)
}

/// Handle an unknown subcommand error from the command line argument parser.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn unknown_subcommand<W>(
    error: clap::Error, cmd_info: &CommandInfo, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let Some(subcommand) = context_string(&error, ContextKind::InvalidSubcommand)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let message = t!("cli_errors.unknown_subcommand", subcommand = subcommand);
    write_cli_error(writer, cmd_info, message)
}

/// Handle an argument conflict error from the command line argument parser.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn arg_conflict<W>(
    error: clap::Error, cmd_info: &CommandInfo, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let Some(arg) = error
        .get(ContextKind::InvalidArg)
        .or_else(|| error.get(ContextKind::InvalidSubcommand))
        .map(|value| value.to_string())
    else
    {
        return generic(error, cmd_info, writer);
    };
    let prior = error.get(ContextKind::PriorArg).map(|v| v.to_string());
    let twice = t!("cli_errors.arg_no_multiple_times", arg = arg);
    let conflict = |c: String| t!("cli_errors.arg_conflict", arg = arg, conflict = c);
    let message = prior.filter(|c| c != &arg).map_or(twice, conflict);

    write_cli_error(writer, cmd_info, message)
}

/// Handle a missing equals error from the command line argument parser.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn no_equals<W>(error: clap::Error, cmd_info: &CommandInfo, writer: &mut W) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let Some(arg) = context_string(&error, ContextKind::InvalidArg)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let message = t!("cli_errors.no_equals", arg = arg);
    write_cli_error(writer, cmd_info, message)
}

/// Handle a missing required argument error from the command line argument
/// parser.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn missing_arg<W>(
    error: clap::Error, cmd_info: &CommandInfo, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let args = match error.get(ContextKind::InvalidArg)
    {
        Some(ContextValue::Strings(values)) => values.join(", "),
        Some(other) => other.to_string(),
        None => return generic(error, cmd_info, writer),
    };
    let message = t!("cli_errors.missing_required_argument", args = args);
    write_cli_error(writer, cmd_info, message)
}

/// Get the context number from an error.
///
/// # Arguments
///
/// * `error` - The error to get the context number from.
/// * `kind` - The context kind to get the number from.
///
/// # Returns
///
/// The context number.
fn context_number(error: &clap::Error, kind: ContextKind) -> Option<isize>
{
    tern!(let Some(ContextValue::Number(n)) = error.get(kind), Some(*n), None)
}

/// Handle a missing subcommand error.
///
/// # Arguments
///
/// * `cmd` - The command whose help should be printed.
/// * `writer` - The writer to write the help to.
///
/// # Returns
///
/// The exit code to return.
fn missing_subcommand<W>(cmd: &Command, writer: &mut W) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let help = cmd.clone().render_help();
    let msg = tern!(writer.supports_color(), help.ansi().to_string(), help.to_string());
    let result = write_line(writer, &msg);
    tern!(result.is_ok(), ExitCode::InvalidUsage, ExitCode::IoError)
}

/// Handle an unrecognized clap error with a localized message and clap detail.
///
/// # Arguments
///
/// * `error` - The clap error.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn generic<W>(error: clap::Error, cmd_info: &CommandInfo, writer: &mut W) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let detail = error.to_string();
    let message = t!("cli_errors.generic_with_detail", detail = detail);
    write_cli_error(writer, cmd_info, message)
}

/// Handle a version request from the command line argument parser.
///
/// # Arguments
///
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the version to.
///
/// # Returns
///
/// The exit code to return.
fn print_version<W>(cmd_info: &CommandInfo, writer: &mut W) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let result =
        write_line(writer, &create_version_string(&cmd_info.name, &cmd_info.version));
    tern!(result.is_ok(), ExitCode::Success, ExitCode::Failure)
}

/// Handle a help request from the command line argument parser.
///
/// # Arguments
///
/// * `error` - The clap error containing rendered help.
/// * `writer` - The writer to write the help to.
///
/// # Returns
///
/// The exit code to return.
fn print_help<W>(error: clap::Error, writer: &mut W) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let help = error.render();
    let msg = tern!(writer.supports_color(), help.ansi().to_string(), help.to_string());
    let result = write_line(writer, &msg);
    tern!(result.is_ok(), ExitCode::Success, ExitCode::Failure)
}

/// Handle a value validation error from the command line argument parser.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn value_validation<W>(
    error: clap::Error, cmd_info: &CommandInfo, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let Some(arg) = context_string(&error, ContextKind::InvalidArg)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let Some(value) = context_string(&error, ContextKind::InvalidValue)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let v = value.escape_debug().to_string();
    let plain = t!("cli_errors.value_validation", arg = arg, value = v);
    let detailed = |reason| {
        t!(
            "cli_errors.value_validation_with_reason",
            arg = arg,
            value = v,
            reason = reason
        )
    };
    let message = error.source().map_or(plain, detailed);

    write_cli_error(writer, cmd_info, message)
}

/// Handle the case that the number of values is greater than the expected
/// number of values.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn more_val_count<W>(
    error: clap::Error, cmd_info: &CommandInfo, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let Some(arg) = context_string(&error, ContextKind::InvalidArg)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let Some(value) = context_string(&error, ContextKind::InvalidValue)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let v = value.escape_debug().to_string();
    let message = t!("cli_errors.more_values_than_expected", arg = arg, value = v);
    write_cli_error(writer, cmd_info, message)
}

/// Handle the case that the number of values is less than the expected number
/// of values.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn less_val_count<W>(
    error: clap::Error, cmd_info: &CommandInfo, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let Some(arg) = context_string(&error, ContextKind::InvalidArg)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let Some(min) = context_number(&error, ContextKind::MinValues)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let Some(actual) = context_number(&error, ContextKind::ActualNumValues)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let message = t!(
        "cli_errors.less_values_than_expected",
        arg = arg,
        min = min,
        actual = actual
    );
    write_cli_error(writer, cmd_info, message)
}

/// Handle the case that the subcommand is unsupported.
/// parser.
///
/// # Arguments
///
/// * `cmd_info` - The command or subcommand information.
/// * `name` - The subcommand name.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
pub fn unsupported_subcommand_error<W>(
    cmd_info: &CommandInfo, name: &str, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let msg = t!("cli_errors.unsupported_subcommand", name = name);
    write_cli_error(writer, cmd_info, msg)
}

/// Handle the case that the number of values is wrong.
///
/// # Arguments
///
/// * `error` - The error to handle.
/// * `cmd_info` - The command or subcommand information.
/// * `writer` - The writer to write the error to.
///
/// # Returns
///
/// The exit code to return.
fn wrong_val_count<W>(
    error: clap::Error, cmd_info: &CommandInfo, writer: &mut W,
) -> ExitCode
where
    W: WriteColor + ?Sized,
{
    let Some(arg) = context_string(&error, ContextKind::InvalidArg)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let Some(expected) = context_number(&error, ContextKind::ExpectedNumValues)
    else
    {
        return generic(error, cmd_info, writer);
    };
    let Some(actual) = context_number(&error, ContextKind::ActualNumValues)
    else
    {
        return generic(error, cmd_info, writer);
    };
    #[rustfmt::skip]
    let message = t!(
        "cli_errors.wrong_number_of_values", arg = arg, expected = expected, actual = actual);
    write_cli_error(writer, cmd_info, message)
}
