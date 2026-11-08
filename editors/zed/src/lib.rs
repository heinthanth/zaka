//! The Zaka extension for Zed: it starts `zakac lsp` for the Zaka files.
//! The `lsp.zakac.binary` setting names another `zakac` and its arguments;
//! with none, the extension takes `zakac` from the PATH of the worktree.

use zakac_rust_sugars::tern;
use zed_extension_api as zed;
use zed_extension_api::settings::LspSettings;

/// The extension; it holds no state.
struct ZakaExtension;

impl zed::Extension for ZakaExtension
{
    fn new() -> Self { ZakaExtension }

    fn language_server_command(
        &mut self, server: &zed::LanguageServerId, worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command>
    {
        let settings = LspSettings::for_worktree(server.as_ref(), worktree);
        let bin = settings.ok().and_then(|s| s.binary);
        let path = bin.as_ref().and_then(|b| b.path.clone());
        let set = path.map(|path| in_worktree(&worktree.root_path(), path));
        let command = set.or_else(|| worktree.which("zakac")).ok_or(NOT_FOUND)?;
        // Zed gives an empty list when the setting names no arguments.
        let args = bin.as_ref().and_then(|b| b.arguments.clone());
        let args = args.filter(|a| !a.is_empty()).unwrap_or_else(|| vec!["lsp".into()]);
        let env = bin.and_then(|b| b.env).map(|env| env.into_iter().collect());
        let env = env.unwrap_or_default();
        Ok(zed::Command { command, args, env })
    }
}

/// A command path from the settings: a relative path such as
/// `target/debug/zakac` starts at the worktree root; a bare name or an
/// absolute path stands as written.
///
/// # Arguments
///
/// * `root` - The worktree root.
/// * `path` - The path from the settings.
///
/// # Returns
///
/// The command path.
fn in_worktree(root: &str, path: String) -> String
{
    let bare = !path.contains(['/', '\\']);
    let absolute = path.starts_with(['/', '\\']) || path.get(1..2) == Some(":");
    tern!(bare || absolute, path, format!("{root}/{path}"))
}

/// The error when no `zakac` is set and none is on the PATH.
const NOT_FOUND: &str = "zakac is not on the PATH; install it with `cargo install \
                         --path compiler/zakac`, or set lsp.zakac.binary.path to a \
                         zakac build";

zed::register_extension!(ZakaExtension);
