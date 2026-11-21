//! A wait on several channels at once: an opener line, then its arms.
//!
//! ```text
//! အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
//! <pipe> ထဲတွင် ဒေတာ ရှိလျှင် <block>                 ← one arm for each channel
//! <n> မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် <block>     ← a time limit, or
//! မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင် <block>        ← no wait
//! ```
//!
//! The closed arm, `မည်သည့်လမ်းကြောင်းထံမှ ဒေတာမရောက်လာနိုင်တော့လျှင် <block>`, can stand
//! right after the arms of the channels, before either of the last two.
//!
//! The arms end at the first statement that is no arm, as the arms of a
//! switch do.

use zakac_ast::stmt::IfArm;
use zakac_ast::stmt::SelectStmt;
use zakac_ast::stmt::SelectTimeout;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_diagnostics::parser as diag;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;

/// The words of the opener between its reserved words, in order: the channels,
/// the data that comes first, the data, and the verb before `မည်`.
const OPENER_WORDS: [&str; 4] = ["ဒေတာလမ်းကြောင်းများ", "ဦးစွာရောက်လာသော", "ဒေတာ", "စောင့်"];

/// The unit of a time limit in milliseconds.
const MILLIS: &str = "မီလီစက္ကန့်";

/// The unit of a time limit in seconds.
const SECONDS: &str = "စက္ကန့်";

/// The words of a time limit after its unit: within, data, and has not come.
const TIMEOUT_WORDS: [&str; 3] = ["အတွင်း", "ဒေတာ", "မရောက်လာလျှင်"];

/// The words of the closed arm, joined: from no channel can data come again.
/// The arm can be written glued or with spaces between its words.
const CLOSED_ARM: &str = "မည်သည့်လမ်းကြောင်းထံမှဒေတာမရောက်လာနိုင်တော့လျှင်";

/// The most tokens the closed arm can take.
const CLOSED_TOKENS: usize = 8;

/// The first word of the no-data arm: in no channel.
const NO_CHANNEL: &str = "မည်သည့်လမ်းကြောင်းတွင်မှ";

/// The most tokens the count of a time limit can take before its unit, a
/// count in parentheses among them.
const COUNT_TOKENS: usize = 8;

impl Parser<'_>
{
    /// Whether the opener of a wait on several channels stands here:
    /// `အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_select(&self) -> bool
    {
        let kw = |n: usize, k: Kw| kw_of(&self.peek_n(n)) == Some(k);
        let [channels, first, data, wait] = OPENER_WORDS;
        kw(0, Kw::Following)
            && self.is_word_at(1, channels)
            && kw(2, Kw::Among)
            && self.is_word_at(3, first)
            && self.is_word_at(4, data)
            && kw(5, Kw::Ko)
            && self.is_word_at(6, wait)
            && kw(7, Kw::Will)
    }

    /// Parse a wait on several channels: the opener, the arms of the channels,
    /// then a time-limit arm or a no-data arm.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    pub(crate) fn parse_select(&mut self) -> PResult<Stmt>
    {
        let start = self.peek().span;
        (0..OPENER_WORDS.len() + 4).for_each(|_| _ = self.bump());
        self.expect_stop()?;
        let mut arms = Vec::new();
        while self.at_channel_arm()
        {
            arms.push(self.parse_channel_arm()?);
        }
        ret_if!(arms.is_empty(), Err(diag::select_no_arm(start)));
        let closed = self.closed_arm().map(|n| self.parse_closed(n)).transpose()?;
        let timeout =
            tern!(self.timeout_unit().is_some(), Some(self.parse_timeout()?), None);
        let idle_at = self.peek().span;
        let idle = tern!(self.at_idle_arm(), Some(self.parse_idle()?), None);
        do_if!(
            timeout.is_some() && idle.is_some(),
            self.diags.push(diag::select_two_ends(idle_at))
        );
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        let node = SelectStmt::builder().arms(arms).maybe_closed(closed).maybe_timeout(timeout)
            .maybe_idle(idle).span(span).build();
        Ok(Stmt::Select(node))
    }

    /// Whether an arm of a channel starts here: `<pipe> ထဲတွင် ဒေတာ ရှိလျှင်`.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    fn at_channel_arm(&self) -> bool
    {
        let kw = |n: usize, k: Kw| kw_of(&self.peek_n(n)) == Some(k);
        kw_of(&self.peek()).is_none()
            && kw(1, Kw::Within)
            && kw(2, Kw::HasData)
            && kw(3, Kw::Then)
    }

