//! Function kinds: the kind of a function value, its parameters (each with
//! its edit mark and its mark) and its result.
//!
//! A function kind writes its short name as `fn(<param>, …) -> <result>`,
//! each parameter as `[mut ]<kind>[(<mark>)]`. Inside another name (a cell, a
//! pointer base, a parameter, a result or a field) it stands in parentheses,
//! `(fn(isize) -> bool)[]`, so the levels after it never read as levels of
//! its result.

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use zakac_ast::name::Name;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::kind::DataKind;

/// One parameter of a function kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnParam
{
    /// The kind.
    pub kind: DataKind,
    /// Whether the function can change the argument.
    pub edit: bool,
    /// The word after the argument in a marked call, when the parameters
    /// have marks.
    pub mark: Option<Name>,
}

/// A function kind: the parameters and the result of a function value. A
/// value is one word: the function it calls, or none.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnKind
{
    /// The parameters, in order.
    pub params: Vec<FnParam>,
    /// The result kind; `None` for a function with no result.
    pub ret: Option<DataKind>,
}

/// Each function kind made so far, so a kind is made once and remains
/// `Copy`.
static FNS: OnceLock<Mutex<HashSet<&'static FnKind>>> = OnceLock::new();

/// The text that opens the short name of a function kind.
const OPEN: &str = "fn(";

/// The text between the parameters and the result in a short name.
const ARROW: &str = " -> ";

impl FnKind
{
    /// The function kind with these parts, made once and kept for the rest
    /// of the run.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters.
    /// * `ret` - The result kind, when the function has one.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn make(params: Vec<FnParam>, ret: Option<DataKind>) -> &'static FnKind
    {
        let wanted = FnKind { params, ret };
        let fns = FNS.get_or_init(Mutex::default);
        let mut fns = fns.lock().unwrap_or_else(PoisonError::into_inner);
        ret_if!(let Some(found) = fns.get(&wanted), found);
        let made: &'static FnKind = Box::leak(Box::new(wanted));
        fns.insert(made);
        made
    }

    /// The kinds of the parameters, in order.
    ///
    /// # Returns
    ///
    /// The kinds.
    pub fn param_kinds(&self) -> Vec<DataKind>
    {
        self.params.iter().map(|p| p.kind).collect()
    }

    /// Whether the function can change each parameter.
    ///
    /// # Returns
    ///
    /// One flag for each parameter.
    pub fn edits(&self) -> Vec<bool> { self.params.iter().map(|p| p.edit).collect() }

    /// The marks of the parameters, when each has one.
    ///
    /// # Returns
    ///
    /// The marks in order, or `None` when the parameters have none.
    pub fn marks(&self) -> Option<Vec<Name>>
    {
        ret_if!(self.params.is_empty(), None);
        self.params.iter().map(|p| p.mark).collect()
    }

    /// The kind with each mark dropped.
    ///
    /// # Returns
    ///
    /// The kind whose parameters have no marks.
    pub fn unmarked(&self) -> &'static FnKind
    {
        let drop = |p: &FnParam| FnParam { mark: None, ..p.clone() };
        FnKind::make(self.params.iter().map(drop).collect(), self.ret)
    }

    /// Whether a function of this kind fits a place of kind `want`: the
    /// same parameter kinds, edit marks and result, and the same marks when
    /// `want` has marks. A kind with no marks takes a function with marks
    /// or without.
    ///
    /// # Arguments
    ///
    /// * `want` - The kind the place holds.
    ///
    /// # Returns
    ///
    /// `true` when it fits.
    pub fn fits(&self, want: &FnKind) -> bool
    {
        ret_if!(self == want, true);
        let marked = want.params.iter().any(|p| p.mark.is_some());
        !marked && std::ptr::eq(self.unmarked(), want.unmarked())
    }

