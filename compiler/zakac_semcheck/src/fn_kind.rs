//! Function kinds: the kind of a function value, its parameters (each with
//! its `ဒေတာ ပြောင်းလဲရန်` flag and its preposition) and its result.
//!
//! A function kind writes its short name as `fn(<param>, …) -> <result>`,
//! each parameter as `[mut ]<kind>[(<preposition>)]`. Inside another name (a
//! cell, a pointer base, a parameter, a result or a field) it stands in
//! parentheses, `(fn(isize) -> bool)[]`, so the levels after it never read as
//! levels of its result.

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use strum::IntoStaticStr;
use zakac_ast::kind::TRAIT_HEAD;
use zakac_ast::name::Name;
use zakac_rust_sugars::do_for;
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
    /// Whether the function writes the argument before it reads it,
    /// `ဒေတာ ထည့်ပေးရန်`, so the caller need not set it; [`FnParam::edit`] holds
    /// too.
    pub fill: bool,
    /// The preposition after the argument in a preposition call, when the
    /// parameter has one.
    pub preposition: Option<Name>,
}

/// How a parameter of a function kind passes its argument, with the text
/// its short name writes before the kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, IntoStaticStr)]
pub enum ParamMode
{
    /// A value.
    #[default]
    #[strum(serialize = "")]
    Plain,
    /// `ဒေတာ ပြောင်းလဲရန်`: the caller's place, set before the call.
    #[strum(serialize = "mut ")]
    Edit,
    /// `ဒေတာ ထည့်ပေးရန်`: the caller's place, which the function sets.
    #[strum(serialize = "out ")]
    Fill,
}

impl ParamMode
{
    /// The mode of a parameter with these marks.
    ///
    /// # Arguments
    ///
    /// * `edit` - Whether the function can change the argument.
    /// * `fill` - Whether the function sets it before it reads it.
    ///
    /// # Returns
    ///
    /// The mode.
    pub fn of(edit: bool, fill: bool) -> ParamMode
    {
        tern!(fill, ParamMode::Fill, tern!(edit, ParamMode::Edit, ParamMode::Plain))
    }
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
    /// The starting phrase a call through a value of the kind writes.
    pub start: Option<Name>,
    /// The ending phrase a call through a value of the kind writes.
    pub end: Option<Name>,
    /// The trait of a trait value, `<trait> အရာ`: its name, then its kinds
    /// after the first as `<k၁,k၂>` when it has more. A trait value is a
    /// function value that runs the function of the kind it holds; `None`
    /// for each other function kind.
    pub of_trait: Option<Name>,
}

/// Each function kind made so far, so a kind is made once and remains
/// `Copy`.
static FNS: OnceLock<Mutex<HashSet<&'static FnKind>>> = OnceLock::new();

/// The text that opens the short name of a function kind, before its
/// phrases or its `(`.
const OPEN: &str = "fn";

