use std::iter::Peekable;
use std::vec::IntoIter;

use bon::Builder;
use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::CowStr;
use pulldown_cmark::Event;
use pulldown_cmark::HeadingLevel;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use pulldown_cmark_escape::escape_html;
use zaka_web_ui_common::paint::Piece;
use zaka_web_ui_common::paint::paint;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use super::Book;
use super::Page;
use super::page_of;
use crate::components::sandbox::run::Fence;
use crate::components::sandbox::run::FenceOptions;

/// The contents file, which a page can link to.
const CONTENTS_FILE: &str = "CONTENTS.md";

/// Element id of the contents on each page of a book.
pub const CONTENTS_ID: &str = "contents";

/// The info string of a fence that holds what the program before it prints.
const OUTPUT_FENCE: &str = "text";

/// The info string of a fence that shows one file of a program in several
/// files: it is painted as Zaka, but a page cannot run it on its own.
pub const FILE_FENCE: &str = "zaka-file";

/// A page of a book, set for the site.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Rendered
{
    /// Its title, from the front matter.
    pub title: String,
    /// One sentence on what it teaches, from the front matter.
    pub description: String,
    /// Its blocks, in order.
    pub blocks: Vec<Block>,
    /// Its second-level heads, for the list of what is on the page.
    pub heads: Vec<Head>,
}

/// A run of a page: text, or a program to run.
#[derive(Debug, Clone, PartialEq)]
pub enum Block
{
    /// Text, as HTML.
    Prose(String),
    /// A program, which the page lets the reader edit and run.
    Zaka(Sample),
}

/// A program of a page.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Sample
{
    /// The program.
    #[builder(into)]
    pub source: String,
    /// What the fence promises the program does.
    pub fence: Fence,
    /// What its fence asks of the compiler.
    #[builder(default)]
    pub options: FenceOptions,
    /// What it prints, from the `text` fence after it; blank when none
    /// follows.
    #[builder(into, default)]
    pub expected: String,
}

/// A second-level head of a page.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Head
{
    /// Its element id.
    pub id: String,
    /// Its text.
    pub text: String,
}

/// The events of a page still to read.
type Rest = Peekable<IntoIter<Event<'static>>>;

/// What the reading of a page has built so far.
#[derive(Default)]
struct Built
{
    /// The finished blocks.
    blocks: Vec<Block>,
    /// The events of the text block in hand.
    prose: Vec<Event<'static>>,
    /// The heads so far.
    heads: Vec<Head>,
}

impl Built
{
    /// Close the text block in hand, when it holds something.
    fn flush(&mut self)
    {
        ret_if!(self.prose.is_empty());
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, self.prose.drain(..));
        self.blocks.push(Block::Prose(html));
    }
}

/// Set a page of a book for the site: its front matter read, its
/// first head left to the page, its programs taken out to run, its links
/// pointed at the pages of the site and the Burmese code in it painted.
///
/// # Arguments
///
/// * `page` - The page.
///
/// # Returns
///
/// The page, rendered.
pub fn render(page: &Page) -> Rendered
{
    let (front, text) = front_matter(page.text);
    let events: Vec<Event<'static>> =
        Parser::new_ext(text, Options::ENABLE_TABLES).collect();
    let mut rest: Rest = events.into_iter().peekable();
    let mut built = Built::default();
    while let Some(event) = rest.next()
    {
        take(page, event, &mut rest, &mut built);
    }
    built.flush();
    let field = |name: &str| {
        front
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.to_string())
            .unwrap_or_default()
    };
    let title = field("title");
    Rendered::builder()
        .title(tern!(title.is_empty(), page.title.to_string(), title))
        .description(field("description"))
        .blocks(built.blocks)
        .heads(built.heads)
        .build()
}

