use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::token::Kw;
use crate::token::Token;
use crate::token::TokenKind;

/// Multi-word reserved phrases, longest first. Whitespace between
/// the pieces does not matter.
#[rustfmt::skip]
pub static PHRASES: &[(&[Kw], Kw)] = &[
    // အောက်ပါတို့ကို လုပ်ဆောင်မည်။ — opens a block, and the steps of a function
    (&[Kw::Following, Kw::ListKo, Kw::WillDo], Kw::BlockOpen),
    // အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။ — opens an unsafe block
    (&[Kw::Following, Kw::ListKo, Kw::UnsafeThough, Kw::WillDo], Kw::UnsafeBlockOpen),
    // လုပ်ဆောင်ချက် ... သည် ... အောက်ပါတို့ကို လုပ်ဆောင်ကာ ... တစ်ခု ကို ပြန်ပေးမည်။
    (&[Kw::Following, Kw::ListKo, Kw::DoAnd], Kw::FnOpenRet),
    // ... လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။
    (&[Kw::OnFail, Kw::Fault, Kw::Ko, Kw::WillReturn], Kw::PassFault),
    // လုပ်ဆောင်ချက် (အောင်မြင်သည်|မအောင်မြင်ပါ)ဟု သတ်မှတ်မည်။
    (&[Kw::Function, Kw::Succeeded, Kw::Hu, Kw::WillMark], Kw::MarkSucceeded),
    (&[Kw::Function, Kw::NotSucceeded, Kw::Hu, Kw::WillMark], Kw::MarkFailed),
    // မင်မိုရီတွင် ... ဘိုက်စာ|ခုစာ နေရာယူ၍ လိပ်စာကို ... တွင် သိမ်းထားမည်။
    (&[Kw::TakeSpace, Kw::Address, Kw::Ko], Kw::AllocTarget),
    (&[Kw::Mem, Kw::At], Kw::AllocHead),
    // လိပ်စာ ... ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
    // လိပ်စာ ... ရှိ မင်မိုရီကို ... ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
    (&[Kw::Located, Kw::Mem, Kw::Ko], Kw::HeapHead),
    // အကုန် ကူးထည့်မည် — the deep duplicate, written apart
    (&[Kw::All, Kw::WillDupIn], Kw::WillDupAllIn),
    // <name> ဟူသော <kind> တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် / ဒေတာ ထည့်ပေးရန် — the marks of a parameter
    (&[Kw::Data, Kw::ToChange], Kw::DataToChange),
    (&[Kw::Data, Kw::ToFill], Kw::DataToFill),
    // မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ — the null pointer
    (&[Kw::Nothing, Kw::PointsAtNo, Kw::Address], Kw::Null),
    // ... လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး — the rest of a C function
    (&[Kw::OtherValues, Kw::Along], Kw::WithOtherValues),
    // ... ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။ / ... ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။
    (&[Kw::CallLeave, Kw::WillGrant], Kw::AllowCall),
    (&[Kw::CallLeave, Kw::WillRevoke], Kw::RevokeCall),
    // လုပ်ဆောင်စရာများ ပြီး(ဆုံး)?ပါပြီ(း)?။
    (&[Kw::Tasks, Kw::Done], Kw::BlockClose),
    // ပရိုဂရမ် ပြီး(ဆုံး)?ပါပြီ(း)?။
    (&[Kw::Program, Kw::Done], Kw::ProgramClose),
    // ... ကို ... ဖြင့် စားသော စားလဒ်
    (&[Kw::DivideHead, Kw::QuotientTail], Kw::Divide),
    // ... ကို ... ဖြင့် စားသော စားကြွင်း
    (&[Kw::DivideHead, Kw::RemainderTail], Kw::Remainder),
    // ... နှင့် ... ၏ ဘစ်အလိုက် ဆုံလဒ်
    (&[Kw::BitwiseHead, Kw::MeetTail], Kw::BitAnd),
    // ... နှင့် ... ၏ ဘစ်အလိုက် ပေါင်းစုလဒ်
    (&[Kw::BitwiseHead, Kw::UnionTail], Kw::BitOr),
    // ... နှင့် ... ၏ ဘစ်အလိုက် ခြားနားလဒ်
    (&[Kw::BitwiseHead, Kw::DifferTail], Kw::BitXor),
    // ... ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်
    (&[Kw::BitwiseHead, Kw::InvertTail], Kw::BitNot),
    // ... ကို ဘယ်သို့|ညာသို့ ... ဘစ် ရွေ့သောရလဒ်
    (&[Kw::ShiftHead, Kw::Result], Kw::Shift),
    // ... ကို ... အဖြစ် ပြောင်းလဲသော ရလဒ်
    (&[Kw::ConvertHead, Kw::Result], Kw::Convert),
    // ... ဟူသော လုပ်ဆောင်ချက်ကို ... တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်
    (&[Kw::CallHead, Kw::Result], Kw::CallResult),
    // ... ဟူသော လုပ်ဆောင်ချက်ကို ... တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း
    (&[Kw::CallHead, Kw::GoodResult], Kw::CallGoodResult),
    // ... ကို သုံး၍ <fn> သည့် ရလဒ်
    (&[Kw::Which, Kw::Result], Kw::UsedResult),
    // လုပ်ဆောင်ချက် <fn> သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်(တစ်ခု)?ဖြစ်(၍|သည်) ...
    (&[Kw::Outside, Kw::From, Kw::Coming], Kw::External),
    // ... ကို သုံး၍ <fn> သည့် ရလဒ်ကောင်း
    (&[Kw::Which, Kw::GoodResult], Kw::UsedGoodResult),
    // <parameters> လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား
    (&[Kw::Function, Kw::KindWord], Kw::FnKind),
    // <name> ဟူသော လုပ်ဆောင်ချက် / <name> မည့် လုပ်ဆောင်ချက်
    (&[Kw::Called, Kw::Function], Kw::FnValue),
    (&[Kw::Named, Kw::Function], Kw::FnValue),
    // <value> ဟူသော အမှားအယွင်း — an error value
    (&[Kw::Called, Kw::Fault], Kw::CalledFault),
    // <x> ထဲတွင် ဒေတာ (မ)ရှိ(နေ) — whether an optional holds data
    (&[Kw::Data, Kw::Located], Kw::HasData),
    (&[Kw::Data, Kw::NotLocated], Kw::LacksData),
    // <x> ထဲတွင် ရလဒ်ကောင်း / အောင်မြင်သည့်သတင်း ရှိ(နေ) — whether a result is good
    (&[Kw::GoodResult, Kw::Located], Kw::HasGood),
    (&[Kw::SuccessNews, Kw::Located], Kw::HasGood),
    // <x> ထဲတွင် အမှားအယွင်း / မအောင်မြင်သည့်သတင်း ရှိ(နေ) — whether a result failed
    (&[Kw::Fault, Kw::Located], Kw::HasFault),
    (&[Kw::FailureNews, Kw::Located], Kw::HasFault),
    // <x> ထဲမှ အမှားအယွင်း ဒေတာ — the error value of a result
    (&[Kw::Fault, Kw::Data], Kw::FaultData),
    // T သို့မဟုတ် အမှားအယွင်းအနေဖြင့် E ရှိနိုင်သော ဒေတာဆဲလ်
    (&[Kw::Fault, Kw::CountAs], Kw::AsFault),
    // ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။
    (&[Kw::Hu, Kw::Also, Kw::CanRegard], Kw::AliasClose),
    // ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပြီး မျိုးပြပစ္စည်းမှာ ... ဖြစ်သည်။
    (&[Kw::Hu, Kw::Also, Kw::CanRegardAnd], Kw::AliasCloseAnd),
    // <name> ဟူသော ဒေတာအစုအဝေးတွင် ... တို့ပါဝင်သည်။
    (&[Kw::Plural, Kw::Contains], Kw::FieldsClose),
    // <name> ဟူသော ဒေတာအစုအဝေးတွင် ... တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ... ဖြစ်သည်။
    (&[Kw::Plural, Kw::ContainsAnd], Kw::FieldsCloseAnd),
    // ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် <variant> — the initial variant
    (&[Kw::Initial, Kw::CountAs, Kw::ToUse], Kw::InitialMark),
    // တန်ဖိုးအနေဖြင့် <value> ရှိသည့် <variant> — the value of a variant
    (&[Kw::Value, Kw::CountAs], Kw::ValueAs), ];

