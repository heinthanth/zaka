use std::path::PathBuf;

use gen_lsp_types::Code;
use gen_lsp_types::Contents;
use gen_lsp_types::Position;
use gen_lsp_types::Uri;
use zakac_lsp::check::Buffers;
use zakac_lsp::check::FileCheck;
use zakac_lsp::check::check_file;
use zakac_lsp::complete::completions;
use zakac_lsp::hints::call_hints;
use zakac_lsp::hover::hover;
use zakac_lsp::lines::LineIndex;
use zakac_lsp::names::definition;
use zakac_lsp::names::references;
use zakac_lsp::names::rename;
use zakac_lsp::outline::outline;
use zakac_lsp::tokens::TOKEN_KINDS;
use zakac_lsp::tokens::semantic_tokens;

/// A folder for the files of a test; nothing is written to it.
fn folder() -> PathBuf { std::env::temp_dir().join("zakac_lsp_tests") }

/// Check a file held open with `text`, beside the other open files.
fn check(name: &str, text: &str, buffers: Buffers) -> FileCheck
{
    let path = folder().join(name);
    let uri = Uri::from_file_path(&path).unwrap();
    check_file(&uri, &path, text, buffers)
}

/// The UTF-16 column of the first `needle` in line `line` of `text`.
fn column(text: &str, line: usize, needle: &str) -> u32
{
    let row = text.lines().nth(line).unwrap();
    row[..row.find(needle).unwrap()].encode_utf16().count() as u32
}

#[test]
fn line_index_counts_utf16_columns()
{
    let text = "ab\nကခ\n";
    let lines = LineIndex::new(text);
    let at = text.find('ခ').unwrap() as u32;
    assert_eq!(lines.position(at), Position { line: 1, character: 1 });
    assert_eq!(lines.offset(Position { line: 1, character: 1 }), at);
    assert_eq!(lines.offset(Position { line: 0, character: 99 }), 2);
}

#[test]
fn an_unknown_name_gets_its_code_and_range()
{
    let text = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nမရှိသောအမည် ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let file = check("unknown.zaka", text, Buffers::default());
    let found = &file.diagnostics[0];
    assert_eq!(found.code, Some(Code::String("ZK0301".into())));
    let start = Position {
        line: 1, character: column(text, 1, "မရှိသောအမည်")
    };
    assert_eq!(found.range.start, start);
}

#[test]
fn an_import_reads_the_unsaved_text()
{
    let lib = "အများသုံး ကိန်းရှင် အကြိမ် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး \
               ဖြစ်ပါစေ။\nအများသုံး ကိန်းရှင် အဆုံး သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး \
               ဖြစ်ပါစေ။\n";
    let text = "\"lib/unsaved.zaka\" မှ အကြိမ် နှင့် အဆုံး တို့ကို ထည့်သွင်းမည်။\nပရိုဂရမ်သည် အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nအကြိမ် ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let missing = check("main.zaka", text, Buffers::default());
    assert!(!missing.diagnostics.is_empty());
    let mut buffers = Buffers::default();
    buffers.open.insert(folder().join("lib").join("unsaved.zaka"), lib.into());
    let found = check("main.zaka", text, buffers);
    assert!(found.diagnostics.is_empty(), "{:?}", found.diagnostics);
}

#[test]
fn comments_color_as_comments()
{
    let text = "// မှတ်ချက်\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let file = check("comment.zaka", text, Buffers::default());
    let first = semantic_tokens(&file)[0];
    assert_eq!(TOKEN_KINDS[first.token_type as usize], "comment");
    assert_eq!((first.delta_line, first.delta_start), (0, 0));
    assert_eq!(first.length, "// မှတ်ချက်".encode_utf16().count() as u32);
}

#[test]
fn hover_gives_the_kind_of_a_variable()
{
    let text = include_str!("../../../samples/basics/text.zaka");
    let file = check("text.zaka", text, Buffers::default());
    let row = text.lines().nth(7).unwrap();
    let at = row[..row.rfind(" န ").unwrap() + 1].encode_utf16().count() as u32;
    let found = hover(&file, Position { line: 7, character: at }).unwrap();
    let Contents::MarkupContent(content) = found.contents
    else
    {
        panic!("a hover holds markup")
    };
    assert!(content.value.starts_with("```zaka\nန: usize\n```"), "{}", content.value);
    assert!(content.value.contains("loop variable"), "{}", content.value);
    assert!(content.value.contains("size = 8, align = 8"), "{}", content.value);
}

