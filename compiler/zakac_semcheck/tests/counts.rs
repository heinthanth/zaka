use std::path::Path;
use std::sync::Arc;

use zakac_ast::s_expr::ToSexpr;
use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::load;
use zakac_semcheck::artifact::check_loaded;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::file_loader::MemFileLoader;
use zakac_span::source_map::SourceMap;

/// Check `/m/main.zaka` and the files it imports, from files held in RAM.
///
/// # Returns
///
/// The top-level statements as trees, with their counts worked out, and the
/// diagnostic codes.
fn checked(files: &[(&str, &str)]) -> (Vec<String>, Vec<Code>)
{
    let mut loader = MemFileLoader::new();
    files.iter().for_each(|(path, text)| loader.insert(*path, text));
    let mut sm = SourceMap::with_loader(Arc::new(loader));
    let file = sm.load_file(Path::new("/m/main.zaka")).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    let out = check_loaded(loaded, &SemcheckOptions::default());
    let trees = out.program.stmts.iter().map(ToSexpr::to_sexpr).collect();
    (trees, out.diagnostics.iter().map(|d| d.code).collect())
}

/// The trees and codes of one file whose start block holds `inside`, after
/// the top-level `head`.
fn one(head: &str, inside: &str) -> (Vec<String>, Vec<Code>)
{
    let text =
        format!("{head}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{inside}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။");
    checked(&[("/m/main.zaka", &text)])
}

/// The count of the list `l` that `inside` declares, after the constant `N`
/// of 3: the tree of the start block, and the codes.
fn count(head: &str, count: &str) -> (String, Vec<Code>)
{
    let decl = format!("ကိန်းရှင် l သည် အကွက် {count} ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။");
    let (trees, codes) = one(&format!("ကိန်းသေ N သည် ၃ ဖြစ်ပါစေ။\n{head}"), &decl);
    (trees.last().cloned().unwrap_or_default(), codes)
}

#[test]
fn counts_fold_with_each_integer_operator()
{
    let cases = [
        ("(N + ၁) * ၂ - N % ၂", "isize[7]"),
        ("N << ၂", "isize[12]"),
        ("(N & ၆) | ၁", "isize[3]"),
        ("N ^ ၇", "isize[4]"),
        ("~(-N)", "isize[2]"),
        ("N နှင့် ၂ ၏ ပေါင်းလဒ်", "isize[5]"),
        ("၂.၇ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ်", "isize[2]"),
        ("ကိန်းပြည့် ၏ အရွယ်အစား", "isize[8]"),
        ("အကွက် N ခုရှိသော ၁၆ ဘစ် ကိန်းပြည့် စာရင်း ၏ အရွယ်အစား", "isize[6]"),
    ];
    for (written, want) in cases
    {
        let (tree, codes) = count("", written);
        assert_eq!(codes, [], "{written}");
        assert!(tree.contains(want), "{written}: {tree}");
    }
}

#[test]
fn counts_follow_the_kind_of_each_constant()
{
    let small = "ကိန်းသေ B သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၀၀ ဖြစ်ပါစေ။";
    assert_eq!(count(small, "B * ၂").1, [Code::ConstantOverflow]);
    assert_eq!(count(small, "B").1, []);
    assert_eq!(count("", "N << ၉၉").1, [Code::ShiftTooFar]);
    assert_eq!(count("", "N / (N - ၃)").1, [Code::ConstantZeroDivisor]);
}

#[test]
fn a_top_level_constant_reaches_structs_above_it_and_functions()
{
    let (trees, codes) = checked(&[(
        "/m/main.zaka",
        "ဒေတာအမျိုးအစား ဇယား တွင် အကွက် N ခုရှိသော ကိန်းပြည့် စာရင်း အမှတ် တို့ ပါဝင်သည်။
ကိန်းသေ N သည် ၂ ဖြစ်ပါစေ။
ကိန်းသေ S သည် ဇယား ၏ အရွယ်အစား ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် f သည်
အကွက် S ခုရှိသော ကိန်းပြည့် စာရင်း l ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းသေ M သည် N + N ဖြစ်ပါစေ။
ကိန်းရှင် m သည် အကွက် M ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။",
    )]);
    assert_eq!(codes, []);
    assert!(trees[0].contains("isize[2]"), "{}", trees[0]);
    assert!(
        trees[3].contains("isize[16] l") && trees[3].contains("isize[4]"),
        "{}",
        trees[3]
    );
}

