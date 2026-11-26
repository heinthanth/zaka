//! The tree-sitter grammar of `editors/treesitter`, written from the tables of
//! the lexer: each reserved word and kind name in each of its spellings, under
//! the role that colors it, and each sign. With `ZAKA_BLESS=1` the test writes
//! the grammar and its highlight queries; without it, the test fails when the
//! files differ from what the lexer gives, so a new reserved word reaches the
//! editors. `mise run editor:treesitter` then writes the parser from the
//! grammar.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use serde_json::json;
use strum::IntoEnumIterator;
use zakac_lexer::aliases::WRITTEN;
use zakac_lexer::ident::is_ident;
use zakac_lexer::reserved::KIND_NAMES;
use zakac_lexer::role::TokenRole;
use zakac_lexer::role::kw_role;
use zakac_lexer::role::punct_role;
use zakac_lexer::spoken::SPOKEN;
use zakac_lexer::token::Punct;

/// The roles that get a rule of their own, in the order the grammar lists
/// them, each with the capture of its highlight. The captures follow the
/// token kinds of the language server.
const ROLES: [(TokenRole, &str); 6] = [
    (TokenRole::Control, "@keyword"),
    (TokenRole::Verb, "@function"),
    (TokenRole::Op, "@operator"),
    (TokenRole::Particle, "@keyword.modifier"),
    (TokenRole::Kind, "@type"),
    (TokenRole::Truth, "@constant.builtin"),
];

/// The brackets, which the grammar keeps as written, for the queries of an
/// editor that matches them.
const BRACKETS: [&str; 4] = ["(", ")", "[", "]"];

/// A word of an identifier: a letter, a mark, `_`, or a Burmese sign other
/// than a digit and `၊ ။ ၏`; after the first, digits as well. The lexer reads
/// the same set (`is_ident`).
const WORD: &str =
    r"[\p{L}\p{M}_\u{104C}-\u{104E}][\p{L}\p{M}\p{N}_\u{104C}-\u{104E}]*";

/// A number: a digit of either script, then digits, letters of a base above
/// 10, a fraction and an exponent.
const NUMBER: &str =
    r"[0-9\u{1040}-\u{1049}][0-9A-Za-z_\u{1040}-\u{1049}]*(\.[0-9\u{1040}-\u{1049}]+)?";

/// The folder of the grammar.
fn folder() -> PathBuf
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../editors/treesitter")
}

/// A rule that matches a string as written.
fn string(text: &str) -> Value { json!({ "type": "STRING", "value": text }) }

/// A rule that matches a pattern.
fn pattern(text: &str) -> Value { json!({ "type": "PATTERN", "value": text }) }

/// A rule that names another.
fn named(name: &str) -> Value { json!({ "type": "SYMBOL", "name": name }) }

/// A rule that matches one of its members.
fn choice(members: Vec<Value>) -> Value
{
    json!({ "type": "CHOICE", "members": members })
}

/// A rule that matches its members in order.
fn seq(members: Vec<Value>) -> Value { json!({ "type": "SEQ", "members": members }) }

/// A rule that matches its content as one token, with no space before it.
fn immediate(content: Value) -> Value
{
    json!({ "type": "IMMEDIATE_TOKEN", "content": { "type": "PREC", "value": 1, "content": content } })
}

/// Each spelling of a reserved word or a kind name, with its role: a spelling
/// that is no whole word, or that a role before it took, is left out.
fn spellings() -> BTreeMap<String, TokenRole>
{
    let written = WRITTEN.iter().map(|w| (w.text.clone(), kw_role(w.kw)));
    let spoken = SPOKEN.iter().map(|s| (s.spoken.clone(), kw_role(s.kw)));
    let kinds = KIND_NAMES.iter().map(|k| (k.to_string(), TokenRole::Kind));
    let mut out = BTreeMap::new();
    for (text, role) in written.chain(spoken).chain(kinds).filter(|(t, _)| is_ident(t))
    {
        out.entry(text).or_insert(role);
    }
    out
}

/// The rule of one role: each word and sign of it.
fn role_rule(role: TokenRole, words: &BTreeMap<String, TokenRole>) -> Value
{
    let signs = Punct::iter().filter(|p| punct_role(*p) == role);
    let signs = signs
        .map(|p| p.as_ref().to_string())
        .filter(|s| !BRACKETS.contains(&s.as_str()));
    let words = words.iter().filter(|(_, r)| **r == role).map(|(w, _)| w.clone());
    choice(words.chain(signs).map(|t| string(&t)).collect())
}

