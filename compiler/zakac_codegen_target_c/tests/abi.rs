//! The interface between generated code and the runtime, against its
//! snapshot: the declarations of the runtime and the inline code of the C
//! target and of a lib, as tokens with no comments. A change of the interface
//! raises `ZK_ABI` in `zaka_runtime/stamp.rs` when it breaks a lib built
//! before; else the snapshot alone is refreshed, with `ZAKA_BLESS=1`.

use std::path::PathBuf;

use zakac_codegen_target_c::RUNTIME_ABI;
use zakac_codegen_target_c::driver::RUNTIME;
use zakac_rust_sugars::do_if;

/// The headers that hold the interface.
const HEADERS: [&str; 3] =
    ["zaka_runtime.h", "zaka_target_c_runtime.h", "zaka_lib_glue.h"];

/// Where a scan of C text stands.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scan
{
    /// In code.
    Code,
    /// In a string or a character, which the quote closes.
    Quoted(char),
    /// In a `//` comment.
    Line,
    /// In a `/* */` comment.
    Block,
}

/// C text with each comment turned into a space, strings kept whole.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The text.
fn no_comments(text: &str) -> String
{
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let (mut scan, mut i) = (Scan::Code, 0);
    while i < chars.len()
    {
        let (c, next) = (chars[i], chars.get(i + 1).copied());
        let (to, kept, step) = step(scan, c, next);
        do_if!(let Some(k) = kept, out.push(k));
        (scan, i) = (to, i + step);
    }
    out
}

/// One step of [`no_comments`].
///
/// # Arguments
///
/// * `scan` - Where the scan stands.
/// * `c` - The character.
/// * `next` - The character after it.
///
/// # Returns
///
/// Where the scan stands next, the character kept, and the count of
/// characters read.
fn step(scan: Scan, c: char, next: Option<char>) -> (Scan, Option<char>, usize)
{
    match (scan, c, next)
    {
        (Scan::Code, '/', Some('/')) => (Scan::Line, Some(' '), 2),
        (Scan::Code, '/', Some('*')) => (Scan::Block, Some(' '), 2),
        (Scan::Code, '"' | '\'', _) => (Scan::Quoted(c), Some(c), 1),
        (Scan::Quoted(_), '\\', Some(n)) => (scan, Some(c), 1 + usize::from(n != '\n')),
        (Scan::Quoted(q), _, _) if c == q => (Scan::Code, Some(c), 1),
        (Scan::Line, '\n', _) => (Scan::Code, Some('\n'), 1),
        (Scan::Block, '*', Some('/')) => (Scan::Code, None, 2),
        (Scan::Line | Scan::Block, _, _) => (scan, None, 1),
        _ => (scan, Some(c), 1),
    }
}

/// The tokens of a line of C: words of letters, digits, `_` and `.`, quoted
/// text whole, and each other mark alone.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// The tokens.
fn tokens(line: &str) -> Vec<String>
{
    let mut out: Vec<String> = Vec::new();
    let (mut quoted, mut gap) = (None::<char>, true);
    let word = |c: char| c.is_alphanumeric() || c == '_' || c == '.';
    for c in line.chars()
    {
        let joins = quoted.is_some()
            || (word(c) && !gap && out.last().is_some_and(|t| t.chars().all(word)));
        quoted = match quoted
        {
            Some(q) if c == q => None,
            Some(q) => Some(q),
            None => (c == '"' || c == '\'').then_some(c),
        };
        gap = c.is_whitespace() && !joins;
        if gap
        {
            continue;
        }
        match (joins, out.last_mut())
        {
            (true, Some(last)) => last.push(c),
            _ => out.push(c.to_string()),
        }
    }
    out
}

/// The interface as the snapshot holds it: the ABI, then each header, a line
/// for each directive and for each run of code up to `;`, `{` or `}`.
///
/// # Returns
///
/// The text.
fn interface() -> String
{
    let mut out = format!("abi {RUNTIME_ABI}\n");
    for name in HEADERS
    {
        let text = RUNTIME.iter().find(|(n, _)| *n == name).map(|(_, t)| *t);
        let text = no_comments(text.expect("the compiler carries each header"));
        let text = text.replace("\r\n", "\n").replace("\\\n", " ");
        out.push_str(&format!("# {name}\n"));
        let mut code: Vec<String> = Vec::new();
        for line in text.lines()
        {
            let line_tokens = tokens(line);
            if line.trim_start().starts_with('#')
            {
                out.push_str(&format!("{}\n", line_tokens.join(" ")));
                continue;
            }
            for t in line_tokens
            {
                let ends = t == ";" || t == "{" || t == "}";
                code.push(t);
                do_if!(
                    ends,
                    out.push_str(&format!(
                        "{}\n",
                        code.drain(..).collect::<Vec<_>>().join(" ")
                    ))
                );
            }
        }
        do_if!(!code.is_empty(), out.push_str(&format!("{}\n", code.join(" "))));
    }
    out
}

#[test]
fn the_interface_matches_its_snapshot_or_the_abi_was_raised()
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/expected/runtime_abi.tokens");
    let now = interface();
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &now).unwrap();
        return;
    }
    let saved = std::fs::read_to_string(&path).unwrap_or_default();
    let saved_abi = saved.lines().next().unwrap_or_default();
    let same_abi = saved_abi == format!("abi {RUNTIME_ABI}");
    let told = match same_abi
    {
        true =>
        {
            "the interface changed: raise ZK_ABI in zaka_runtime/stamp.rs when the \
             change breaks a lib built before, then refresh the snapshot with \
             ZAKA_BLESS=1"
        }
        false => "ZK_ABI changed: refresh the snapshot with ZAKA_BLESS=1",
    };
    assert!(now == saved, "{told}");
}

#[test]
fn comments_go_and_strings_stand_whole()
{
    let text = "int a; /* gone */ char *s = \"// kept\"; // gone\n#define X 1";
    let clean = no_comments(text);
    assert!(!clean.contains("gone"), "{clean}");
    assert!(clean.contains("\"// kept\""), "{clean}");
    assert_eq!(tokens("char *s = \"a b\";"), ["char", "*", "s", "=", "\"a b\"", ";"]);
    assert_eq!(tokens("#define zk_a zk_b"), ["#", "define", "zk_a", "zk_b"]);
    assert_eq!(
        tokens("zk_f(x->z, 1.5)"),
        ["zk_f", "(", "x", "-", ">", "z", ",", "1.5", ")"]
    );
}
