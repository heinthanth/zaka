use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NodeId;
use zakac_ast::stmt::Program;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex_at;
use zakac_lexer::reserved::PLURAL;
use zakac_lexer::spelling::canonical;
use zakac_lexer::spelling::spellings;
use zakac_lexer::spoken::spoken;
use zakac_lexer::token::Kw;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::Punct;
use zakac_lexer::token::StrLit;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;
use zakac_span::pos::Pos;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

use crate::cursor::Cursor;
use crate::describe::kw_text;
use crate::describe::one_of;
use crate::describe::quote;
use crate::generics::angle_len_with;
use crate::glued::GluedEnds;
use crate::glued::clash_warnings;
use crate::glued::glued_spoken;
use crate::glued::split_glued;
use crate::heap::HEAP_FNS;
use crate::hint::Hint;
use crate::hint::declared_hints;
use crate::load::CORE_SOURCE;
use crate::stmt::FN_MARKS;
use crate::threads::past_marks;
use crate::using::CallWords;
use crate::using::Functions;
use crate::using::preposition_words;
use crate::verb_call::VERB_FNS;

/// Result of a parse step; the error is the diagnostic to report.
pub(crate) type PResult<T> = Result<T, Diagnostic>;

/// The whole parser output for one source file.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct ParseOutput
{
    /// The program; statements with errors are left out.
    pub program: Program,
    /// Parser diagnostics, in source order.
    #[builder(default)]
    pub diagnostics: Vec<Diagnostic>,
    /// The number the next expression node would take, so the parse of
    /// another file can go on from it.
    #[builder(default)]
    pub next_id: u32,
    /// The spoken words the parser read as their second word,
    /// as `ရှိတဲ့` read as `ရှိသည့်`.
    #[builder(default)]
    pub also_taken: Vec<Span>,
    /// The spellings of the ends glued to calls.
    #[builder(default)]
    pub glued_ends: GluedEnds,
}

/// Parse the tokens of a source file.
///
/// # Arguments
///
/// * `source` - The source file the tokens come from.
/// * `lexed` - The lexer output for it, with absolute spans.
///
/// # Returns
///
/// The program and the parser diagnostics, with absolute spans.
pub fn parse(source: &SourceFile, lexed: &LexOutput) -> ParseOutput
{
    parse_at(&source.src, source.start_pos, lexed)
}

/// Parse the tokens of one file of a program read with its imports.
///
/// # Arguments
///
/// * `source` - The source file the tokens come from.
/// * `lexed` - The lexer output for it, with absolute spans.
/// * `first_id` - The number for the first expression node, so the nodes of
///   each file of the program get different numbers.
/// * `structs` - The struct names the file imports, so each of them can start a
///   kind as the names it declares do.
/// * `functions` - The functions the file imports, with the prepositions of
///   each, so a preposition call of each can be read and a glued call end
///   splits off.
/// * `hints` - The written-as hints of the kinds and traits the file imports.
///
/// # Returns
///
/// The program and the parser diagnostics, with absolute spans.
pub fn parse_with(
    source: &SourceFile, lexed: &LexOutput, first_id: u32, structs: &HashSet<String>,
    functions: &Functions, hints: &[Hint],
) -> ParseOutput
{
    let (text, base) = (source.src.as_str(), source.start_pos);
    let span = Span::new(base, base + text.len());
    ret_if!(lexed.tokens.is_empty(), blank_output(span));
    let mut known = declared_functions(text, base, lexed);
    let imported = functions.iter().flat_map(|(name, lists)| {
        spellings(name).into_iter().map(move |n| (n, lists.clone()))
    });
    imported.for_each(|(name, lists)| known.entry(name).or_default().extend(lists));
    let heap = with_core(&mut known);
    let split = split_glued(text, base, lexed, &known);
    let mut parser = Parser::new(text, base, &split);
    parser.core_heap = heap;
    parser.next_id = first_id;
    parser.structs.extend(structs.iter().flat_map(|s| spellings(s)));
    parser.diags.extend(clash_warnings(text, base, lexed, &known));
    let (spoken, ends) = glued_spoken(text, base, lexed, &known);
    parser.diags.extend(spoken);
    parser.glued_ends = ends;
    parser.set_functions(known);
    let own = declared_hints(text, base, &lexed.tokens);
    parser.set_hints(hints.iter().cloned().chain(own));
    finish(parser, span)
}

/// The functions of the operator traits of the core, which `+` and the
/// word operators call on a struct, as `ပေါင်း` for `+` and `ပေါင်းလဒ်`.
const OPERATOR_FNS: &[&str] = &[
    "ပေါင်း",
    "နုတ်",
    "မြှောက်",
    "စား",
    "အကြွင်းရှာ",
    "ဘစ်ဆုံ",
    "ဘစ်ပေါင်းစု",
    "ဘစ်ခြားနား",
    "ဘစ်ပြောင်းပြန်",
    "ဘစ်ဘယ်ရွေ့",
    "ဘစ်ညာရွေ့",
];