    /// Parse an arm of a channel: the test, `လျှင်`, then the block.
    ///
    /// # Returns
    ///
    /// The arm, or a diagnostic.
    fn parse_channel_arm(&mut self) -> PResult<IfArm>
    {
        let cond = self.parse_expr()?;
        self.expect(Kw::Then)?;
        let block = self.parse_branch(false)?;
        let span = cond.span().to(&block.span);
        Ok(IfArm::builder().cond(cond).block(block).span(span).build())
    }

    /// The unit of the time limit that starts here: the count, then
    /// `မီလီစက္ကန့်` or `စက္ကန့်`, `အတွင်း`, `ဒေတာ` and `မရောက်လာလျှင်`.
    ///
    /// # Returns
    ///
    /// The place of the unit and whether it counts seconds, or `None` when no
    /// time limit starts here.
    fn timeout_unit(&self) -> Option<(usize, bool)>
    {
        let [within, data, not_come] = TIMEOUT_WORDS;
        let tail = |n: usize| {
            self.is_word_at(n + 1, within)
                && self.is_word_at(n + 2, data)
                && self.is_word_at(n + 3, not_come)
        };
        (1..=COUNT_TOKENS).find_map(|n| {
            let unit = self.is_word_at(n, MILLIS) || self.is_word_at(n, SECONDS);
            (unit && tail(n)).then(|| (n, self.is_word_at(n, SECONDS)))
        })
    }

    /// Parse the time-limit arm: the count, its unit and the words after it,
    /// then the block.
    ///
    /// # Returns
    ///
    /// The arm, or a diagnostic.
    fn parse_timeout(&mut self) -> PResult<SelectTimeout>
    {
        let start = self.peek().span;
        let seconds = self.timeout_unit().is_some_and(|(_, s)| s);
        let count = self.parse_operand()?;
        (0..TIMEOUT_WORDS.len() + 1).for_each(|_| _ = self.bump());
        let block = self.parse_branch(false)?;
        let span = start.to(&block.span);
        #[rustfmt::skip]
        let arm = SelectTimeout::builder().count(count).seconds(seconds).block(block).span(span)
            .build();
        Ok(arm)
    }

    /// The count of tokens of the closed arm that starts here: its words,
    /// glued or apart, up to `လျှင်`.
    ///
    /// # Returns
    ///
    /// The count, or `None` when no closed arm starts here.
    fn closed_arm(&self) -> Option<usize>
    {
        let joined = (0..CLOSED_TOKENS).scan(String::new(), |text, n| {
            text.push_str(self.text_of(self.peek_n(n).span));
            Some(text.len())
        });
        let whole =
            |(n, len): (usize, usize)| (len == CLOSED_ARM.len()).then_some(n + 1);
        let n = joined.enumerate().find_map(whole)?;
        let text: String = (0..n).map(|k| self.text_of(self.peek_n(k).span)).collect();
        (text == CLOSED_ARM).then_some(n)
    }

    /// Parse the closed arm: its words, then the block.
    ///
    /// # Arguments
    ///
    /// * `n` - The count of tokens of its words.
    ///
    /// # Returns
    ///
    /// The block, or a diagnostic.
    fn parse_closed(&mut self, n: usize) -> PResult<zakac_ast::stmt::BlockStmt>
    {
        (0..n).for_each(|_| _ = self.bump());
        self.parse_branch(false)
    }

    /// Whether the no-data arm starts here:
    /// `မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင်`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_idle_arm(&self) -> bool
    {
        let kw = |n: usize, k: Kw| kw_of(&self.peek_n(n)) == Some(k);
        self.is_word_at(0, NO_CHANNEL)
            && self.is_word_at(1, "ဒေတာ")
            && kw(2, Kw::NotLocated)
            && kw(3, Kw::Then)
    }

    /// Parse the no-data arm: its words, then the block.
    ///
    /// # Returns
    ///
    /// The block, or a diagnostic.
    fn parse_idle(&mut self) -> PResult<zakac_ast::stmt::BlockStmt>
    {
        (0..4).for_each(|_| _ = self.bump());
        self.parse_branch(false)
    }
}
