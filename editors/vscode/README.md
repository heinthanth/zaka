# Zaka for VS Code

Errors while editing, colors and the kind of a value on hover for Zaka (စံကား)
programs, from the language server in `zakac`.

## Setup

1. Build `zakac` (`cargo build -p zakac`) and set `zaka.server.path`, or install
   it on the `PATH` (`cargo install --path compiler/zakac` in the Zaka repo).
2. Install this extension: `mise run vscode:build`, then
   `code --install-extension editors/vscode/zaka-<version>.vsix`.
3. Open a `.zaka` or `.zk` file. Zaka files get the Zaka logo as their icon when the
   icon theme has none for them.

The setting `zaka.server.path` names a different `zakac`: a name on the `PATH`, an
absolute path, or a path from the first workspace folder, such as
`target/debug/zakac`.
