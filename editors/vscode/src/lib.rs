//! The Zaka extension for VS Code: it starts `zakac lsp` and hands the open
//! `.zaka` files to it through `vscode-languageclient`. wasm-pack builds it
//! for Node.js; VS Code loads the glue it writes.

use std::cell::RefCell;

use bon::Builder;
use js_sys::Promise;
use js_sys::Reflect;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "vscode-languageclient/node")]
extern "C" {
    /// The LSP client of `vscode-languageclient`.
    type LanguageClient;

    /// Make a client.
    #[wasm_bindgen(constructor)]
    fn new(id: &str, name: &str, server: &JsValue, client: &JsValue) -> LanguageClient;

    /// Start the server and connect to it.
    #[wasm_bindgen(method)]
    fn start(this: &LanguageClient) -> Promise;

    /// Stop the server.
    #[wasm_bindgen(method)]
    fn stop(this: &LanguageClient) -> Promise;
}

#[wasm_bindgen(module = "vscode")]
extern "C" {
    /// The settings of a section.
    type Settings;

    /// The settings of `section`.
    #[wasm_bindgen(js_namespace = workspace, js_name = getConfiguration)]
    fn settings(section: &str) -> Settings;

    /// The value of a setting, or `undefined`.
    #[wasm_bindgen(method)]
    fn get(this: &Settings, name: &str) -> JsValue;

    /// The workspace of the window.
    #[wasm_bindgen(thread_local_v2, js_name = workspace)]
    static WORKSPACE: JsValue;
}

#[wasm_bindgen(module = "path")]
extern "C" {
    /// Whether a path starts at the root of a drive.
    #[wasm_bindgen(js_name = isAbsolute)]
    fn is_absolute(path: &str) -> bool;

    /// A path joined onto a folder.
    fn join(folder: &str, path: &str) -> String;
}

/// How the client starts the server: a command and its arguments.
#[derive(Serialize, Builder)]
struct ServerRun
{
    /// The command.
    command: String,
    /// Its arguments.
    args: [&'static str; 1],
}

/// Which files the client hands to the server.
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct ClientSetup
{
    /// The files: Zaka files on the disk.
    document_selector: [FileFilter; 1],
}

/// A filter on the files of the editor.
#[derive(Serialize, Builder)]
struct FileFilter
{
    /// The URI scheme.
    scheme: &'static str,
    /// The language id.
    language: &'static str,
}

thread_local! {
    /// The running client, kept to stop it.
    static CLIENT: RefCell<Option<LanguageClient>> = const { RefCell::new(None) };
}

/// Start the language server, with the command of the `zaka.server.path`
/// setting.
///
/// # Arguments
///
/// * `_context` - The extension context; unused.
///
/// # Returns
///
/// A promise that settles once the client runs.
#[wasm_bindgen]
pub fn activate(_context: JsValue) -> Result<Promise, JsValue>
{
    let command = settings("zaka").get("server.path").as_string();
    let command = command.map_or_else(|| "zakac".into(), in_workspace);
    let run = ServerRun::builder().command(command).args(["lsp"]).build();
    let filter = FileFilter::builder().scheme("file").language("zaka").build();
    let setup = ClientSetup::builder().document_selector([filter]).build();
    let server = serde_wasm_bindgen::to_value(&run)?;
    let client_setup = serde_wasm_bindgen::to_value(&setup)?;
    let client = LanguageClient::new("zaka", "Zaka", &server, &client_setup);
    let started = client.start();
    CLIENT.with(|slot| slot.replace(Some(client)));
    Ok(started)
}

/// A command path from the settings: a relative path such as
/// `target/debug/zakac` starts at the first workspace folder; a bare name or
/// an absolute path stands as written.
///
/// # Arguments
///
/// * `path` - The path from the settings.
///
/// # Returns
///
/// The command path.
fn in_workspace(path: String) -> String
{
    let bare = !path.contains(['/', '\\']);
    let folder = (!bare && !is_absolute(&path)).then(first_folder).flatten();
    folder.map(|folder| join(&folder, &path)).unwrap_or(path)
}

/// The disk path of the first workspace folder.
///
/// # Returns
///
/// The path, or `None` when the window has no folder open.
fn first_folder() -> Option<String>
{
    let field = |value: &JsValue, name: &str| Reflect::get(value, &name.into()).ok();
    let folders = WORKSPACE.with(|w| field(w, "workspaceFolders"))?;
    let first = Reflect::get(&folders, &0.into()).ok().filter(|f| !f.is_undefined())?;
    field(&field(&first, "uri")?, "fsPath")?.as_string()
}

/// Stop the language server.
///
/// # Returns
///
/// A promise that settles once the server stops, or `undefined` when none
/// runs.
#[wasm_bindgen]
pub fn deactivate() -> JsValue
{
    let client = CLIENT.with(|slot| slot.take());
    client.map_or(JsValue::UNDEFINED, |c| c.stop().into())
}
