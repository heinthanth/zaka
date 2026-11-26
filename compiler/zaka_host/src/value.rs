/// A value that crosses between the VM and the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostValue
{
    /// Text.
    Text(String),
    /// The bits of a 64-bit float.
    Float(u64),
    /// A signed 64-bit integer.
    Int(i64),
    /// A stream: the handle number its struct holds.
    Stream(u32),
    /// A member of an enumeration, counted from 0.
    Member(u32),
    /// A plain list of text.
    Texts(Vec<String>),
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
    /// The 64-bit float at a place.
    pub(crate) fn float(&self, n: usize) -> f64
    {
        match self.0.get(n)
        {
            Some(HostValue::Float(raw)) => f64::from_bits(*raw),
            other => panic!("argument {n} is no float: {other:?}"),
        }
    }

    /// The signed 64-bit integer at a place.
    pub(crate) fn int(&self, n: usize) -> i64
    {
        match self.0.get(n)
        {
            Some(HostValue::Int(value)) => *value,
            other => panic!("argument {n} is no integer: {other:?}"),
        }
    }

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

    /// The list of text argument at a place.
    ///
    /// # Arguments
    ///
    /// * `n` - The place, counted from 0.
    ///
    /// # Returns
    ///
    /// The texts.
    ///
    /// # Panics
    ///
    /// When the argument is no list of text, which a matched declaration
    /// rules out.
    pub fn texts(&self, n: usize) -> &[String]
    {
        match self.0.get(n)
        {
            Some(HostValue::Texts(texts)) => texts,
            other => panic!("argument {n} is no list of text: {other:?}"),
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
