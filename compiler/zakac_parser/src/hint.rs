//! Written-as hints: the words a generic kind or trait is written with,
//! `ရေးသားရာတွင် K ဖြင့် ညွှန်းသော V ဇယားဟု ရေးသားနိုင်သည်`. Each word of a hint
//! that is a kind parameter of its declaration is a slot, where a use writes a
//! kind; the other words stand as written, and the name of the kind or trait
//! ends the hint.

use std::sync::OnceLock;

use bon::Builder;
use zakac_ast::expr::Ident;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::TableParts;
use zakac_ast::kind::Wrap;
use zakac_ast::stmt::KindParam;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_diagnostics::parser as diag;
use zakac_diagnostics::parser::HintFault;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex_at;
use zakac_lexer::reserved::LIST_NAME;
use zakac_lexer::reserved::TABLE_NAME;
use zakac_lexer::spelling::canonical;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

use crate::describe::quote;
use crate::fn_kind::FnForm;
use crate::load::CORE_SOURCE;
use crate::parser::PResult;
use crate::parser::Parser;

/// One part of a written-as hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintPart
{
    /// A kind a use writes here: the index of the kind argument it gives.
    Slot(usize),
    /// A word a use writes as it stands.
    Word(String),
}

/// The written-as hint of one generic kind or trait.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Hint
{
    /// The name of the kind or trait.
    pub name: String,
    /// The parts in order; the name is the last.
    pub parts: Vec<HintPart>,
    /// The number of kind arguments a use gives: each kind parameter, but
    /// the first of a trait.
    pub arg_count: usize,
    /// The name of the kind parameter of each slot index, for reports.
    #[builder(default)]
    pub slots: Vec<String>,
    /// Whether the hint writes a trait, not a kind.
    #[builder(default)]
    pub is_trait: bool,
}

impl Hint
{
    /// Read a hint from its words, when the words keep the rules of its form.
    ///
    /// # Arguments
    ///
    /// * `name` - The kind or trait.
    /// * `params` - The names of its kind parameters, in order.
    /// * `words` - The words of the hint.
    /// * `is_trait` - Whether it writes a trait.
    ///
    /// # Returns
    ///
    /// The hint, or `None` for one that breaks a rule.
    pub fn read(
        name: &str, params: &[&str], words: &[&str], is_trait: bool,
    ) -> Option<Hint>
    {
        ret_if!(hint_fault(name, params, words, is_trait).is_some(), None);
        let given = given_params(params, is_trait);
        let part = |w: &&str| {
            let slot = given.iter().position(|p| p == w);
            slot.map_or_else(|| HintPart::Word((*w).to_string()), HintPart::Slot)
        };
        let parts = words.iter().map(part).collect();
        #[rustfmt::skip]
        let slots = given.iter().map(|p| (*p).to_string()).collect();
        return Some(
            Hint::builder()
                .name(name.to_string())
                .parts(parts)
                .arg_count(given.len())
                .slots(slots)
                .is_trait(is_trait)
                .build(),
        );
    }

    /// Whether a hint starts with a slot, so it follows a kind as a level.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn starts_with_slot(&self) -> bool
    {
        matches!(self.parts.first(), Some(HintPart::Slot(_)))
    }
}

