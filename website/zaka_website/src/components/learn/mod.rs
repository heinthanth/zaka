use leptos::html::a;
use leptos::html::article;
use leptos::html::aside;
use leptos::html::div;
use leptos::html::h1;
use leptos::html::h2;
use leptos::html::header;
use leptos::html::li;
use leptos::html::main;
use leptos::html::nav;
use leptos::html::ol;
use leptos::html::p;
use leptos::html::section;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;

use crate::book::Book;
use crate::book::Chapter;
use crate::book::Page;
use crate::book::chapter_of;
use crate::book::chapters;
use crate::book::render::Block;
use crate::book::render::Head;
use crate::book::render::Rendered;
use crate::components::contents::contents;
use crate::components::contents::contents_sheet;
use crate::components::contents::number;
use crate::components::footer::footer;
use crate::components::header::SiteHeader;
use crate::components::header::SiteHeaderProps;
use crate::components::header::header_sheet;
use crate::components::pager::pager;
use crate::components::sandbox::Sandbox;
use crate::components::sandbox::SandboxProps;
use crate::components::sandbox::sandbox_sheet;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/learn/mod.sass");

/// Rules of the pages of a book.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-learn", STYLE_SHEET);

/// A page of a book: the header, the contents on the left, the page in
/// the middle with its programs to run, what is on the page on the right, and
/// the footer. The title page opens on a gold field with the mark of its book
/// and the title, and lists the chapters after its text.
///
/// # Arguments
///
/// * `page` - The page.
/// * `rendered` - The page, rendered.
///
/// # Returns
///
/// The view.
pub fn book_page(page: Page, rendered: Rendered) -> impl IntoView
{
    let intro = page.is_title().then(|| intro(page.book, &rendered));
    let frame = div().class(ClassName::LEARN_FRAME).child((
        contents(&page),
        article_view(&page, &rendered),
        on_page(&rendered.heads),
    ));
    (
        (header_sheet(), contents_sheet(), sandbox_sheet(), sheet(&SHEET)),
        SiteHeader(
            SiteHeaderProps::builder().here(page.book.root_url().to_string()).build(),
        )
        .into_any(),
        main()
            .id("top")
            .class(ClassName::LEARN)
            .attr("data-page", "learn")
            .child((intro, frame)),
        footer().into_any(),
    )
}

/// The gold field of the title page: the mark of the book, the title and
/// the description.
///
/// # Arguments
///
/// * `book` - The book.
/// * `rendered` - The title page, rendered.
///
/// # Returns
///
/// The view.
fn intro(book: Book, rendered: &Rendered) -> impl IntoView + use<>
{
    let mark = match book
    {
        Book::Learn => t!("website.learn.mark"),
        Book::Manual => t!("website.docs.mark"),
    };
    section().class(ClassName::LEARN_INTRO).attr("data-field", "gold").child((
        p().class(ClassName::LEARN_INTRO_MARK).child(mark),
        h1().class(ClassName::LEARN_INTRO_HEAD).child(rendered.title.clone()),
        p().class(ClassName::LEARN_INTRO_TEXT).child(rendered.description.clone()),
    ))
}

/// The page itself: its head, its blocks, the chapters on the title page,
/// and the links to the pages before and after it.
///
/// # Arguments
///
/// * `page` - The page.
/// * `rendered` - The page, rendered.
///
/// # Returns
///
/// The view.
fn article_view(page: &Page, rendered: &Rendered) -> impl IntoView + use<>
{
    let head = (!page.is_title()).then(|| page_head(page, rendered));
    let blocks: Vec<_> = rendered.blocks.iter().cloned().map(block).collect();
    let book = page.book;
    let grid = page.is_title().then(|| chapter_grid(book));
    article().class(ClassName::LEARN_ARTICLE).child((head, blocks, grid, pager(page)))
}