    /// The short name of the kind: `fn(<param>, …) -> <result>`.
    ///
    /// # Arguments
    ///
    /// * `full` - Whether each struct is written as its signature.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn spell(&self, full: bool) -> String
    {
        let param = |p: &FnParam| {
            let edit = tern!(p.edit, "mut ", "");
            let mark = p.mark.map_or(String::new(), |m| format!("({m})"));
            format!("{edit}{}{mark}", p.kind.spell_inner(full))
        };
        let params: Vec<String> = self.params.iter().map(param).collect();
        let ret = self
            .ret
            .map_or(String::new(), |r| format!("{ARROW}{}", r.spell_inner(full)));
        format!("{OPEN}{}){ret}", params.join(", "))
    }
}

/// The text form of a function kind, split into its parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnText<'a>
{
    /// Each parameter: whether `mut` marks it, its kind text and its mark.
    pub params: Vec<(bool, &'a str, Option<&'a str>)>,
    /// The result kind text.
    pub ret: Option<&'a str>,
}

/// How a character moves the bracket depth of a kind name: `(`, `{` and `<`
/// open, their twins close, and the `>` of `->` does neither.
///
/// # Arguments
///
/// * `prev` - The character before, when there is one.
/// * `ch` - The character.
///
/// # Returns
///
/// 1 to open, -1 to close, 0 otherwise.
pub fn bracket_step(prev: Option<char>, ch: char) -> i32
{
    match ch
    {
        '(' | '{' | '<' => 1,
        '>' if prev == Some('-') => 0,
        ')' | '}' | '>' => -1,
        _ => 0,
    }
}

/// Split text at each `sep` outside brackets.
///
/// # Arguments
///
/// * `text` - The text.
/// * `sep` - The separator.
///
/// # Returns
///
/// The parts.
pub fn split_top(text: &str, sep: char) -> Vec<&str>
{
    let mut parts = Vec::new();
    let (mut depth, mut start, mut prev) = (0i32, 0, None);
    for (at, ch) in text.char_indices()
    {
        depth += bracket_step(prev, ch);
        prev = Some(ch);
        if ch == sep && depth == 0
        {
            parts.push(&text[start..at]);
            start = at + ch.len_utf8();
        }
    }
    parts.push(&text[start..]);
    parts
}

/// Take off the parentheses around a whole name, as a function kind inside
/// another name stands.
///
/// # Arguments
///
/// * `text` - The name.
///
/// # Returns
///
/// The name inside, or the name as it is.
pub fn unwrap_parens(text: &str) -> &str
{
    let Some(inner) = text.strip_prefix('(').and_then(|t| t.strip_suffix(')'))
    else
    {
        return text;
    };
    tern!(closes_at_end(text), inner, text)
}

/// Whether the `(` that opens `text` closes at its last character.
///
/// # Arguments
///
/// * `text` - Text that starts with `(`.
///
/// # Returns
///
/// `true` if it does.
fn closes_at_end(text: &str) -> bool
{
    let (mut depth, mut prev) = (0i32, None);
    for (at, ch) in text.char_indices()
    {
        depth += bracket_step(prev, ch);
        prev = Some(ch);
        ret_if!(depth == 0, at + ch.len_utf8() == text.len());
    }
    false
}

/// Split the short name of a function kind, `fn(<param>, …) -> <result>`,
/// into its parts.
///
/// # Arguments
///
/// * `text` - The name, with no parentheses around it.
///
/// # Returns
///
/// The parts, or `None` for text that is no function kind.
pub fn split_fn(text: &str) -> Option<FnText<'_>>
{
    let rest = text.strip_prefix(OPEN)?;
    let close = matching_close(rest)?;
    let (inner, tail) = (&rest[..close], &rest[close + 1..]);
    let ret = tern!(tail.is_empty(), None, Some(tail.strip_prefix(ARROW)?));
    let parts = tern!(inner.is_empty(), Vec::new(), split_top(inner, ','));
    let params = parts.into_iter().map(|p| split_param(p.trim())).collect();
    Some(FnText { params, ret })
}

/// The place of the `)` that closes text whose `(` came just before it.
///
/// # Arguments
///
/// * `text` - The text after the `(`.
///
/// # Returns
///
/// The octet offset of the `)`, or `None`.
fn matching_close(text: &str) -> Option<usize>
{
    let (mut depth, mut prev) = (1i32, Some('('));
    for (at, ch) in text.char_indices()
    {
        depth += bracket_step(prev, ch);
        prev = Some(ch);
        ret_if!(depth == 0, Some(at));
    }
    None
}

