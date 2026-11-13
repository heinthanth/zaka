use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::io;
use std::io::BufRead;

use bon::Builder;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::kinds::IS_FILE;
use crate::kinds::IS_FOLDER;
use crate::kinds::IS_OTHER;
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

    /// Add text at the end of a file, making the file when it is not there.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file.
    /// * `data` - The text.
    ///
    /// # Returns
    ///
    /// `Ok`, or the error.
    fn append_file(&mut self, path: &str, data: &str) -> io::Result<()>;

    /// Remove a file, or a folder that holds nothing.
    ///
    /// # Arguments
    ///
    /// * `path` - The path.
    ///
    /// # Returns
    ///
    /// `Ok`, or the error.
    fn remove(&mut self, path: &str) -> io::Result<()>;

    /// Make a folder and each folder above it that is not there.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the folder.
    ///
    /// # Returns
    ///
    /// `Ok`, or the error.
    fn make_folder(&mut self, path: &str) -> io::Result<()>;

    /// The names a folder holds, sorted.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the folder.
    ///
    /// # Returns
    ///
    /// The names, or the error.
    fn list_folder(&mut self, path: &str) -> io::Result<Vec<String>>;

    /// What a path holds: [`IS_FILE`], [`IS_FOLDER`] or [`IS_OTHER`].
    ///
    /// # Arguments
    ///
    /// * `path` - The path.
    ///
    /// # Returns
    ///
    /// The member, or the error for a path that holds nothing.
    fn path_kind(&mut self, path: &str) -> io::Result<u32>;

    /// Move a file or a folder, replacing what the target holds.
    ///
    /// # Arguments
    ///
    /// * `from` - The old path.
    /// * `to` - The new path.
    ///
    /// # Returns
    ///
    /// `Ok`, or the error.
    fn rename(&mut self, from: &str, to: &str) -> io::Result<()>;

    /// Give standard input more text, as a terminal does when a line is
    /// entered. A world that cannot take more does nothing.
    ///
    /// # Arguments
    ///
    /// * `text` - The text, with its line break.
    fn give_input(&mut self, text: &str) { let _ = text; }

    /// Close standard input, so a read past what it holds fails with the end
    /// of the input. A world that cannot take more does nothing.
    fn close_input(&mut self) {}

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
    /// Whether a read found no input so far in a world that waits for more: the
    /// VM then runs the call again once input comes.
    pub blocked: bool,
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
        let line = self.world.read_line();
        self.blocked |=
            line.as_ref().is_err_and(|e| e.kind() == io::ErrorKind::WouldBlock);
        line
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

/// A world in storage: input lines given before the run, or line after line
/// while it runs, and files that live as long as the world. The browser and
/// the tests run in it, and a VM runs in a blank one unless it is given
/// another.
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
    /// Whether more input can come, as at a terminal: a read past the input
    /// waits ([`io::ErrorKind::WouldBlock`]) instead of failing with its end.
    #[builder(default)]
    pub open: bool,
    /// Each folder made with no file in it so far; a folder that holds a file
    /// is there through the path of the file.
    #[builder(default)]
    pub folders: BTreeSet<String>,
}

// shortcut: MemWorld stores paths as written, with no `.`, `..` or `\` handled;
// normalize them when the explorer grows a view of its files.
impl MemWorld
{
    /// Whether a path names a folder: one made, or one above a file or a
    /// folder made.
    ///
    /// # Arguments
    ///
    /// * `path` - The path, with no `/` at its end.
    ///
    /// # Returns
    ///
    /// `true` for a folder.
    fn is_folder(&self, path: &str) -> bool
    {
        ret_if!(path.is_empty(), true);
        let prefix = format!("{path}/");
        let names = self.files.keys().chain(self.folders.iter());
        self.folders.contains(path) || names.into_iter().any(|k| k.starts_with(&prefix))
    }

    /// Each path the world holds, files and folders made.
    ///
    /// # Returns
    ///
    /// The paths.
    fn paths(&self) -> impl Iterator<Item = &String>
    {
        self.files.keys().chain(self.folders.iter())
    }
}

/// A path with no `/` at its end and no `./` at its start, as [`MemWorld`]
/// stores it.
///
/// # Arguments
///
/// * `path` - The path.
///
/// # Returns
///
/// The stored path.
fn mem_path(path: &str) -> String
{
    let path = path.strip_prefix("./").unwrap_or(path);
    let path = path.trim_end_matches('/');
    tern!(path == ".", String::new(), path.to_string())
}

