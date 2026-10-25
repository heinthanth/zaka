use zaka_web_ui_common::paint::COMMENT_CLASS;
use zaka_web_ui_common::paint::GAP_CLASS;
use zaka_web_ui_common::paint::paint;

/// Classes of the painted words of `source`, the gaps left out.
///
/// # Arguments
///
/// * `source` - The source text.
///
/// # Returns
///
/// The classes, in order.
fn classes(source: &str) -> Vec<&'static str>
{
    let pieces = paint(source);
    pieces.iter().filter(|p| p.class != GAP_CLASS).map(|p| p.class).collect()
}

#[test]
fn the_pieces_cover_the_text()
{
    let source = "၆ * ၇ ကို ပြသပါ။\n// မှတ်ချက်\n";
    let joined: String = paint(source).iter().map(|p| p.text.as_str()).collect();
    assert_eq!(joined, source);
}

#[test]
fn words_take_the_color_of_their_role()
{
    let found = classes("၆ * ၇ ကို ပြသပါ။");
    let roles =
        ["tok-number", "tok-op", "tok-number", "tok-particle", "tok-verb", "tok-delim"];
    assert_eq!(found, roles);
}

#[test]
fn comments_are_painted_as_comments()
{
    assert_eq!(classes("// မှတ်ချက်"), [COMMENT_CLASS]);
}