/// Split one parameter of a function kind name: `[mut ]<kind>[(<mark>)]`.
///
/// # Arguments
///
/// * `text` - The parameter.
///
/// # Returns
///
/// Whether `mut` marks it, the kind text and the mark.
fn split_param(text: &str) -> (bool, &str, Option<&str>)
{
    let (edit, text) = text.strip_prefix("mut ").map_or((false, text), |t| (true, t));
    let Some(open) = mark_open(text)
    else
    {
        return (edit, text, None);
    };
    (edit, &text[..open], Some(&text[open + 1..text.len() - 1]))
}

/// The place of the `(` of a mark at the end of a parameter: a `)` last,
/// whose `(` does not open the parameter.
///
/// # Arguments
///
/// * `text` - The parameter, `mut` taken off.
///
/// # Returns
///
/// The octet offset of the `(`, or `None` for a parameter with no mark.
fn mark_open(text: &str) -> Option<usize>
{
    ret_if!(!text.ends_with(')'), None);
    let open = text.rfind('(')?;
    // A mark is a word with no brackets in it.
    let word = &text[open + 1..text.len() - 1];
    let plain = !word.contains(['(', ')', '{', '}', '<', '>', '[', ']']);
    (open > 0 && plain && !text[..open].ends_with("fn")).then_some(open)
}

/// Write the short name of a function kind from the texts of its parts.
///
/// # Arguments
///
/// * `parts` - The parts, each kind in text form.
///
/// # Returns
///
/// The name.
pub fn join_fn(parts: &FnText<'_>) -> String
{
    let param = |(edit, kind, mark): &(bool, &str, Option<&str>)| {
        let mark = mark.map_or(String::new(), |m| format!("({m})"));
        format!("{}{kind}{mark}", tern!(*edit, "mut ", ""))
    };
    let params: Vec<String> = parts.params.iter().map(param).collect();
    let ret = parts.ret.map_or(String::new(), |r| format!("{ARROW}{r}"));
    format!("{OPEN}{}){ret}", params.join(", "))
}

/// Read the short name of a function kind, each part read with `read`.
///
/// # Arguments
///
/// * `text` - The name, with no parentheses around it.
/// * `read` - Reads the name of a part.
///
/// # Returns
///
/// The kind, or `None` for text that does not read.
pub fn parse_fn_with(
    text: &str, read: &mut impl FnMut(&str) -> Option<DataKind>,
) -> Option<DataKind>
{
    let parts = split_fn(text)?;
    let mut params = Vec::new();
    for (edit, kind, mark) in &parts.params
    {
        let kind = read(kind)?;
        params.push(FnParam { kind, edit: *edit, mark: mark.map(Name::new) });
    }
    let ret = match parts.ret
    {
        Some(r) => Some(read(r)?),
        None => None,
    };
    Some(DataKind::Fn(FnKind::make(params, ret)))
}

/// Whether text names a function kind, in parentheses or not.
///
/// # Arguments
///
/// * `text` - The name.
///
/// # Returns
///
/// `true` if it does.
pub fn is_fn_text(text: &str) -> bool { unwrap_parens(text).starts_with(OPEN) }

/// Map each part of the short name of a function kind, and write the name
/// again in parentheses, as it stands inside another name.
///
/// # Arguments
///
/// * `text` - The name, in parentheses or not.
/// * `f` - Maps the text of a part.
///
/// # Returns
///
/// The name, or `None` when the text or a part does not read.
pub fn map_fn_text(
    text: &str, f: &mut impl FnMut(&str) -> Option<String>,
) -> Option<String>
{
    let parts = split_fn(unwrap_parens(text))?;
    let mut params = Vec::new();
    for (edit, kind, mark) in &parts.params
    {
        params.push((*edit, f(kind)?, *mark));
    }
    let ret = match parts.ret
    {
        Some(r) => Some(f(r)?),
        None => None,
    };
    let params = params.iter().map(|(e, k, m)| (*e, k.as_str(), *m)).collect();
    Some(format!("({})", join_fn(&FnText { params, ret: ret.as_deref() })))
}
