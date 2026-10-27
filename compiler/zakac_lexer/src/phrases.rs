use zakac_rust_sugars::ret_if;

use crate::token::Kw;
use crate::token::Token;
use crate::token::TokenKind;

/// Multi-word reserved phrases, longest first. Whitespace between
/// the pieces does not matter.
#[rustfmt::skip]
pub static PHRASES: &[(&[Kw], Kw)] = &[
    // လုပ်ဆောင်ချက် ... သည် ... အောက်ပါတို့ကို လုပ်ဆောင်မည်။
    (&[Kw::Following, Kw::ListKo, Kw::WillDo], Kw::FnOpen),
    // လုပ်ဆောင်ချက် ... သည် ... အောက်ပါတို့ကို လုပ်ဆောင်ကာ ... ရလဒ် ကို ပြန်ပေးမည်။
    (&[Kw::Following, Kw::ListKo, Kw::DoAnd], Kw::FnOpenRet),
    // အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
    (&[Kw::Following, Kw::ListKo, Kw::Call], Kw::BlockOpen),
    // ... ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
    (&[Kw::Result, Kw::As, Kw::Return], Kw::ReturnValue),
    // မင်မိုရီ ... ဘိုက်စာ|ခုစာ နေရာယူ၍ လိပ်စာကို ... တွင် သိမ်းထားမည်။
    (&[Kw::TakeSpace, Kw::Address, Kw::Ko], Kw::AllocTarget),
    // ... လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
    (&[Kw::AtAddress, Kw::Mem, Kw::Ko], Kw::FreeHead),
    // ... လိပ်စာရှိကို မင်မိုရီ ... ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
    (&[Kw::AtAddress, Kw::Ko], Kw::ResizeHead),
    // လုပ်ဆောင်စရာများ ပြီး(ဆုံး)?ပါပြီ(း)?။
    (&[Kw::Tasks, Kw::Done], Kw::BlockClose),
    // ... ကို ... ဖြင့် စားသော စားလဒ်
    (&[Kw::DivideHead, Kw::QuotientTail], Kw::Divide),
    // ... ကို ... ဖြင့် စားသော စားကြွင်း
    (&[Kw::DivideHead, Kw::RemainderTail], Kw::Remainder),
    // ... နှင့် ... ၏ ဘစ်အလိုက် ဆုံလဒ်
    (&[Kw::BitwiseHead, Kw::MeetTail], Kw::BitAnd),
    // ... နှင့် ... ၏ ဘစ်အလိုက် ပေါင်းစုလဒ်
    (&[Kw::BitwiseHead, Kw::UnionTail], Kw::BitOr),
    // ... နှင့် ... ၏ ဘစ်အလိုက် ခြားနားလဒ်
    (&[Kw::BitwiseHead, Kw::DifferTail], Kw::BitXor),
    // ... ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်
    (&[Kw::BitwiseHead, Kw::InvertTail], Kw::BitNot),
    // ... ကို ဘယ်သို့|ညာသို့ ... ဘစ် ရွေ့သောရလဒ်
    (&[Kw::ShiftHead, Kw::Result], Kw::Shift),
    // ... ကို ... အဖြစ် ပြောင်းလဲသော ရလဒ်
    (&[Kw::ConvertHead, Kw::Result], Kw::Convert),
    // လုပ်ဆောင်ချက် ... ကို ... တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်
    (&[Kw::CallHead, Kw::Result], Kw::CallResult), ];

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
pub fn join_phrases(tokens: Vec<Token>) -> Vec<Token>
{
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