#[test]
fn a_module_needs_no_start_block()
{
    let text = include_str!("../../../samples/modules/lib/point.zaka");
    let found = check("point.zaka", text, Buffers::default());
    assert!(found.diagnostics.is_empty(), "{:?}", found.diagnostics);
}

#[test]
fn a_program_with_no_start_block_is_reported()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\n";
    let found = check("nostart.zaka", text, Buffers::default());
    let codes: Vec<_> = found.diagnostics.iter().map(|d| d.code.clone()).collect();
    assert!(codes.contains(&Some(Code::String("ZK0327".into()))), "{codes:?}");
}

/// The point module and a program that imports it, both open.
fn points() -> (FileCheck, &'static str)
{
    let lib = include_str!("../../../samples/modules/lib/point.zaka");
    let text = include_str!("../../../samples/modules/points.zaka");
    let mut buffers = Buffers::default();
    buffers.open.insert(folder().join("lib").join("point.zaka"), lib.into());
    (check("points.zaka", text, buffers), text)
}

/// The position of the first `needle` on line `line` of `text`.
fn at(text: &str, line: usize, needle: &str) -> Position
{
    Position { line: line as u32, character: column(text, line, needle) }
}

#[test]
fn a_local_goes_to_its_declaration()
{
    let text = include_str!("../../../samples/basics/text.zaka");
    let file = check("text.zaka", text, Buffers::default());
    let row = text.lines().nth(8).unwrap();
    let use_at = row[..row.rfind(" န ").unwrap() + 1].encode_utf16().count() as u32;
    let found = definition(&file, Position { line: 8, character: use_at }).unwrap();
    assert_eq!(found.range.start, at(text, 6, "န "));
}

#[test]
fn an_imported_function_goes_to_its_file()
{
    let (file, text) = points();
    let found = definition(&file, at(text, 13, "ရွှေ့")).unwrap();
    assert!(found.uri.path().ends_with("lib/point.zaka"), "{}", found.uri);
}

#[test]
fn a_struct_name_and_a_field_go_to_their_declarations()
{
    let (file, text) = points();
    let kind = definition(&file, at(text, 3, "နေရာ")).unwrap();
    let field = definition(&file, at(text, 3, "x အနေဖြင့် ၃")).unwrap();
    assert!(kind.uri.path().ends_with("lib/point.zaka"));
    assert_eq!((kind.range.start.line, field.range.start.line), (2, 2));
}

#[test]
fn references_reach_each_file()
{
    let (file, text) = points();
    let found = references(std::iter::once(&file), &file, at(text, 13, "ရွှေ့"), true);
    let in_lib = found.iter().filter(|l| l.uri.path().ends_with("lib/point.zaka"));
    assert_eq!(in_lib.count(), 1, "{found:?}");
    assert!(found.len() >= 3, "{found:?}");
}

#[test]
fn a_rename_to_a_reserved_word_is_refused()
{
    let (file, text) = points();
    let pos = at(text, 13, "ရွှေ့");
    assert!(rename(std::iter::once(&file), &file, pos, "ပြီးဆုံးပါပြီး").is_err());
    let edit = rename(std::iter::once(&file), &file, pos, "ရွှေ့ပါ").unwrap();
    assert_eq!(edit.changes.unwrap().len(), 2);
}

#[test]
fn the_outline_holds_the_functions_and_the_start_block()
{
    let text = include_str!("../../../samples/basics/text.zaka");
    let file = check("text.zaka", text, Buffers::default());
    let names: Vec<String> = outline(&file).into_iter().map(|s| s.name).collect();
    assert_eq!(names, ["ကွက်လပ်ရေ", "ပရိုဂရမ်"]);
}

#[test]
fn completion_offers_reserved_words_and_declared_names()
{
    let (file, text) = points();
    let labels: Vec<String> = completions(Some(&file), at(text, 17, "နှစ်ထပ်"))
        .into_iter()
        .map(|c| c.label)
        .collect();
    assert!(labels.iter().any(|l| l == "ပရိုဂရမ်"));
    assert!(labels.iter().any(|l| l == "နှစ်ထပ်"));
}

#[test]
fn an_alias_goes_to_its_declaration()
{
    let text = include_str!("../../../samples/structs/aliases.zaka");
    let file = check("aliases.zaka", text, Buffers::default());
    let found = definition(&file, at(text, 15, "ကျောင်းသား")).unwrap();
    assert_eq!(found.range.start, at(text, 2, "ကျောင်းသား"));
}

#[test]
fn a_constant_above_its_declaration_goes_to_it()
{
    let text = "ကိန်းရှင် အမှတ်များ သည် အကွက် အရေ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။\nကိန်းသေ အရေ \
                သည် ၃ ဖြစ်ပါစေ။\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nအမှတ်များ ကို ပြသမည်။\nပရိုဂရမ် \
                ပြီးဆုံးပါပြီး။\n";
    let file = check("soon.zaka", text, Buffers::default());
    let found = definition(&file, at(text, 0, "အရေ")).unwrap();
    assert_eq!(found.range.start, at(text, 1, "အရေ"));
}

#[test]
fn completion_keeps_to_the_scope_at_the_cursor()
{
    let text = include_str!("../../../samples/basics/text.zaka");
    let file = check("text.zaka", text, Buffers::default());
    let names = |line| -> Vec<String> {
        let found = completions(Some(&file), Position { line, character: 0 });
        let names = found.into_iter().filter(|c| c.detail.is_none());
        names.map(|c| c.label).collect()
    };
    assert_eq!(names(7), ["ကွက်လပ်ရေ", "စာ", "န", "အဖြေ"]);
    assert_eq!(names(15), ["ကွက်လပ်ရေ", "နှုတ်ခွန်း"]);
}

/// The labels completion offers at the end of line `line` of `text`.
fn offered_at_end(text: &str, line: usize) -> Vec<String>
{
    let file = check("writing.zaka", text, Buffers::default());
    let row = text.lines().nth(line).unwrap();
    let pos =
        Position { line: line as u32, character: row.encode_utf16().count() as u32 };
    completions(Some(&file), pos).into_iter().map(|c| c.label).collect()
}

#[test]
fn completion_after_ei_offers_the_fields_of_a_chain()
{
    let text = "နေရာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု \
                တို့ပါဝင်သည်။\nလူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အိမ် ဟူသော နေရာ တစ်ခု \
                တို့ပါဝင်သည်။\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် က သည် လူ တစ်ခု ဖြစ်ပါစေ။\nက ၏ \
                \nက ၏ အိမ် ၏ \nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    assert_eq!(offered_at_end(text, 4), ["အသက်", "အိမ်"]);
    assert_eq!(offered_at_end(text, 5), ["x", "y"]);
}

#[test]
fn completion_after_among_offers_the_members()
{
    let text = "အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။\nပရိုဂရမ်သည် အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nကိန်းရှင် က သည် အရောင်များထဲမှ \nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    assert_eq!(offered_at_end(text, 2), ["နီ", "ဝါ", "စိမ်း"]);
}

#[test]
fn a_kind_from_a_value_gives_fields_while_a_line_does_not_parse()
{
    let lib = include_str!("../../../samples/modules/lib/point.zaka");
    let text = "\"lib/point.zaka\" မှ နေရာ နှင့် ရွှေ့ တို့ကို ထည့်သွင်းမည်။\nပရိုဂရမ်သည် အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nကိန်းရှင် က သည် နေရာ တစ်ခု ဖြစ်ပါစေ။\nကိန်းရှင် ခ သည် ရွှေ့ ဟူသော \
                လုပ်ဆောင်ချက်ကို က နှင့် က တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\nခ ၏ \nပရိုဂရမ် \
                ပြီးဆုံးပါပြီး။\n";
    let mut buffers = Buffers::default();
    buffers.open.insert(folder().join("lib").join("point.zaka"), lib.into());
    let file = check("value.zaka", text, buffers);
    let pos = Position { line: 4, character: "ခ ၏ ".encode_utf16().count() as u32 };
    let labels: Vec<String> =
        completions(Some(&file), pos).into_iter().map(|c| c.label).collect();
    assert_eq!(labels, ["x", "y"]);
}

#[test]
fn completion_after_ei_goes_through_list_cells()
{
    let text = "နေရာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု \
                တို့ပါဝင်သည်။\nလူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အိမ် ဟူသော နေရာ တစ်ခု \
                တို့ပါဝင်သည်။\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် လူများ သည် အကွက် ၃ ခုရှိသော လူ \
                စာရင်း တစ်ခု ဖြစ်ပါစေ။\nကိန်းရှင် n သည် ၁ ဖြစ်ပါစေ။\nလူများ ၏ အကွက် n + ၁ ၏ \
                \nလူများ ၏ နောက်ဆုံးအကွက် ၏ အိမ် ၏ \nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    assert_eq!(offered_at_end(text, 5), ["အသက်", "အိမ်"]);
    assert_eq!(offered_at_end(text, 6), ["x", "y"]);
}

#[test]
fn hover_shows_the_doc_lines_above_a_declaration()
{
    let text = include_str!("../../../samples/lists/sort.zaka");
    let file = check("sort.zaka", text, Buffers::default());
    let line = text.lines().position(|l| l.contains("အစဉ်လိုက်စီ ဟူသော လုပ်ဆောင်ချက်ကို")).unwrap();
    let found = hover(&file, at(text, line, "အစဉ်လိုက်စီ")).unwrap();
    let Contents::MarkupContent(content) = found.contents
    else
    {
        panic!("a hover holds markup")
    };
    assert!(
        content.value.contains("\n---\n\nစာရင်းကို ငယ်စဉ်ကြီးလိုက် စီသည်"),
        "{}",
        content.value
    );
    assert!(!content.value.contains("ပူဖောင်း"), "{}", content.value);
}

/// A program with a function that can change its first parameter, and an
/// unsafe function that takes a block over.
const MARKED: &str = "လုပ်ဆောင်ချက် တိုး သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ခ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စိတ်မချရသော လုပ်ဆောင်ချက် ရှင်း သည်
ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် န သည် ၁ ဖြစ်ပါစေ။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို န နှင့် ၂ တို့ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

#[test]
fn call_hints_mark_the_arguments_a_call_can_change()
{
    let file = check("marked.zaka", MARKED, Buffers::default());
    assert!(file.diagnostics.is_empty(), "{:?}", file.diagnostics);
    let found = call_hints(&file);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].position, at(MARKED, 12, "န နှင့်"));
}