/// The mark between the starting and the ending phrase in a short name,
/// where the arguments stand.
const ARGS: &str = "…";

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
        FnKind::make_with(params, ret, None, None)
    }

    /// The function kind with these parts and phrases, made once and kept for
    /// the rest of the run.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters.
    /// * `ret` - The result kind, when the function has one.
    /// * `start` - The starting phrase, when written.
    /// * `end` - The ending phrase, when written.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn make_with(
        params: Vec<FnParam>, ret: Option<DataKind>, start: Option<Name>,
        end: Option<Name>,
    ) -> &'static FnKind
    {
        FnKind::make_full(FnKind { params, ret, start, end, of_trait: None })
    }

    /// The kind of a trait value of a trait: a function value that takes the
    /// number of the trait function to run and the address of its
    /// arguments, made once and kept for the rest of the run.
    ///
    /// # Arguments
    ///
    /// * `marker` - The trait, as [`FnKind::of_trait`] writes it.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn trait_value(marker: Name) -> &'static FnKind
    {
        let param =
            |kind| FnParam { kind, edit: false, fill: false, preposition: None };
        let params =
            vec![param(DataKind::USIZE), param(DataKind::OPAQUE.with_outer_raw())];
        let wanted = FnKind {
            params,
            ret: None,
            start: None,
            end: None,
            of_trait: Some(marker),
        };
        FnKind::make_full(wanted)
    }

    /// The function kind, made once and kept for the rest of the run.
    ///
    /// # Arguments
    ///
    /// * `wanted` - The kind.
    ///
    /// # Returns
    ///
    /// The kind kept.
    fn make_full(wanted: FnKind) -> &'static FnKind
    {
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

    /// Whether the function sets each parameter before it reads it.
    ///
    /// # Returns
    ///
    /// One flag for each parameter.
    pub fn fills(&self) -> Vec<bool> { self.params.iter().map(|p| p.fill).collect() }

    /// The prepositions of the parameters.
    ///
    /// # Returns
    ///
    /// The preposition of each parameter in order, `None` where it has none.
    pub fn prepositions(&self) -> Vec<Option<Name>>
    {
        self.params.iter().map(|p| p.preposition).collect()
    }

    /// The kind with each preposition and each phrase dropped.
    ///
    /// # Returns
    ///
    /// The kind whose calls write no words beside the arguments.
    pub fn without_words(&self) -> &'static FnKind
    {
        ret_if!(self.of_trait.is_some(), FnKind::make_full(self.clone()));
        let drop = |p: &FnParam| FnParam { preposition: None, ..p.clone() };
        FnKind::make(self.params.iter().map(drop).collect(), self.ret)
    }

    /// The phrases of the kind as its short name writes them, the arguments
    /// as `…`: `{ဖိုင် … စာကို}`; blank with no phrase.
    ///
    /// # Returns
    ///
    /// The text.
    fn phrases_text(&self) -> String
    {
        ret_if!(self.start.is_none() && self.end.is_none(), String::new());
        let words = [
            self.start.map(|s| s.to_string()),
            Some(ARGS.to_string()),
            self.end.map(|e| e.to_string()),
        ];
        let words: Vec<String> = words.into_iter().flatten().collect();
        format!("{{{}}}", words.join(" "))
    }

    /// Whether a function of this kind fits a place of kind `want`: the
    /// same parameter kinds, `ဒေတာ ပြောင်းလဲရန်` flags and result, and the same
    /// prepositions when `want` has prepositions. A kind with no prepositions
    /// takes a function with prepositions or without.
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
        ret_if!(self.of_trait.is_some() || want.of_trait.is_some(), false);
        let phrased = want.start.is_some() || want.end.is_some();
        let given = phrased || want.params.iter().any(|p| p.preposition.is_some());
        let (mine, theirs) =
            tern!(given, (self, want), (self.without_words(), want.without_words()));
        // A function that sets an argument fits a kind that wants it set
        // first, but a function that reads it fits no kind that leaves it
        // unset.
        let param = |(m, w): (&FnParam, &FnParam)| {
            let words = m.preposition == w.preposition;
            m.kind == w.kind && m.edit == w.edit && words && (m.fill || !w.fill)
        };
        let phrases = (mine.start, mine.end) == (theirs.start, theirs.end);
        let same = mine.ret == theirs.ret && phrases;
        let count = mine.params.len() == theirs.params.len();
        same && count && mine.params.iter().zip(&theirs.params).all(param)
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
        ret_if!(!full && self.of_trait.is_some(), self.trait_value_name());
        let marker = self.of_trait.map(|t| t.to_string());
        let param = |p: &FnParam| {
            let edit: &str = ParamMode::of(p.edit, p.fill).into();
            let preposition = p.preposition.map_or(String::new(), |m| format!("({m})"));
            format!("{edit}{}{preposition}", p.kind.spell_inner(full))
        };
        let params: Vec<String> = self.params.iter().map(param).collect();
        let ret = self
            .ret
            .map_or(String::new(), |r| format!("{ARROW}{}", r.spell_inner(full)));
        let marker = marker.map_or(String::new(), |t| format!("<{t}>"));
        format!("{OPEN}{marker}{}({}){ret}", self.phrases_text(), params.join(", "))
    }

    /// The short name of a trait value kind: the trait, its kinds after the
    /// first under their short names, then `အရာ`.
    ///
    /// # Returns
    ///
    /// The name.
    fn trait_value_name(&self) -> String
    {
        let marker = self.of_trait.map_or_else(String::new, |t| t.to_string());
        let Some((name, args)) = self.trait_parts().filter(|(_, a)| !a.is_empty())
        else
        {
            return format!("{marker} {TRAIT_HEAD}");
        };
        let args: Vec<String> = args.iter().map(|k| k.name()).collect();
        format!("{name}<{}> {TRAIT_HEAD}", args.join("၊ "))
    }

    /// The trait of a trait value and its kinds after the first.
    ///
    /// # Returns
    ///
    /// The name of the trait and its kinds, or `None` for a plain function
    /// kind or a marker that does not read.
    pub fn trait_parts(&self) -> Option<(Name, Vec<DataKind>)>
    {
        let marker = self.of_trait?;
        let text = marker.as_str();
        let Some((name, args)) = text.split_once('<')
        else
        {
            return Some((marker, Vec::new()));
        };
        let args = args.strip_suffix('>')?;
        let kinds: Option<Vec<DataKind>> = split_top(args, ',')
            .into_iter()
            .map(|a| DataKind::parse(a.trim()))
            .collect();
        Some((Name::new(name), kinds?))
    }
}