/// Replace each run of tokens that spells a phrase with one token.
///
/// The joined token spans the whole run, is joined when the first piece is,
/// and keeps the particle of the last piece.
///
/// # Arguments
///
/// * `tokens` - The classified tokens.
///
/// # Returns
///
/// The tokens with phrases joined.
pub fn join_phrases(mut tokens: Vec<Token>) -> Vec<Token>
{
    free_data_words(&mut tokens);
    let mut out = Vec::with_capacity(tokens.len());
    let mut idx = 0;
    while idx < tokens.len()
    {
        let (token, len) = next_token(&tokens[idx..]);
        out.push(token);
        idx += len;
    }
    out
}

/// The token at the start of `tokens`, joined with the pieces that follow it
/// when the run spells a phrase.
///
/// # Arguments
///
/// * `tokens` - The tokens from the current position; not blank.
///
/// # Returns
///
/// The token to keep and the number of input tokens it replaces.
fn next_token(tokens: &[Token]) -> (Token, usize)
{
    phrase_at(tokens)
        .map(|(len, kw)| (joined_token(&tokens[..len], kw), len))
        .unwrap_or((tokens[0], 1))
}

/// Find the phrase that starts at the first token.
///
/// # Arguments
///
/// * `tokens` - The tokens from the current position.
///
/// # Returns
///
/// The number of tokens in the phrase and its joined [`Kw`], or `None`.
fn phrase_at(tokens: &[Token]) -> Option<(usize, Kw)>
{
    // `<name> ဟူသော လုပ်ဆောင်ချက် အမျိုးအစား` keeps `ဟူသော` apart, so the function
    // kind after it joins on its own.
    let kind_after = spells(tokens, &[Kw::Called, Kw::Function, Kw::KindWord]);
    ret_if!(kind_after, None);
    PHRASES
        .iter()
        .find(|(pieces, _)| spells(tokens, pieces))
        .map(|(pieces, kw)| (pieces.len(), *kw))
}