/// Read one event of a page, and the events that belong with it.
///
/// # Arguments
///
/// * `page` - The page.
/// * `event` - The event.
/// * `rest` - The events after it.
/// * `built` - What the reading has built so far.
fn take(page: &Page, event: Event<'static>, rest: &mut Rest, built: &mut Built)
{
    match event
    {
        Event::Start(Tag::Heading { level: HeadingLevel::H1, .. }) =>
        {
            _ = inner(rest, TagEnd::Heading(HeadingLevel::H1));
        }
        Event::Start(Tag::Heading {
            level: HeadingLevel::H2, classes, attrs, ..
        }) =>
        {
            heading(rest, built, classes, attrs);
        }
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) =>
        {
            code_block(rest, built, info);
        }
        Event::Start(Tag::Link { link_type, dest_url, title, id }) =>
        {
            let dest_url = link_target(page, &dest_url).into();
            built.prose.push(Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }));
        }
        Event::Code(code) if is_burmese(&code) =>
        {
            built.prose.push(Event::InlineHtml(painted_code(&code).into()));
        }
        event => built.prose.push(event),
    }
}

/// Take a second-level head, with an id for the list of what is on the page.
///
/// # Arguments
///
/// * `rest` - The events after its start.
/// * `built` - What the reading has built so far.
/// * `classes` - Its classes.
/// * `attrs` - Its attributes.
fn heading(
    rest: &mut Rest, built: &mut Built, classes: Vec<CowStr<'static>>,
    attrs: Vec<(CowStr<'static>, Option<CowStr<'static>>)>,
)
{
    let end = TagEnd::Heading(HeadingLevel::H2);
    let content = inner(rest, end);
    let text: String = content.iter().filter_map(plain_text).collect();
    let id = anchor(&text, &built.heads);
    let level = HeadingLevel::H2;
    let start = Tag::Heading { level, id: Some(id.clone().into()), classes, attrs };
    built.prose.push(Event::Start(start));
    built.prose.extend(content);
    built.prose.push(Event::End(end));
    built.heads.push(Head::builder().id(id).text(text).build());
}

/// Take a fenced block: a program of Zaka becomes a block of its own, with
/// what it prints from the `text` fence after it; a file of a longer
/// program is painted in place; others stand as text.
///
/// # Arguments
///
/// * `rest` - The events after its start.
/// * `built` - What the reading has built so far.
/// * `info` - The info string of the fence.
fn code_block(rest: &mut Rest, built: &mut Built, info: CowStr<'static>)
{
    ret_if!(&*info == FILE_FENCE, shown_file(rest, built));
    let Some((fence, options)) = Fence::parse(&info)
    else
    {
        built.prose.push(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))));
        return;
    };
    let source = block_text(rest);
    let fenced = |e: &Event| match e
    {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(i))) => &**i == OUTPUT_FENCE,
        _ => false,
    };
    let output = rest.peek().is_some_and(fenced);
    let expected = output.then(|| {
        rest.next();
        block_text(rest)
    });
    built.flush();
    let source = source.trim_end().to_string();
    let expected = expected.unwrap_or_default();
    let sample = Sample::builder()
        .source(source)
        .fence(fence)
        .options(options)
        .expected(expected)
        .build();
    built.blocks.push(Block::Zaka(sample));
}

/// Take a fenced block that shows one file of a program: painted, in the
/// prose, with no run button.
///
/// # Arguments
///
/// * `rest` - The events after its start.
/// * `built` - What the reading has built so far.
fn shown_file(rest: &mut Rest, built: &mut Built)
{
    let source = block_text(rest);
    let html = format!("<pre>{}</pre>\n", painted_code(source.trim_end()));
    built.prose.push(Event::Html(html.into()));
}

/// The text of a code block, up to its end.
///
/// # Arguments
///
/// * `rest` - The events after its start.
///
/// # Returns
///
/// The text.
fn block_text(rest: &mut Rest) -> String
{
    inner(rest, TagEnd::CodeBlock).iter().filter_map(plain_text).collect()
}

/// The events up to an end, which is read but left out.
///
/// # Arguments
///
/// * `rest` - The events.
/// * `end` - The end.
///
/// # Returns
///
/// The events before the end.
fn inner(rest: &mut Rest, end: TagEnd) -> Vec<Event<'static>>
{
    let content: Vec<_> = rest.by_ref().take_while(|e| *e != Event::End(end)).collect();
    content
}

