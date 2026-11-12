use bon::Builder;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoEnumIterator;
use strum::IntoStaticStr;
use zakac_borrow_checker::options::OverflowMode;

/// The levels of the run-time checks: each level trades speed for the stops
/// that point at a mistake. `--checks` takes a level, then checks to add
/// or drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Checks
{
    /// No check that C itself lacks: a zero divisor, a shift too far or a
    /// cell outside its list does what C does with it.
    None,
    /// The checks of the language: a zero divisor, a shift too far, a step
    /// that is not positive, a cell or a range outside its list, and the
    /// rest of the stops a sound program never meets.
    #[default]
    Default,
    /// The checks of the language, a check on each access through an
    /// address for the null address, and the sanitizers of the C compiler
    /// for the rest of the mem checks of the VM.
    Mem,
}

/// One run-time check: a kind of stop that `--checks` adds or drops on its
/// own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum RunCheck
{
    /// A zero divisor.
    Divide,
    /// A shift as wide as its kind or wider.
    Shift,
    /// A cell, a place, a slice or a range outside its list or its text.
    Bounds,
    /// A range of text that cuts a character, and a list of u8s that holds
    /// no UTF-8.
    Text,
    /// A step of a loop that is not positive.
    Step,
    /// More nested calls than `--max-depth`.
    Depth,
    /// An access through the null address.
    Null,
    /// A block copied onto a block it overlaps.
    Overlap,
    /// The sanitizers of the C compiler, for the rest of the mem checks of
    /// the VM.
    Sanitizers,
}

impl RunCheck
{
    /// The C macro that turns this check on or off in the runtime.
    ///
    /// # Returns
    ///
    /// The macro name, such as `ZK_CHECK_DIVIDE`.
    pub fn macro_name(self) -> String
    {
        let name: &'static str = self.into();
        format!("ZK_CHECK_{}", name.to_uppercase())
    }

    /// The bit of this check in [`RunChecks`].
    ///
    /// # Returns
    ///
    /// The bit.
    const fn bit(self) -> u16 { 1 << self as u16 }
}

/// The checks of the language, which `default` keeps.
const LANGUAGE_CHECKS: &[RunCheck] = &[
    RunCheck::Divide,
    RunCheck::Shift,
    RunCheck::Bounds,
    RunCheck::Text,
    RunCheck::Step,
    RunCheck::Depth,
];

/// The run-time checks an executable keeps, each check on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunChecks(u16);

impl RunChecks
{
    /// Whether a check is on.
    ///
    /// # Arguments
    ///
    /// * `check` - The check.
    ///
    /// # Returns
    ///
    /// `true` when it is.
    pub const fn has(self, check: RunCheck) -> bool { self.0 & check.bit() != 0 }

    /// The checks with one check turned on or off.
    ///
    /// # Arguments
    ///
    /// * `check` - The check.
    /// * `on` - Whether it is on.
    ///
    /// # Returns
    ///
    /// The checks.
    pub const fn with(self, check: RunCheck, on: bool) -> Self
    {
        Self(
            if on
            {
                self.0 | check.bit()
            }
            else
            {
                self.0 & !check.bit()
            },
        )
    }

    /// The checks after one item of `--checks`: a level (`none`, `default`,
    /// `mem`) sets each check; a check alone or with `+` turns it on, and
    /// with `-` turns it off.
    ///
    /// # Arguments
    ///
    /// * `item` - The item.
    ///
    /// # Returns
    ///
    /// The checks, or `None` when the item names nothing.
    pub fn with_item(self, item: &str) -> Option<Self>
    {
        let item = item.trim();
        if let Ok(level) = item.parse::<Checks>()
        {
            return Some(level.into());
        }
        let (on, name) =
            item.strip_prefix('-').map_or((true, item), |rest| (false, rest));
        let name = name.strip_prefix('+').filter(|_| on).unwrap_or(name);
        name.parse::<RunCheck>().ok().map(|check| self.with(check, on))
    }

    /// The checks the items of `--checks` give, from the default level, in
    /// order.
    ///
    /// # Arguments
    ///
    /// * `items` - The items.
    ///
    /// # Returns
    ///
    /// The checks, or `None` when an item names nothing.
    pub fn from_items<'a>(items: impl IntoIterator<Item = &'a str>) -> Option<Self>
    {
        items.into_iter().try_fold(Self::default(), Self::with_item)
    }
}

impl Default for RunChecks
{
    /// The checks of the default level.
    ///
    /// # Returns
    ///
    /// The checks.
    fn default() -> Self { Checks::default().into() }
}

impl From<Checks> for RunChecks
{
    /// The checks a level keeps.
    ///
    /// # Arguments
    ///
    /// * `level` - The level.
    ///
    /// # Returns
    ///
    /// The checks.
    fn from(level: Checks) -> Self
    {
        let none = Self(0);
        let all = RunCheck::iter().fold(none, |c, f| c.with(f, true));
        let language = LANGUAGE_CHECKS.iter().fold(none, |c, f| c.with(*f, true));
        match level
        {
            Checks::None => none,
            Checks::Default => language,
            Checks::Mem => all,
        }
    }
}

/// What a build makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Emit
{
    /// The C files, then the executable the C compiler makes of them.
    #[default]
    Bin,
    /// The C files alone.
    C,
}

/// The choices of the C target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Builder)]
pub struct TargetCOptions
{
    /// The run-time checks the executable keeps.
    #[builder(default, into)]
    pub checks: RunChecks,
    /// Whether an overflow wraps or stops the program.
    #[builder(default)]
    pub overflow: OverflowMode,
    /// Bit count of the platform-sized integer kinds, which must equal the
    /// width of an address on the host.
    #[builder(default = 64)]
    pub platform_bits: u16,
    /// The most nested calls before the program stops, under the default
    /// checks and the mem checks.
    #[builder(default = 4096)]
    pub max_depth: u32,
    /// Whether the executable keeps the source line of each statement and
    /// the calls in progress, to name them when it stops; a release build
    /// leaves them out.
    #[builder(default = true)]
    pub debug_info: bool,
    /// The most octets of heap blocks the program holds at once; 0 for no
    /// cap.
    #[builder(default)]
    pub max_heap: u64,
}

impl Default for TargetCOptions
{
    /// The default checks, a stop on overflow, a 64-bit platform.
    ///
    /// # Returns
    ///
    /// The options.
    fn default() -> Self { TargetCOptions::builder().build() }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn items_of_checks_take_effect_in_order()
    {
        let mem = RunChecks::from(Checks::Mem);
        assert_eq!(RunChecks::from_items(["mem"]), Some(mem));
        let no_san = RunChecks::from_items(["mem", "-sanitizers"]).unwrap();
        assert!(no_san.has(RunCheck::Null) && !no_san.has(RunCheck::Sanitizers));
        let bounds = RunChecks::from_items(["none", "+bounds"]).unwrap();
        assert!(bounds.has(RunCheck::Bounds) && !bounds.has(RunCheck::Divide));
        let null = RunChecks::from_items(["null"]).unwrap();
        assert!(null.has(RunCheck::Null) && null.has(RunCheck::Depth));
        assert_eq!(RunChecks::from_items(["mem", "+nothing"]), None);
        assert_eq!(RunChecks::from_items(["-+divide"]), None);
    }

    #[test]
    fn the_default_level_keeps_the_language_checks_alone()
    {
        let checks = RunChecks::default();
        assert!(LANGUAGE_CHECKS.iter().all(|f| checks.has(*f)));
        assert!(!checks.has(RunCheck::Null) && !checks.has(RunCheck::Sanitizers));
    }
}
