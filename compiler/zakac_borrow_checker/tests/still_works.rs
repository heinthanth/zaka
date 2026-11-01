mod common;

use std::path::Path;
use std::path::PathBuf;

use common::file_codes;
use common::lines;
use common::ok;
use common::repo_root;
use common::zaka_files;
use zakac_diagnostics::Code;

/// The text of a repo file.
fn repo_file(path: &str) -> String
{
    std::fs::read_to_string(repo_root().join(path)).unwrap()
}

/// The sample groups whose programs are meant to fail.
const FAILING_GROUPS: [&str; 2] = ["mistakes", "diagnostics"];

/// Two borrows into one block, both writing: the block reversed in place
/// with a front and a back pointer.
const REVERSE: &str = "\
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ ပ + န ၏ တန်ဖိုး ကို န ဟု ထားပါ။
ကိန်းရှင် ရှေ့ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ကိန်းရှင် နောက် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ + ၃ ဖြစ်ပါစေ။
ရှေ့ သည် နောက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ရှေ့ ၏ တန်ဖိုး ဖြစ်ပါစေ။
ရှေ့ ၏ တန်ဖိုး ကို နောက် ၏ တန်ဖိုး ဟု ထားပါ။
နောက် ၏ တန်ဖိုး ကို ယာယီ ဟု ထားပါ။
ရှေ့ ကို ရှေ့ + ၁ ဟု ထားပါ။
နောက် ကို နောက် - ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ ပ + န ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
";

/// A swap that reads and writes through both borrows, given one slot
/// twice and one block lent twice.
const SWAP_ITSELF: &str = "\
လုပ်ဆောင်ချက် လဲလှယ် သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ က နှင့် ငှားထားသော ကိန်းပြည့် လိပ်စာ ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုး ဖြစ်ပါစေ။
က ၏ တန်ဖိုး ကို ခ ၏ တန်ဖိုး ဟု ထားပါ။
ခ ၏ တန်ဖိုး ကို ယာယီ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပထမ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် လဲလှယ် ကို ပထမ ၏ လိပ်စာ၊ ပထမ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ပထမ ကို ပြသပါ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၂ ဟု ထားပါ။
လုပ်ဆောင်ချက် လဲလှယ် ကို ပ ၏ တန်ဖိုးအငှား၊ ပ ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်ပါ။
ပ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
";

/// The arena: one owned block of 1024 octets, handed out in raw pieces
/// rounded up to 8 octets. It prints `၄၂`, `၇` and `၄`, as the second
/// piece starts four integers past the first.
const ARENA: &str = "\
ကိန်းရှင် အခင်း သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် သုံးပြီး သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
မင်မိုရီတွင် ၁၀၂၄ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို အခင်း တွင် သိမ်းထားမည်။

/// hands out the next n octets of the arena, or ဗလာ when it is full
လုပ်ဆောင်ချက် ယူ သည်
လက္ခဏာမဲ့ ကိန်းပြည့် အရွယ် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လိပ်စာအကြမ်း ရလဒ် ကို ပြန်ပေးမည်။
အကယ်၍ သုံးပြီး + အရွယ် သည် ၁၀၂၄ ထက် ကြီးလျှင် ဗလာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
ရလဒ် ကို အခင်း + သုံးပြီး ကို လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
သုံးပြီး ကို သုံးပြီး + (အရွယ် + ၇) / ၈ * ၈ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ယူ ကို ကိန်းပြည့် ၏ အရွယ်အစား * ၄ တို့ဖြင့် \
                     လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ယူ ကို ကိန်းပြည့် ၏ အရွယ်အစား * ၈ တို့ဖြင့် \
                     လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
က + ၃ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားပါ။
ခ + ၁ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
က + ၃ ၏ တန်ဖိုး ကို ပြသပါ။
ခ + ၁ ၏ တန်ဖိုး ကို ပြသပါ။
ခ - က ကို ပြသပါ။
";

/// The line that frees the whole arena at once.
const FREE_ARENA: &str = "လိပ်စာ အခင်း ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။\n";

/// A borrow of the arena block, still read after the block is freed.
const STALE_LOOK: &str = "\
ကိန်းရှင် ကြည့် သည် ငှားထားသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် အခင်း ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လိပ်စာ အခင်း ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ကြည့် ၏ တန်ဖိုး ကို ပြသပါ။
";

/// Whether a sample sits in a group whose programs are meant to fail.
fn meant_to_fail(samples: &Path, file: &Path) -> bool
{
    let group = file.strip_prefix(samples).unwrap().components().next();
    group.is_some_and(|g| FAILING_GROUPS.iter().any(|f| g.as_os_str() == *f))
}

#[test]
fn two_writing_borrows_are_fine() { ok(REVERSE); }

#[test]
fn swap_with_itself_is_fine() { ok(SWAP_ITSELF); }

#[test]
fn walk_is_fine() { ok(&repo_file("samples/heap/walk.zaka")); }

#[test]
fn record_cast_is_fine() { ok(&repo_file("samples/heap/record.zaka")); }

#[test]
fn divide_out_params_are_fine() { ok(&repo_file("samples/heap/divide.zaka")); }

#[test]
fn pascal_nesting_is_fine() { ok(&repo_file("samples/heap/pascal.zaka")); }

#[test]
fn arena_with_raw_pieces_is_fine()
{
    ok(&format!("{ARENA}{FREE_ARENA}"));
    let stale = format!("{ARENA}{STALE_LOOK}");
    let free_line = ARENA.lines().count() + 2;
    assert_eq!(lines(&stale), [(Code::InvalidatedLoan, free_line)]);
}

#[test]
fn repo_samples_are_clean()
{
    let samples = repo_root().join("samples");
    let files: Vec<PathBuf> = zaka_files(&samples)
        .into_iter()
        .filter(|f| !meant_to_fail(&samples, f))
        .collect();
    assert!(files.len() > 40, "{} samples", files.len());
    let found: Vec<(String, Vec<Code>)> = files
        .iter()
        .map(|f| (f.display().to_string(), file_codes(f)))
        .filter(|(_, c)| !c.is_empty())
        .collect();
    assert!(found.is_empty(), "{found:#?}");
}
