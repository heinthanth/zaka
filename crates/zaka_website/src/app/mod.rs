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
use leptos::html::title;
use leptos::prelude::*;
use leptos::tachys::html::doctype;
use leptos_meta::MetaTags;
use leptos_meta::provide_meta_context;
use leptos_router::SsrMode;
use leptos_router::components::Route;
use leptos_router::components::RouteChildren;
use leptos_router::components::RouteProps;
use leptos_router::components::Router;
use leptos_router::components::RouterProps;
use leptos_router::components::Routes;
use leptos_router::components::RoutesProps;
use leptos_router::path;
use leptos_router::static_routes::StaticRoute;
use leptos_scoped_style::ComponentStyle;
use leptos_scoped_style::scss_file;
use zaka_web_ui_common::font::face::Face;
use zaka_web_ui_common::shared_sheets;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

use crate::FONT_ROOT;
use crate::assets::LOGO_LIGHT;
use crate::assets::font_faces;
use crate::components::bands::bands;
use crate::components::close::close;
use crate::components::footer::footer;
use crate::components::header::SiteHeader;
use crate::components::header::header_sheet;
use crate::components::hero::hero;
use crate::components::name::name;

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
/// scenes once they start.
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
    let sheet = |w| Face::SpaceMono.sheet_path(w).unwrap_or_default();
    let fonts = START_WEIGHTS
        .map(|w| link().rel("stylesheet").href(format!("{FONT_ROOT}{}", sheet(w))));
    let head = head().child((
        meta().charset("utf-8"),
        meta().name("viewport").content("width=device-width, initial-scale=1"),
        // As inner HTML, so no marker of the islands lands in the title text.
        title().inner_html(t!("website.page.title").to_string()),
        meta().name("description").content(t!("website.page.description")),
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

/// The site: one page for now, written to `index.html` when the site is
/// built.
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
    // The routes read the router from context, so the router makes them.
    let routes = move || {
        Routes(
            RoutesProps::builder()
                .fallback(|| "Not found.")
                .children(RouteChildren::to_children(home_route))
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
    (
        header_sheet(),
        SiteHeader().into_any(),
        main().id("top").child(sections),
        footer().into_any(),
    )
}
