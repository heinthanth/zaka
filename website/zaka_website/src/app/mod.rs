use leptos::children::ToChildren;
use leptos::children::TypedChildren;
use leptos::html::body;
use leptos::html::head;
use leptos::html::html;
use leptos::html::link;
use leptos::html::main;
use leptos::html::meta;
use leptos::html::script;
use leptos::html::style;
use leptos::prelude::*;
use leptos::tachys::html::doctype;
use leptos_meta::Link;
use leptos_meta::LinkProps;
use leptos_meta::Meta;
use leptos_meta::MetaProps;
use leptos_meta::MetaTags;
use leptos_meta::Title;
use leptos_meta::TitleProps;
use leptos_meta::provide_meta_context;
use leptos_router::SsrMode;
use leptos_router::components::Route;
use leptos_router::components::RouteChildren;
use leptos_router::components::RouteProps;
use leptos_router::components::Router;
use leptos_router::components::RouterProps;
use leptos_router::components::Routes;
use leptos_router::components::RoutesProps;
use leptos_router::hooks::use_params_map;
use leptos_router::path;
use leptos_router::static_routes::StaticParamsMap;
use leptos_router::static_routes::StaticRoute;
use leptos_scoped_style::ComponentStyle;
use leptos_scoped_style::scss_file;
use zaka_web_ui_common::font::FONT_ROOT;
use zaka_web_ui_common::font::face::Face;
use zaka_web_ui_common::shared_sheets;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

use crate::ORIGIN_VAR;
use crate::assets::COVER_CARD;
use crate::assets::LOGO_LIGHT;
use crate::assets::font_faces;
use crate::components::bands::bands;
use crate::components::close::close;
use crate::components::footer::footer;
use crate::components::header::SiteHeader;
use crate::components::header::SiteHeaderProps;
use crate::components::header::header_sheet;
use crate::components::hero::hero;
use crate::components::learn::learn_page;
use crate::components::name::name;
use crate::learn::page_at;
use crate::learn::render::render;
use crate::learn::slugs;

/// Name of the route parameter that holds the address of a Learn page.
const PAGE_PARAM: &str = "page";

/// The address the site is served from, which the pages read to name
/// themselves in full.
#[derive(Debug, Clone)]
struct Origin(String);

/// Page rules: the frame, the text steps, the button and the entrances.
static PAGE: ComponentStyle =
    ComponentStyle::global_css("site-page", scss_file!("src/app/mod.sass"));

/// Set the theme the visitor picked before the page paints, so a dark pick
/// never flashes light. It reads the storage name the explorer shares; with
/// no pick, or Auto, the OS preference stands.
const PREPAINT: &str = "try{var m=localStorage.getItem('zaka-theme');if(m&&m!=='auto'\
                        )document.documentElement.dataset.theme=m}catch(e){}";

/// Weights of Space Mono, the Latin face of the page, whose sheets load
/// first, so the first paint is set in it; the Burmese and code faces come
/// from the page sheet.
const START_WEIGHTS: [u16; 2] = [400, 700];

/// The libraries of the motion code, in order: GSAP, its scroll plugin and
/// Lenis, from npm through `vendor/Trunk.toml`, each in a folder named after
/// its version. Deferred, so the page paints first; the islands set the
/// scenes once started.
pub const MOTION: [&str; 3] = [
    "/vendor/gsap/3.12.5/gsap.min.js",
    "/vendor/gsap/3.12.5/ScrollTrigger.min.js",
    "/vendor/lenis/1.1.13/lenis.min.js",
];

/// The HTML document around the page.
///
/// # Arguments
///
/// * `options` - The site options cargo-leptos sets.
///
/// # Returns
///
/// The view.
pub fn shell(options: LeptosOptions) -> impl IntoView
{
    provide_context(Origin(site_origin(&options)));
    let sheet = |w| Face::SpaceMono.sheet_path(w).unwrap_or_default();
    let fonts = START_WEIGHTS
        .map(|w| link().rel("stylesheet").href(format!("{FONT_ROOT}{}", sheet(w))));
    let head = head().child((
        meta().charset("utf-8"),
        meta().name("viewport").content("width=device-width, initial-scale=1"),
        link().rel("icon").r#type("image/svg+xml").href(LOGO_LIGHT),
        script().inner_html(PREPAINT),
        fonts,
        style().inner_html(font_faces()),
        MOTION.map(|src| script().src(src).attr("defer", true)),
        AutoReload(AutoReloadProps::builder().options(options.clone()).build()),
        HydrationScripts(
            HydrationScriptsProps::builder().options(options).islands(true).build(),
        ),
        MetaTags(),
    ));
    (doctype("html"), html().lang("en").child((head, body().child(app()))))
}

/// The address the site is served from: [`ORIGIN_VAR`], else the address the
/// server listens on, for a run on this machine.
///
/// # Arguments
///
/// * `options` - The site options cargo-leptos sets.
///
/// # Returns
///
/// The address, with no trailing slash.
pub fn site_origin(options: &LeptosOptions) -> String
{
    let set = std::env::var(ORIGIN_VAR).map(|o| o.trim_end_matches('/').to_string());
    set.unwrap_or_else(|_| format!("http://{}", options.site_addr))
}

