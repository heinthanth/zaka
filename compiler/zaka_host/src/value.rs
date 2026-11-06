/// A value that crosses between the VM and the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostValue
{
    /// Text.
    Text(String),
    /// A stream: the handle number its struct holds.
    Stream(u32),
    /// A member of an enumeration, counted from 0.
    Member(u32),
    /// A good result, with its value when it holds one.
    Good(Option<Box<HostValue>>),
    /// A failed result, with its error value.
    Fault(Box<HostValue>),
}

/// The arguments of a host call, in the order of the parameters. The check
/// matched the declaration to its row, so each one has the kind of its
/// parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args(pub Vec<HostValue>);

impl Args
{
    /// The text argument at a place.
    ///
    /// # Arguments
    ///
    /// * `n` - The place, counted from 0.
    ///
    /// # Returns
    ///
    /// The text.
    ///
    /// # Panics
    ///
    /// When the argument is no text, which a matched declaration rules out.
    pub fn text(&self, n: usize) -> &str
    {
        match self.0.get(n)
        {
            Some(HostValue::Text(text)) => text,
            other => panic!("argument {n} is no text: {other:?}"),
        }
    }

    /// The stream argument at a place.
    ///
    /// # Arguments
    ///
    /// * `n` - The place, counted from 0.
    ///
    /// # Returns
    ///
    /// The handle of the stream.
    ///
    /// # Panics
    ///
    /// When the argument is no stream, which a matched declaration rules out.
    pub fn stream(&self, n: usize) -> u32
    {
        match self.0.get(n)
        {
            Some(HostValue::Stream(handle)) => *handle,
            other => panic!("argument {n} is no stream: {other:?}"),
        }
    }
}
