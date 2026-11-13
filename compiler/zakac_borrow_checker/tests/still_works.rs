use std::path::Path;
use std::path::PathBuf;

use zakac_diagnostics::Code;

use crate::common::file_codes;
use crate::common::lines;
use crate::common::ok;
use crate::common::repo_root;
use crate::common::zaka_files;

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
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၄ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ ပ + န ၏ တန်ဖိုး ကို န ဟု ထားမည်။
ကိန်းရှင် ရှေ့ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။
ကိန်းရှင် နောက် သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ + ၃ ဖြစ်ပါစေ။
ရှေ့ သည် နောက် အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရှေ့ ၏ တန်ဖိုး ဖြစ်ပါစေ။
ရှေ့ ၏ တန်ဖိုး ကို နောက် ၏ တန်ဖိုး ဟု ထားမည်။
နောက် ၏ တန်ဖိုး ကို ယာယီ ဟု ထားမည်။
ရှေ့ ကို ရှေ့ + ၁ ဟု ထားမည်။
နောက် ကို နောက် - ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
၀ မှ ၃ အထိ န အနေဖြင့် ရေတွက်၍ ပ + န ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
";

/// A swap that reads and writes through both borrows, given one slot
/// twice and one block lent twice.
const SWAP_ITSELF: &str = "\
လုပ်ဆောင်ချက် လဲလှယ် သည်
က ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ယာယီ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုး ဖြစ်ပါစေ။
က ၏ တန်ဖိုး ကို ခ ၏ တန်ဖိုး ဟု ထားမည်။
ခ ၏ တန်ဖိုး ကို ယာယီ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပထမ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
လဲလှယ် ဟူသော လုပ်ဆောင်ချက်ကို ပထမ ၏ လိပ်စာ၊ ပထမ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်မည်။
ပထမ ကို ပြသမည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို ၂ ဟု ထားမည်။
လဲလှယ် ဟူသော လုပ်ဆောင်ချက်ကို ပ၊ ပ တို့ဖြင့် လုပ်ဆောင်မည်။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
";

/// The arena: one owned block of 1024 octets, handed out in raw pieces
/// rounded up to 8 octets. It prints `၄၂`, `၇` and `၄`, as the second
/// piece starts four integers past the first.
const ARENA: &str = "\
ကိန်းရှင် အခင်း သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် သုံးပြီး သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
မင်မိုရီတွင် ၁၀၂၄ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို အခင်း တွင် သိမ်းထားမည်။

/// hands out the next n octets of the arena, or null when it is full
လုပ်ဆောင်ချက် ယူ သည်
အရွယ် ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ သုံးပြီး + အရွယ် သည် ၁၀၂၄ ထက် ကြီးလျှင် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခင်း + သုံးပြီး ကို လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
သုံးပြီး ကို သုံးပြီး + (အရွယ် + ၇) / ၈ * ၈ ဟု ထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား * ၄ တို့ဖြင့် \
                     လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား * ၈ တို့ဖြင့် \
                     လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
က + ၃ ၏ တန်ဖိုး ကို ၄၂ ဟု ထားမည်။
ခ + ၁ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
က + ၃ ၏ တန်ဖိုး ကို ပြသမည်။
ခ + ၁ ၏ တန်ဖိုး ကို ပြသမည်။
ခ - က ကို ပြသမည်။
";

/// The line that frees the whole arena at once.
const FREE_ARENA: &str = "လိပ်စာ အခင်း ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။\n";

/// A borrow of the arena block, still read after the block is freed.
const STALE_LOOK: &str = "\
ကိန်းရှင် ကြည့် သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အခင်း ဖြစ်ပါစေ။
လိပ်စာ အခင်း ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ကြည့် ၏ တန်ဖိုး ကို ပြသမည်။
";

/// Whether a sample sits in a group whose programs are meant to fail.
fn meant_to_fail(samples: &Path, file: &Path) -> bool
{
    let group = file.strip_prefix(samples).unwrap().components().next();
    group.is_some_and(|g| FAILING_GROUPS.iter().any(|f| g.as_os_str() == *f))
}

/// Whether a sample is a module in a `lib` folder, which has no start block:
/// the programs that import it check it.
fn is_lib_module(file: &Path) -> bool
{
    file.parent().and_then(Path::file_name).is_some_and(|d| d == "lib")
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
        .filter(|f| !meant_to_fail(&samples, f) && !is_lib_module(f))
        .collect();
    assert!(files.len() > 40, "{} samples", files.len());
    let found: Vec<(String, Vec<Code>)> = files
        .iter()
        .map(|f| (f.display().to_string(), file_codes(f)))
        .filter(|(_, c)| !c.is_empty())
        .collect();
    assert!(found.is_empty(), "{found:#?}");
}