/// The kind parameters a use of a hint writes: each one, but the first of a
/// trait, which the sentence around the use names.
///
/// # Arguments
///
/// * `params` - The names of the kind parameters.
/// * `is_trait` - Whether the parameters belong to a trait.
///
/// # Returns
///
/// The names.
fn given_params<'p>(params: &'p [&'p str], is_trait: bool) -> &'p [&'p str]
{
    match is_trait
    {
        true => params.get(1..).unwrap_or_default(),
        false => params,
    }
}

/// The rule of its form that a hint breaks.
///
/// # Arguments
///
/// * `name` - The kind or trait.
/// * `params` - The names of its kind parameters, in order.
/// * `words` - The words of the hint.
/// * `is_trait` - Whether it writes a trait.
///
/// # Returns
///
/// The rule, or `None` for a hint that keeps each rule.
pub(crate) fn hint_fault(
    name: &str, params: &[&str], words: &[&str], is_trait: bool,
) -> Option<HintFault>
{
    let given = given_params(params, is_trait);
    let count = |p: &str| words.iter().filter(|w| **w == p).count();
    let first = params.first().filter(|p| is_trait && count(p) > 0);
    ret_if!(let Some(p) = first, Some(HintFault::First(p.to_string())));
    let missing = given.iter().find(|p| count(p) == 0);
    ret_if!(let Some(p) = missing, Some(HintFault::Missing(p.to_string())));
    // The name compares in the spelling the compiler keeps: `ဒေတာ` in a name
    // reads as `အချက်အလက်`.
    let names = words.iter().filter(|w| canonical(w) == canonical(name)).count();
    let last =
        words.last().is_some_and(|w| canonical(w) == canonical(name)) && names == 1;
    ret_if!(!last, Some(HintFault::Last));
    let slot = |w: &&str| given.contains(w);
    let meet = words.windows(2).find(|pair| slot(&pair[0]) && slot(&pair[1]));
    meet.map(|pair| HintFault::Meet(pair[0].to_string(), pair[1].to_string()))
}

/// The words of hints that a use can spell in a second form, each with its
/// twin: `<kind> ပို့ဆောင်နိုင်သော ဒေတာလမ်းကြောင်း` reads as `<kind> ပို့နိုင်သော
/// ဒေတာလမ်းကြောင်း`.
const HINT_TWINS: [(&str, &str); 1] = [("ပို့နိုင်သော", "ပို့ဆောင်နိုင်သော")];

/// The word that closes a hint, `ဟု`, which can stand glued to the name the
/// hint ends in: `အတွဲဟု`.
const HINT_CLOSE: &str = "ဟု";

/// The words of a hint from its tokens after `ရေးသားရာတွင်`, up to the `ဟု`
/// that closes it, or up to the word that ends the clause when `ဟု` stands
/// glued to the last word.
///
/// # Arguments
///
/// * `tokens` - The tokens after `ရေးသားရာတွင်`.
/// * `word` - The text of a token.
///
/// # Returns
///
/// The words and the number of tokens the words take with their `ဟု`, or `None`
/// when no `ဟု` closes them.
fn hint_words(
    tokens: &[Token], word: impl Fn(&Token) -> String,
) -> Option<(Vec<String>, usize)>
{
    let ends =
        |t: &Token| matches!(t.kind, TokenKind::Kw(Kw::CanWrite | Kw::CanWriteAnd));
    let end = tokens.iter().position(|t| t.kind == TokenKind::Kw(Kw::Hu) || ends(t))?;
    let mut words: Vec<String> = tokens[..end].iter().map(&word).collect();
    ret_if!(tokens[end].kind == TokenKind::Kw(Kw::Hu), Some((words, end + 1)));
    let last = words.pop()?;
    words.push(last.strip_suffix(HINT_CLOSE)?.to_string());
    Some((words, end))
}

/// The hints of the core, read once from its tokens.
///
/// # Returns
///
/// The hints.
pub(crate) fn core_hints() -> &'static [Hint]
{
    static HINTS: OnceLock<Vec<Hint>> = OnceLock::new();
    HINTS.get_or_init(|| {
        let src = CORE_SOURCE;
        let lexed = lex_at(src, Pos::ZERO, &LexerOptions::default());
        declared_hints(src, Pos::ZERO, &lexed.tokens)
    })
}

/// The hints a file declares, read from its tokens before it is parsed, so a
/// use can come before the declaration. A hint that breaks a rule is left
/// out; its declaration reports it.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `tokens` - The tokens of `text`.
///
/// # Returns
///
/// The hints.
pub(crate) fn declared_hints(text: &str, base: Pos, tokens: &[Token]) -> Vec<Hint>
{
    let word = |t: &Token| {
        let (lo, hi) = ((t.span.lo - base) as usize, (t.span.hi - base) as usize);
        text[lo..hi].to_string()
    };
    let opens = tokens.iter().enumerate();
    let opens = opens.filter(|(_, t)| t.kind == TokenKind::Kw(Kw::WritingOpen));
    let read = |at: usize| {
        let stop = TokenKind::Punct(Punct::FullStop);
        let start =
            tokens[..at].iter().rposition(|t| t.kind == stop).map_or(0, |n| n + 1);
        let head = &tokens[start..at];
        let (words, _) = hint_words(&tokens[at + 1..], word)?;
        let name = word(head.first()?);
        let params: Vec<String> = head_params(head).iter().map(|t| word(t)).collect();
        let is_trait = head.iter().any(|t| t.kind == TokenKind::Kw(Kw::Trait));
        let params: Vec<&str> = params.iter().map(String::as_str).collect();
        let words: Vec<&str> = words.iter().map(String::as_str).collect();
        Hint::read(&name, &params, &words, is_trait)
    };
    opens.filter_map(|(at, _)| read(at)).collect()
}

/// The kind parameters in the head of a declaration: the name after each
/// `ယေဘုယျဒေတာအမျိုးအစား`, or the names in the `<…>` after the declared name.
///
/// # Arguments
///
/// * `head` - The tokens of the declaration before its hint.
///
/// # Returns
///
/// The tokens of the parameter names.
fn head_params(head: &[Token]) -> Vec<&Token>
{
    let pairs = head.windows(2);
    let pairs = pairs.filter(|pair| pair[0].kind == TokenKind::Kw(Kw::GenericKind));
    let words: Vec<&Token> = pairs.map(|pair| &pair[1]).collect();
    ret_if!(!words.is_empty(), words);
    let angle = head.get(1).is_some_and(|t| t.kind == TokenKind::Punct(Punct::Lt));
    ret_if!(!angle, Vec::new());
    let inside = head[2..].iter().take_while(|t| t.kind != TokenKind::Punct(Punct::Gt));
    inside.filter(|t| t.kind == TokenKind::Ident).collect()
}

/// The hints of the kinds and traits a file exports.
///
/// # Arguments
///
/// * `program` - The program of the file.
///
/// # Returns
///
/// The hints.
pub(crate) fn exported_hints(program: &Program) -> Vec<Hint>
{
    let read = |name: &Ident, params: &[KindParam], words: &[Ident], is_trait| {
        let params: Vec<&str> = params.iter().map(|p| p.name.name.as_str()).collect();
        let words: Vec<&str> = words.iter().map(|w| w.name.as_str()).collect();
        ret_if!(words.is_empty(), None);
        Hint::read(name.name.as_str(), &params, &words, is_trait)
    };
    let hints = program.stmts.iter().filter_map(|s| match s
    {
        Stmt::Struct(d) if d.exported => read(&d.name, &d.kind_params, &d.hint, false),
        Stmt::Trait(d) if d.exported => read(&d.name, &d.kind_params, &d.hint, true),
        _ => None,
    });
    hints.collect()
}

impl Parser<'_>
{
    /// Take the hints the parser knows: those of the core, then the given
    /// ones, a later hint of a name in place of an earlier one.
    ///
    /// # Arguments
    ///
    /// * `hints` - The hints the file imports, then its own.
    pub(crate) fn set_hints(&mut self, hints: impl IntoIterator<Item = Hint>)
    {
        let mut known = core_hints().to_vec();
        for hint in hints
        {
            known.retain(|h| h.name != hint.name || h.is_trait != hint.is_trait);
            known.push(hint);
        }
        self.hints = known;
    }

    /// Whether the token `n` places ahead is the word `word`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    /// * `word` - The word.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    pub(crate) fn is_word_at(&self, n: usize, word: &str) -> bool
    {
        let tok = self.peek_n(n);
        let plain = matches!(tok.kind, TokenKind::Ident | TokenKind::Kw(_));
        plain && self.text_of(tok.span) == word
    }

    /// Whether the token `n` places ahead is a word of a hint, or a twin
    /// spelling of it.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    /// * `word` - The word the hint holds.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    fn is_hint_word_at(&self, n: usize, word: &str) -> bool
    {
        let twin = HINT_TWINS.iter().find(|(w, _)| *w == word);
        self.is_word_at(n, word) || twin.is_some_and(|(_, t)| self.is_word_at(n, t))
    }

    /// Read the word `word`, which must come next.
    ///
    /// # Arguments
    ///
    /// * `word` - The word.
    ///
    /// # Returns
    ///
    /// Its token, or an `expected-token` diagnostic.
    pub(crate) fn expect_word(&mut self, word: &str) -> PResult<Token>
    {
        ret_if!(!self.is_word_at(0, word), Err(self.err_expected(&quote(word))));
        Ok(self.bump())
    }

    /// Whether the token `n` places ahead names the list or the table kind
    /// of the core, which no declaration of the file hides.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    /// * `name` - [`LIST_NAME`] or [`TABLE_NAME`].
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn is_builtin_word_at(&self, n: usize, name: &str) -> bool
    {
        self.peek_n(n).kind == TokenKind::Ident
            && self.is_word_at(n, name)
            && !self.structs.contains(name)
    }

    /// Whether `စာရင်း<…>` or `ဇယား<…>` starts `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn generic_builtin_at(&self, n: usize) -> bool
    {
        self.is_generic_name_at(n) && self.angle_len(n + 1).is_some()
    }

    /// Whether the token `n` places ahead is `စာရင်း` or `ဇယား`, the generic
    /// kinds of the core, which no declaration of the file hides.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    pub(crate) fn is_generic_name_at(&self, n: usize) -> bool
    {
        [LIST_NAME, TABLE_NAME].iter().any(|w| self.is_builtin_word_at(n, w))
    }

    /// Read `စာရင်း` or `ဇယား` with its kind arguments as the list or the
    /// table kind; leave each other kind as it is.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The kind.
    pub(crate) fn builtin_generic(&self, mut kind: Kind) -> Kind
    {
        let name = kind.name.as_ref().filter(|_| kind.base == BaseKind::Named);
        let Some(name) = name.map(|n| n.name.as_str().to_string())
        else
        {
            return kind;
        };
        ret_if!(self.structs.contains(&name), kind);
        match (name.as_str(), kind.args.len())
        {
            (LIST_NAME, 1) =>
            {
                let mut cell = kind.args.remove(0);
                cell.wraps.push(Wrap::List(Cells::Inferred));
                cell.wraps.extend(kind.wraps);
                cell.classifier = kind.classifier;
                cell.span = kind.span;
                cell
            }
            (TABLE_NAME, 2) =>
            {
                let value = kind.args.pop().expect("two kind arguments");
                let index = kind.args.pop().expect("two kind arguments");
                let parts = TableParts::builder().index(index).value(value).build();
                #[rustfmt::skip]
                return Kind::builder().base(BaseKind::Table).table(Box::new(parts))
                    .wraps(kind.wraps).maybe_classifier(kind.classifier).span(kind.span).build();
            }
            _ => kind,
        }
    }

    /// The token count of hint parts written `at` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `parts` - The parts.
    /// * `at` - How far ahead.
    /// * `form` - The bare function kinds a slot can be.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when the parts are not written there.
    fn parts_len(&self, parts: &[HintPart], at: usize, form: FnForm) -> Option<usize>
    {
        let mut n = at;
        for part in parts
        {
            n += match part
            {
                HintPart::Word(w) => self.is_hint_word_at(n, w).then_some(1)?,
                HintPart::Slot(_) => self.kind_len_at(n, form)?,
            };
        }
        Some(n - at)
    }

    /// The longest hint written `at` tokens ahead, among those `pick` takes.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    /// * `pick` - Which hints to try, and the count of their first parts read
    ///   before.
    ///
    /// # Returns
    ///
    /// The index of the hint and the token count of its unread parts, or
    /// `None` when none is written there.
    fn longest_hint(
        &self, at: usize, pick: impl Fn(&Hint) -> Option<usize>,
    ) -> Option<(usize, usize)>
    {
        let form = self.fn_form();
        let found = self.hints.iter().enumerate().filter_map(|(i, h)| {
            let skip = pick(h)?;
            self.parts_len(&h.parts[skip..], at, form).map(|len| (i, len))
        });
        found.max_by_key(|(_, len)| *len)
    }

    /// The hint of a kind that follows the kind before `at` as a level, as
    /// `ဖြင့် ညွှန်းသော V ဇယား` follows `K`. The list of the core is read as
    /// a level of its own, with its counts.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead the part after the first slot starts.
    ///
    /// # Returns
    ///
    /// The index of the hint and the token count after the first slot.
    pub(crate) fn hint_level_at(&self, at: usize) -> Option<(usize, usize)>
    {
        let builtin_list = !self.structs.contains(LIST_NAME);
        self.longest_hint(at, |h| {
            let list = builtin_list && h.name == LIST_NAME;
            (!h.is_trait && !list && h.starts_with_slot()).then_some(1)
        })
    }

    /// The hint of a kind that starts with a word, written `at` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The index of the hint and its token count.
    pub(crate) fn hint_kind_at(&self, at: usize) -> Option<(usize, usize)>
    {
        self.longest_hint(at, |h| (!h.is_trait && !h.starts_with_slot()).then_some(0))
    }

    /// The hint of a trait written `at` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The index of the hint and its token count.
    pub(crate) fn hint_trait_at(&self, at: usize) -> Option<(usize, usize)>
    {
        self.longest_hint(at, |h| h.is_trait.then_some(0))
    }

    /// Parse a use of a hint: its parts, the slot that a level hint follows
    /// given as `first`.
    ///
    /// # Arguments
    ///
    /// * `idx` - The index of the hint.
    /// * `first` - The kind before a level hint, or `None`.
    /// * `start` - Where the kind phrase starts.
    /// * `form` - The bare function kinds a slot can be.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    pub(crate) fn parse_hint(
        &mut self, idx: usize, first: Option<Kind>, start: Span, form: FnForm,
    ) -> PResult<Kind>
    {
        let hint = self.hints[idx].clone();
        let mut args: Vec<Option<Kind>> = vec![None; hint.arg_count];
        let mut parts = hint.parts.iter();
        if let Some(kind) = first
            && let Some(HintPart::Slot(i)) = parts.next()
        {
            args[*i] = Some(kind);
        }
        let mut name_span = start;
        for part in parts
        {
            match part
            {
                HintPart::Word(_) => name_span = self.bump().span,
                HintPart::Slot(i) =>
                {
                    let kind = self.parse_kind_with(form)?;
                    self.fill_slot(&hint, &mut args[*i], *i, kind);
                }
            }
        }
        let name = Ident::builder().name(hint.name.as_str()).span(name_span).build();
        let args = args.into_iter().flatten().collect();
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        let kind = Kind::builder().base(BaseKind::Named).name(name).args(args).span(span)
            .build();
        Ok(self.builtin_generic(kind))
    }

    /// Give a slot of a hint use its kind. A kind parameter that stands twice
    /// in the hint takes the kind its first place gives; another kind in a
    /// later place is ZK0253.
    ///
    /// # Arguments
    ///
    /// * `hint` - The hint.
    /// * `arg` - The kind argument of the slot, when an earlier place gave it.
    /// * `slot` - The index of the slot.
    /// * `kind` - The kind this place writes.
    fn fill_slot(
        &mut self, hint: &Hint, arg: &mut Option<Kind>, slot: usize, kind: Kind,
    )
    {
        let Some(held) = arg
        else
        {
            *arg = Some(kind);
            return;
        };
        ret_if!(held.same_as(&kind));
        let param = hint.slots.get(slot).map_or("", String::as_str);
        self.diags.push(diag::hint_slot_two_kinds(kind.span, &hint.name, param));
    }

    /// Parse the hint of a declaration, `ရေးသားရာတွင် <words> ဟု`, when it
    /// comes next, and report a rule it breaks.
    ///
    /// # Arguments
    ///
    /// * `name` - The kind or trait the declaration names.
    /// * `params` - Its kind parameters.
    /// * `is_trait` - Whether it is a trait.
    ///
    /// # Returns
    ///
    /// The words, none when no hint comes next, or a diagnostic for a sign
    /// among them or a missing `ဟု`.
    pub(crate) fn parse_hint_decl(
        &mut self, name: &Ident, params: &[KindParam], is_trait: bool,
    ) -> PResult<Vec<Ident>>
    {
        ret_if!(!self.eat(Kw::WritingOpen), Ok(Vec::new()));
        let text = name.name.as_str();
        let tokens = self.cur.rest();
        let found = hint_words(tokens, |t| self.text_of(t.span).to_string());
        let Some((texts, len)) = found
        else
        {
            return Err(self.err_expected(&quote(HINT_CLOSE)));
        };
        let sign = tokens[..texts.len()].iter().find(|t| !is_hint_word(t));
        if let Some(tok) = sign
        {
            let fault = HintFault::Sign(self.text_of(tok.span).to_string());
            return Err(diag::bad_hint(tok.span, text, &fault));
        }
        let ident = |(t, w): (&Token, &String)| {
            Ident::builder().name(w.as_str()).span(t.span).build()
        };
        let words: Vec<Ident> = tokens.iter().zip(&texts).map(ident).collect();
        do_for!(_ in 0..len, self.advance());
        let span = words.first().map(|w| w.span.to(&words[words.len() - 1].span));
        let params: Vec<&str> = params.iter().map(|p| p.name.name.as_str()).collect();
        let texts: Vec<&str> = texts.iter().map(String::as_str).collect();
        let fault = hint_fault(text, &params, &texts, is_trait);
        let fault = fault.map(|f| diag::bad_hint(span.unwrap_or(name.span), text, &f));
        self.diags.extend(fault);
        Ok(words)
    }
}

/// Whether a token can be a word of a hint: a name or a reserved word.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// `true` if it can.
fn is_hint_word(tok: &Token) -> bool
{
    matches!(tok.kind, TokenKind::Ident | TokenKind::Kw(_))
}
