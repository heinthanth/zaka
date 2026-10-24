use strum::IntoEnumIterator;
use zakac_explorer::compile::Options;
use zakac_explorer::compile::PLATFORM_BITS;
use zakac_explorer::compile::Phase;
use zakac_explorer::compile::samples::SAMPLES;
use zakac_explorer::state::permalink::Link;
use zakac_explorer::state::permalink::decode;
use zakac_explorer::state::permalink::encode;
use zakac_explorer::state::settings::AstMode;

/// A link to a source with the default options.
fn link(source: &str) -> Link
{
    Link::builder().source(source).options(Options::default()).build()
}

/// A link to `x` that opens a stage and an AST view.
fn link_at(stage: Phase, ast_mode: AstMode) -> Link
{
    let options = Options::default();
    Link::builder().source("x").options(options).stage(stage).ast_mode(ast_mode).build()
}

#[test]
fn all_samples_and_options_round_trip()
{
    for sample in SAMPLES
    {
        for bits in PLATFORM_BITS
        {
            let options = Options::builder().platform_bits(bits).build();
            let link = Link::builder().source(sample.text).options(options).build();
            let decoded = decode(&format!("#{}", encode(&link)));
            assert_eq!(decoded, Some(link), "{}", sample.name);
        }
    }
}

#[test]
fn links_keep_the_stage_and_the_ast_view()
{
    for stage in Phase::iter()
    {
        for ast_mode in AstMode::iter()
        {
            let link = link_at(stage, ast_mode);
            assert_eq!(decode(&encode(&link)), Some(link));
        }
    }
    let shown = encode(&link_at(Phase::Parse, AstMode::Graph));
    assert!(shown.starts_with("3.64.parse.graph."), "{shown}");
}

#[test]
fn older_links_open_with_the_default_view()
{
    let old = "2.32.eA";
    let decoded = decode(old).unwrap();
    assert_eq!(decoded.source, "x");
    assert_eq!(decoded.options.platform_bits, 32);
    assert_eq!(
        (decoded.stage, decoded.ast_mode),
        (Phase::default(), AstMode::default())
    );
}

#[test]
fn links_are_url_safe()
{
    let hash = encode(&link("ကိန်းရှင် ?/+"));
    assert!(hash.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)));
}

#[test]
fn malformed_links_decode_to_none()
{
    assert!(decode(&encode(&link("x"))).is_some());
    for bad in [
        "",
        "#",
        "1.0.64.eA",
        "4.64.lex.tree.eA",
        "3.12.lex.tree.eA",
        "3.64.lexer.tree.eA",
        "3.64.lex.forest.eA",
        "3.64.lex.tree.@@",
        "3.64.lex.tree",
        "2.12.eA",
        "2.64",
    ]
    {
        assert_eq!(decode(bad), None, "{bad}");
    }
    assert_eq!(decode("3.64.lex.tree._w"), None);
}