#[test]
fn an_imported_constant_works_out()
{
    let lib = "အများသုံး ကိန်းသေ N သည် ၅ ဖြစ်ပါစေ။\nကိန်းသေ ဝှက် သည် ၉ ဖြစ်ပါစေ။";
    let main = |count: &str| {
        format!(
            "\"lib.zaka\" မှ N ကို ထည့်သွင်းပါ။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အကွက် {count} ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။"
        )
    };
    let (trees, codes) = checked(&[("/m/main.zaka", &main("N")), ("/m/lib.zaka", lib)]);
    assert_eq!(codes, []);
    assert!(trees.last().unwrap().contains("isize[5]"), "{trees:?}");
    let (_, codes) = checked(&[("/m/main.zaka", &main("ဝှက်")), ("/m/lib.zaka", lib)]);
    assert_eq!(codes, [Code::UndeclaredName]);
}

#[test]
fn counts_that_do_not_fold_have_their_codes()
{
    let (_, codes) = count("", "N - ၃");
    assert_eq!(codes, [Code::BadListCount]);
    let (_, codes) = one(
        "ကိန်းသေ N သည် ၃ ဖြစ်ပါစေ။",
        "ကိန်းရှင် N သည် ၅ ဖြစ်ပါစေ။\nကိန်းရှင် l သည် အကွက် N ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။",
    );
    assert_eq!(codes, [Code::BadListCount]);
    let circle = "ဒေတာအမျိုးအစား ဇယား တွင် အကွက် N ခုရှိသော ကိန်းပြည့် စာရင်း အမှတ် တို့ ပါဝင်သည်။
ကိန်းသေ N သည် ဇယား ၏ အရွယ်အစား ဖြစ်ပါစေ။";
    assert_eq!(one(circle, "N ကို ပြသပါ။").1, [Code::BadListCount]);
}

#[test]
fn counts_read_constant_lists_and_structs()
{
    let head = "ဒေတာအမျိုးအစား နေရာ တွင် ကိန်းပြည့် x၊ ကိန်းပြည့် y တို့ ပါဝင်သည်။
ကိန်းသေ L သည် [၂၊ ၃၊ ၄] ဖြစ်ပါစေ။
ကိန်းသေ P သည် နေရာ(x: ၂) ဖြစ်ပါစေ။
ကိန်းသေ T သည် [[၁၊ ၂]၊ [၃၊ ၄]] ဖြစ်ပါစေ။";
    let cases = [
        ("L ၏ အကွက် ၁", "isize[3]"),
        ("L ၏ ၃ ခုမြောက်အကွက်", "isize[4]"),
        ("(P ၏ x) + (P ၏ y) + ၁", "isize[3]"),
        ("T ၏ အကွက် ၁ ၏ အကွက် ၀", "isize[3]"),
        ("L ၏ အကွက်အရေအတွက်", "isize[3]"),
    ];
    for (written, want) in cases
    {
        let (tree, codes) = count(head, written);
        assert_eq!(codes, [], "{written}");
        assert!(tree.contains(want), "{written}: {tree}");
    }
    assert_eq!(count(head, "L ၏ အကွက် ၉").1, [Code::CellOutOfRange]);
}

#[test]
fn a_counted_list_gives_its_count_and_unknown_lists_do_not()
{
    let (trees, codes) = one(
        "ကိန်းသေ T သည် \"abc\" ဖြစ်ပါစေ။",
        "ကိန်းရှင် v သည် အကွက် ၄ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် l သည် အကွက် v ၏ အကွက်အရေအတွက် ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။",
    );
    assert_eq!(codes, []);
    assert!(trees.last().unwrap().contains("(var l isize[4])"), "{trees:?}");
    let plain = "ကိန်းရှင် w သည် [၁၊ ၂] ဖြစ်ပါစေ။
ကိန်းသေ W သည် [၁၊ (w ၏ အကွက် ၀)] ဖြစ်ပါစေ။";
    let decl = |count: &str| {
        format!("{plain}\nကိန်းရှင် l သည် အကွက် {count} ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။")
    };
    // The folder knows neither the count of a variable, nor a cell of a list
    // with a cell it does not know, nor the count of text.
    for written in ["w ၏ အကွက်အရေအတွက်", "W ၏ အကွက် ၀", "T ၏ အကွက်အရေအတွက်"]
    {
        let (_, codes) = one("ကိန်းသေ T သည် \"abc\" ဖြစ်ပါစေ။", &decl(written));
        assert_eq!(codes, [Code::BadListCount], "{written}");
    }
}
