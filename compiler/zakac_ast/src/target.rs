//! The target a program is built for: the facts a compile-time conditional
//! tests and the prelude's marked constants hold.

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;
use strum::IntoStaticStr;
use strum::VariantNames;

/// The OS a program runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(IntoStaticStr, EnumString, VariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TargetOs
{
    /// Windows.
    Windows,
    /// macOS.
    Macos,
    /// Linux.
    Linux,
}

/// The CPU architecture a program runs on, as Rust's `std::env::consts::ARCH`
/// spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(IntoStaticStr, EnumString, VariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TargetArch
{
    /// 64-bit x86.
    X86_64,
    /// 64-bit ARM.
    Aarch64,
    /// 32-bit x86.
    X86,
    /// 32-bit ARM.
    Arm,
    /// 64-bit RISC-V.
    Riscv64,
    /// 32-bit WebAssembly.
    Wasm32,
}

/// The ABI a program is built for: the C runtime and the calling contract
/// that its machine code and the libs it links share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(IntoStaticStr, EnumString, VariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TargetAbi
{
    /// The GNU ABI: glibc on Linux, MinGW on Windows.
    Gnu,
    /// The ABI of Microsoft's C compiler, on Windows.
    Msvc,
    /// The ABI of Apple's platforms.
    Apple,
    /// The ABI of WebAssembly.
    Wasm,
}

impl TargetAbi
{
    /// The ABI an OS and a CPU take when none is given: a 32-bit WebAssembly
    /// CPU takes `Wasm`, macOS takes `Apple`, Windows takes the ABI of this
    /// compiler on a Windows host and `Gnu` elsewhere, and Linux takes `Gnu`.
    ///
    /// # Arguments
    ///
    /// * `os` - The OS.
    /// * `arch` - The CPU architecture.
    ///
    /// # Returns
    ///
    /// The ABI.
    pub fn usual(os: TargetOs, arch: TargetArch) -> TargetAbi
    {
        let msvc_host = cfg!(all(windows, target_env = "msvc"));
        match (os, arch)
        {
            (_, TargetArch::Wasm32) => TargetAbi::Wasm,
            (TargetOs::Macos, _) => TargetAbi::Apple,
            (TargetOs::Windows, _) if msvc_host => TargetAbi::Msvc,
            _ => TargetAbi::Gnu,
        }
    }
}

/// The code the compiler gives out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(IntoStaticStr, EnumString, VariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum BuildTarget
{
    /// Ops for the VM.
    Vm,
    /// C, built into a native executable.
    C,
    /// Machine code from Cranelift: run at once in `zakac` (JIT), or linked
    /// into a native executable (AOT).
    Native,
}

/// The facts of the target a program is built for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Target
{
    /// The OS.
    pub os: TargetOs,
    /// The CPU architecture.
    pub arch: TargetArch,
    /// The ABI; the usual one of the OS and the CPU when the builder leaves
    /// it out.
    #[builder(default = TargetAbi::usual(os, arch))]
    pub abi: TargetAbi,
    /// The bit count of the platform-sized kinds and addresses.
    pub bits: u16,
    /// The code the compiler gives out.
    pub output: BuildTarget,
}

impl Target
{
    /// The host the compiler runs on, 64-bit, building for the VM.
    ///
    /// # Returns
    ///
    /// The target.
    pub fn host() -> Target
    {
        #[rustfmt::skip]
        return Target::builder().os(host_os()).arch(host_arch()).bits(64).output(BuildTarget::Vm)
            .build();
    }
}

impl Default for Target
{
    fn default() -> Self { Target::host() }
}

impl Target
{
    /// Whether the target is the host's: its OS, CPU and ABI are those of this
    /// compiler.
    ///
    /// # Returns
    ///
    /// `true` for the host.
    pub fn is_host(&self) -> bool
    {
        let host = Target::host();
        (self.os, self.arch, self.abi) == (host.os, host.arch, host.abi)
    }

    /// The OS, the CPU and the ABI as a reader sees them, such as
    /// `windows/x86_64/gnu`.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn shown(&self) -> String
    {
        let words: [&'static str; 3] =
            [self.os.into(), self.arch.into(), self.abi.into()];
        words.join("/")
    }
}

/// The OS the compiler runs on.
///
/// # Returns
///
/// The OS; Linux for a host this list does not name.
pub fn host_os() -> TargetOs
{
    // shortcut: a BSD or another unix host reads as Linux; add it to
    // `TargetOs` when a program needs to tell it apart.
    std::env::consts::OS.parse().unwrap_or(TargetOs::Linux)
}

/// The CPU architecture the compiler runs on.
///
/// # Returns
///
/// The architecture; x86-64 for a host this list does not name.
pub fn host_arch() -> TargetArch
{
    std::env::consts::ARCH.parse().unwrap_or(TargetArch::X86_64)
}

/// The constant of the prelude that holds the OS.
pub const OS_FACT: &str = "အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ်";

/// The constant of the prelude that holds the CPU architecture.
pub const ARCH_FACT: &str = "အလုပ်လုပ်မည့်စီပီယူဗိသုကာပုံစံ";

/// The constant of the prelude that holds the ABI.
pub const ABI_FACT: &str = "အလုပ်လုပ်မည့်အပလီကေးရှင်းဘိုင်နရီကြားခံစနစ်";

/// The constant of the prelude that holds the platform bit count.
pub const BITS_FACT: &str = "အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ်";

/// The constant of the prelude that holds the code the compiler gives out.
pub const OUTPUT_FACT: &str = "ကွန်ပိုင်လာအထွက်ကုဒ်အမျိုးအစား";

/// The names of the five facts of a target.
pub const FACTS: [&str; 5] = [OS_FACT, ARCH_FACT, ABI_FACT, BITS_FACT, OUTPUT_FACT];

/// The value of one fact of a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactValue
{
    /// Text, with each text the fact can hold.
    Text(&'static str, &'static [&'static str]),
    /// The platform bit count.
    Bits(u16),
}

impl Target
{
    /// The value of a fact, from its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the fact.
    ///
    /// # Returns
    ///
    /// The value, or `None` for a name that is no fact.
    pub fn fact(&self, name: &str) -> Option<FactValue>
    {
        match name
        {
            OS_FACT => Some(FactValue::Text(self.os.into(), TargetOs::VARIANTS)),
            ARCH_FACT => Some(FactValue::Text(self.arch.into(), TargetArch::VARIANTS)),
            ABI_FACT => Some(FactValue::Text(self.abi.into(), TargetAbi::VARIANTS)),
            BITS_FACT => Some(FactValue::Bits(self.bits)),
            OUTPUT_FACT =>
            {
                Some(FactValue::Text(self.output.into(), BuildTarget::VARIANTS))
            }
            _ => None,
        }
    }
}