/// Whether `tokens` starts with the reserved words `pieces`.
///
/// # Arguments
///
/// * `tokens` - The tokens to check.
/// * `pieces` - The phrase pieces.
///
/// # Returns
///
/// `true` if the first tokens match `pieces` one to one.
fn spells(tokens: &[Token], pieces: &[Kw]) -> bool
{
    ret_if!(tokens.len() < pieces.len(), false);
    pieces.iter().zip(tokens).all(|(kw, tok)| tok.kind == TokenKind::Kw(*kw))
}

/// Build the token that replaces a phrase.
///
/// # Arguments
///
/// * `run` - The tokens of the phrase.
/// * `kw` - The joined reserved word.
///
/// # Returns
///
/// The joined token.
fn joined_token(run: &[Token], kw: Kw) -> Token
{
    let first = run[0];
    let last = run[run.len() - 1];
    #[rustfmt::skip]
    return Token::builder()
        .kind(TokenKind::Kw(kw)).span(first.span.to(&last.span))
        .particle(last.particle).joined(first.joined).build();
}

/// Turn each `ဒေတာ` back into a name, save one after `ထဲမှ`, `ထဲတွင်` or
/// `အမှားအယွင်း`, where it reads or tests the data of an optional or a result,
/// and one before `ပြောင်းလဲရန်` or `ထည့်ပေးရန်`, where it marks a parameter.
/// The word remains reserved for the split of joined words alone, so a program
/// can still name a variable `ဒေတာ`.
///
/// # Arguments
///
/// * `tokens` - The tokens, changed in place.
fn free_data_words(tokens: &mut [Token])
{
    let mut prev = None;
    for at in 0..tokens.len()
    {
        let after =
            matches!(prev, Some(TokenKind::Kw(Kw::Among | Kw::Within | Kw::Fault)));
        let next = tokens.get(at + 1).map(|t| t.kind);
        let mark = matches!(next, Some(TokenKind::Kw(Kw::ToChange | Kw::ToFill)));
        let tok = &mut tokens[at];
        do_if!(
            tok.kind == TokenKind::Kw(Kw::Data) && !after && !mark,
            tok.kind = TokenKind::Ident
        );
        prev = Some(tok.kind);
    }
}
