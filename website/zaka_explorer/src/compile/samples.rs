/// A sample program to start from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample
{
    /// The group in the picker.
    pub group: &'static str,
    /// Name in the picker.
    pub name: &'static str,
    /// The source text.
    pub text: &'static str,
}

impl Sample
{
    /// The `group/name` path, the value of the option in the picker.
    ///
    /// # Returns
    ///
    /// The path.
    pub fn path(&self) -> String { format!("{}/{}", self.group, self.name) }
}

/// The path of a sample file, relative to this source file.
#[rustfmt::skip]
macro_rules! sample_path { ($group:literal, $name:literal) => {
concat!("../../../../samples/", $group, "/", $name, ".zaka") }; }

/// Embed a file of the `samples` folder at the repo root.
#[rustfmt::skip]
macro_rules! sample { ($group:literal, $name:literal) => {
Sample { group: $group, name: $name, text: include_str!(sample_path!($group, $name)) } };}

/// The group of the samples that are meant to report errors.
pub const DIAGNOSTICS: &str = "diagnostics";

/// The group of the samples that go wrong: the ownership check rejects the
/// pointer mistakes, and with the check on warn or off each sample traps at
/// run time or leaves heap room behind.
pub const MISTAKES: &str = "mistakes";

/// The groups, in picker order.
pub const GROUPS: &[&str] = &[
    "basics",
    "control",
    "functions",
    "heap",
    "lists",
    "programs",
    MISTAKES,
    DIAGNOSTICS,
];

/// The samples, in picker order.
#[rustfmt::skip]
pub const SAMPLES: &[Sample] = &[
    sample!("basics", "hello"),
    sample!("basics", "variables"),
    sample!("basics", "literals"),
    sample!("basics", "arithmetic"),
    sample!("basics", "comparisons"),
    sample!("basics", "logic"),
    sample!("basics", "bits"),
    sample!("basics", "conversion"),
    sample!("basics", "comments"),
    sample!("basics", "folding"),
    sample!("control", "branches"),
    sample!("control", "scopes"),
    sample!("control", "while"),
    sample!("control", "counted"),
    sample!("control", "generic"),
    sample!("control", "break_continue"),
    sample!("functions", "procedures"),
    sample!("functions", "results"),
    sample!("functions", "recursion"),
    sample!("functions", "mixed_kinds"),
    sample!("heap", "swap"),
    sample!("heap", "divide"),
    sample!("heap", "walk"),
    sample!("heap", "growable"),
    sample!("heap", "record"),
    sample!("heap", "pascal"),
    sample!("heap", "arena"),
    sample!("lists", "basics"),
    sample!("lists", "copies"),
    sample!("lists", "equal"),
    sample!("lists", "grid"),
    sample!("lists", "borrow"),
    sample!("lists", "sort"),
    sample!("programs", "gradebook"),
    sample!("programs", "fizzbuzz"),
    sample!("programs", "gcd"),
    sample!("programs", "primes"),
    sample!("programs", "fibonacci"),
    sample!("programs", "power"),
    sample!("programs", "collatz"),
    sample!("programs", "digits"),
    sample!("programs", "palindrome"),
    sample!("programs", "popcount"),
    sample!("programs", "isqrt"),
    sample!("programs", "temperature"),
    sample!("programs", "leap"),
    sample!("programs", "max_of_three"),
    sample!("programs", "triangle"),
    sample!("mistakes", "leak"),
    sample!("mistakes", "null_read"),
    sample!("mistakes", "past_the_end"),
    sample!("mistakes", "use_after_free"),
    sample!("mistakes", "double_free"),
    sample!("mistakes", "bad_free"),
    sample!("mistakes", "zero_divide"),
    sample!("mistakes", "endless_recursion"),
    sample!("mistakes", "endless_loop"),
    sample!("diagnostics", "lexer_errors"),
    sample!("diagnostics", "parser_errors"),
    sample!("diagnostics", "check_errors"),
    sample!("diagnostics", "platform_bits"),
    sample!("diagnostics", "fold_errors"),];

/// The sample shown when the page opens without a link.
///
/// # Returns
///
/// The first sample.
pub fn default_sample() -> &'static Sample { &SAMPLES[0] }

/// The samples of a group, in picker order.
///
/// # Arguments
///
/// * `group` - The group.
///
/// # Returns
///
/// The samples.
pub fn in_group(group: &str) -> impl Iterator<Item = &'static Sample>
{
    SAMPLES.iter().filter(move |s| s.group == group)
}

/// Find a sample on its `group/name` path.
///
/// # Arguments
///
/// * `path` - The path.
///
/// # Returns
///
/// The sample, or `None`.
pub fn find_sample(path: &str) -> Option<&'static Sample>
{
    SAMPLES.iter().find(|s| s.path() == path)
}
