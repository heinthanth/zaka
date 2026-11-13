use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use zakac_ast::expr::Expr;
use zakac_ast::expr::NodeId;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_expr;
use zakac_diagnostics::Code;
use zakac_host_errors::Error;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::Loaded;
use zakac_parser::load::load;
use zakac_span::file_loader::MemFileLoader;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

use crate::common::codes;
use crate::common::stmt;

/// Read `root` and its imports from files held in RAM.
fn load_files(files: &[(&str, &str)], root: &str) -> zakac_host_errors::Result<Loaded>
{
    let mut loader = MemFileLoader::new();
    files.iter().for_each(|(path, text)| loader.insert(*path, text));
    let mut sm = SourceMap::with_loader(Arc::new(loader));
    let file = sm.load_file(Path::new(root)).unwrap();
    load(&file, &mut sm, &LexerOptions::default())
}

/// The diagnostic codes of a loaded program.
fn loaded_codes(loaded: &Loaded) -> Vec<Code>
{
    loaded.diagnostics.iter().map(|d| d.code).collect()
}

/// Collects the number of each expression node.
#[derive(Default)]
struct Ids(Vec<NodeId>);

impl Visitor for Ids
{
    fn visit_expr(&mut self, e: &Expr)
    {
        self.0.push(e.id());
        walk_expr(self, e);
    }
}

#[test]
fn import_sentences()
{
    let several = "\"ဂဏန်း.zaka\" မှ ပေါင်း နှင့် နုတ် တို့ကို ထည့်သွင်းမည်။";
    assert_eq!(stmt(several), "(import \"ဂဏန်း.zaka\" ပေါင်း နုတ်)");
    let one = "\"ဂဏန်း.zaka\" မှ ပေါင်း ကို ထည့်သွင်းမည်။";
    assert_eq!(stmt(one), "(import \"ဂဏန်း.zaka\" ပေါင်း)");
}

