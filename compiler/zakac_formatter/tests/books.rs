//! The formatter over the code fences of Learn and Docs.

use std::path::Path;
use std::path::PathBuf;

use zakac_formatter::FmtOptions;
use zakac_formatter::format_text;
use zakac_formatter::options::SpeechMode;
use zakac_rust_sugars::tern;

use crate::common::repo;

/// The Markdown pages of a folder and its subfolders.
///
/// # Arguments
///
/// * `dir` - The folder.
///
/// # Returns
///
/// The paths, sorted.
fn pages(dir: &Path) -> Vec<PathBuf>
{
    let entries = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path());
    let mut found: Vec<PathBuf> = entries
        .flat_map(|p| if p.is_dir() { pages(&p) } else { vec![p] })
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    found.sort();
    found
}

/// The fences of a page that hold a program meant to compile: `zaka`,
/// `zaka-warning` and `zaka-stop`, each with its info line.
///
/// # Arguments
///
/// * `page` - The text of the page.
///
/// # Returns
///
/// The info line and the code of each fence.
fn fences(page: &str) -> Vec<(String, String)>
{
    let mut found = Vec::new();
    let mut open: Option<(String, String)> = None;
    for line in page.lines()
    {
        let fence = line.trim_start().strip_prefix("```");
        match (fence, open.take())
        {
            (Some(_), Some(done)) => found.push(done),
            (Some(info), None) if compiles(info) =>
            {
                open = Some((info.to_string(), String::new()))
            }
            (None, Some((info, code))) => open = Some((info, code + line + "\n")),
            _ =>
            {}
        }
    }
    found
}

/// Whether a fence holds a program meant to compile.
///
/// # Arguments
///
/// * `info` - The info line after the backticks.
///
/// # Returns
///
/// `true` for such a fence.
fn compiles(info: &str) -> bool
{
    let word = info.split_whitespace().next().unwrap_or("");
    matches!(word, "zaka" | "zaka-warning" | "zaka-stop")
}

/// The pages whose fences put statements on one line on purpose, to show
/// that statements can share a line.
const SHARED_LINES: [&str; 1] = ["words.md"];

/// Each fence that compiles is formatted already.
#[test]
fn book_fences_are_formatted()
{
    let books = [pages(&repo().join("docs/learn")), pages(&repo().join("docs/manual"))];
    let mut changed = Vec::new();
    for page in books.iter().flatten()
    {
        let shared = SHARED_LINES.iter().any(|p| page.ends_with(p));
        let text = std::fs::read_to_string(page).unwrap();
        let off = fences(&text).into_iter().filter(|(info, code)| {
            // A fence that allows spoken words is written in them.
            let spoken = info.contains("lint-allow=spoken");
            let speech = tern!(spoken, SpeechMode::Spoken, SpeechMode::Written);
            #[rustfmt::skip]
            let options = FmtOptions::builder().one_statement_per_line(!shared)
                .speech_mode(speech).build();
            !matches!(format_text(code, &options), Ok(out) if out == *code)
        });
        changed.extend(off.map(|(info, _)| format!("{} [{info}]", page.display())));
    }
    assert!(changed.is_empty(), "not formatted:\n{}", changed.join("\n"));
}
