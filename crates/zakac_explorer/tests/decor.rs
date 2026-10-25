use zakac_explorer::compile::Options;
use zakac_explorer::compile::compile;
use zakac_explorer::monaco::decor::decorations;
use zakac_explorer::monaco::decor::markers;

#[test]
fn decorations_cover_tokens_and_comments_in_order()
{
    let out = compile("// note\nက ကို ပြသပါ။", &Options::default());
    let marks = decorations(&out);
    assert_eq!(marks.len(), out.tokens.len() + out.comments.len());
    assert!(marks.windows(2).all(|w| w[0].hi <= w[1].lo));
    assert_eq!(marks[0].class, "tok-comment");
    assert_eq!((marks[0].lo, marks[0].hi), (0, 7));
    // `က` starts after the comment line; UTF-16 offsets equal char counts here.
    assert_eq!((marks[1].lo, marks[1].hi), (8, 9));
}

#[test]
fn markers_use_monaco_levels()
{
    let out = compile("က ; ခ", &Options::default());
    let found = markers(&out);
    // The lone `;` is a lexer error; the parser then reports the broken
    // statement too.
    assert_eq!(found[0].code, "ZK0101");
    assert_eq!((found[0].lo, found[0].hi, found[0].level), (2, 3, 8));
    assert!(found.iter().all(|m| m.hi > m.lo));
}
