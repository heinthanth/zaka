use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::path::PathBuf;

use gen_lsp_types::DidChangeTextDocumentParams;
use gen_lsp_types::DidCloseTextDocumentParams;
use gen_lsp_types::DidOpenTextDocumentParams;
use gen_lsp_types::PublishDiagnosticsParams;
use gen_lsp_types::ReferenceParams;
use gen_lsp_types::RenameParams;
use gen_lsp_types::SemanticTokens;
use gen_lsp_types::TextDocumentContentChangeEvent;
use gen_lsp_types::TextDocumentIdentifier;
use gen_lsp_types::TextDocumentPositionParams;
use gen_lsp_types::Uri;
use lsp_server::Connection;
use lsp_server::ErrorCode;
use lsp_server::Message;
use lsp_server::Notification;
use lsp_server::Request;
use lsp_server::RequestId;
use lsp_server::Response;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use zakac_rust_sugars::ret_if;

use crate::check::Buffers;
use crate::check::FileCheck;
use crate::check::check_file;
use crate::complete::completions;
use crate::hints::call_hints;
use crate::hover::hover;
use crate::names::definition;
use crate::names::references;
use crate::names::rename;
use crate::names::rename_range;
use crate::outline::outline;
use crate::tokens::TOKEN_KINDS;
use crate::tokens::semantic_tokens;

/// The error that stops the server.
pub type ServeError = Box<dyn Error + Send + Sync>;

/// What the server holds between messages.
#[derive(Default)]
struct State
{
    /// The text of each open file.
    docs: HashMap<Uri, String>,
    /// The last check of each open file.
    checks: HashMap<Uri, FileCheck>,
    /// The open files whose check is out of date.
    stale: HashSet<Uri>,
}

/// Answer an editor over a connection until it shuts the server down.
///
/// # Arguments
///
/// * `connection` - The connection to the editor.
///
/// # Returns
///
/// `Ok` after a clean shutdown, or the error that stopped the server.
pub fn serve(connection: &Connection) -> Result<(), ServeError>
{
    connection.initialize(capabilities())?;
    let mut state = State::default();
    for message in &connection.receiver
    {
        match message
        {
            Message::Request(req) =>
            {
                ret_if!(connection.handle_shutdown(&req)?, Ok(()));
                refresh(connection, &mut state)?;
                connection.sender.send(answer_guarded(&state, req).into())?;
            }
            Message::Notification(note) => take_note(&mut state, note),
            Message::Response(_) => (),
        }
        // A burst of edits gets one check, once no other message waits.
        refresh_when_idle(connection, &mut state)?;
    }
    Ok(())
}

/// Check the stale files when no message waits.
///
/// # Arguments
///
/// * `connection` - The connection to the editor.
/// * `state` - What the server holds.
///
/// # Returns
///
/// `Ok`, or the error of a send.
fn refresh_when_idle(
    connection: &Connection, state: &mut State,
) -> Result<(), ServeError>
{
    ret_if!(!connection.receiver.is_empty(), Ok(()));
    refresh(connection, state)
}

/// What the server can do: the whole text on each change, hover, call
/// hints, colored tokens, the outline, completions, definitions, references
/// and renames, with positions in UTF-16 columns.
///
/// # Returns
///
/// The capabilities.
fn capabilities() -> serde_json::Value
{
    json!({
        "positionEncoding": "utf-16",
        "textDocumentSync": 1,
        "hoverProvider": true,
        "inlayHintProvider": true,
        "documentSymbolProvider": true,
        "completionProvider": {},
        "definitionProvider": true,
        "referencesProvider": true,
        "renameProvider": { "prepareProvider": true },
        "semanticTokensProvider": {
            "legend": { "tokenTypes": TOKEN_KINDS, "tokenModifiers": [] },
            "full": true,
            "range": true,
        },
    })
}

