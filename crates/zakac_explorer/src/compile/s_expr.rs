use bon::Builder;
use strum::IntoStaticStr;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

/// Chars a form can take on one line.
pub const SEXPR_WIDTH: usize = 60;

/// What a chunk of the laid-out text is; its static name is a CSS class
/// suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Ink
{
    /// An open or close bracket.
    Bracket,
    /// The first word of a `(` list.
    Head,
    /// Another word.
    Atom,
    /// A quoted string or char.
    Quote,
    /// Spaces and line breaks.
    Gap,
}

/// A run of laid-out text with one ink.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Chunk
{
    /// What the text is.
    pub ink: Ink,
    /// The text.
    #[builder(into)]
    pub text: String,
}

/// A form read from the flat text.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Form
{
    /// A word or a quoted literal, with its ink.
    Atom(String, Ink),
    /// A bracketed list.
    List(char, char, Vec<Form>),
}

/// Arrange the flat S-expression form of a program as Lisp code is indented: a
/// form that fits is kept on one line; a longer list puts its head and first
/// argument on the first line and lines the rest up under the argument; a
/// `block`, `program`, or one of the IR lists `module`, `init` and `update`.of
/// two or more statements puts each on its own line, two columns in.
///
/// # Arguments
///
/// * `flat` - The flat text, as `to_sexpr` writes it.
/// * `width` - Chars a form can take on one line.
///
/// # Returns
///
/// The laid-out text in chunks.
pub fn arrange(flat: &str, width: usize) -> Vec<Chunk>
{
    let mut writer = Writer { chunks: Vec::new(), width };
    let forms = read_all(flat);
    for (idx, form) in forms.iter().enumerate()
    {
        do_if!(idx > 0, writer.put(Ink::Gap, "\n"));
        writer.form(form, 0, false);
    }
    writer.chunks
}

/// Read all top-level forms of the flat text.
///
/// # Arguments
///
/// * `flat` - The flat text.
///
/// # Returns
///
/// The forms; an unmatched close bracket becomes an atom of its own.
fn read_all(flat: &str) -> Vec<Form>
{
    let mut reader = Reader { chars: flat.chars().collect(), at: 0 };
    let mut forms = Vec::new();
    while !reader.done()
    {
        let form = reader.next_form().unwrap_or_else(|| reader.lone_close());
        forms.push(form);
    }
    forms
}

/// The close bracket of an open bracket.
///
/// # Arguments
///
/// * `open` - The char.
///
/// # Returns
///
/// The close bracket, or `None` when `open` is not an open bracket.
fn closer(open: char) -> Option<char>
{
    match open
    {
        '(' => Some(')'),
        '[' => Some(']'),
        _ => None,
    }
}

/// Whether a char is a close bracket.
///
/// # Arguments
///
/// * `ch` - The char.
///
/// # Returns
///
/// `true` for `)` and `]`.
fn is_close(ch: char) -> bool { matches!(ch, ')' | ']') }

/// Whether a char ends a word.
///
/// # Arguments
///
/// * `ch` - The char.
///
/// # Returns
///
/// `true` for whitespace and brackets.
fn ends_word(ch: char) -> bool
{
    ch.is_whitespace() || is_close(ch) || closer(ch).is_some()
}

/// Reader of forms over the chars of the flat text.
struct Reader
{
    /// The chars.
    chars: Vec<char>,
    /// Index of the next char.
    at: usize,
}

impl Reader
{
    /// The next char, without moving on.
    ///
    /// # Returns
    ///
    /// The char, or `None` at the end.
    fn peek(&self) -> Option<char> { self.chars.get(self.at).copied() }

    /// Move past whitespace.
    fn skip_gaps(&mut self)
    {
        do_while!(self.peek().is_some_and(char::is_whitespace), self.at += 1);
    }

    /// Whether nothing but whitespace is left.
    ///
    /// # Returns
    ///
    /// `true` at the end.
    fn done(&mut self) -> bool
    {
        self.skip_gaps();
        self.at >= self.chars.len()
    }

    /// The text from `lo` to the current char.
    ///
    /// # Arguments
    ///
    /// * `lo` - Index of the first char.
    ///
    /// # Returns
    ///
    /// The text.
    fn since(&self, lo: usize) -> String { self.chars[lo..self.at].iter().collect() }

    /// Read the next form.
    ///
    /// # Returns
    ///
    /// The form, or `None` at the end or at a close bracket.
    fn next_form(&mut self) -> Option<Form>
    {
        self.skip_gaps();
        let ch = self.peek()?;
        ret_if!(is_close(ch), None);
        let lo = self.at;
        self.at += 1;
        ret_if!(let Some(close) = closer(ch), Some(self.list(ch, close)));
        ret_if!(ch == '"', Some(self.quoted(lo, '"')));
        ret_if!(ch == '\'', Some(self.quoted(lo, '\'')));
        do_while!(self.peek().is_some_and(|c| !ends_word(c)), self.at += 1);
        Some(Form::Atom(self.since(lo), Ink::Atom))
    }

    /// Read the items of a list after its open bracket, and its close
    /// bracket.
    ///
    /// # Arguments
    ///
    /// * `open` - The open bracket.
    /// * `close` - The close bracket.
    ///
    /// # Returns
    ///
    /// The list.
    fn list(&mut self, open: char, close: char) -> Form
    {
        let mut items = Vec::new();
        while let Some(item) = self.next_form()
        {
            items.push(item);
        }
        do_if!(self.peek().is_some_and(is_close), self.at += 1);
        Form::List(open, close, items)
    }