/// The marker of a trait value kind, [`FnKind::of_trait`], from the trait's
/// name and its kinds after the first.
///
/// # Arguments
///
/// * `name` - The name of the trait.
/// * `args` - Its kinds after the first.
///
/// # Returns
///
/// The marker.
pub fn trait_marker(name: &str, args: &[DataKind]) -> Name
{
    ret_if!(args.is_empty(), Name::new(name));
    let args: Vec<String> = args.iter().map(|k| k.spell_inner(true)).collect();
    Name::from(format!("{name}<{}>", args.join(",")))
}

/// The text form of a function kind, split into its parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnText<'a>
{
    /// Each parameter: how it passes its argument, its kind text and its
    /// preposition.
    pub params: Vec<(ParamMode, &'a str, Option<&'a str>)>,
    /// The result kind text.
    pub ret: Option<&'a str>,
    /// The phrases inside their braces, `ဖိုင် … စာကို`, when written.
    pub phrases: Option<&'a str>,
    /// The trait of a trait value inside its angle brackets, when written.
    pub of_trait: Option<&'a str>,
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
    let angled = rest.strip_prefix('<').and_then(|r| {
        let close = matching_angle(r)?;
        Some((&r[..close], &r[close + 1..]))
    });
    let of_trait = angled.map(|(t, _)| t);
    let rest = angled.map_or(rest, |(_, r)| r);
    let braced = rest.strip_prefix('{').and_then(|r| r.split_once('}'));
    let phrases = braced.map(|(p, _)| p);
    let rest = braced.map_or(rest, |(_, r)| r).strip_prefix('(')?;
    let close = matching_close(rest)?;
    let (inner, tail) = (&rest[..close], &rest[close + 1..]);
    let ret = tern!(tail.is_empty(), None, Some(tail.strip_prefix(ARROW)?));
    let parts = tern!(inner.is_empty(), Vec::new(), split_top(inner, ','));
    let params = parts.into_iter().map(|p| split_param(p.trim())).collect();
    Some(FnText { params, ret, phrases, of_trait })
}

/// The place of the `>` that closes text whose `<` came just before it.
///
/// # Arguments
///
/// * `text` - The text after the `<`.
///
/// # Returns
///
/// The octet offset of the `>`, or `None`.
fn matching_angle(text: &str) -> Option<usize>
{
    let (mut depth, mut prev) = (1i32, Some('<'));
    for (at, ch) in text.char_indices()
    {
        depth += bracket_step(prev, ch);
        prev = Some(ch);
        ret_if!(depth == 0, Some(at));
    }
    None
}

