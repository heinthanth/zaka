use std::path::PathBuf;

use bon::Builder;
use zakac_i18n::t;

/// Possible errors that can occur during a Zakac operation.
#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error("{}", t!("errors.file_not_found", p = path.display().to_string()))]
    FileNotFound
    {
        path: PathBuf
    },
    #[error("{}", t!("errors.permission_denied", p = path.display().to_string()))]
    PermissionDenied
    {
        path: PathBuf
    },
    #[error("{}", t!("errors.file_is_a_folder", p = path.display().to_string()))]
    FileIsAFolder
    {
        path: PathBuf
    },
    #[error("{}", t!("errors.folder_is_a_file", p = path.display().to_string()))]
    FolderIsAFile
    {
        path: PathBuf
    },
    #[error("{}", t!("errors.failed_to_read_file", p = path.display().to_string()))]
    FailedToReadFile
    {
        path: PathBuf
    },
    #[error("{}", t!("errors.failed_to_write_file", p = path.display().to_string()))]
    FailedToWriteFile
    {
        path: PathBuf
    },
    #[error("{}", t!("errors.generic_io_error", p = path.display().to_string(), e = format!("{:?}", error)))]
    GenericIOError
    {
        path: PathBuf,
        error: Box<dyn std::error::Error>,
    },
    #[error("{}", t!("errors.generic_error", e = format!("{:?}", error)))]
    InternalError
    {
        error: Box<dyn std::error::Error>
    },
    #[error("{}", t!("errors.unsupported_artifact_format", p = path.display().to_string()))]
    UnsupportedArtifactFormat
    {
        path: PathBuf
    },
    #[error("{}", t!("errors.failed_to_decode_artifact", p = path.display().to_string(), e = msg))]
    FailedToDecodeArtifact
    {
        path: PathBuf, msg: String
    },
    #[error("{}", t!("errors.runtime_trap", e = msg))]
    RuntimeTrap
    {
        msg: String
    },
    #[error("{}", t!("errors.no_c_compiler"))]
    NoCCompiler,
    #[error("{}", t!("errors.flag_not_for_target", flag = flag, target = target))]
    FlagNotForTarget
    {
        flag: String, target: String
    },
    #[error("{}", t!("errors.tool_not_found", tool = tool))]
    ToolNotFound
    {
        tool: String
    },
    #[error("{}", t!("errors.c_compiler_failed", cc = cc))]
    CCompilerFailed
    {
        cc: String
    },
    #[error("{}", t!("errors.not_for_target_c", what = what))]
    NotForTargetC
    {
        what: String
    },
    #[error("{}", t!("errors.config_not_toml", p = path, e = msg))]
    ConfigNotToml
    {
        path: String, msg: String
    },
    #[error("{}", t!("errors.config_unknown_name", p = path, name = name))]
    ConfigUnknownName
    {
        path: String, name: String
    },
    #[error("{}", t!("errors.config_bad_value", p = path, name = name))]
    ConfigBadValue
    {
        path: String, name: String
    },
    #[error("{}", t!("errors.unknown_lint_name", name = name))]
    UnknownLintName
    {
        name: String
    },
    #[error("{}", t!("errors.not_a_lint", code = code))]
    NotALint
    {
        code: String
    },
    #[error("{}", t!("errors.unknown_check", name = name))]
    UnknownCheck
    {
        name: String
    },
}

/// An action performed on an IO resource.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IOErrorAction
{
    Read,
    Write,
}

/// A context for an IO error.
#[derive(Debug, Clone, Builder)]
pub struct IOErrorContext
{
    /// The path of the resource.
    pub path: PathBuf,
    /// The action performed on the resource.
    pub action: IOErrorAction,
}

impl IOErrorContext
{
    /// Create a new IO error context.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the resource.
    /// * `action` - The action performed on the resource.
    ///
    /// # Returns
    ///
    /// The new IO error context.
    pub fn new(path: PathBuf, action: IOErrorAction) -> Self
    {
        Self::builder().path(path).action(action).build()
    }
}

/// Convert an IO error to a Zakac error.
///
/// # Arguments
///
/// * `error` - The IO error.
/// * `context` - The context for the IO error.
///
/// # Returns
///
/// The corresponding Zakac error.
fn m_io_err(error: std::io::Error, context: IOErrorContext) -> Error
{
    let path = context.path;
    match error.kind()
    {
        std::io::ErrorKind::NotFound => Error::FileNotFound { path },
        std::io::ErrorKind::PermissionDenied => Error::PermissionDenied { path },
        std::io::ErrorKind::IsADirectory => Error::FileIsAFolder { path },
        std::io::ErrorKind::NotADirectory => Error::FolderIsAFile { path },
        _ if context.action == IOErrorAction::Read => Error::FailedToReadFile { path },
        _ if context.action == IOErrorAction::Write =>
        {
            Error::FailedToWriteFile { path }
        }
        _ => Error::GenericIOError { path, error: Box::new(error) },
    }
}

impl Error
{
    /// Convert an IO error to a Zakac error.
    ///
    /// # Arguments
    ///
    /// * `e` - The IO error.
    /// * `ctx` - The context for the IO error.
    ///
    /// # Returns
    ///
    /// The corresponding Zakac error.
    pub fn from_io(e: std::io::Error, ctx: IOErrorContext) -> Self { m_io_err(e, ctx) }
}