#[test]
fn hover_shows_the_marks_of_a_function_and_its_parameters()
{
    let file = check("marked.zaka", MARKED, Buffers::default());
    let text = |pos| {
        let Some(Contents::MarkupContent(c)) = hover(&file, pos).map(|h| h.contents)
        else
        {
            panic!("a hover holds markup")
        };
        c.value
    };
    let edit = text(at(MARKED, 0, "တိုး"));
    assert!(edit.contains("က: ဒေတာ ပြောင်းလဲရန် isize"), "{edit}");
    let owned = text(at(MARKED, 5, "ရှင်း"));
    assert!(owned.starts_with("```zaka\nစိတ်မချရသော လုပ်ဆောင်ချက် ရှင်း"), "{owned}");
    assert!(owned.contains("ပ: အပိုင်အနေဖြင့် isize*"), "{owned}");
}

/// A function of the stdlib goes to a replica of its module in the cache
/// folder, and the replica reads as the module, with no diagnostic.
#[test]
fn a_stdlib_function_goes_to_a_replica_of_its_module()
{
    let text = "\"စံကား/အဝင်အထွက်\" မှ စာကြောင်းတစ်ကြောင်းဖတ် နှင့် စံပြုအဝင် တို့ကို ထည့်သွင်းမည်။\n\nပရိုဂရမ်သည် \
                အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် လိုင်း သည် စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် \
                ဖြစ်ပါစေ။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let file = check("stdcopy.zaka", text, Buffers::default());
    let found = definition(&file, at(text, 3, "စာကြောင်းတစ်ကြောင်းဖတ် သည့်")).unwrap();
    let path = found.uri.to_file_path().unwrap();
    assert!(path.ends_with("io.zaka"), "{path:?}");
    let replica = std::fs::read_to_string(&path).unwrap();
    let uri = Uri::from_file_path(&path).unwrap();
    let opened = check_file(&uri, &path, &replica, Buffers::default());
    let codes: Vec<_> = opened.diagnostics.iter().map(|d| d.code.clone()).collect();
    assert!(codes.is_empty(), "{codes:?}");
}