/// Add the functions of the core that a file does not declare or import
/// itself: each file sees them, and a function of its own hides one.
///
/// # Arguments
///
/// * `known` - The functions of the file.
///
/// # Returns
///
/// The heap functions of the core the file sees, which no function of
/// its own hides.
fn with_core(known: &mut Functions) -> HashSet<String>
{
    let heap = |name: &str| HEAP_FNS.iter().any(|(n, _)| *n == name);
    // A count the compiler gives is a reading of the clause engine, not a
    // preposition call, so its prepositions remain outside the preposition
    // calls of the file; so are the functions of the operator traits, which
    // the operators call. The heap functions and the verbs are read as the
    // sentences their heads spell.
    let sentence = |name: &str| heap(name) || VERB_FNS.contains(&name);
    let verb = |name: &str| {
        OPERATOR_FNS.contains(&name)
            || (core_builtins().contains(name) && !sentence(name))
    };
    let lists = |name: &str, lists: &Vec<CallWords>| {
        tern!(verb(name), vec![CallWords::default()], lists.clone())
    };
    let seen: HashSet<String> = HEAP_FNS
        .iter()
        .map(|(n, _)| n.to_string())
        .filter(|n| !known.contains_key(n))
        .collect();
    let core = core_functions().iter();
    core.for_each(|(name, l)| _ = known.entry(name.clone()).or_insert(lists(name, l)));
    // A verb of the core keeps its sentences beside the functions of its
    // name the file declares, which add overloads.
    let verbs =
        core_functions().iter().filter(|(name, _)| VERB_FNS.contains(&name.as_str()));
    verbs.for_each(|(name, l)| {
        let have = known.entry(name.clone()).or_default();
        let missing: Vec<CallWords> =
            l.iter().filter(|w| !have.contains(w)).cloned().collect();
        have.extend(missing);
    });
    seen
}

/// The names of the functions the core declares with the compiler mark,
/// read once from its tokens: each header from `လုပ်ဆောင်ချက်` to its full stop
/// that holds `ကွန်ပိုင်လာထဲတွင်`.
///
/// # Returns
///
/// The names.
pub(crate) fn core_builtins() -> &'static HashSet<String>
{
    static NAMES: OnceLock<HashSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let src = CORE_SOURCE;
        let lexed = lex_at(src, Pos::ZERO, &LexerOptions::default());
        let tokens = &lexed.tokens;
        let word = |t: &Token| {
            src[(t.span.lo - Pos::ZERO) as usize..(t.span.hi - Pos::ZERO) as usize]
                .to_string()
        };
        let heads = (0..tokens.len()).filter(|&at| fn_head_at(&tokens[at..]).is_some());
        let marked = heads.filter_map(|at| {
            let header = tokens[at..]
                .iter()
                .take_while(|t| t.kind != TokenKind::Punct(Punct::FullStop));
            let builtin =
                header.clone().any(|t| t.kind == TokenKind::Kw(Kw::InCompiler));
            let name = Some(&tokens[at]).filter(|t| t.kind == TokenKind::Ident);
            name.filter(|_| builtin).map(word)
        });
        marked.collect()
    })
}

/// The functions the core declares, with their prepositions, read once.
///
/// # Returns
///
/// The functions.
fn core_functions() -> &'static Functions
{
    static FNS: OnceLock<Functions> = OnceLock::new();
    FNS.get_or_init(|| {
        let src = CORE_SOURCE;
        let lexed = lex_at(src, Pos::ZERO, &LexerOptions::default());
        declared_functions(src, Pos::ZERO, &lexed)
    })
}

/// The functions a file declares, with the prepositions of each.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `lexed` - The lexer output for `text`.
///
/// # Returns
///
/// The functions, each name in both spellings of `ဒေတာ`.
fn declared_functions(text: &str, base: Pos, lexed: &LexOutput) -> Functions
{
    let parser = Parser::new(text, base, lexed);
    let heads =
        (0..lexed.tokens.len()).filter_map(|at| parser.fn_header(&lexed.tokens, at));
    let mut found = Functions::new();
    for (name, prepositions) in heads
    {
        let names = spellings(&name).into_iter();
        names.for_each(|n| found.entry(n).or_default().push(prepositions.clone()));
    }
    found
}