/// Follow an open, an edit or a close of a file. Each edit marks each open
/// file stale, as an open file can import the edited one.
///
/// # Arguments
///
/// * `state` - What the server holds.
/// * `note` - The notification.
fn take_note(state: &mut State, note: Notification)
{
    match note.method.as_str()
    {
        "textDocument/didOpen" =>
        {
            let Some(p) = params::<DidOpenTextDocumentParams>(note.params)
            else
            {
                return;
            };
            state.docs.insert(p.text_document.uri, p.text_document.text);
        }
        "textDocument/didChange" =>
        {
            let Some(p) = params::<DidChangeTextDocumentParams>(note.params)
            else
            {
                return;
            };
            let uri = p.text_document.text_document_identifier.uri;
            let last = p.content_changes.into_iter().last();
            let Some(text) = last.and_then(whole_text)
            else
            {
                return;
            };
            state.docs.insert(uri, text);
        }
        "textDocument/didClose" =>
        {
            let Some(p) = params::<DidCloseTextDocumentParams>(note.params)
            else
            {
                return;
            };
            state.docs.remove(&p.text_document.uri);
            state.checks.remove(&p.text_document.uri);
            state.stale.insert(p.text_document.uri);
        }
        _ => return,
    }
    state.stale.extend(state.docs.keys().cloned());
}

/// The whole text of a change; the server asks for the whole text, so a
/// partial change does not come.
///
/// # Arguments
///
/// * `change` - The change.
///
/// # Returns
///
/// The text, or `None` for a partial change.
fn whole_text(change: TextDocumentContentChangeEvent) -> Option<String>
{
    match change
    {
        TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(c) =>
        {
            Some(c.text)
        }
        TextDocumentContentChangeEvent::TextDocumentContentChangePartial(_) => None,
    }
}

/// Check each stale file and send its diagnostics; a closed file gets an
/// blank list, which clears what the editor shows.
///
/// # Arguments
///
/// * `connection` - The connection to the editor.
/// * `state` - What the server holds.
///
/// # Returns
///
/// `Ok`, or the error of a send.
fn refresh(connection: &Connection, state: &mut State) -> Result<(), ServeError>
{
    let buffers = Buffers { open: open_paths(&state.docs) };
    for uri in std::mem::take(&mut state.stale)
    {
        let file = state.docs.get(&uri).and_then(|text| {
            let path = uri.to_file_path().ok()?;
            Some(check_file(&uri, &path, text, buffers.clone()))
        });
        let diagnostics = file.as_ref().map(|a| a.diagnostics.clone());
        let published = PublishDiagnosticsParams {
            uri: uri.clone(),
            version: None,
            diagnostics: diagnostics.unwrap_or_default(),
        };
        let note =
            Notification::new("textDocument/publishDiagnostics".into(), published);
        connection.sender.send(note.into())?;
        file.map(|a| state.checks.insert(uri, a));
    }
    Ok(())
}

/// The path and text of each open file that lives on the disk.
///
/// # Arguments
///
/// * `docs` - The open files.
///
/// # Returns
///
/// The text of each, on its path.
fn open_paths(docs: &HashMap<Uri, String>) -> HashMap<PathBuf, String>
{
    let on_disk =
        |(uri, text): (&Uri, &String)| Some((uri.to_file_path().ok()?, text.clone()));
    docs.iter().filter_map(on_disk).collect()
}

