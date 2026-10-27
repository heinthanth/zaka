use zaka_website::components::sandbox::run::Fence;
use zaka_website::components::sandbox::run::run_to_end;
use zaka_website::manual::PAGES;
use zaka_website::manual::chapters;
use zaka_website::manual::neighbours;
use zaka_website::manual::page_at;
use zaka_website::manual::render::Block;
use zaka_website::manual::render::Head;
use zaka_website::manual::render::Sample;
use zaka_website::manual::render::anchor;
use zaka_website::manual::render::link_target;
use zaka_website::manual::render::render;
use zaka_website::manual::slugs;

/// Each program of the manual with the page it is on.
fn samples() -> Vec<(&'static str, Sample)>
{
    let zaka = |b: Block| match b
    {
        Block::Zaka(s) => Some(s),
        Block::Prose(_) => None,
    };
    let page_samples = |p: &zaka_website::manual::Page| {
        let path = p.path;
        render(p).blocks.into_iter().filter_map(zaka).map(move |s| (path, s))
    };
    PAGES.iter().flat_map(page_samples).collect()
}

/// What is wrong with one program, measured against what its page promises.
fn mismatch(path: &str, sample: &Sample) -> Option<String>
{
    let (codes, out) = run_to_end(&sample.source);
    let missing: Vec<_> =
        sample.fence.codes().iter().filter(|c| !codes.contains(c)).collect();
    let wrong = match &sample.fence
    {
        Fence::Run | Fence::Stop if out != sample.expected =>
        {
            Some(format!("prints\n{out}\nnot\n{}", sample.expected))
        }
        Fence::Error(_) | Fence::Warning(_) if !missing.is_empty() =>
        {
            Some(format!("reports {codes:?}, not {missing:?}"))
        }
        _ => None,
    };
    wrong.map(|w| format!("{path}: {w}\n---\n{}\n", sample.source))
}

#[test]
fn each_program_of_the_manual_does_what_its_page_promises()
{
    let all = samples();
    assert!(all.len() > 100, "the manual holds its programs: {}", all.len());
    let wrong: Vec<String> = all.iter().filter_map(|(p, s)| mismatch(p, s)).collect();
    assert!(
        wrong.is_empty(),
        "{} of {} programs differ:\n\n{}",
        wrong.len(),
        all.len(),
        wrong.join("\n")
    );
}

#[test]
fn the_runs_and_stops_promise_an_output()
{
    let blank: Vec<_> = samples()
        .into_iter()
        .filter(|(_, s)| {
            matches!(s.fence, Fence::Run | Fence::Stop) && s.expected.is_empty()
        })
        .map(|(p, s)| format!("{p}: {}", s.source))
        .collect();
    assert!(blank.is_empty(), "{blank:#?}");
}

#[test]
fn each_link_to_a_page_of_the_manual_reaches_one()
{
    let stale: Vec<String> = PAGES
        .iter()
        .flat_map(|p| {
            let prose = render(p).blocks.into_iter().filter_map(|b| match b
            {
                Block::Prose(html) => Some(html),
                Block::Zaka(_) => None,
            });
            prose
                .filter(|h| h.contains(".md\"") || h.contains(".md#"))
                .map(|_| p.path.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(stale.is_empty(), "links to Markdown files remain on {stale:?}");
}

#[test]
fn links_point_at_the_pages_of_the_site()
{
    let page = page_at("control-flow/while").expect("the page is there");
    assert_eq!(
        link_target(&page, "counted-loops.md"),
        "/learn/control-flow/counted-loops/"
    );
    assert_eq!(link_target(&page, "../hello/README.md"), "/learn/hello/");
    assert_eq!(link_target(&page, "README.md#top"), "/learn/control-flow/#top");
    assert_eq!(link_target(&page, "https://zaka-lang.org/explorer/"), "/explorer/");
    assert_eq!(
        link_target(&page, "https://www.rust-lang.org/"),
        "https://www.rust-lang.org/"
    );
    let title = page_at("").expect("the title page is there");
    assert_eq!(link_target(&title, "CONTENTS.md"), "/learn/#contents");
}

#[test]
fn heads_get_ids_of_their_own()
{
    let first = Head::builder().id("a-loop".into()).text("A loop".into()).build();
    assert_eq!(anchor("A loop", &[]), "a-loop");
    assert_eq!(anchor("A loop", std::slice::from_ref(&first)), "a-loop-2");
    assert_eq!(anchor("`x` and (z)", &[]), "x-and-z");
}

#[test]
fn the_addresses_of_the_pages_differ()
{
    let mut all = slugs();
    let count = all.len();
    all.sort();
    all.dedup();
    assert_eq!(all.len(), count);
    assert_eq!(count, PAGES.len());
    assert!(all.contains(&String::new()), "the title page is at the root");
}

#[test]
fn the_contents_hold_each_page_once()
{
    let listed: usize = chapters().iter().map(|c| 1 + c.pages.len()).sum();
    assert_eq!(listed + 1, PAGES.len(), "each page but the title page is in a chapter");
    let numbered = chapters().iter().filter(|c| c.number.is_some()).count();
    assert!(numbered >= 9, "{numbered}");
    let first = page_at("").expect("the title page is there");
    assert_eq!(neighbours(&first).0, None);
    assert_eq!(neighbours(&first).1.map(|p| p.slug), Some("hello"));
}
