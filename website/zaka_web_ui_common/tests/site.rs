use zaka_web_ui_common::site::Release;
use zaka_web_ui_common::site::moved_path;
use zaka_web_ui_common::site::read_releases;

#[test]
fn the_versions_file_lists_its_releases_newest_first()
{
    let text = "[[release]]\nversion = \"0.3\"\npath = \"/\"\n\n[[release]]\nversion \
                = \"0.2\"\npath = \"/v0.2/\"\n";
    let releases = read_releases(text);
    let release = |v: &str, p: &str| Release::builder().version(v).path(p).build();
    assert_eq!(releases, [release("0.3", "/"), release("0.2", "/v0.2/")]);
}

#[test]
fn a_versions_file_that_does_not_parse_lists_no_release()
{
    assert!(read_releases("[[release]]\nversion = 3\n").is_empty());
    assert!(read_releases("").is_empty());
}

#[test]
fn a_pick_opens_the_same_page_in_the_other_release()
{
    assert_eq!(moved_path("/docs/x/", "/", "/v0.2/"), "/v0.2/docs/x/");
    assert_eq!(moved_path("/v0.2/learn/", "/v0.2/", "/"), "/learn/");
    assert_eq!(moved_path("/v0.2/", "/v0.2/", "/v0.1/"), "/v0.1/");
    assert_eq!(moved_path("/elsewhere/", "/v0.2/", "/"), "/");
}