/// Parse the program of a parser and collect its output.
///
/// # Arguments
///
/// * `parser` - The parser at the first token.
/// * `span` - The file span.
///
/// # Returns
///
/// The program, the parser diagnostics and the next node number.
fn finish(mut parser: Parser<'_>, span: Span) -> ParseOutput
{
    let stmts = parser.parse_program();
    let program = Program::builder().stmts(stmts).span(span).build();
    #[rustfmt::skip]
    return ParseOutput::builder().program(program).diagnostics(parser.diags)
        .next_id(parser.next_id).also_taken(parser.also_taken)
        .glued_ends(parser.glued_ends).build();
}

/// Parse the tokens of `text`, which starts at the absolute position `base`.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `lexed` - The lexer output for `text`.
///
/// # Returns
///
/// The program and the parser diagnostics.
pub fn parse_at(text: &str, base: Pos, lexed: &LexOutput) -> ParseOutput
{
    let span = Span::new(base, base + text.len());
    ret_if!(lexed.tokens.is_empty(), blank_output(span));
    let mut known = declared_functions(text, base, lexed);
    let heap = with_core(&mut known);
    let split = split_glued(text, base, lexed, &known);
    let mut parser = Parser::new(text, base, &split);
    parser.core_heap = heap;
    parser.diags.extend(clash_warnings(text, base, lexed, &known));
    let (spoken, ends) = glued_spoken(text, base, lexed, &known);
    parser.diags.extend(spoken);
    parser.glued_ends = ends;
    parser.set_functions(known);
    parser.set_hints(declared_hints(text, base, &lexed.tokens));
    finish(parser, span)
}

/// Parse `text` as one expression that takes all of its tokens.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `lexed` - The lexer output for `text`.
///
/// # Returns
///
/// The expression, or the first diagnostic.
pub fn parse_lone_expr(
    text: &str, base: Pos, lexed: &LexOutput,
) -> Result<Expr, Diagnostic>
{
    let at_base = Span::new(base, base);
    ret_if!(lexed.tokens.is_empty(), Err(diag::expected_operand(at_base, "")));
    let mut known = Functions::new();
    _ = with_core(&mut known);
    let split = split_glued(text, base, lexed, &known);
    let mut parser = Parser::new(text, base, &split);
    parser.set_functions(known);
    parser.set_hints(Vec::new());
    let expr = parser.parse_expr()?;
    ret_if!(!parser.at_eof(), Err(parser.err_expected(&t!("parser.eof"))));
    Ok(expr)
}

/// Output for a token list without even an end-of-input token.
///
/// # Arguments
///
/// * `span` - The file span.
///
/// # Returns
///
/// A blank program.
fn blank_output(span: Span) -> ParseOutput
{
    ParseOutput::builder().program(Program::builder().span(span).build()).build()
}

/// Whether the head of a function declaration starts at the first token:
/// `<name> [<…>] ဟူသော [<marks>] လုပ်ဆောင်ချက် သည်`, with the marks of
/// [`FN_MARKS`].
///
/// # Arguments
///
/// * `tokens` - The tokens from the first one on.
///
/// # Returns
///
/// The index of `လုပ်ဆောင်ချက်` in `tokens`, or `None` for no head.
pub(crate) fn fn_head_at(tokens: &[Token]) -> Option<usize>
{
    let kind_at = |n: usize| tokens.get(n).map_or(TokenKind::Eof, |t| t.kind);
    ret_if!(!matches!(kind_at(0), TokenKind::Ident | TokenKind::RawIdent), None);
    let called = 1 + angle_len_with(1, kind_at).unwrap_or(0);
    ret_if!(kind_at(called) != TokenKind::Kw(Kw::Called), None);
    let mark = |n: usize| FN_MARKS.iter().any(|m| kind_at(n) == TokenKind::Kw(*m));
    // A `၊` between two marks belongs to the marks.
    let sep = |n: usize| kind_at(n) == TokenKind::Punct(Punct::ListSep) && mark(n + 1);
    let noun = (called + 1..).find(|n| !mark(*n) && !sep(*n))?;
    let head = kind_at(noun) == TokenKind::Kw(Kw::Function)
        && kind_at(noun + 1) == TokenKind::Kw(Kw::Thi);
    tern!(head, Some(noun), None)
}

/// The reserved word of a token.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// The reserved word, or `None` for another token.
pub(crate) fn kw_of(tok: &Token) -> Option<Kw>
{
    match tok.kind
    {
        TokenKind::Kw(kw) => Some(kw),
        _ => None,
    }
}

/// Whether the token at `at` is the name of a kind declaration: a struct or
/// an enumeration, `<name> ဟူသော [marks] ဒေတာအစုအဝေး|ဒေတာအမျိုးအစားကွဲ`, or an
/// alias, `ဒေတာအမျိုးအစား <name> ကို`, or `ဒေတာအမျိုးအစား <name> သည်` before
/// its kind parameters in the word form.
///
/// # Arguments
///
/// * `tokens` - The tokens of the file.
/// * `at` - The index of the name.
///
/// # Returns
///
/// `true` for such a name.
fn declares_kind(
    tokens: &[Token], at: usize, text_of: &dyn Fn(&Token) -> String,
) -> bool
{
    let kw = |n: usize| match tokens.get(n).map(|t| t.kind)
    {
        Some(TokenKind::Kw(kw)) => Some(kw),
        _ => None,
    };
    ret_false!(tokens[at].kind != TokenKind::Ident);
    // A generic declaration has its kind parameters right after its name.
    let kind_at = |n: usize| tokens.get(n).map_or(TokenKind::Eof, |t| t.kind);
    let next = at + 1 + angle_len_with(at + 1, kind_at).unwrap_or(0);
    let alias = at > 0 && kw(at - 1) == Some(Kw::DataKindOpen);
    ret_true!(alias && matches!(kw(next), Some(Kw::Ko | Kw::Thi)));
    ret_false!(kw(next) != Some(Kw::Called));
    matches!(
        kw(past_marks(tokens, next + 1, text_of)),
        Some(Kw::StructWord | Kw::EnumWord)
    )
}

/// Whether a token is a number.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// `true` for a number token.
pub(crate) fn is_number(tok: &Token) -> bool
{
    matches!(tok.kind, TokenKind::Number(_))
}

/// The most notation expressions, and the most blocks, that nest inside
/// one another: deeper input gets a diagnostic, not a stack overflow, in
/// the editor too, whose stack cannot grow.
pub(crate) const MAX_NEST: u32 = 128;

/// Parser state: the cursor, the source text and the diagnostics so far.
#[derive(Builder)]
pub(crate) struct Parser<'t>
{
    /// Position in the tokens.
    pub(crate) cur: Cursor<'t>,
    /// The normalized source text.
    pub(crate) text: &'t str,
    /// Absolute position of `text`.
    pub(crate) base: Pos,
    /// Decoded string literals, indexed with [`TokenKind::Str`].
    pub(crate) strings: &'t [StrLit],
    /// Diagnostics recorded after recovering from an error.
    #[builder(default)]
    pub(crate) diags: Vec<Diagnostic>,
    /// The spoken words read as their second word, as
    /// `ရှိတဲ့` read as `ရှိသည့်`.
    #[builder(default)]
    pub(crate) also_taken: Vec<Span>,
    /// The spellings of the ends glued to calls.
    #[builder(default)]
    pub(crate) glued_ends: GluedEnds,
    /// Number for the next expression node.
    #[builder(default)]
    pub(crate) next_id: u32,
    /// The number of blocks that enclose the current statement.
    #[builder(default)]
    pub(crate) depth: u32,
    /// The number of blocks of compile-time conditionals at the top level
    /// that enclose the current statement: their statements stand at the top
    /// level all the same.
    #[builder(default)]
    pub(crate) top: u32,
    /// The number of notation expressions that enclose the current one.
    #[builder(default)]
    pub(crate) nest: u32,
    /// The struct names the program declares or imports, each in both
    /// spellings of `ဒေတာ`, so a name can start a kind however it is written.
    #[builder(default)]
    pub(crate) structs: HashSet<String>,
    /// The classifier each struct of this file declares, under the name in
    /// the spelling the compiler keeps, for the help of a declaration that
    /// leaves it out.
    #[builder(default)]
    pub(crate) classifiers: HashMap<String, &'t str>,
    /// The functions the program declares or imports, each name in both
    /// spellings of `ဒေတာ`, with the prepositions of each function of that
    /// name, so the names of functions with prepositions end a preposition
    /// call.
    #[builder(default)]
    pub(crate) functions: Functions,
    /// Each word of a preposition of a function in [`Self::functions`].
    #[builder(default)]
    pub(crate) preposition_words: HashSet<String>,
    /// The first word of each preposition of a function in [`Self::functions`],
    /// and so each word a preposition call can go on from after an
    /// argument.
    #[builder(default)]
    pub(crate) first_words: HashSet<String>,
    /// The first word of each starting phrase of a function in
    /// [`Self::functions`], and so each word a call written as a sentence can
    /// start from.
    #[builder(default)]
    pub(crate) start_words: HashSet<String>,
    /// Whether the header of a function the compiler gives is being parsed:
    /// its prepositions spell the words of a sentence, so the places of `၏` and
    /// of `ကို သုံး၍` are not checked.
    #[builder(default)]
    pub(crate) in_builtin: bool,
    /// Whether a list count is being parsed: its `ရှိသော` closes the count,
    /// so no struct literal starts inside it.
    #[builder(default)]
    pub(crate) in_count: bool,
    /// Whether the next notation expression takes no `၏` reading after it: a
    /// field name after `၏`, or a part of a larger read.
    #[builder(default)]
    pub(crate) bare_notation: bool,
    /// The depth of notation at which a field read or a call after `၏` does
    /// not bind to the value before it: the top of the number of a cell, or
    /// of the operand after a `၏` of a clause, whose reads go with the whole
    /// read.
    pub(crate) bare_level: Option<u32>,
    /// The heap functions of the core the file sees, which no function of
    /// its own hides: a call statement of one is its heap statement.
    #[builder(default)]
    pub(crate) core_heap: HashSet<String>,
    /// Whether a list of parameters or fields is open: a function kind with
    /// parameters then stands in parentheses.
    #[builder(default)]
    pub(crate) in_list: bool,
    /// Whether an item of a list in words is being parsed: a `နှင့်` outside
    /// parentheses ends the item, so an item with a `နှင့်` of its own stands in
    /// parentheses.
    #[builder(default)]
    pub(crate) in_word_list: bool,
    /// Whether the first half of a `>>` closed a `<…>` group, so its second
    /// half closes the group around it.
    #[builder(default)]
    pub(crate) half_close: bool,
    /// Whether the kinds of a list of kind arguments in the word form are
    /// being parsed: no kind in it starts another such list.
    #[builder(default)]
    pub(crate) in_kind_args: bool,
    /// Whether a scan for kind arguments in the word form is running, so the
    /// scan of a kind inside it does not start another.
    #[builder(default)]
    pub(crate) scanning_kind_args: std::cell::Cell<bool>,
    /// The token where a trait value kind is being scanned or read, so the
    /// kind slot its trait's hint starts with, at that same token, is read as
    /// a plain kind.
    #[builder(default)]
    pub(crate) trait_value_at: std::cell::Cell<Option<usize>>,
    /// Whether a statement other than an import has been parsed at the top
    /// level, so a later import is out of place.
    #[builder(default)]
    pub(crate) imports_closed: bool,
    /// The written-as hints the file sees: those of the core, its imports
    /// and its own.
    #[builder(default)]
    pub(crate) hints: Vec<Hint>,
}

impl<'t> Parser<'t>
{
    /// Create a parser over non-blank lexer output.
    ///
    /// # Arguments
    ///
    /// * `text` - The normalized source text.
    /// * `base` - Absolute position of `text`.
    /// * `lexed` - The lexer output.
    ///
    /// # Returns
    ///
    /// The parser at the first token.
    pub(crate) fn new(text: &'t str, base: Pos, lexed: &'t LexOutput) -> Self
    {
        let cur = Cursor::builder().tokens(&lexed.tokens).build();
        #[rustfmt::skip]
        let mut parser = Parser::builder().cur(cur).text(text).base(base).strings(&lexed.strings).build();
        let decls: Vec<usize> = (0..lexed.tokens.len())
            .filter(|at| {
                declares_kind(&lexed.tokens, *at, &|t: &Token| {
                    parser.text_of(t.span).to_string()
                })
            })
            .collect();
        let names: Vec<&str> =
            decls.iter().map(|at| parser.text_of(lexed.tokens[*at].span)).collect();
        // the stem of an enumeration names it too: `အရောင်` for `အရောင်များ`
        let stems = names.iter().filter_map(|name| name.strip_suffix(PLURAL));
        parser.structs =
            names.iter().copied().chain(stems).flat_map(spellings).collect();
        let found = decls
            .iter()
            .filter_map(|at| parser.kind_classifier(&lexed.tokens, *at))
            .map(|(name, word)| (canonical(name).into_owned(), word))
            .collect();
        parser.classifiers = found;
        parser
    }

    /// The name and the prepositions of the function declaration whose
    /// name sits at token `at`: the string after each `ဝိဘတ်` before the full
    /// stop of the header, read as its words.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The tokens of the file.
    /// * `at` - The index of the name in them.
    ///
    /// # Returns
    ///
    /// The pair, with no prepositions for a header with none, or `None` for a
    /// call.
    pub(crate) fn fn_header(
        &self, tokens: &[Token], at: usize,
    ) -> Option<(String, CallWords)>
    {
        let tokens = tokens.get(at..)?;
        fn_head_at(tokens)?;
        let name = &tokens[0];
        let raw = name.kind == TokenKind::RawIdent;
        let stop = TokenKind::Punct(Punct::FullStop);
        let header: Vec<&Token> =
            tokens.iter().take_while(|t| t.kind != stop).collect();
        // A preposition at depth 0 prepositions the header; one inside the
        // parentheses of a function kind prepositions a parameter of
        // that kind.
        let mut depth = 0usize;
        let mut words = CallWords::default();
        let (mut accepts, mut joins) = (false, 0usize);
        for (n, tok) in header.iter().enumerate()
        {
            let word = |kw: Kw| depth == 0 && tok.kind == TokenKind::Kw(kw);
            let next = || self.preposition_at(header.get(n + 1));
            // The parameter a preposition marks is the one the count of
            // `လည်းကောင်း` before it reaches.
            if word(Kw::PrepositionWord)
            {
                words.prepositions.resize(joins + 1, None);
                words.prepositions[joins] = next();
            }
            do_if!(word(Kw::StartPhrase), words.start = next());
            do_if!(word(Kw::EndPhrase), words.end = next());
            accepts |= word(Kw::Accepting) || word(Kw::WillAccept);
            joins += usize::from(word(Kw::AsWellAs));
            match tok.kind
            {
                TokenKind::Punct(Punct::LParen) => depth += 1,
                TokenKind::Punct(Punct::RParen) => depth = depth.saturating_sub(1),
                _ => (),
            }
        }
        // Two or more parameters each end in `လည်းကောင်း`; a lone one need not.
        words.params = tern!(accepts, joins.max(1), 0);
        words.prepositions.resize(words.params, None);
        let name = unquote_raw(raw, self.text_of(name.span)).to_string();
        Some((name, words))
    }

    /// The preposition a string token after `ဝိဘတ်` holds, its words joined with
    /// one space.
    ///
    /// # Arguments
    ///
    /// * `tok` - The token after `ဝိဘတ်`.
    ///
    /// # Returns
    ///
    /// The preposition, or `None` for no string.
    fn preposition_at(&self, tok: Option<&&Token>) -> Option<String>
    {
        let TokenKind::Str(idx) = tok?.kind
        else
        {
            return None;
        };
        let text = &self.strings[idx as usize].value;
        Some(preposition_words(text).map_or_else(|| text.clone(), |w| w.join(" ")))
    }

    /// Add functions of a name to the ones the parser knows, with the words
    /// of their prepositions.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `lists` - The prepositions of each function.
    pub(crate) fn add_function(&mut self, name: &str, lists: Vec<CallWords>)
    {
        let mut functions = std::mem::take(&mut self.functions);
        functions.entry(name.to_string()).or_default().extend(lists);
        self.set_functions(functions);
    }

    /// Take the functions the parser knows, and the words of their
    /// prepositions.
    ///
    /// # Arguments
    ///
    /// * `functions` - The functions.
    pub(crate) fn set_functions(&mut self, functions: Functions)
    {
        let all = functions.values().flatten();
        let texts = all.clone().flat_map(CallWords::texts);
        let words = texts.flat_map(|m| m.split(' ').map(str::to_string));
        self.preposition_words = words.collect();
        // A call goes on after an argument from a preposition or an ending
        // phrase; a starting phrase stands before the first argument alone.
        let after_arg =
            all.clone().flat_map(|w| w.prepositions.iter().flatten().chain(&w.end));
        let first_word = |m: &String| m.split(' ').next().map(str::to_string);
        self.first_words = after_arg.filter_map(first_word).collect();
        let starts = all.filter_map(|w| w.start.as_ref());
        self.start_words = starts.filter_map(first_word).collect();
        self.functions = functions;
    }

    /// The name and the classifier of the struct, enumeration or alias
    /// declaration whose name sits at token `at`.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The tokens of the file.
    /// * `at` - The index of the name in them.
    ///
    /// # Returns
    ///
    /// The pair, or `None` for a declaration with no classifier clause.
    fn kind_classifier(&self, tokens: &[Token], at: usize)
    -> Option<(&'t str, &'t str)>
    {
        let tokens = tokens.get(at..)?;
        let stop = TokenKind::Punct(Punct::FullStop);
        let clause = tokens.iter().take_while(|t| t.kind != stop);
        let is =
            clause.clone().position(|t| t.kind == TokenKind::Kw(Kw::ClassifierIs))?;
        let word = tokens.get(is + 1)?;
        Some((self.text_of(tokens[0].span), self.text_of(word.span)))
    }

    /// Whether the token `n` places ahead is a declared struct or
    /// enumeration name that starts a kind. A name before `ထဲမှ` names a
    /// member instead: `အမှားများထဲမှ မတွေ့`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far to look ahead.
    ///
    /// # Returns
    ///
    /// `true` for such a name.
    pub(crate) fn is_struct_at(&self, n: usize) -> bool
    {
        let tok = self.peek_n(n);
        let named = tok.kind == TokenKind::Ident
            && self.structs.contains(self.text_of(tok.span));
        named && self.peek_n(n + 1).kind != TokenKind::Kw(Kw::Among)
    }

    /// The current token.
    ///
    /// # Returns
    ///
    /// The token.
    pub(crate) fn peek(&self) -> Token { self.cur.peek_n(0) }

    /// The token `n` places ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far to look ahead.
    ///
    /// # Returns
    ///
    /// The token.
    pub(crate) fn peek_n(&self, n: usize) -> Token { self.cur.peek_n(n) }

    /// Move past the current token.
    ///
    /// # Returns
    ///
    /// The token moved past.
    pub(crate) fn bump(&mut self) -> Token { self.cur.bump() }

    /// Move past the current token without returning it.
    pub(crate) fn advance(&mut self) { self.cur.bump(); }

    /// After a mark of a declaration head, eat the `၊` before the next mark
    /// of the head: two marks need one between them (ZK0266).
    ///
    /// # Arguments
    ///
    /// * `is_mark` - Whether a mark of the head starts some tokens ahead.
    ///
    /// # Returns
    ///
    /// `Ok`, or ZK0266 for a mark right after the one before.
    pub(crate) fn mark_gap(
        &mut self, is_mark: impl Fn(&Self, usize) -> bool,
    ) -> PResult<()>
    {
        let sep = self.at_punct(Punct::ListSep) && is_mark(self, 1);
        do_if!(sep, self.advance());
        ret_if!(sep, Ok(()));
        let glued = diag::marks_need_separator(self.peek().span, self.prev_span().hi);
        ret_if!(is_mark(self, 0), Err(glued));
        Ok(())
    }

    /// Whether the token `n` ahead is the sign `punct`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    /// * `punct` - The sign.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    pub(crate) fn at_punct_n(&self, n: usize, punct: Punct) -> bool
    {
        self.peek_n(n).kind == TokenKind::Punct(punct)
    }

    /// Whether the token `n` ahead is the reserved word `kw`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    /// * `kw` - The reserved word.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    pub(crate) fn at_kw_n(&self, n: usize, kw: Kw) -> bool
    {
        kw_of(&self.peek_n(n)) == Some(kw)
    }

    /// Whether a mark of a function head starts `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn fn_mark_at(&self, n: usize) -> bool
    {
        kw_of(&self.peek_n(n)).is_some_and(|k| FN_MARKS.contains(&k))
    }

    /// Whether the current token is the reserved word `kw`.
    ///
    /// # Arguments
    ///
    /// * `kw` - The reserved word.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    /// Whether the current statement stands inside a block, not at the top
    /// level of the file. The blocks of a compile-time conditional at the top
    /// level count as the top level.
    ///
    /// # Returns
    ///
    /// `true` inside a block.
    pub(crate) fn nested(&self) -> bool { self.depth > self.top }

    pub(crate) fn at(&self, kw: Kw) -> bool { self.peek().kind == TokenKind::Kw(kw) }

    /// Whether the current token is the punctuation `punct`.
    ///
    /// # Arguments
    ///
    /// * `punct` - The punctuation.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    pub(crate) fn at_punct(&self, punct: Punct) -> bool
    {
        self.peek().kind == TokenKind::Punct(punct)
    }

    /// Whether the input is used up.
    ///
    /// # Returns
    ///
    /// `true` at the end-of-input token.
    pub(crate) fn at_eof(&self) -> bool { self.peek().kind == TokenKind::Eof }

    /// Move past the reserved word `kw` if it is the current token.
    ///
    /// # Arguments
    ///
    /// * `kw` - The reserved word.
    ///
    /// # Returns
    ///
    /// `true` if it was there.
    pub(crate) fn eat(&mut self, kw: Kw) -> bool
    {
        let found = self.at(kw);
        do_if!(found, self.advance());
        found
    }

    /// Move past the punctuation `punct` if it is the current token.
    ///
    /// # Arguments
    ///
    /// * `punct` - The punctuation.
    ///
    /// # Returns
    ///
    /// `true` if it was there.
    pub(crate) fn eat_punct(&mut self, punct: Punct) -> bool
    {
        let found = self.at_punct(punct);
        do_if!(found, self.advance());
        found
    }

    /// Move past the reserved word `kw`, or fail.
    ///
    /// # Arguments
    ///
    /// * `kw` - The reserved word.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic.
    pub(crate) fn expect(&mut self, kw: Kw) -> PResult<Token>
    {
        self.expect_one_of(&[kw])
    }

    /// Whether a token is `kw`, or a spoken spelling that also stands for it,
    /// such as `ရှိတဲ့` for `ရှိသည့်`.
    ///
    /// # Arguments
    ///
    /// * `tok` - The token.
    /// * `kw` - The reserved word the grammar asks for.
    ///
    /// # Returns
    ///
    /// `true` when it is.
    pub(crate) fn reads_as(&self, tok: &Token, kw: Kw) -> bool
    {
        let also = spoken(self.text_of(tok.span)).and_then(|s| s.also);
        kw_of(tok) == Some(kw) || also.is_some_and(|(other, _)| other == kw)
    }

    /// Move past `kw`, or a spoken spelling that also stands for it.
    ///
    /// # Arguments
    ///
    /// * `kw` - The reserved word the grammar asks for.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic.
    pub(crate) fn expect_reading(&mut self, kw: Kw) -> PResult<Token>
    {
        let tok = self.peek();
        ret_if!(!self.reads_as(&tok, kw), self.expect(kw));
        do_if!(kw_of(&tok) != Some(kw), self.also_taken.push(tok.span));
        Ok(self.bump())
    }

    /// Whether the token `n` places ahead is the identifier `word`: a kind
    /// name the compiler gives that the grammar also reads as a noun, as
    /// `စာလုံး` in `စာ ၏ ၂ လုံးမြောက် စာလုံး`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    /// * `word` - The name.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    pub(crate) fn name_at(&self, n: usize, word: &str) -> bool
    {
        let tok = self.peek_n(n);
        tok.kind == TokenKind::Ident && self.text_of(tok.span) == word
    }

    /// Move past the identifier `word` the grammar reads as a noun, or fail.
    ///
    /// # Arguments
    ///
    /// * `word` - The name.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic that names it.
    pub(crate) fn expect_name(&mut self, word: &str) -> PResult<Token>
    {
        ret_if!(self.name_at(0, word), Ok(self.bump()));
        Err(self.err_expected(&quote(word)))
    }

    /// Move past one of the reserved words `kws`, or fail.
    ///
    /// # Arguments
    ///
    /// * `kws` - The accepted reserved words.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic that lists them.
    pub(crate) fn expect_one_of(&mut self, kws: &[Kw]) -> PResult<Token>
    {
        ret_if!(kws.iter().any(|kw| self.at(*kw)), Ok(self.bump()));
        let names: Vec<String> = kws.iter().map(|kw| quote(&kw_text(*kw))).collect();
        Err(self.err_expected(&one_of(&names)))
    }

    /// Move past the punctuation `punct`, or fail.
    ///
    /// # Arguments
    ///
    /// * `punct` - The punctuation.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic.
    pub(crate) fn expect_punct(&mut self, punct: Punct) -> PResult<Token>
    {
        ret_if!(self.at_punct(punct), Ok(self.bump()));
        Err(self.err_expected(&quote(punct.as_ref())))
    }

    /// Move past the full stop `။` that ends a statement, or fail.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic.
    pub(crate) fn expect_stop(&mut self) -> PResult<Token>
    {
        self.expect_punct(Punct::FullStop)
    }

    /// Source text of a span.
    ///
    /// # Arguments
    ///
    /// * `span` - An absolute span inside the text.
    ///
    /// # Returns
    ///
    /// The text.
    pub(crate) fn text_of(&self, span: Span) -> &'t str
    {
        let lo = (span.lo - self.base) as usize;
        let hi = (span.hi - self.base) as usize;
        &self.text[lo..hi]
    }

    /// Description of the current token for a message.
    ///
    /// # Returns
    ///
    /// The quoted token text, or the words for the end of input.
    pub(crate) fn found(&self) -> String
    {
        let tok = self.peek();
        ret_if!(tok.kind == TokenKind::Eof, t!("parser.eof").to_string());
        quote(self.text_of(tok.span))
    }

    /// A `nesting-too-deep` diagnostic at the current token.
    ///
    /// # Returns
    ///
    /// The diagnostic.
    pub(crate) fn err_too_deep(&self) -> Diagnostic
    {
        diag::nesting_too_deep(self.peek().span, MAX_NEST)
    }

    /// An `expected-token` diagnostic at the current token.
    ///
    /// # Arguments
    ///
    /// * `expected` - What the grammar allows here.
    ///
    /// # Returns
    ///
    /// The diagnostic.
    pub(crate) fn err_expected(&self, expected: &str) -> Diagnostic
    {
        let found = diag::expected_token(self.peek().span, expected, &self.found());
        diag::with_missing_word(found, expected, self.prev_span().hi)
    }

    /// Number for a new expression node.
    ///
    /// # Returns
    ///
    /// The next unused number.
    pub(crate) fn new_id(&mut self) -> NodeId
    {
        self.next_id += 1;
        NodeId(self.next_id - 1)
    }

    /// Span of the last token moved past.
    ///
    /// # Returns
    ///
    /// The span, or the current token span at the start.
    pub(crate) fn prev_span(&self) -> Span
    {
        self.cur.back(1).map_or(self.peek().span, |tok| tok.span)
    }

    /// Parse a name: an identifier or a raw identifier.
    ///
    /// # Returns
    ///
    /// The name without its braces, or an `expected-token` diagnostic.
    pub(crate) fn parse_ident(&mut self) -> PResult<Ident>
    {
        let tok = self.peek();
        let raw = tok.kind == TokenKind::RawIdent;
        let is_name = tok.kind == TokenKind::Ident || raw;
        ret_if!(!is_name, Err(self.err_expected(&t!("parser.name"))));
        self.bump();
        let text = self.text_of(tok.span);
        let name = unquote_raw(raw, text);
        Ok(Ident::builder().name(name).span(tok.span).build())
    }
}

/// The name inside a raw identifier, or the text as is.
///
/// # Arguments
///
/// * `raw` - Whether the text is a raw identifier in braces.
/// * `text` - The token text.
///
/// # Returns
///
/// The name.
pub(crate) fn unquote_raw(raw: bool, text: &str) -> &str
{
    ret_if!(!raw, text);
    text.strip_prefix('{').and_then(|t| t.strip_suffix('}')).unwrap_or(text)
}
