pub mod render;

pub use zaka_web_ui_common::site::DOCS_URL;
pub use zaka_web_ui_common::site::LEARN_URL;

/// A book of pages on the site, each built from a folder of Markdown under
/// `docs/` with a contents file that lists its pages in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Book
{
    /// Learn: Zaka taught through examples, at [`LEARN_URL`].
    Learn,
    /// Docs: the manual of the language, the compiler and the machine, at
    /// [`DOCS_URL`].
    Manual,
}

impl Book
{
    /// Each book, in the order the site lists them.
    pub const ALL: [Book; 2] = [Book::Learn, Book::Manual];

    /// The pages of the book, in reading order.
    ///
    /// # Returns
    ///
    /// The pages.
    pub fn pages(self) -> &'static [Page]
    {
        match self
        {
            Book::Learn => &learn_pages::PAGES,
            Book::Manual => &manual_pages::PAGES,
        }
    }

    /// The address of the title page of the book.
    ///
    /// # Returns
    ///
    /// The address, ending in a slash.
    pub fn root_url(self) -> &'static str
    {
        match self
        {
            Book::Learn => LEARN_URL,
            Book::Manual => DOCS_URL,
        }
    }

    /// The folder of the book, under `docs/`.
    ///
    /// # Returns
    ///
    /// The name of the folder.
    pub fn folder(self) -> &'static str
    {
        match self
        {
            Book::Learn => "learn",
            Book::Manual => "manual",
        }
    }

    /// The book whose folder has a name.
    ///
    /// # Arguments
    ///
    /// * `folder` - The name of the folder, under `docs/`.
    ///
    /// # Returns
    ///
    /// The book, or `None` when no book lives there.
    pub fn in_folder(folder: &str) -> Option<Book>
    {
        Book::ALL.into_iter().find(|b| b.folder() == folder)
    }
}

/// One page of a book, as its contents file lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page
{
    /// The book it belongs to.
    pub book: Book,
    /// Its address under the root of its book, with no slash at either end;
    /// blank for the title page.
    pub slug: &'static str,
    /// Its file, from the folder of its book.
    pub path: &'static str,
    /// Its title in the contents.
    pub title: &'static str,
    /// How deep it sits in the contents: 0 for a chapter, 1 for a page in one.
    pub depth: usize,
    /// Whether it comes after the rule of the contents, with the reference
    /// pages.
    pub appendix: bool,
    /// Its Markdown text.
    pub text: &'static str,
}

/// The pages of Learn.
mod learn_pages
{
    use super::Book;
    use super::Page;

    include!(concat!(env!("OUT_DIR"), "/learn.rs"));
}

/// The pages of the manual.
mod manual_pages
{
    use super::Book;
    use super::Page;

    include!(concat!(env!("OUT_DIR"), "/manual.rs"));
}

impl Page
{
    /// The address of the page.
    ///
    /// # Returns
    ///
    /// The address, ending in a slash.
    pub fn url(&self) -> String
    {
        let root = self.book.root_url();
        match self.slug
        {
            "" => root.to_string(),
            slug => format!("{root}{slug}/"),
        }
    }

    /// Whether this is the title page.
    ///
    /// # Returns
    ///
    /// `true` for the title page.
    pub fn is_title(&self) -> bool { self.slug.is_empty() }
}

/// A chapter of the contents: a page at depth 0, or a reference page, with
/// the pages under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter
{
    /// Its number, from 1; the reference pages have none.
    pub number: Option<usize>,
    /// The chapter page.
    pub head: Page,
    /// The pages under it.
    pub pages: Vec<Page>,
}

/// The page of a book at an address.
///
/// # Arguments
///
/// * `book` - The book.
/// * `slug` - The address under the root of the book; slashes at either end are
///   fine.
///
/// # Returns
///
/// The page, or `None` when no page has that address.
pub fn page_at(book: Book, slug: &str) -> Option<Page>
{
    let slug = slug.trim_matches('/');
    book.pages().iter().find(|p| p.slug == slug).copied()
}

/// The page of a book with a file.
///
/// # Arguments
///
/// * `book` - The book.
/// * `path` - The file, from the folder of the book.
///
/// # Returns
///
/// The page, or `None` when the contents do not list the file.
pub fn page_of(book: Book, path: &str) -> Option<Page>
{
    book.pages().iter().find(|p| p.path == path).copied()
}

/// The pages before and after a page, in reading order.
///
/// # Arguments
///
/// * `page` - The page.
///
/// # Returns
///
/// The page before, and the page after.
pub fn neighbours(page: &Page) -> (Option<Page>, Option<Page>)
{
    let pages = page.book.pages();
    let Some(at) = pages.iter().position(|p| p == page)
    else
    {
        return (None, None);
    };
    (at.checked_sub(1).map(|i| pages[i]), pages.get(at + 1).copied())
}

/// The chapters of the contents of a book, the title page left out: each
/// numbered chapter with its pages, then each reference page as a chapter of
/// its own.
///
/// # Arguments
///
/// * `book` - The book.
///
/// # Returns
///
/// The chapters, in order.
pub fn chapters(book: Book) -> Vec<Chapter>
{
    let mut chapters: Vec<Chapter> = Vec::new();
    book.pages().iter().filter(|p| !p.is_title()).for_each(|page| {
        let under = page.depth > 0 && !chapters.is_empty();
        let Some(last) = chapters.last_mut().filter(|_| under)
        else
        {
            let count = chapters.iter().filter(|c| c.number.is_some()).count();
            let number = (!page.appendix).then_some(count + 1);
            let head = *page;
            chapters.push(Chapter { number, head, pages: Vec::new() });
            return;
        };
        last.pages.push(*page);
    });
    chapters
}

/// The chapter a page belongs to.
///
/// # Arguments
///
/// * `page` - The page.
///
/// # Returns
///
/// The chapter, or `None` for the title page.
pub fn chapter_of(page: &Page) -> Option<Chapter>
{
    chapters(page.book).into_iter().find(|c| c.head == *page || c.pages.contains(page))
}

/// The address of each page of a book, for the static build: the title
/// page and each page after it.
///
/// # Arguments
///
/// * `book` - The book.
///
/// # Returns
///
/// The address of each page under the root of the book, each ending in a
/// slash but the title page, which is blank.
pub fn slugs(book: Book) -> Vec<String>
{
    book.pages()
        .iter()
        .map(|p| match p.slug
        {
            "" => String::new(),
            slug => format!("{slug}/"),
        })
        .collect()
}
