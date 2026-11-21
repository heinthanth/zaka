use bon::Builder;
use js_sys::Array;
use js_sys::Promise;
use leptos::ev;
use leptos::html::a;
use leptos::html::button;
use leptos::html::input;
use leptos::html::li;
use leptos::html::p;
use leptos::html::span;
use leptos::html::ul;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_scoped_style::ComponentStyle;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use zaka_web_ui_common::controls::modal::Modal;
use zaka_web_ui_common::controls::modal::modal;
use zaka_web_ui_common::icon::search as search_icon;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::book::LEARN_URL;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/search/mod.sass");

/// Rules of the search.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-search", STYLE_SHEET);

/// The inline script of the page shell: a global that imports the Pagefind
/// module, which `pages:build` writes into `/pagefind/`. The import waits
/// for the first search, so a page that never searches never fetches it.
pub const LOADER: &str = "window.zakaPagefind=()=>import('/pagefind/pagefind.js')";

/// Element id of the title of the search modal, which names the dialog.
const TITLE_ID: &str = "search-title";

/// The most pages a search lists.
const LIMIT: usize = 8;

#[wasm_bindgen]
extern "C" {
    /// The global [`LOADER`] defines; it throws on a page without it.
    #[wasm_bindgen(catch, js_name = zakaPagefind)]
    fn load_pagefind() -> Result<Promise, JsValue>;

    /// The Pagefind module.
    type Pagefind;
    /// A search that waits a moment after the last change; it gives `null`
    /// when a later search took its place.
    #[wasm_bindgen(method, js_name = debouncedSearch)]
    fn debounced_search(this: &Pagefind, words: &str) -> Promise;

    /// What a search found.
    type Found;
    #[wasm_bindgen(method, getter)]
    fn results(this: &Found) -> Array;

    /// One page a search found; its data loads on request.
    type Hit;
    #[wasm_bindgen(method)]
    fn data(this: &Hit) -> Promise;

    /// The data of a page a search found.
    type HitData;
    #[wasm_bindgen(method, getter)]
    fn url(this: &HitData) -> String;
    /// The lines around the words, as HTML with each word in a `mark`.
    #[wasm_bindgen(method, getter)]
    fn excerpt(this: &HitData) -> String;
    #[wasm_bindgen(method, getter)]
    fn meta(this: &HitData) -> HitMeta;

    /// The fields Pagefind reads from a page.
    type HitMeta;
    #[wasm_bindgen(method, getter)]
    fn title(this: &HitMeta) -> Option<String>;
}

/// A page a search found, as the list shows it.
#[derive(Clone, PartialEq, Builder)]
struct Match
{
    /// The address of the page.
    url: String,
    /// The title of the page.
    title: String,
    /// The lines around the words, as HTML.
    excerpt: String,
}

/// Where a search stands.
#[derive(Clone, PartialEq)]
enum Outcome
{
    /// The pages the last search found; none before the first search.
    Pages(Vec<Match>),
    /// The page has no index: the dev server, which skips `pages:build`.
    NoIndex,
}

/// The sheet of the search. It mounts outside the island of the header, as
/// the sheet of the header does.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn search_sheet() -> impl IntoView { sheet(&SHEET) }

/// The search of the Learn and Docs pages: a button in the bar and the modal
/// it opens, with a field and the pages that match its words. Pagefind
/// searches an index of the books in the browser, with no server.
///
/// # Returns
///
/// The view.
pub fn search_button() -> impl IntoView
{
    let open = RwSignal::new(false);
    let spec = Modal::builder()
        .open(open)
        .class(ClassName::SEARCH_MODAL)
        .title_id(TITLE_ID)
        .title(t!("website.search.title"))
        .close_label(t!("website.search.close"))
        .build();
    let trigger = button()
        .class(ClassName::SEARCH_BUTTON)
        .r#type("button")
        .attr("aria-haspopup", "dialog")
        .attr("aria-label", t!("website.search.open"))
        .title(t!("website.search.open"))
        .on(ev::click, move |_| open.set(true))
        .child(search_icon());
    (trigger, modal(spec, search_panel()))
}