/// The text an event adds, for a head or a code block.
///
/// # Arguments
///
/// * `event` - The event.
///
/// # Returns
///
/// The text, or `None` for an event with none.
fn plain_text(event: &Event<'static>) -> Option<String>
{
    match event
    {
        Event::Text(text) | Event::Code(text) => Some(text.to_string()),
        _ => None,
    }
}

/// An element id for a head: its words, lower case, joined with dashes, and
/// a number after it when an earlier head took the same id.
///
/// # Arguments
///
/// * `text` - The text of the head.
/// * `taken` - The heads before it.
///
/// # Returns
///
/// The id.
pub fn anchor(text: &str, taken: &[Head]) -> String
{
    let word = |w: &str| -> String {
        w.chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
            .to_lowercase()
    };
    let words: Vec<String> =
        text.split_whitespace().map(word).filter(|w| !w.is_empty()).collect();
    let base = words.join("-");
    let used = |id: &str| taken.iter().any(|h| h.id == id);
    ret_if!(!used(&base), base);
    (2..).map(|n| format!("{base}-{n}")).find(|id| !used(id)).unwrap_or(base)
}

/// Where a link of a page points on the site: a page of either book at its
/// address and the contents of a book at the list on each of its pages; a
/// path from the site root, as `/explorer/`, stands as written. A link names a
/// file from the folder of its page, so `../../manual/reference/errors.md` on
/// a Learn page reaches the manual.
///
/// # Arguments
///
/// * `page` - The page the link is on.
/// * `dest` - The target the Markdown names.
///
/// # Returns
///
/// The target on the site; the same target when it is not a page.
pub fn link_target(page: &Page, dest: &str) -> String
{
    let local = !dest.contains("://") && !dest.starts_with(['#', '/']);
    let (file, mark) = dest.split_once('#').unwrap_or((dest, ""));
    ret_if!(!local || !file.ends_with(".md"), dest.to_string());
    let mark = tern!(mark.is_empty(), String::new(), format!("#{mark}"));
    let from = format!("{}/{}", page.book.folder(), page.path);
    let path = joined(&from, file);
    let found = path.split_once('/').and_then(|(folder, rest)| {
        Book::in_folder(folder).map(|book| (book, rest.to_string()))
    });
    let Some((book, rest)) = found
    else
    {
        return dest.to_string();
    };
    ret_if!(rest == CONTENTS_FILE, format!("{}#{CONTENTS_ID}", book.root_url()));
    page_of(book, &rest)
        .map_or_else(|| dest.to_string(), |p| format!("{}{mark}", p.url()))
}

/// A file named from a page, as a path from the `docs/` folder.
///
/// # Arguments
///
/// * `from` - The file of the page.
/// * `file` - The file it names, from its own folder.
///
/// # Returns
///
/// The path, with `.` and `..` gone.
fn joined(from: &str, file: &str) -> String
{
    let dir = from.rsplit_once('/').map_or("", |(dir, _)| dir);
    let parts = dir.split('/').chain(file.split('/')).filter(|p| !p.is_empty());
    let mut path: Vec<&str> = Vec::new();
    parts.for_each(|part| match part
    {
        "." =>
        {}
        ".." => _ = path.pop(),
        part => path.push(part),
    });
    path.join("/")
}

/// Whether a text holds a Burmese letter.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// `true` when it does.
fn is_burmese(text: &str) -> bool
{
    text.chars().any(|c| ('\u{1000}'..='\u{109F}').contains(&c))
}

/// A code span of Burmese, painted with the lexer, as HTML.
///
/// # Arguments
///
/// * `code` - The code.
///
/// # Returns
///
/// The HTML.
fn painted_code(code: &str) -> String
{
    let piece = |p: Piece| {
        let mut text = String::new();
        _ = escape_html(&mut text, &p.text);
        format!("<span class=\"{}\">{text}</span>", p.class)
    };
    let spans: String = paint(code).into_iter().map(piece).collect();
    format!("<code class=\"painted\" lang=\"my\">{spans}</code>")
}

/// Split the front matter off a page: the `name: value` lines between two
/// `---` lines at its start.
///
/// # Arguments
///
/// * `text` - The page.
///
/// # Returns
///
/// The pairs of the front matter, and the text after it.
fn front_matter(text: &'static str)
-> (Vec<(&'static str, &'static str)>, &'static str)
{
    let Some((head, rest)) =
        text.strip_prefix("---\n").and_then(|t| t.split_once("\n---\n"))
    else
    {
        return (Vec::new(), text);
    };
    let pairs = head.lines().filter_map(|l| l.split_once(':'));
    (pairs.map(|(k, v)| (k.trim(), v.trim())).collect(), rest)
}
