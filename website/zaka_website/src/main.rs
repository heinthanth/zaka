use axum::Router;
use leptos::prelude::*;
use leptos_axum::LeptosRoutes;
use leptos_axum::generate_route_list_with_ssg;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use zaka_website::ORIGIN_VAR;
use zaka_website::app::shell;
use zaka_website::set_origin;

/// Env var that makes the server write the pages and stop, for a build.
const ONCE_VAR: &str = "ZAKA_WEBSITE_ONCE";

/// The Monaco editor, as the build of `vendor` writes it, served under
/// `/explorer/monaco` while the site runs on this machine: the explorer
/// names it from beside its page.
const MONACO_DIST: &str = "target/vendor/monaco";

/// The folders of the site root the hashed pictures and fonts go to: the
/// pages name them under `/assets`, and the explorer from beside its page,
/// under `/explorer/assets`.
const ASSET_DIRS: [&str; 2] = ["assets", "explorer/assets"];

/// The web fonts the website and the explorer share, as the build of
/// `zaka_web_ui_common/fonts` writes them, served under `/fonts` while the
/// site runs on this machine.
const FONTS_DIST: &str = "target/fonts";

/// The pictures and fonts under hashed names, as the build script writes
/// them; the server places them in the site root under `/assets`.
const ASSETS: &str = concat!(env!("OUT_DIR"), "/assets");

/// The motion libraries Trunk fetches from npm, served under `/vendor` while
/// the site runs on this machine.
const VENDOR_DIST: &str = "target/vendor";

/// The page the build writes for an address with no page, in the site root.
const MISSING_PAGE: &str = "404.html";

/// The rules for crawlers, written to `robots.txt` in the site root: each
/// crawler can read each page.
const ROBOTS: &str = "User-agent: *\nAllow: /\n";

/// Write the static pages and `robots.txt` into the site root; then, unless
/// [`ONCE_VAR`] is set, serve the site with Monaco, the motion libraries and
/// the web fonts beside it, and [`MISSING_PAGE`] with the code
/// 404 for an address with no page, as the host does.
/// A build with [`ONCE_VAR`] is for the web, so it needs [`ORIGIN_VAR`].
#[tokio::main]
async fn main()
{
    let once = std::env::var_os(ONCE_VAR).is_some();
    let origin = set_origin().is_some_and(|o| o.contains("://"));
    assert!(
        !once || origin,
        "set {ORIGIN_VAR} to the address the site is served from, as https://zaka-lang.org"
    );
    let conf = get_configuration(None).expect("cargo-leptos sets the site options");
    let options = conf.leptos_options;
    let page = options.clone();
    let (routes, pages) = generate_route_list_with_ssg(move || shell(page.clone()));
    pages.generate(&options).await;
    let root: &str = options.site_root.as_ref();
    place_assets(root);
    place_robots(root);
    if once
    {
        return;
    }
    let missing = ServeFile::new(std::path::Path::new(root).join(MISSING_PAGE));
    let site = ServeDir::new(root).not_found_service(missing);
    let page = options.clone();
    let app = Router::new()
        .leptos_routes(&options, routes, move || shell(page.clone()))
        .nest_service("/explorer/monaco", ServeDir::new(MONACO_DIST))
        .nest_service("/vendor", ServeDir::new(VENDOR_DIST))
        .nest_service("/fonts", ServeDir::new(FONTS_DIST))
        .fallback_service(site)
        .with_state(options.clone());
    let listener = tokio::net::TcpListener::bind(&options.site_addr)
        .await
        .expect("the site address is free");
    println!("serving the website on http://{}", options.site_addr);
    axum::serve(listener, app).await.expect("the server runs");
}

/// Put the hashed pictures and fonts in each of [`ASSET_DIRS`] in the site
/// root, in place of the ones an earlier build left there.
///
/// # Arguments
///
/// * `site_root` - The site root.
fn place_assets(site_root: &str)
{
    ASSET_DIRS
        .iter()
        .for_each(|d| place_assets_in(&std::path::Path::new(site_root).join(d)));
}

/// Put the hashed pictures and fonts in a folder, in place of the ones an
/// earlier build left there.
///
/// # Arguments
///
/// * `dir` - The folder.
fn place_assets_in(dir: &std::path::Path)
{
    _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).expect("the site root can be written");
    let files = std::fs::read_dir(ASSETS).expect("the build script wrote the assets");
    files.flatten().for_each(|f| {
        std::fs::copy(f.path(), dir.join(f.file_name()))
            .expect("the site root can be written");
    });
}

/// Write [`ROBOTS`] to `<site root>/robots.txt`.
///
/// # Arguments
///
/// * `site_root` - The site root.
fn place_robots(site_root: &str)
{
    let path = std::path::Path::new(site_root).join("robots.txt");
    std::fs::write(path, ROBOTS).expect("the site root can be written");
}
