/// The text of a file, from the root of the website crate.
fn read(name: &str) -> String
{
    std::fs::read_to_string(format!("{}/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// The English text of a page item in the locale table.
fn page_text(id: &str) -> String
{
    let table: toml::Table = read("locales/website.toml").parse().unwrap();
    table["website"]["page"][id]["en"].as_str().unwrap().to_string()
}

#[test]
fn the_cover_card_reads_the_page_description()
{
    let cover = read("vendor/cover.html");
    let (_, rest) = cover.split_once("<p class=\"lede\">").unwrap();
    let (lede, _) = rest.split_once("</p>").unwrap();
    assert_eq!(lede.replace("<br />", " "), page_text("description"));
}

#[test]
fn the_explorer_shares_the_page_description_and_cover_text()
{
    let index = read("../zaka_explorer/index.html");
    let count = |text: String| index.matches(&format!("content=\"{text}\"")).count();
    // The page, Open Graph and X descriptions; the Open Graph and X picture
    // text.
    assert_eq!(count(page_text("description")), 3);
    assert_eq!(count(page_text("cover")), 2);
}