/// The starting and the ending phrase of the phrases of a short name,
/// `ဖိုင် … စာကို`.
///
/// # Arguments
///
/// * `text` - The text inside the braces.
///
/// # Returns
///
/// The two phrases, each `None` when not written.
fn split_phrases(text: &str) -> (Option<Name>, Option<Name>)
{
    let (start, end) = text.split_once(ARGS).unwrap_or((text, ""));
    let word = |w: &str| Some(w.trim()).filter(|w| !w.is_empty()).map(Name::new);
    (word(start), word(end))
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

/// Split one parameter of a function kind name: `[mut
/// ]<kind>[(<preposition>)]`.
///
/// # Arguments
///
/// * `text` - The parameter.
///
/// # Returns
///
/// Whether `mut` marks it, the kind text and the mark.
fn split_param(text: &str) -> (ParamMode, &str, Option<&str>)
{
    let marked = [ParamMode::Edit, ParamMode::Fill].into_iter().find_map(|m| {
        let prefix: &str = m.into();
        text.strip_prefix(prefix).map(|t| (m, t))
    });
    let (edit, text) = marked.unwrap_or((ParamMode::Plain, text));
    let Some(open) = mark_open(text)
    else
    {
        return (edit, text, None);
    };
    (edit, &text[..open], Some(&text[open + 1..text.len() - 1]))
}

/// The place of the `(` of a preposition at the end of a parameter: a `)` last,
/// whose `(` does not open the parameter.
///
/// # Arguments
///
/// * `text` - The parameter, `mut` taken off.
///
/// # Returns
///
/// The octet offset of the `(`, or `None` for a parameter with no preposition.
fn mark_open(text: &str) -> Option<usize>
{
    ret_if!(!text.ends_with(')'), None);
    let open = text.rfind('(')?;
    // A preposition is a word with no brackets in it.
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
    let param = |(mode, kind, preposition): &(ParamMode, &str, Option<&str>)| {
        let preposition = preposition.map_or(String::new(), |m| format!("({m})"));
        let mode: &str = (*mode).into();
        format!("{mode}{kind}{preposition}")
    };
    let params: Vec<String> = parts.params.iter().map(param).collect();
    let ret = parts.ret.map_or(String::new(), |r| format!("{ARROW}{r}"));
    let phrases = parts.phrases.map_or(String::new(), |p| format!("{{{p}}}"));
    let marker = parts.of_trait.map_or(String::new(), |t| format!("<{t}>"));
    format!("{OPEN}{marker}{phrases}({}){ret}", params.join(", "))
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
    for (edit, kind, preposition) in &parts.params
    {
        let kind = read(kind)?;
        params.push(FnParam {
            kind,
            edit: *edit != ParamMode::Plain,
            fill: *edit == ParamMode::Fill,
            preposition: preposition.map(Name::new),
        });
    }
    let ret = match parts.ret
    {
        Some(r) => Some(read(r)?),
        None => None,
    };
    let (start, end) = parts.phrases.map_or((None, None), split_phrases);
    let of_trait = match parts.of_trait
    {
        Some(t) => Some(read_marker(t, read)?),
        None => None,
    };
    let wanted = FnKind { params, ret, start, end, of_trait };
    Some(DataKind::Fn(FnKind::make_full(wanted)))
}

/// Read the marker of a trait value, each kind in it read with `read` and
/// written again as its signature, so the marker reads back alone.
///
/// # Arguments
///
/// * `text` - The marker.
/// * `read` - Reads the name of a kind.
///
/// # Returns
///
/// The marker, or `None` when a kind does not read.
fn read_marker(
    text: &str, read: &mut impl FnMut(&str) -> Option<DataKind>,
) -> Option<Name>
{
    let Some((name, args)) = text.split_once('<')
    else
    {
        return Some(Name::new(text));
    };
    let args = args.strip_suffix('>')?;
    let mut kinds = Vec::new();
    do_for!(a in split_top(args, ','), kinds.push(read(a.trim())?));
    Some(trait_marker(name, &kinds))
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
pub fn is_fn_text(text: &str) -> bool
{
    let rest = unwrap_parens(text).strip_prefix(OPEN);
    rest.is_some_and(|r| r.starts_with(['(', '{', '<']))
}

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
    for (edit, kind, preposition) in &parts.params
    {
        params.push((*edit, f(kind)?, *preposition));
    }
    let ret = match parts.ret
    {
        Some(r) => Some(f(r)?),
        None => None,
    };
    let of_trait = match parts.of_trait
    {
        Some(t) => Some(map_marker(t, f)?),
        None => None,
    };
    let params = params.iter().map(|(e, k, m)| (*e, k.as_str(), *m)).collect();
    #[rustfmt::skip]
    let parts = FnText { params, ret: ret.as_deref(), phrases: parts.phrases, of_trait: of_trait.as_deref() };
    Some(format!("({})", join_fn(&parts)))
}

/// Map each kind of the marker of a trait value.
///
/// # Arguments
///
/// * `text` - The marker.
/// * `f` - Maps the text of a kind.
///
/// # Returns
///
/// The marker, or `None` when a kind does not read.
fn map_marker(text: &str, f: &mut impl FnMut(&str) -> Option<String>)
-> Option<String>
{
    let Some((name, args)) = text.split_once('<')
    else
    {
        return Some(text.to_string());
    };
    let args = args.strip_suffix('>')?;
    let mut kinds = Vec::new();
    do_for!(a in split_top(args, ','), kinds.push(f(a.trim())?));
    Some(format!("{name}<{}>", kinds.join(",")))
}