    /// Read a quoted literal up to its closing mark.
    ///
    /// # Arguments
    ///
    /// * `lo` - Index of the opening mark.
    /// * `mark` - The quote char.
    ///
    /// # Returns
    ///
    /// The literal, both marks included, with a suffix that follows them.
    fn quoted(&mut self, lo: usize, mark: char) -> Form
    {
        // A char literal holds one char, which can be a quote or a bracket.
        do_if!(mark == '\'', self.at += 1);
        do_while!(self.peek().is_some_and(|c| c != mark), self.at += 1);
        self.at = (self.at + 1).min(self.chars.len());
        // A suffix such as the `:char` of an IR constant stays attached.
        do_while!(self.peek().is_some_and(|c| !ends_word(c)), self.at += 1);
        Form::Atom(self.since(lo), Ink::Quote)
    }

    /// Take an unmatched close bracket as an atom.
    ///
    /// # Returns
    ///
    /// The atom.
    fn lone_close(&mut self) -> Form
    {
        let lo = self.at;
        self.at += 1;
        Form::Atom(self.since(lo), Ink::Bracket)
    }
}

/// Chars of a form on one line.
///
/// # Arguments
///
/// * `form` - The form.
///
/// # Returns
///
/// The width.
fn flat_len(form: &Form) -> usize
{
    match form
    {
        Form::Atom(text, _) => text.chars().count(),
        Form::List(_, _, items) =>
        {
            let inner: usize = items.iter().map(flat_len).sum();
            2 + inner + items.len().saturating_sub(1)
        }
    }
}

/// Whether the items of a `(` list are a statement list: the head is
/// `block`, `program`, or one of the IR lists `module`, `init` and `update`.
///
/// # Arguments
///
/// * `items` - The items of the list.
///
/// # Returns
///
/// `true` for a statement list.
fn is_stmt_list(items: &[Form]) -> bool
{
    let Some(Form::Atom(head, _)) = items.first()
    else
    {
        return false;
    };
    matches!(head.as_str(), "block" | "program" | "module" | "init" | "update")
}

/// Whether a form must break whatever the width: a statement list of two or
/// more statements, or a list that holds one.
///
/// # Arguments
///
/// * `form` - The form.
///
/// # Returns
///
/// `true` when the form cannot sit on one line.
fn must_break(form: &Form) -> bool
{
    let Form::List(open, _, items) = form
    else
    {
        return false;
    };
    let long_list = *open == '(' && is_stmt_list(items) && items.len() > 2;
    long_list || items.iter().any(must_break)
}

/// Writer of the chunks.
struct Writer
{
    /// The chunks so far.
    chunks: Vec<Chunk>,
    /// Chars a form can take on one line.
    width: usize,
}

impl Writer
{
    /// Add a chunk.
    ///
    /// # Arguments
    ///
    /// * `ink` - What the text is.
    /// * `text` - The text.
    fn put(&mut self, ink: Ink, text: impl Into<String>)
    {
        self.chunks.push(Chunk::builder().ink(ink).text(text).build());
    }

    /// Write a form that starts at column `col`.
    ///
    /// # Arguments
    ///
    /// * `form` - The form.
    /// * `col` - Its column.
    /// * `head` - Whether it is the head of a `(` list.
    fn form(&mut self, form: &Form, col: usize, head: bool)
    {
        let fits = col + flat_len(form) <= self.width && !must_break(form);
        match form
        {
            Form::Atom(text, ink) =>
            {
                self.put(tern!(head, Ink::Head, *ink), text.clone())
            }
            Form::List(open, close, items) if fits || items.is_empty() =>
            {
                self.flat(*open, *close, items)
            }
            Form::List(open, close, items) => self.broken(*open, *close, items, col),
        }
    }

    /// Write a list on one line.
    ///
    /// # Arguments
    ///
    /// * `open` - The open bracket.
    /// * `close` - The close bracket.
    /// * `items` - The items.
    fn flat(&mut self, open: char, close: char, items: &[Form])
    {
        self.put(Ink::Bracket, open);
        for (idx, item) in items.iter().enumerate()
        {
            do_if!(idx > 0, self.put(Ink::Gap, " "));
            self.form(item, 0, idx == 0 && open == '(');
        }
        self.put(Ink::Bracket, close);
    }

    /// Write a list over several lines. A `(` list with a short word head
    /// keeps its first argument on the head line and lines the rest up
    /// under it; a statement list and a `(` list with a long head put each
    /// item after the head two columns in; other lists put their first item
    /// after the bracket and the rest one column in.
    ///
    /// # Arguments
    ///
    /// * `open` - The open bracket.
    /// * `close` - The close bracket.
    /// * `items` - The items; at least one.
    /// * `col` - Column of the open bracket.
    fn broken(&mut self, open: char, close: char, items: &[Form], col: usize)
    {
        let call = open == '(';
        let head_len = flat_len(&items[0]);
        let aligned = col + 2 + head_len;
        let hang = call
            && !is_stmt_list(items)
            && items.len() > 1
            && matches!(items[0], Form::Atom(..))
            && aligned <= self.width / 2;
        let indent = tern!(hang, aligned, tern!(call, col + 2, col + 1));
        let lead = tern!(hang, 2, 1);
        self.put(Ink::Bracket, open);
        self.form(&items[0], col + 1, call);
        do_if!(hang, {
            self.put(Ink::Gap, " ");
            self.form(&items[1], indent, false);
        });
        do_for!(item in &items[lead..], {
            self.put(Ink::Gap, format!("\n{}", " ".repeat(indent)));
            self.form(item, indent, false);
        });
        self.put(Ink::Bracket, close);
    }
}
