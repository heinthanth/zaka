use axum::Router;
use leptos::prelude::*;
use leptos_axum::LeptosRoutes;
use leptos_axum::generate_route_list_with_ssg;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use zaka_web_ui_common::site::EXPLORER_URL;
use zaka_website::ORIGIN_VAR;
use zaka_website::app::shell;
use zaka_website::book::Book;
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

/// The file the build writes the address of each page to, in the site root,
/// for crawlers.
const SITEMAP: &str = "sitemap.xml";

/// Write the static pages, `robots.txt` and [`SITEMAP`] into the site root;
/// then, unless
/// [`ONCE_VAR`] is set, serve the site with Monaco, the motion libraries and
/// the web fonts beside it, and [`MISSING_PAGE`] with the code
/// 404 for an address with no page, as the host does.
/// A build with [`ONCE_VAR`] is for the web, so it needs [`ORIGIN_VAR`].
#[tokio::main]
async fn main()
{
    let once = std::env::var_os(ONCE_VAR).is_some();
    let origin = set_origin().filter(|o| o.contains("://"));
    assert!(
        !once || origin.is_some(),
        "set {ORIGIN_VAR} to the address the site is served from, as https://zaka-lang.org"
    );
    let conf = get_configuration(None).expect("cargo-leptos sets the site options");
    let options = conf.leptos_options;
    let page = options.clone();
    let (routes, pages) = generate_route_list_with_ssg(move || shell(page.clone()));
    pages.generate(&options).await;
    let root: &str = options.site_root.as_ref();
    place_assets(root);
    place_robots(root, origin.as_deref());
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

/// Write [`ROBOTS`] to `<site root>/robots.txt`. With an origin, also write
/// [`SITEMAP`] and name it in `robots.txt`; a sitemap needs full addresses, so
/// a build with no origin writes none.
///
/// # Arguments
///
/// * `site_root` - The site root.
/// * `origin` - The address the site is served from, with no trailing slash.
fn place_robots(site_root: &str, origin: Option<&str>)
{
    let root = std::path::Path::new(site_root);
    let robots = origin.map_or(ROBOTS.to_string(), |o| {
        std::fs::write(root.join(SITEMAP), sitemap(o))
            .expect("the site root can be written");
        format!("{ROBOTS}\nSitemap: {o}/{SITEMAP}\n")
    });
    std::fs::write(root.join("robots.txt"), robots)
        .expect("the site root can be written");
}

/// The sitemap of the site: the home page, the explorer and each page of each
/// book, at the address its `canonical` link names. The page for an address
/// with no page is left out.
///
/// # Arguments
///
/// * `origin` - The address the site is served from, with no trailing slash.
///
/// # Returns
///
/// The XML text.
fn sitemap(origin: &str) -> String
{
    let books = Book::ALL.iter().flat_map(|b| b.pages().iter().map(|p| p.url()));
    let entries: String = ["/".to_string(), EXPLORER_URL.to_string()]
        .into_iter()
        .chain(books)
        .map(|path| format!("  <url><loc>{origin}{path}</loc></url>\n"))
        .collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n\
         {entries}</urlset>\n"
    )
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn sitemap_names_home_explorer_and_book_pages()
    {
        let map = sitemap("https://e.org");
        let first = Book::Learn.pages()[0].url();
        ["/", EXPLORER_URL, first.as_str()].iter().for_each(|path| {
            let entr = format!("<loc>https://e.org{path}</loc>");
            assert!(map.contains(&entr), "{entr} missing from {map}");
        });
        assert!(!map.contains("/404"));
    }
}
