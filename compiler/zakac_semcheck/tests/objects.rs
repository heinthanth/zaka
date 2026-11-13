use zakac_diagnostics::Code;

use crate::common::codes;

/// A struct counted with `ယောက်`, and a value of it.
const PERSON: &str = "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ \
                      ယောက် ဖြစ်သည်။
ကိန်းရှင် က သည် လူ(အသက်: ၂၀) ဖြစ်ပါစေ။
";

/// The measure word of a deep duplicate is the classifier of the kind: the
/// one a struct declares, `ခု` for each other kind; another word is ZK0509.
#[test]
fn a_deep_duplicate_takes_the_classifier_of_its_kind()
{
    let dup =
        |word: &str| codes(&format!("{PERSON}ကိန်းရှင် ခ သည် က {word} ၏ မိတ္တူ ဖြစ်ပါစေ။"));
    assert_eq!(dup("တစ်ယောက်လုံး"), []);
    assert_eq!(dup("တစ်ခုလုံး"), [Code::ClassifierMismatch]);
    let list = "ကိန်းရှင် လ သည် [၁၊ ၂] ဖြစ်ပါစေ။\n";
    assert_eq!(codes(&format!("{list}ကိန်းရှင် ဃ သည် လ တစ်ခုလုံး ၏ မိတ္တူ ဖြစ်ပါစေ။")), []);
    let wrong = codes(&format!("{list}ကိန်းရှင် ဃ သည် လ တစ်ယောက်လုံး ၏ မိတ္တူ ဖြစ်ပါစေ။"));
    assert_eq!(wrong, [Code::ClassifierMismatch]);
}
