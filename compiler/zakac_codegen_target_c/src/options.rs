use bon::Builder;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;
use zakac_borrow_checker::options::OverflowMode;

/// The run-time checks an executable keeps: each level trades speed for
/// the stops that point at a mistake.
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

impl Checks
{
    /// The name of the C macro that selects this level in the runtime.
    ///
    /// # Returns
    ///
    /// The macro name.
    pub fn macro_name(self) -> &'static str
    {
        match self
        {
            Checks::None => "ZK_CHECKS_NONE",
            Checks::Default => "ZK_CHECKS_DEFAULT",
            Checks::Mem => "ZK_CHECKS_MEM",
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
    #[builder(default)]
    pub checks: Checks,
    /// Whether an overflow wraps or stops the program.
    #[builder(default)]
    pub overflow: OverflowMode,
    /// Bit count of the platform-sized integer kinds, which must equal the
    /// width of an address on the host.
    #[builder(default = 64)]
    pub platform_bits: u16,
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
