use zaka_explorer::compile::Options;
use zaka_explorer::compile::compile;
use zaka_explorer::compile::samples::find_sample;
use zaka_explorer::monaco::decor::decorations;
use zaka_explorer::monaco::decor::markers;

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

#[test]
fn a_label_shows_as_a_note_marker_beside_its_error()
{
    let text = find_sample("mistakes/use_after_free").unwrap().text;
    let out = compile(text, &Options::default());
    let found = markers(&out);
    let levels: Vec<u32> = found.iter().map(|m| m.level).collect();
    assert_eq!(levels, [8, 2], "an error, then a note");
    let (range, label) = &out.diagnostics[0].labels[0];
    let (lo, hi) = out.table.range_to_utf16(range);
    let note = &found[1];
    assert_eq!((note.lo, note.hi), (lo, hi));
    assert_eq!((note.code.as_str(), note.message.as_str()), ("ZK0704", label.as_str()));
}
