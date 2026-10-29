use zaka_website::components::sandbox::run::Fence;
use zaka_website::components::sandbox::run::FenceOptions;
use zaka_website::components::sandbox::run::run_to_end;
use zaka_website::learn::PAGES;
use zaka_website::learn::chapters;
use zaka_website::learn::neighbours;
use zaka_website::learn::page_at;
use zaka_website::learn::render::Block;
use zaka_website::learn::render::Head;
use zaka_website::learn::render::Sample;
use zaka_website::learn::render::anchor;
use zaka_website::learn::render::link_target;
use zaka_website::learn::render::render;
use zaka_website::learn::slugs;
use zakac_borrow_checker::options::MemCheck;

/// Each program of the Learn pages with the page it is on.
fn samples() -> Vec<(&'static str, Sample)>
{
    let zaka = |b: Block| match b
    {
        Block::Zaka(s) => Some(s),
        Block::Prose(_) => None,
    };
    let page_samples = |p: &zaka_website::learn::Page| {
        let path = p.path;
        render(p).blocks.into_iter().filter_map(zaka).map(move |s| (path, s))
    };
    PAGES.iter().flat_map(page_samples).collect()
}

/// What is wrong with one program, measured against what its page promises.
fn mismatch(path: &str, sample: &Sample) -> Option<String>
{
    let (codes, out) = run_to_end(&sample.source, &sample.options);
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
fn each_program_of_the_learn_pages_does_what_its_page_promises()
{
    let all = samples();
    assert!(all.len() > 100, "the Learn pages hold their programs: {}", all.len());
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
fn each_link_to_a_learn_page_reaches_one()
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

#[test]
fn fence_words_with_an_equals_sign_are_options()
{
    let (fence, options) = Fence::parse("zaka-stop mem-check=warn").expect("a fence");
    assert_eq!(fence, Fence::Stop);
    assert_eq!(options, FenceOptions::builder().check(MemCheck::Warn).build());

    let (fence, options) =
        Fence::parse("zaka-error ZK0717 no-raw=on").expect("a fence");
    assert_eq!(fence.codes(), ["ZK0717"]);
    assert!(options.no_raw);
    assert_eq!(options.check, MemCheck::Error);

    let (_, options) = Fence::parse("zaka mem-strict=on").expect("a fence");
    assert!(options.strict);

    let (_, plain) = Fence::parse("zaka").expect("a fence");
    assert_eq!(plain, FenceOptions::default());

    assert_eq!(Fence::parse("zaka-error mem-check=loud"), None);
    assert_eq!(Fence::parse("zaka no-raw=off"), None);
    assert_eq!(Fence::parse("zaka mem-lint=on"), None);
    assert_eq!(Fence::parse("zaka-stop mem-check"), None);
    assert_eq!(Fence::parse("zaka-stop mem-check:off"), None);
    assert_eq!(Fence::parse("zaka-error ZK07"), None);
    assert_eq!(Fence::parse("zaka-error zk0709"), None);
    assert_eq!(Fence::parse("text mem-check=warn"), None);
}

#[test]
fn a_warn_fence_runs_a_program_the_check_refuses()
{
    let source = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။\n\"အစ\" ဟု ပြသပါ။\nပ ၏ တန်ဖိုး ကို ပြသပါ။";
    let (codes, out) = run_to_end(source, &FenceOptions::default());
    assert!(codes.contains(&"ZK0709".to_string()), "{codes:?}");
    assert_eq!(out, "");

    let warn = FenceOptions::builder().check(MemCheck::Warn).build();
    let (codes, out) = run_to_end(source, &warn);
    assert!(codes.contains(&"ZK0709".to_string()), "{codes:?}");
    assert!(out.starts_with("အစ\n"), "{out}");
    assert!(out.contains("null address"), "{out}");
}

#[test]
fn the_pointer_chapter_teaches_owners_borrows_and_raw_addresses()
{
    let page = page_at("pointers/ownership").expect("the page is there");
    let zaka = |b: Block| match b
    {
        Block::Zaka(s) => Some(s),
        Block::Prose(_) => None,
    };
    let codes: Vec<String> = render(&page)
        .blocks
        .into_iter()
        .filter_map(zaka)
        .flat_map(|s| s.fence.codes().to_vec())
        .collect();
    let wanted = ["ZK0701", "ZK0703", "ZK0711", "ZK0714", "ZK0715", "ZK0717"];
    let missing: Vec<_> =
        wanted.iter().filter(|c| !codes.contains(&c.to_string())).collect();
    assert!(missing.is_empty(), "codes the page does not show: {missing:?}");
    let words = ["လိပ်စာအကြမ်း", "တန်ဖိုးအငှား"];
    let absent: Vec<_> = words.iter().filter(|w| !page.text.contains(*w)).collect();
    assert!(absent.is_empty(), "words the page does not teach: {absent:?}");
}

#[test]
fn each_zaka_fence_of_the_learn_pages_parses()
{
    let offenders: Vec<String> = PAGES
        .iter()
        .flat_map(|p| {
            p.text
                .lines()
                .filter_map(|l| l.strip_prefix("```"))
                .filter(|info| info.starts_with("zaka") && Fence::parse(info).is_none())
                .map(|info| format!("{}: {info}", p.path))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(offenders.is_empty(), "fences that do not parse: {offenders:#?}");
}
