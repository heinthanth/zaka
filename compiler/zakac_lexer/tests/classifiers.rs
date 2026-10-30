mod common;

use common::kinds;
use zakac_lexer::token::Kw;
use zakac_lexer::token::TokenKind;

const DECL: &str = "ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။";

#[test]
fn struct_declaration_lexes_to_its_words()
{
    let ident = TokenKind::Ident;
    let kw = TokenKind::Kw;
    #[rustfmt::skip]
    let want = [
        kw(Kw::StructOpen), ident, kw(Kw::At), kw(Kw::IntKind), ident,
        kw(Kw::FieldsCloseAnd), kw(Kw::ClassifierIs), ident, kw(Kw::Is),
        TokenKind::Punct(zakac_lexer::token::Punct::FullStop),
    ];
    assert_eq!(kinds(DECL), want);
    assert_eq!(kinds("အချက်အလက်အမျိုးအစား")[0], kw(Kw::StructOpen));
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
