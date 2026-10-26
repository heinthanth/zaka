pub mod render;

/// Where the manual lives on the site.
pub const LEARN_URL: &str = zaka_web_ui_common::site::MANUAL_URL;

/// One page of the manual, as its contents file lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page
{
    /// Its address under [`LEARN_URL`], with no slash at either end; blank
    /// for the title page.
    pub slug: &'static str,
    /// Its file, from the folder of the manual.
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

include!(concat!(env!("OUT_DIR"), "/manual.rs"));

impl Page
{
    /// The address of the page.
    ///
    /// # Returns
    ///
    /// The address, ending in a slash.
    pub fn url(&self) -> String
    {
        match self.slug
        {
            "" => LEARN_URL.to_string(),
            slug => format!("{LEARN_URL}{slug}/"),
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

/// The page at an address.
///
/// # Arguments
///
/// * `slug` - The address under [`LEARN_URL`]; slashes at either end are fine.
///
/// # Returns
///
/// The page, or `None` when no page has that address.
pub fn page_at(slug: &str) -> Option<Page>
{
    let slug = slug.trim_matches('/');
    PAGES.iter().find(|p| p.slug == slug).copied()
}

/// The page with a file.
///
/// # Arguments
///
/// * `path` - The file, from the folder of the manual.
///
/// # Returns
///
/// The page, or `None` when the contents do not list the file.
pub fn page_of(path: &str) -> Option<Page>
{
    PAGES.iter().find(|p| p.path == path).copied()
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
    let Some(at) = PAGES.iter().position(|p| p == page)
    else
    {
        return (None, None);
    };
    (at.checked_sub(1).map(|i| PAGES[i]), PAGES.get(at + 1).copied())
}

/// The chapters of the contents, the title page left out: each numbered
/// chapter with its pages, then each reference page as a chapter of its own.
///
/// # Returns
///
/// The chapters, in order.
pub fn chapters() -> Vec<Chapter>
{
    let mut chapters: Vec<Chapter> = Vec::new();
    PAGES.iter().filter(|p| !p.is_title()).for_each(|page| {
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
    chapters().into_iter().find(|c| c.head == *page || c.pages.contains(page))
}

/// The address of each page of the manual, for the static build: the title
/// page and each page after it.
///
/// # Returns
///
/// The address of each page under [`LEARN_URL`], each ending in a slash but
/// the title page, which is blank.
pub fn slugs() -> Vec<String>
{
    PAGES
        .iter()
        .map(|p| match p.slug
        {
            "" => String::new(),
            slug => format!("{slug}/"),
        })
        .collect()
}