#[test]
fn the_export_mark_sets_the_flag_of_a_declaration()
{
    let var = "အများသုံး ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။";
    assert!(stmt(var).starts_with("(export (var က"), "{}", stmt(var));
    let record = "လူ ဟူသော အများသုံး ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။";
    assert!(stmt(record).starts_with("(export (struct လူ"), "{}", stmt(record));
    let func =
        "အများသုံး လုပ်ဆောင်ချက် ဖ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n၁ ကို ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert!(stmt(func).starts_with("(export (fn ဖ"), "{}", stmt(func));
    let plain = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။";
    assert!(stmt(plain).starts_with("(var က"), "{}", stmt(plain));
}

#[test]
fn the_export_mark_before_a_statement_is_zk0229()
{
    assert_eq!(codes("အများသုံး ၁ ကို ပြသမည်။"), [Code::ExportNotOnDecl]);
    let nested = "အကယ်၍ မှန် ဖြစ်လျှင် အများသုံး ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။";
    assert_eq!(codes(nested), [Code::ExportNotOnDecl]);
}

#[test]
fn an_import_below_a_statement_is_zk0230()
{
    let late = "၁ ကို ပြသမည်။\n\"က.zaka\" မှ က ကို ထည့်သွင်းမည်။";
    assert_eq!(codes(late), [Code::ImportNotFirst]);
    let first = "\"က.zaka\" မှ က ကို ထည့်သွင်းမည်။\n\"ခ.zaka\" မှ ခ ကို ထည့်သွင်းမည်။\n၁ ကို ပြသမည်။";
    assert_eq!(codes(first), []);
}

#[test]
fn a_file_imported_twice_is_read_once_and_imports_come_first()
{
    let files = [
        (
            "/m/main.zaka",
            "\"a.zaka\" မှ က ကို ထည့်သွင်းမည်။\n\"lib/b.zaka\" မှ ခ ကို ထည့်သွင်းမည်။\nက + ခ ကို ပြသမည်။",
        ),
        (
            "/m/a.zaka",
            "\"lib/c.zaka\" မှ ဂ ကို ထည့်သွင်းမည်။\nအများသုံး ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \
             ၁ + ၁ ဖြစ်ပါစေ။",
        ),
        (
            "/m/lib/b.zaka",
            "\"./c.zaka\" မှ ဂ ကို ထည့်သွင်းမည်။\nအများသုံး ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ \
             + ၂ ဖြစ်ပါစေ။",
        ),
        ("/m/lib/c.zaka", "အများသုံး ကိန်းရှင် ဂ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ + ၃ ဖြစ်ပါစေ။"),
    ];
    let loaded = load_files(&files, "/m/main.zaka").unwrap();
    assert_eq!(loaded_codes(&loaded), []);
    let program = &loaded.program;
    let names: Vec<&str> = program.units.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(names, ["lib/c.zaka", "a.zaka", "lib/b.zaka", "main.zaka"]);
    let imported: Vec<&str> =
        loaded.imports.iter().map(|f| f.source.as_str()).collect();
    assert_eq!(imported.len(), 3, "{imported:?}");
    let targets: Vec<Option<u32>> = program
        .stmts
        .iter()
        .filter_map(|s| match s
        {
            Stmt::Import(n) => Some(n.unit),
            _ => None,
        })
        .collect();
    assert_eq!(targets, [Some(0), Some(0), Some(1), Some(2)]);
    let mut ids = Ids::default();
    ids.visit_program(program);
    let unique: HashSet<NodeId> = ids.0.iter().copied().collect();
    assert_eq!(unique.len(), ids.0.len(), "node numbers repeat across files");
    assert!(program.is_root(3));
    assert_eq!(program.unit_of(program.stmts.len() - 1), 3);
}

#[test]
fn files_that_import_each_other_are_zk0231()
{
    let files = [
        ("/m/a.zaka", "\"b.zaka\" မှ ခ ကို ထည့်သွင်းမည်။"),
        (
            "/m/b.zaka",
            "\"a.zaka\" မှ က ကို ထည့်သွင်းမည်။\nအများသုံး ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။",
        ),
    ];
    let loaded = load_files(&files, "/m/a.zaka").unwrap();
    assert_eq!(loaded_codes(&loaded), [Code::CircularImport]);
    assert!(loaded.diagnostics[0].message.contains("a.zaka → b.zaka → a.zaka"));
}

#[test]
fn a_missing_import_is_a_host_error()
{
    let files = [("/m/a.zaka", "\"nope.zaka\" မှ ခ ကို ထည့်သွင်းမည်။")];
    let err = load_files(&files, "/m/a.zaka").unwrap_err();
    assert!(
        matches!(err, Error::FileNotFound { ref path } if path == Path::new("/m/nope.zaka"))
    );
}

#[test]
fn a_program_not_read_from_a_file_cannot_import()
{
    let mut sm = SourceMap::new();
    let text = "\"a.zaka\" မှ ခ ကို ထည့်သွင်းမည်။\n၁ ကို ပြသမည်။";
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    assert_eq!(loaded_codes(&loaded), [Code::ImportNeedsFile]);
    assert!(loaded.program.units.is_empty());
}

#[test]
fn an_imported_struct_name_starts_a_kind()
{
    let files = [
        (
            "/m/main.zaka",
            "\"lib.zaka\" မှ နေရာ ကို ထည့်သွင်းမည်။\nကိန်းရှင် န သည် နေရာ တစ်ခု ဖြစ်ပါစေ။",
        ),
        ("/m/lib.zaka", "နေရာ ဟူသော အများသုံး ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။"),
    ];
    let loaded = load_files(&files, "/m/main.zaka").unwrap();
    assert_eq!(loaded_codes(&loaded), []);
    let last = loaded.program.stmts.last().unwrap().to_sexpr();
    assert!(last.contains("နေရာ"), "{last}");
}

/// A file of the `stdlib` folder of the source tree, opened from storage as an
/// editor opens it, reads as its module, so its compiler marks hold.
#[test]
fn a_stdlib_file_read_from_storage_reads_as_its_module()
{
    let tree =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/core_verbs.zaka");
    let mut sm = SourceMap::new();
    let file = sm.load_file(&tree).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    let names: Vec<&str> =
        loaded.program.units.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(names.last(), Some(&"စံကား/အခြေခံအုတ်မြစ်"));
}
