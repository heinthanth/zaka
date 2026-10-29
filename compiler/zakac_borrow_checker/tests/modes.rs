mod common;

use common::main_forms;
use common::off;
use common::run;
use common::run_with;
use common::strict;
use common::warn;
use zakac_diagnostics::Code;
use zakac_diagnostics::Level;
use zakac_diagnostics::has_errors;

/// A borrow read after its block is freed. Strict mode
/// doubts the store into the fresh allocation and the read through the
/// borrow lent from it, neither compared with `ဗလာ`.
const FREED_READ: &str = include_str!("inputs/invalidated_loan.zaka");

/// A list of three, each cell written in a counted loop over its numbers.
const CELLS: &str = "\
ကိန်းရှင် တန်း သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။
၀ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍ တန်း ၏ အကွက် န ကို န ဟု ထားပါ။
";

/// A function that frees the block its hook holds, and a call of it.
const FREES: &str = "\
လုပ်ဆောင်ချက် ရှင်း သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ၏ တန်ဖိုး လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။

ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ကြည့် သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ရှင်း ကို ပ ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။
ကြည့် ၏ တန်ဖိုး ကို ပြသပါ။
";

/// The code and the level of each diagnostic.
fn found(
    text: &str, options: &zakac_borrow_checker::options::BorrowCheckOptions,
) -> Vec<(Code, Level)>
{
    let run = run_with(text, options);
    run.out.diagnostics.iter().map(|d| (d.code, d.level)).collect()
}

/// Whether the main statements of a text, as checked, mark a cell
/// `checked`.
fn marks_checked(
    text: &str, options: &zakac_borrow_checker::options::BorrowCheckOptions,
) -> bool
{
    let run = run_with(text, options);
    let module = run.out.lowered.folded.expect("the text lowers");
    main_forms(&module).iter().any(|f| f.contains("checked"))
}

#[test]
fn warn_mode_downgrades_to_warnings()
{
    let run = run_with(FREED_READ, &warn());
    let codes: Vec<Code> = run.out.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(codes, [Code::InvalidatedLoan]);
    assert!(!has_errors(&run.out.diagnostics));
    assert_eq!(found(FREED_READ, &warn()), [(Code::InvalidatedLoan, Level::Warning)]);
    assert_eq!(
        found(FREED_READ, &Default::default()),
        [(Code::InvalidatedLoan, Level::Error)]
    );
}

#[test]
fn off_mode_drops_codes_but_keeps_checked()
{
    assert_eq!(found(FREED_READ, &off()), []);
    assert!(marks_checked(CELLS, &off()));
    assert!(marks_checked(CELLS, &Default::default()));
    let both = format!("{FREED_READ}{CELLS}");
    assert_eq!(found(&both, &off()), []);
    assert!(!marks_checked(&both, &off()));
}

#[test]
fn strict_adds_zk0711_without_changing_errors()
{
    let plain = found(FREED_READ, &Default::default());
    let doubted = found(FREED_READ, &strict());
    let errors = |all: &[(Code, Level)]| -> Vec<(Code, Level)> {
        all.iter().copied().filter(|(_, l)| *l == Level::Error).collect()
    };
    assert_eq!(errors(&doubted), errors(&plain));
    let added: Vec<(Code, Level)> =
        doubted.iter().copied().filter(|d| !plain.contains(d)).collect();
    let doubt = (Code::NullDerefPossible, Level::Warning);
    assert_eq!(added, [doubt, doubt]);
}

#[test]
fn off_mode_keeps_digests()
{
    let checked = run(FREES);
    let quiet = run_with(FREES, &off());
    assert!(!checked.out.diagnostics.is_empty());
    assert!(quiet.out.diagnostics.is_empty());
    assert_eq!(quiet.out.digests, checked.out.digests);
    assert!(!quiet.out.digests[0].frees.is_empty());
}
