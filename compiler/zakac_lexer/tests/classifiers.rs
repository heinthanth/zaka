use zakac_lexer::token::Kw;
use zakac_lexer::token::TokenKind;

use crate::common::kinds;

const DECL: &str =
    "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။";

#[test]
fn struct_declaration_lexes_to_its_words()
{
    let ident = TokenKind::Ident;
    let kw = TokenKind::Kw;
    #[rustfmt::skip]
    let want = [
        ident, kw(Kw::Called), kw(Kw::StructWord), kw(Kw::At), ident, kw(Kw::Called),
        ident, ident, kw(Kw::FieldsCloseAnd), kw(Kw::ClassifierIs), ident, kw(Kw::Is),
        TokenKind::Punct(zakac_lexer::token::Punct::FullStop),
    ];
    assert_eq!(kinds(DECL), want);
    assert_eq!(kinds("အချက်အလက်အမျိုးအစား")[0], kw(Kw::DataKindOpen));
}

#[test]
fn a_declared_classifier_makes_a_count_word()
{
    let text = format!("{DECL} လူ ၃ ယောက်စာ ကောင်စာ");
    let got = kinds(&text);
    let tail = &got[got.len() - 2..];
    assert_eq!(tail, [TokenKind::Kw(Kw::Items), TokenKind::Ident]);
}

#[test]
fn without_a_declaration_the_count_word_is_a_name()
{
    assert_eq!(kinds("ယောက်စာ"), [TokenKind::Ident]);
}

#[test]
fn an_alias_closes_with_one_phrase_joined_or_spaced()
{
    let kw = TokenKind::Kw;
    let stop = TokenKind::Punct(zakac_lexer::token::Punct::FullStop);
    #[rustfmt::skip]
    let want = [
        kw(Kw::DataKindOpen), TokenKind::Ident, kw(Kw::Ko), TokenKind::Ident,
        kw(Kw::AliasClose), stop,
    ];
    let joined = "ဒေတာအမျိုးအစား ပုဂ္ဂိုလ် ကို လူ ဟုလည်း မှတ်ယူနိုင်ပါသည်။";
    let spaced = "ဒေတာအမျိုးအစား ပုဂ္ဂိုလ် ကို လူ ဟုလည်း မှတ်ယူနိုင်ပါသည်။";
    assert_eq!(kinds(joined), want);
    assert_eq!(kinds(spaced), want);
}