impl World for MemWorld
{
    fn read_line(&mut self) -> io::Result<Option<String>>
    {
        let rest = &self.input[self.read..];
        let waits = rest.is_empty() && self.open;
        ret_if!(waits, Err(io::ErrorKind::WouldBlock.into()));
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

    fn append_file(&mut self, path: &str, data: &str) -> io::Result<()>
    {
        self.files.entry(mem_path(path)).or_default().push_str(data);
        Ok(())
    }

    fn remove(&mut self, path: &str) -> io::Result<()>
    {
        let stored = mem_path(path);
        ret_if!(self.files.remove(&stored).is_some(), Ok(()));
        ret_if!(!self.is_folder(&stored), Err(io::ErrorKind::NotFound.into()));
        let prefix = format!("{stored}/");
        let held = self.paths().any(|p| p.starts_with(&prefix));
        ret_if!(held, Err(io::Error::other(format!("the folder {path} is not empty"))));
        self.folders.remove(&stored);
        Ok(())
    }

    fn make_folder(&mut self, path: &str) -> io::Result<()>
    {
        let stored = mem_path(path);
        let a_file = self.files.contains_key(&stored);
        ret_if!(a_file, Err(io::Error::other(format!("{path} is a file"))));
        // Each folder above it is made too, as on disk.
        let ends = stored.match_indices('/').map(|(at, _)| at).chain([stored.len()]);
        let made: Vec<String> =
            ends.filter(|at| *at > 0).map(|at| stored[..at].to_string()).collect();
        self.folders.extend(made);
        Ok(())
    }

    fn list_folder(&mut self, path: &str) -> io::Result<Vec<String>>
    {
        let stored = mem_path(path);
        ret_if!(!self.is_folder(&stored), Err(io::ErrorKind::NotFound.into()));
        let prefix = tern!(stored.is_empty(), String::new(), format!("{stored}/"));
        let under = self.paths().filter_map(|p| p.strip_prefix(&prefix));
        let names =
            under.filter_map(|rest| rest.split('/').next()).filter(|n| !n.is_empty());
        let names: BTreeSet<String> = names.map(str::to_string).collect();
        Ok(names.into_iter().collect())
    }

    fn path_kind(&mut self, path: &str) -> io::Result<u32>
    {
        let stored = mem_path(path);
        ret_if!(self.files.contains_key(&stored), Ok(IS_FILE));
        ret_if!(self.is_folder(&stored), Ok(IS_FOLDER));
        Err(io::ErrorKind::NotFound.into())
    }

    fn rename(&mut self, from: &str, to: &str) -> io::Result<()>
    {
        let (from, to) = (mem_path(from), mem_path(to));
        if let Some(data) = self.files.remove(&from)
        {
            self.files.insert(to, data);
            return Ok(());
        }
        ret_if!(!self.is_folder(&from), Err(io::ErrorKind::NotFound.into()));
        let prefix = format!("{from}/");
        let moved =
            |p: &String| p.strip_prefix(&prefix).map(|rest| format!("{to}/{rest}"));
        let files: Vec<(String, String)> = self
            .files
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (old, data) in files
        {
            self.files.remove(&old);
            do_if!(let Some(new) = moved(&old), _ = self.files.insert(new, data));
        }
        let folders: Vec<String> = self
            .folders
            .iter()
            .filter(|f| **f == from || f.starts_with(&prefix))
            .cloned()
            .collect();
        for old in folders
        {
            self.folders.remove(&old);
            let new = tern!(old == from, to.clone(), moved(&old).unwrap_or_default());
            self.folders.insert(new);
        }
        Ok(())
    }

    fn give_input(&mut self, text: &str) { self.input.push_str(text); }

    fn close_input(&mut self) { self.open = false; }

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

    fn append_file(&mut self, path: &str, data: &str) -> io::Result<()>
    {
        use std::io::Write;
        let mut file =
            std::fs::OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(data.as_bytes())
    }

    fn remove(&mut self, path: &str) -> io::Result<()>
    {
        let folder = std::fs::symlink_metadata(path)?.is_dir();
        tern!(folder, std::fs::remove_dir(path), std::fs::remove_file(path))
    }

    fn make_folder(&mut self, path: &str) -> io::Result<()>
    {
        std::fs::create_dir_all(path)
    }

    fn list_folder(&mut self, path: &str) -> io::Result<Vec<String>>
    {
        let text = |e: std::fs::DirEntry| {
            let name = e.file_name().into_string();
            name.map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
        };
        let entries = std::fs::read_dir(path)?;
        let names = entries.map(|e| e.and_then(text));
        let mut names = names.collect::<io::Result<Vec<String>>>()?;
        names.sort();
        Ok(names)
    }

    fn path_kind(&mut self, path: &str) -> io::Result<u32>
    {
        let meta = std::fs::metadata(path)?;
        let kind =
            tern!(meta.is_dir(), IS_FOLDER, tern!(meta.is_file(), IS_FILE, IS_OTHER));
        Ok(kind)
    }

    fn rename(&mut self, from: &str, to: &str) -> io::Result<()>
    {
        std::fs::rename(from, to)
    }

    fn boxed(&self) -> Box<dyn World> { Box::new(*self) }
}