/// The tags of the head that name a page: its title and description, its
/// canonical address, and the tags a chat app or a feed reads to draw a card
/// of it.
///
/// # Arguments
///
/// * `path` - The address of the page on the site.
/// * `title` - Its title.
/// * `description` - What it is, in one sentence.
///
/// # Returns
///
/// The view.
fn page_tags(path: &str, title: String, description: String) -> impl IntoView + use<>
{
    let origin = use_context::<Origin>().map(|o| o.0).unwrap_or_default();
    let page = format!("{origin}{path}");
    let cover = format!("{origin}{COVER_CARD}");
    let name = |name: &'static str, content: String| {
        Meta(MetaProps::builder().name(name).content(content).build())
    };
    (
        Title(TitleProps::builder().text(title.clone()).build()),
        name("description", description.clone()),
        Link(LinkProps::builder().rel("canonical").href(page.clone()).build()),
        open_graph_tags(page, title.clone(), description.clone(), cover.clone()),
        twitter_tags(title, description, cover),
    )
}

/// The Open Graph tags, which Facebook, LinkedIn, Discord, Slack, Telegram and
/// most chat apps read: the page, its title and description, and the cover
/// picture, 1200 x 630.
///
/// # Arguments
///
/// * `page` - The address of the page.
/// * `title` - Its title.
/// * `description` - What it is, in one sentence.
/// * `cover` - The address of the cover picture.
///
/// # Returns
///
/// The view.
fn open_graph_tags(
    page: String, title: String, description: String, cover: String,
) -> impl IntoView
{
    let tag = |name: &'static str, content: String| {
        Meta(MetaProps::builder().property(name).content(content).build())
    };
    (
        tag("og:type", "website".into()),
        tag("og:site_name", t!("website.page.site_name").to_string()),
        tag("og:locale", "en_US".into()),
        tag("og:url", page),
        tag("og:title", title),
        tag("og:description", description),
        tag("og:image", cover),
        tag("og:image:type", "image/png".into()),
        tag("og:image:width", "1200".into()),
        tag("og:image:height", "630".into()),
        tag("og:image:alt", t!("website.page.cover").to_string()),
    )
}

/// The tags X reads for its large picture card; it takes the rest from the
/// Open Graph tags, but some readers want them under its own names.
///
/// # Arguments
///
/// * `title` - The title of the page.
/// * `description` - What it is, in one sentence.
/// * `cover` - The address of the cover picture.
///
/// # Returns
///
/// The view.
fn twitter_tags(title: String, description: String, cover: String) -> impl IntoView
{
    let tag = |name: &'static str, content: String| {
        Meta(MetaProps::builder().name(name).content(content).build())
    };
    (
        tag("twitter:card", "summary_large_image".into()),
        tag("twitter:title", title),
        tag("twitter:description", description),
        tag("twitter:image", cover),
        tag("twitter:image:alt", t!("website.page.cover").to_string()),
    )
}

/// The site: the landing page and the Learn pages, each written to
/// its own `index.html` when the site is built.
///
/// # Returns
///
/// The view.
pub fn app() -> impl IntoView
{
    provide_meta_context();
    let home_route = || {
        Route(
            RouteProps::builder()
                .path(path!(""))
                .view(home)
                .ssr(SsrMode::Static(StaticRoute::new()))
                .build(),
        )
    };
    // One route for all the Learn pages; the build writes a page for each
    // address the contents list.
    let learn_route = || {
        let pages = || async {
            let mut params = StaticParamsMap::new();
            params.insert(PAGE_PARAM, slugs());
            params
        };
        Route(
            RouteProps::builder()
                .path(path!("/learn/*page"))
                .view(learn)
                .ssr(SsrMode::Static(StaticRoute::new().prerender_params(pages)))
                .build(),
        )
    };
    // The routes read the router from context, so the router makes them.
    let routes = move || {
        Routes(
            RoutesProps::builder()
                .fallback(|| "Not found.")
                .children(RouteChildren::to_children(move || {
                    (home_route(), learn_route())
                }))
                .build(),
        )
    };
    let router = Router(
        RouterProps::builder().children(TypedChildren::to_children(routes)).build(),
    );
    (shared_sheets(), sheet(&PAGE), router)
}

/// The landing page, as a run of sentences with their proof.
///
/// # Returns
///
/// The view.
fn home() -> impl IntoView
{
    // Each section is erased to one kind of view, which keeps the page small
    // enough for the compiler.
    let sections =
        [hero().into_any(), name().into_any(), bands().into_any(), close().into_any()];
    let title = t!("website.page.title").to_string();
    (
        page_tags("/", title, t!("website.page.description").to_string()),
        header_sheet(),
        SiteHeader(SiteHeaderProps::builder().home(true).build()).into_any(),
        main().id("top").child(sections),
        footer().into_any(),
    )
}

/// A Learn page, from the address in the route.
///
/// # Returns
///
/// The view; a short note for an address with no page.
fn learn() -> impl IntoView
{
    let slug =
        use_params_map().with_untracked(|p| p.get(PAGE_PARAM).unwrap_or_default());
    let Some(page) = page_at(&slug)
    else
    {
        return "Not found.".into_any();
    };
    let rendered = render(&page);
    let title = t!("website.learn.page_title", page = rendered.title).to_string();
    let tags = page_tags(&page.url(), title, rendered.description.clone());
    (tags, learn_page(page, rendered)).into_any()
}