/// The field and the pages that match its words. The field takes the focus
/// when the modal opens.
///
/// # Returns
///
/// The view.
fn search_panel() -> impl IntoView
{
    let words = RwSignal::new(String::new());
    let outcome = RwSignal::new(Outcome::Pages(Vec::new()));
    let field = input()
        .class(ClassName::SEARCH_FIELD)
        .r#type("search")
        .attr("autofocus", "")
        .attr("aria-label", t!("website.search.title"))
        .placeholder(t!("website.search.hint"))
        .on(ev::input, move |e| ask(event_target_value(&e), words, outcome));
    (field, move || outcome_view(words, outcome))
}

/// Start a search for the words of the field. A search that ends after the
/// field changed leaves the list as it is.
///
/// # Arguments
///
/// * `text` - The text of the field.
/// * `words` - The words of the last search.
/// * `outcome` - Where the search stands.
fn ask(text: String, words: RwSignal<String>, outcome: RwSignal<Outcome>)
{
    let asked = text.trim().to_string();
    words.set(asked.clone());
    ret_if!(asked.is_empty(), outcome.set(Outcome::Pages(Vec::new())));
    spawn_local(async move {
        let found = find(&asked).await;
        ret_if!(words.get_untracked() != asked);
        match found
        {
            Ok(Some(pages)) => outcome.set(Outcome::Pages(pages)),
            Ok(None) => (),
            Err(_) => outcome.set(Outcome::NoIndex),
        }
    });
}

/// Search the index for words.
///
/// # Arguments
///
/// * `asked` - The words.
///
/// # Returns
///
/// The pages that match, best first; `None` when a later search took the
/// place of this one; an error when the page has no index.
async fn find(asked: &str) -> Result<Option<Vec<Match>>, JsValue>
{
    let pagefind: Pagefind = JsFuture::from(load_pagefind()?).await?.unchecked_into();
    let found = JsFuture::from(pagefind.debounced_search(asked)).await?;
    ret_if!(found.is_null(), Ok(None));
    let loads: Array = found
        .unchecked_into::<Found>()
        .results()
        .iter()
        .take(LIMIT)
        .map(|hit| hit.unchecked_into::<Hit>().data())
        .collect();
    let datas: Array = JsFuture::from(Promise::all(&loads)).await?.unchecked_into();
    Ok(Some(datas.iter().map(|d| to_match(d.unchecked_into())).collect()))
}

/// The page a search found, as the list shows it.
///
/// # Arguments
///
/// * `data` - Its data from Pagefind.
///
/// # Returns
///
/// The page; a page with no title shows its address.
fn to_match(data: HitData) -> Match
{
    let url = data.url();
    Match::builder()
        .title(data.meta().title().unwrap_or_else(|| url.clone()))
        .excerpt(data.excerpt())
        .url(url)
        .build()
}

/// The list of pages, or a note when no page matches or the page has no
/// index.
///
/// # Arguments
///
/// * `words` - The words of the last search.
/// * `outcome` - Where the search stands.
///
/// # Returns
///
/// The view.
fn outcome_view(words: RwSignal<String>, outcome: RwSignal<Outcome>) -> AnyView
{
    let note = |id: &str| p().class(ClassName::SEARCH_NOTE).child(t!(id.to_string()));
    let pages = match outcome.get()
    {
        Outcome::NoIndex => return note("website.search.no_index").into_any(),
        Outcome::Pages(pages) => pages,
    };
    ret_if!(words.with(String::is_empty), ().into_any());
    ret_if!(pages.is_empty(), note("website.search.none").into_any());
    ul().class(ClassName::SEARCH_LIST)
        .child(pages.into_iter().map(match_link).collect_view())
        .into_any()
}

/// One page in the list: its title over the lines around the words.
///
/// # Arguments
///
/// * `found` - The page.
///
/// # Returns
///
/// The view.
fn match_link(found: Match) -> impl IntoView
{
    // Pagefind escapes the text of the page and adds the `mark` tags alone.
    let excerpt = p().class(ClassName::SEARCH_EXCERPT).inner_html(found.excerpt);
    // Both books hold pages of the same title, so each match names its book.
    let book = tern!(
        found.url.starts_with(LEARN_URL),
        t!("website.header.learn"),
        t!("website.header.docs")
    );
    let head = p().class(ClassName::SEARCH_HEAD).child((
        span().class(ClassName::SEARCH_TITLE).child(found.title),
        span().class(ClassName::SEARCH_BOOK).child(book),
    ));
    li().child(
        a().class(ClassName::SEARCH_MATCH).href(found.url).child((head, excerpt)),
    )
}