/// The head of a page: its chapter, its title and what it teaches.
///
/// # Arguments
///
/// * `page` - The page.
/// * `rendered` - The page, rendered.
///
/// # Returns
///
/// The view.
fn page_head(page: &Page, rendered: &Rendered) -> impl IntoView + use<>
{
    let chapter = chapter_of(page).map(|c| {
        let mark = c
            .number
            .map(number)
            .unwrap_or_else(|| t!("website.learn.reference").to_string());
        let name = (c.head != *page).then_some(c.head.title);
        p().class(ClassName::LEARN_KICKER)
            .child((span().child(mark), name.map(|n| span().child(n))))
    });
    header().class(ClassName::LEARN_HEAD).child((
        chapter,
        h1().class(ClassName::LEARN_TITLE).child(rendered.title.clone()),
        p().class(ClassName::LEARN_LEDE).child(rendered.description.clone()),
    ))
}

/// One block of a page: text, or a program in a sandbox.
///
/// # Arguments
///
/// * `block` - The block.
///
/// # Returns
///
/// The view.
fn block(block: Block) -> AnyView
{
    match block
    {
        Block::Prose(html) =>
        {
            div().class(ClassName::LEARN_PROSE).inner_html(html).into_any()
        }
        Block::Zaka(sample) => Sandbox(
            SandboxProps::builder()
                .source(sample.source)
                .fence(sample.fence)
                .options(sample.options)
                .expected(sample.expected)
                .build(),
        )
        .into_any(),
    }
}

/// The chapters on the title page, one block each: its number, its title and
/// its pages; then one block for the reference pages, each a link.
///
/// # Arguments
///
/// * `book` - The book of the title page.
///
/// # Returns
///
/// The view.
fn chapter_grid(book: Book) -> impl IntoView
{
    let (numbered, appendix): (Vec<Chapter>, Vec<Chapter>) =
        chapters(book).into_iter().partition(|c| c.number.is_some());
    let item = |c: Chapter| {
        let pages: Vec<_> = c.pages.iter().map(|p| li().child(p.title)).collect();
        li().child(a().class(ClassName::LEARN_CHAPTER).href(c.head.url()).child((
            span().class(ClassName::LEARN_CHAPTER_MARK).child(c.number.map(number)),
            span().class(ClassName::LEARN_CHAPTER_NAME).child(c.head.title),
            ol().class(ClassName::LEARN_CHAPTER_PAGES).child(pages),
        )))
    };
    let link = |c: Chapter| li().child(a().href(c.head.url()).child(c.head.title));
    let reference = (!appendix.is_empty()).then(|| {
        li().child(
            div()
                .class(format!(
                    "{} {}",
                    ClassName::LEARN_CHAPTER,
                    ClassName::LEARN_REFERENCE
                ))
                .child((
                    span().class(ClassName::LEARN_CHAPTER_MARK).child("§"),
                    span()
                        .class(ClassName::LEARN_CHAPTER_NAME)
                        .child(t!("website.learn.reference")),
                    ol().class(ClassName::LEARN_CHAPTER_PAGES)
                        .child(appendix.into_iter().map(link).collect::<Vec<_>>()),
                )),
        )
    });
    let items: Vec<_> = numbered.into_iter().map(item).collect();
    section().class(ClassName::LEARN_CHAPTERS).child((
        h2().child(t!("website.learn.chapters")),
        ol().class(ClassName::LEARN_CHAPTER_GRID).child((items, reference)),
    ))
}

/// What is on the page: a link to each second-level head, in the right
/// margin of wide screens.
///
/// # Arguments
///
/// * `heads` - The heads of the page.
///
/// # Returns
///
/// The view; none for a page with fewer than two heads.
fn on_page(heads: &[Head]) -> Option<impl IntoView + use<>>
{
    ret_if!(heads.len() < 2, None);
    let link =
        |h: &Head| li().child(a().href(format!("#{}", h.id)).child(h.text.clone()));
    let links: Vec<_> = heads.iter().map(link).collect();
    let label = t!("website.learn.on_page").to_string();
    Some(
        aside()
            .class(ClassName::LEARN_ON_PAGE)
            .attr("aria-label", label.clone())
            .child((
                p().class(ClassName::LEARN_ON_PAGE_HEAD).child(label),
                nav().child(ol().child(links)),
            )),
    )
}