/// Answer a request: colored tokens, a hover, call hints, the outline,
/// completions, a definition, references or a rename; each other method is
/// refused.
///
/// # Arguments
///
/// * `state` - What the server holds.
/// * `req` - The request.
///
/// # Returns
///
/// The response.
fn answer(state: &State, req: Request) -> Response
{
    let id = req.id.clone();
    let file = |p: &Doc| state.checks.get(&p.text_document.uri);
    let req_params = req.params.clone();
    let doc = params::<Doc>(req.params.clone());
    let at = params::<TextDocumentPositionParams>(req.params);
    let result = match req.method.as_str()
    {
        // A range request gets the tokens of the whole file, which clients take.
        "textDocument/semanticTokens/full" | "textDocument/semanticTokens/range" => doc
            .map(|d| {
                let data = file(&d).map(semantic_tokens).unwrap_or_default();
                json!(SemanticTokens { result_id: None, data })
            }),
        "textDocument/inlayHint" =>
        {
            doc.map(|d| json!(file(&d).map(call_hints).unwrap_or_default()))
        }
        "textDocument/documentSymbol" =>
        {
            doc.map(|d| json!(file(&d).map(outline).unwrap_or_default()))
        }
        "textDocument/completion" => at.map(|p| {
            let found = state.checks.get(&p.text_document.uri);
            json!(completions(found, p.position))
        }),
        "textDocument/hover" => at.map(|p| {
            let found = state.checks.get(&p.text_document.uri);
            json!(found.and_then(|f| hover(f, p.position)))
        }),
        "textDocument/definition" => at.map(|p| {
            let found = state.checks.get(&p.text_document.uri);
            json!(found.and_then(|f| definition(f, p.position)))
        }),
        "textDocument/references" => params::<ReferenceParams>(req_params).map(|p| {
            let at = p.text_document_position_params;
            let with_decl = p.context.include_declaration;
            let found = state.checks.get(&at.text_document.uri);
            let all = state.checks.values();
            json!(found.map(|f| references(all, f, at.position, with_decl)))
        }),
        "textDocument/prepareRename" => at.map(|p| {
            let found = state.checks.get(&p.text_document.uri);
            json!(found.and_then(|f| rename_range(f, p.position)))
        }),
        "textDocument/rename" =>
        {
            let Some(p) = params::<RenameParams>(req_params)
            else
            {
                return invalid(id);
            };
            let at = p.text_document_position_params;
            let found = state.checks.get(&at.text_document.uri);
            let all = state.checks.values();
            let edit = found.map(|f| rename(all, f, at.position, &p.new_name));
            match edit
            {
                Some(Err(reason)) =>
                {
                    return Response::new_err(id, REQUEST_FAILED, reason);
                }
                edit => Some(json!(edit.and_then(Result::ok))),
            }
        }
        _ =>
        {
            let message = format!("unknown method {}", req.method);
            return Response::new_err(id, ErrorCode::MethodNotFound as i32, message);
        }
    };
    match result
    {
        Some(value) => Response::new_ok(id, value),
        None => invalid(id),
    }
}

/// The error code of a request the server understood but could not do,
/// such as a rename to a reserved word.
const REQUEST_FAILED: i32 = -32803;

/// The answer to a request whose params do not match its method.
///
/// # Arguments
///
/// * `id` - The request.
///
/// # Returns
///
/// The response.
fn invalid(id: RequestId) -> Response
{
    let message = "invalid params".to_string();
    Response::new_err(id, ErrorCode::InvalidParams as i32, message)
}

/// Answer a request; a compiler panic while answering becomes an error
/// response, and the server goes on.
///
/// # Arguments
///
/// * `state` - What the server holds.
/// * `req` - The request.
///
/// # Returns
///
/// The response.
fn answer_guarded(state: &State, req: Request) -> Response
{
    let id = req.id.clone();
    let answered = catch_unwind(AssertUnwindSafe(|| answer(state, req)));
    answered.unwrap_or_else(|_| {
        let message = "the server failed on this request".to_string();
        Response::new_err(id, ErrorCode::InternalError as i32, message)
    })
}

/// The file a request is about: the `textDocument` field each request on a
/// file carries.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Doc
{
    /// The file.
    text_document: TextDocumentIdentifier,
}

/// Read the params of a message.
///
/// # Arguments
///
/// * `value` - The params as JSON.
///
/// # Returns
///
/// The params, or `None` when the JSON does not match.
fn params<P: DeserializeOwned>(value: serde_json::Value) -> Option<P>
{
    serde_json::from_value(value).ok()
}
