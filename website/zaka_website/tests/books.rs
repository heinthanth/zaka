use zaka_website::book::Book;
use zaka_website::book::Page;
use zaka_website::book::chapters;
use zaka_website::book::neighbours;
use zaka_website::book::page_at;
use zaka_website::book::render::Block;
use zaka_website::book::render::FILE_FENCE;
use zaka_website::book::render::Head;
use zaka_website::book::render::Sample;
use zaka_website::book::render::anchor;
use zaka_website::book::render::link_target;
use zaka_website::book::render::render;
use zaka_website::book::slugs;
use zaka_website::components::sandbox::run::Fence;
use zaka_website::components::sandbox::run::FenceOptions;
use zaka_website::components::sandbox::run::run_to_end;
use zakac_borrow_checker::options::MemCheck;
use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Each page of each book.
fn all_pages() -> impl Iterator<Item = &'static Page>
{
    Book::ALL.into_iter().flat_map(Book::pages)
}

/// Each program of the pages of a book with the page it is on.
fn samples_of(book: Book) -> Vec<(&'static str, Sample)>
{
    let zaka = |b: Block| match b
    {
        Block::Zaka(s) => Some(s),
        Block::Prose(_) => None,
    };
    let page_samples = |p: &Page| {
        let path = p.path;
        render(p).blocks.into_iter().filter_map(zaka).map(move |s| (path, s))
    };
    book.pages().iter().flat_map(page_samples).collect()
}

/// Each program of the pages of each book with the page it is on.
fn samples() -> Vec<(&'static str, Sample)>
{
    Book::ALL.into_iter().flat_map(samples_of).collect()
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
fn each_program_of_the_books_does_what_its_page_promises()
{
    let learn = samples_of(Book::Learn).len();
    assert!(learn > 100, "the Learn pages hold their programs: {learn}");
    let all = samples();
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
fn each_link_to_a_page_of_a_book_reaches_one()
{
    let stale: Vec<String> = all_pages()
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

/// The ids each page holds besides the ids of its heads.
const PAGE_IDS: [&str; 2] = ["top", "contents"];

/// The targets of the links in a piece of HTML.
fn hrefs(html: &str) -> Vec<String>
{
    html.split("href=\"")
        .skip(1)
        .filter_map(|rest| rest.split_once('"').map(|(href, _)| href.to_string()))
        .collect()
}

/// The page of the site at an address, such as `/docs/language/kinds/`.
fn page_at_url(url: &str) -> Option<Page>
{
    Book::ALL.into_iter().find_map(|book| {
        url.strip_prefix(book.root_url()).and_then(|slug| page_at(book, slug))
    })
}

/// What is wrong with one link to a mark on a page.
fn broken_mark(from: &Page, href: &str) -> Option<String>
{
    let (url, mark) = href.split_once('#')?;
    let target = match url
    {
        "" => Some(*from),
        url => page_at_url(url),
    };
    let Some(target) = target
    else
    {
        return (url.starts_with("/learn/") || url.starts_with("/docs/"))
            .then(|| format!("{}: {href} names no page", from.path));
    };
    let heads = render(&target).heads;
    let found = PAGE_IDS.contains(&mark) || heads.iter().any(|h| h.id == mark);
    (!found).then(|| format!("{}: {href} names no head", from.path))
}

#[test]
fn each_link_to_a_mark_reaches_a_head()
{
    let broken: Vec<String> = all_pages()
        .flat_map(|p| {
            let prose = render(p).blocks.into_iter().filter_map(|b| match b
            {
                Block::Prose(html) => Some(html),
                Block::Zaka(_) => None,
            });
            prose
                .flat_map(|h| hrefs(&h))
                .filter_map(|href| broken_mark(p, &href))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(broken.is_empty(), "links to marks that are not there: {broken:#?}");
}

#[test]
fn links_point_at_the_pages_of_the_site()
{
    let page = page_at(Book::Learn, "control-flow/while").expect("the page is there");
    assert_eq!(
        link_target(&page, "counted-loops.md"),
        "/learn/control-flow/counted-loops/"
    );
    assert_eq!(link_target(&page, "../hello/README.md"), "/learn/hello/");
    assert_eq!(link_target(&page, "README.md#top"), "/learn/control-flow/#top");
    assert_eq!(link_target(&page, "/explorer/"), "/explorer/");
    assert_eq!(
        link_target(&page, "https://www.rust-lang.org/"),
        "https://www.rust-lang.org/"
    );
    let title = page_at(Book::Learn, "").expect("the title page is there");
    assert_eq!(link_target(&title, "CONTENTS.md"), "/learn/#contents");
}

#[test]
fn links_cross_from_one_book_to_the_other()
{
    let learn = page_at(Book::Learn, "hello").expect("the page is there");
    assert_eq!(
        link_target(&learn, "../../manual/reference/errors.md"),
        "/docs/reference/errors/"
    );
    let errors = page_at(Book::Manual, "reference/errors").expect("the page is there");
    assert_eq!(
        link_target(&errors, "../../learn/values/literals.md#top"),
        "/learn/values/literals/#top"
    );
    assert_eq!(link_target(&errors, "precedence.md"), "/docs/reference/precedence/");
    assert_eq!(link_target(&errors, "../CONTENTS.md"), "/docs/#contents");
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
    Book::ALL.into_iter().for_each(|book| {
        let mut all = slugs(book);
        let count = all.len();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), count, "{book:?}");
        assert_eq!(count, book.pages().len(), "{book:?}");
        assert!(all.contains(&String::new()), "the title page is at the root");
    });
}

#[test]
fn the_contents_hold_each_page_once()
{
    Book::ALL.into_iter().for_each(|book| {
        let listed: usize = chapters(book).iter().map(|c| 1 + c.pages.len()).sum();
        assert_eq!(
            listed + 1,
            book.pages().len(),
            "{book:?}: a page outside a chapter"
        );
        let first = page_at(book, "").expect("the title page is there");
        assert_eq!(neighbours(&first).0, None);
    });
    let numbered = chapters(Book::Learn).iter().filter(|c| c.number.is_some()).count();
    assert!(numbered >= 9, "{numbered}");
    let first = page_at(Book::Learn, "").expect("the title page is there");
    assert_eq!(neighbours(&first).1.map(|p| p.slug), Some("hello"));
}

#[test]
fn the_error_page_lists_each_code()
{
    let page = page_at(Book::Manual, "reference/errors").expect("the page is there");
    let missing: Vec<&str> = Code::ALL
        .iter()
        .map(|c| c.id())
        .filter(|id| !page.text.contains(&format!("| {id} |")))
        .collect();
    assert!(missing.is_empty(), "codes the error page does not list: {missing:?}");
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
    let (_, options) = Fence::parse("zaka-stop overflow=stop").expect("a fence");
    assert_eq!(options.overflow, zakac_borrow_checker::options::OverflowMode::Stop);
    assert_eq!(Fence::parse("zaka overflow=trap"), None);
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
    let page = page_at(Book::Learn, "pointers/ownership").expect("the page is there");
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
fn each_zaka_fence_of_the_books_parses()
{
    let offenders: Vec<String> = all_pages()
        .flat_map(|p| {
            p.text
                .lines()
                .filter_map(|l| l.strip_prefix("```"))
                .filter(|info| info.starts_with("zaka") && *info != FILE_FENCE)
                .filter(|info| Fence::parse(info).is_none())
                .map(|info| format!("{}: {info}", p.path))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(offenders.is_empty(), "fences that do not parse: {offenders:#?}");
}

/// The text of each fence of a page that shows one file of a longer
/// program.
fn shown_files(text: &str) -> Vec<String>
{
    let open = format!("```{FILE_FENCE}\n");
    let starts = text.match_indices(&open).map(|(at, _)| at + open.len());
    let texts = starts.filter_map(|at| text[at..].split_once("```").map(|(t, _)| t));
    texts.map(str::to_string).collect()
}

#[test]
fn each_shown_file_of_the_books_lexes_without_errors()
{
    let mut sm = SourceMap::new();
    let mut count = 0;
    for page in all_pages()
    {
        for shown in shown_files(page.text)
        {
            let name = FileName::Custom(page.path.to_string());
            let file = sm.new_source_file(name, shown.clone());
            let lexed = lex(&file, &LexerOptions::default());
            assert!(lexed.diagnostics.is_empty(), "{}:\n{shown}", page.path);
            count += 1;
        }
    }
    assert!(count >= 10, "the module pages show their files: {count}");
}

#[test]
fn a_shown_file_is_painted_in_the_prose()
{
    let page = all_pages().find(|p| p.path.ends_with("modules/README.md"));
    let rendered = render(page.expect("the modules page"));
    let prose: String = rendered
        .blocks
        .iter()
        .filter_map(|b| match b
        {
            Block::Prose(html) => Some(html.as_str()),
            Block::Zaka(_) => None,
        })
        .collect();
    assert!(prose.contains("<pre><code class=\"painted\" lang=\"my\">"), "{prose}");
    assert!(prose.contains("tok-kw"));
    assert!(!prose.contains("zaka-file"));
}
