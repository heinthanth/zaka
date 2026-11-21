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

/// The group of the samples that are meant to report errors.
pub const DIAGNOSTICS: &str = "diagnostics";

/// The group of the samples that go wrong: the checker rejects the
/// pointer mistakes, and with the check on warn or off each sample traps at
/// run time or leaves heap room behind.
pub const MISTAKES: &str = "mistakes";

include!(concat!(env!("OUT_DIR"), "/samples.rs"));

/// The sample shown when the page opens without a link.
///
/// # Returns
///
/// The hello sample, or the first available sample.
pub fn default_sample() -> &'static Sample
{
    find_sample("basics/hello").unwrap_or(&SAMPLES[0])
}

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
