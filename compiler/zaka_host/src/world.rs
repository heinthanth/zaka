use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io;
use std::io::BufRead;

use bon::Builder;
use zakac_rust_sugars::ret_if;

use crate::kinds::STDERR;
use crate::kinds::STDIN;
use crate::kinds::STDOUT;

/// The effects outside the VM that host functions use. The VM keeps its world
/// with the rest of its state and clones it along, so a world that the
/// explorer winds back is a world in storage.
pub trait World: Debug + Send + Sync
{
    /// Read one line of standard input, without its line break.
    ///
    /// # Returns
    ///
    /// The line, `None` at the end of the input, or the error.
    fn read_line(&mut self) -> io::Result<Option<String>>;

    /// Read a whole file as text.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file.
    ///
    /// # Returns
    ///
    /// The text, or the error; octets that are not UTF-8 give
    /// [`io::ErrorKind::InvalidData`].
    fn read_file(&mut self, path: &str) -> io::Result<String>;

    /// Write text as the whole of a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file.
    /// * `data` - The text.
    ///
    /// # Returns
    ///
    /// `Ok`, or the error.
    fn write_file(&mut self, path: &str, data: &str) -> io::Result<()>;

    /// A clone of the world in a box, so a VM that holds one can be cloned.
    ///
    /// # Returns
    ///
    /// The clone.
    fn boxed(&self) -> Box<dyn World>;
}

impl Clone for Box<dyn World>
{
    fn clone(&self) -> Self { self.boxed() }
}

/// What a host function reaches while it runs: the world, and the output and
/// error text of the VM, so the writes of a program to its standard streams
/// keep their order with what it prints.
pub struct Effects<'a>
{
    /// The world.
    pub world: &'a mut dyn World,
    /// What the program wrote to standard output and the VM did not hand on
    /// so far.
    pub out: &'a mut String,
    /// What the program wrote to standard error and the VM did not hand on
    /// so far.
    pub err: &'a mut String,
}

impl Effects<'_>
{
    /// Read one line of an input stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - The handle of the stream.
    ///
    /// # Returns
    ///
    /// The line, `None` at the end of the input, or the error.
    pub fn read_line(&mut self, stream: u32) -> io::Result<Option<String>>
    {
        ret_if!(stream != STDIN, Err(no_stream(stream)));
        self.world.read_line()
    }

    /// Read a whole file as text.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file.
    ///
    /// # Returns
    ///
    /// The text, or the error.
    pub fn read_file(&mut self, path: &str) -> io::Result<String>
    {
        self.world.read_file(path)
    }

    /// Write text as the whole of a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file.
    /// * `data` - The text.
    ///
    /// # Returns
    ///
    /// `Ok`, or the error.
    pub fn write_file(&mut self, path: &str, data: &str) -> io::Result<()>
    {
        self.world.write_file(path, data)
    }

    /// Write text to an output stream, with no line break.
    ///
    /// # Arguments
    ///
    /// * `stream` - The handle of the stream.
    /// * `data` - The text.
    ///
    /// # Returns
    ///
    /// `Ok`, or the error for a handle that names no output stream.
    pub fn write(&mut self, stream: u32, data: &str) -> io::Result<()>
    {
        match stream
        {
            STDOUT => self.out.push_str(data),
            STDERR => self.err.push_str(data),
            _ => return Err(no_stream(stream)),
        }
        Ok(())
    }
}

/// The error for a handle that names no stream of the right direction.
///
/// # Arguments
///
/// * `stream` - The handle.
///
/// # Returns
///
/// The error.
fn no_stream(stream: u32) -> io::Error
{
    io::Error::other(format!("stream {stream} is not open in this direction"))
}

/// A world in storage: input lines given before the run, and files that
/// live as long as the world. The browser and the tests run in it, and a VM
/// runs in a blank one unless it is given another.
#[derive(Debug, Clone, Default, PartialEq, Eq, Builder)]
pub struct MemWorld
{
    /// What standard input holds, line after line.
    #[builder(default, into)]
    pub input: String,
    /// How far standard input has been read, in octets.
    #[builder(default)]
    pub read: usize,
    /// Each file, under its path.
    #[builder(default)]
    pub files: BTreeMap<String, String>,
}

impl World for MemWorld
{
    fn read_line(&mut self) -> io::Result<Option<String>>
    {
        let rest = &self.input[self.read..];
        ret_if!(rest.is_empty(), Ok(None));
        let line = rest.split_inclusive('\n').next().unwrap_or(rest);
        self.read += line.len();
        let line = line.strip_suffix('\n').unwrap_or(line);
        Ok(Some(line.strip_suffix('\r').unwrap_or(line).to_string()))
    }

    fn read_file(&mut self, path: &str) -> io::Result<String>
    {
        let found = self.files.get(path).cloned();
        found.ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn write_file(&mut self, path: &str, data: &str) -> io::Result<()>
    {
        self.files.insert(path.to_string(), data.to_string());
        Ok(())
    }

    fn boxed(&self) -> Box<dyn World> { Box::new(self.clone()) }
}

/// The world of a program run from the terminal: the standard input of the
/// process and the files on disk. A clone shares them.
#[derive(Debug, Clone, Copy, Default)]
pub struct TermWorld;

impl World for TermWorld
{
    fn read_line(&mut self) -> io::Result<Option<String>>
    {
        let mut line = String::new();
        let read = io::stdin().lock().read_line(&mut line)?;
        ret_if!(read == 0, Ok(None));
        let line = line.strip_suffix('\n').unwrap_or(&line);
        Ok(Some(line.strip_suffix('\r').unwrap_or(line).to_string()))
    }

    fn read_file(&mut self, path: &str) -> io::Result<String>
    {
        std::fs::read_to_string(path)
    }

    fn write_file(&mut self, path: &str, data: &str) -> io::Result<()>
    {
        std::fs::write(path, data)
    }

    fn boxed(&self) -> Box<dyn World> { Box::new(*self) }
}