/// The rules of the grammar, in the order written: `tree-sitter generate`
/// starts from the first.
struct Rules(Vec<(&'static str, Value)>);

impl Rules
{
    /// Add a rule after the others.
    fn insert(&mut self, name: &'static str, rule: Value) { self.0.push((name, rule)); }
}

impl Serialize for Rules
{
    fn serialize<S: Serializer>(&self, out: S) -> Result<S::Ok, S::Error>
    {
        out.collect_map(self.0.iter().map(|(name, rule)| (name, rule)))
    }
}

/// The grammar, as `tree-sitter generate` reads it, with its fields in the
/// order of the schema.
#[derive(Serialize)]
struct Grammar
{
    #[serde(rename = "$schema")]
    schema: &'static str,
    name: &'static str,
    word: &'static str,
    rules: Rules,
    extras: Vec<Value>,
}

/// The grammar of Zaka.
fn grammar() -> Grammar
{
    let words = spellings();
    let mut rules = Rules(Vec::new());
    let mut items: Vec<Value> =
        ROLES.iter().map(|(r, _)| named(<&str>::from(*r))).collect();
    items.extend(
        ["identifier", "raw_identifier", "number", "char", "string", "delimiter"]
            .map(named),
    );
    items.extend(BRACKETS.map(string));
    rules.insert("source_file", json!({ "type": "REPEAT", "content": named("_item") }));
    rules.insert("_item", choice(items));
    for (role, _) in ROLES
    {
        rules.insert(<&str>::from(role), role_rule(role, &words));
    }
    rules.insert("delimiter", role_rule(TokenRole::Delim, &words));
    rules.insert("identifier", pattern(WORD));
    rules.insert("raw_identifier", pattern(r"\{[^}\n]*\}"));
    rules.insert("number", pattern(NUMBER));
    rules.insert("char", pattern(r"'([^'\\\n]|\\.)*'"));
    let parts = choice(
        ["string_content", "escape_sequence", "template_slot"].map(named).to_vec(),
    );
    let inner = json!({ "type": "REPEAT", "content": parts });
    rules.insert("string", seq(vec![string("\""), inner, immediate(string("\""))]));
    rules.insert("string_content", immediate(pattern(r#"[^"\\{}\n]+"#)));
    rules.insert("escape_sequence", immediate(pattern(r"\\.|\{\{|\}\}")));
    rules.insert("template_slot", immediate(pattern(r"\{[^{}\n]*\}")));
    // `မှတ်ချက်` is a comment marker at the start of a word too, as the
    // lexer reads it; the rest of the line joins the comment after a space.
    let comment = choice(vec![pattern(r"//[^\n]*"), pattern(r"မှတ်ချက်([ \t][^\n]*)?")]);
    let comment = json!({ "type": "PREC", "value": 1, "content": comment });
    rules.insert("comment", json!({ "type": "TOKEN", "content": comment }));
    Grammar {
        schema: "https://tree-sitter.github.io/tree-sitter/assets/schemas/grammar.schema.json",
        name: "zaka",
        word: "identifier",
        rules,
        extras: vec![pattern(r"[\s\u{200B}]"), named("comment")],
    }
}

/// The highlight queries: each role with its capture, then the literals.
fn highlights() -> String
{
    let roles = ROLES.iter().map(|(r, c)| format!("({}) {c}\n", <&str>::from(*r)));
    let rest = [
        "(identifier) @variable",
        "(raw_identifier) @variable",
        "(number) @number",
        "(char) @string",
        "(string) @string",
        "(escape_sequence) @string.escape",
        "(template_slot) @punctuation.special",
        "(comment) @comment",
        "(delimiter) @punctuation.delimiter",
        "[\"(\" \")\" \"[\" \"]\"] @punctuation.bracket",
    ];
    roles.chain(rest.iter().map(|l| format!("{l}\n"))).collect()
}

/// Compare a file of the grammar with what the lexer gives, or write it with
/// `ZAKA_BLESS=1`.
fn check(path: &str, found: &str)
{
    let path = folder().join(path);
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::write(&path, found).unwrap();
    }
    let held = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(held, found, "{} is stale; run with ZAKA_BLESS=1", path.display());
}

#[test]
fn the_tree_sitter_grammar_holds_each_reserved_word()
{
    let text = serde_json::to_string_pretty(&grammar()).unwrap() + "\n";
    check("src/grammar.json", &text);
}

#[test]
fn the_highlight_queries_name_each_role()
{
    check("queries/highlights.scm", &highlights());
}
