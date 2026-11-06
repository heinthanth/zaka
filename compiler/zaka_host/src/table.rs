use std::io;

use serde::Deserialize;
use serde::Serialize;
use strum::EnumIter;
use strum::IntoStaticStr;

use crate::kinds::BAD_TEXT;
use crate::kinds::END_OF_INPUT;
use crate::kinds::INPUT_STREAM;
use crate::kinds::IO_ERROR;
use crate::kinds::NO_PERMISSION;
use crate::kinds::NOT_FOUND;
use crate::kinds::OTHER;
use crate::kinds::OUTPUT_STREAM;
use crate::kinds::TEXT;
use crate::value::Args;
use crate::value::HostValue;
use crate::world::Effects;

/// One host function: what a program declares it with, and the Rust that
/// runs it.
#[derive(Debug, Clone, Copy)]
pub struct HostDef
{
    /// Which one it is.
    pub func: HostFn,
    /// The name a program declares it with.
    pub name: &'static str,
    /// The kind name of each parameter.
    pub params: &'static [&'static str],
    /// The kind name of the good result; `None` for a good result that holds
    /// no value. Each host function can fail with [`IO_ERROR`].
    pub ok: Option<&'static str>,
    /// The work: the result for the arguments.
    pub run: fn(&Args, &mut Effects<'_>) -> HostValue,
}

/// Make [`HostFn`], one variant for each row, and [`HOST_FNS`], the rows in
/// the same order.
macro_rules! host_fns {
    ($( $(#[$doc:meta])* $id:ident = $name:literal ($($param:expr),*) -> $ok:expr, $run:expr; )*) => {
        /// A function the host serves.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[derive(EnumIter, IntoStaticStr)]
        #[serde(rename_all = "snake_case")]
        #[strum(serialize_all = "snake_case")]
        pub enum HostFn
        {
            $( $(#[$doc])* $id, )*
        }

        /// Each host function, in the order of [`HostFn`].
        pub static HOST_FNS: &[HostDef] = &[
            $( HostDef { func: HostFn::$id, name: $name, params: &[$($param),*], ok: $ok, run: $run }, )*
        ];
    };
}

host_fns! {
    /// Read one line of an input stream, without its line break.
    ReadLine = "စာကြောင်းတစ်ကြောင်းဖတ်"(INPUT_STREAM) -> Some(TEXT), read_line;
    /// Read a whole file as text.
    ReadFile = "ဖိုင်ကိုဖတ်"(TEXT) -> Some(TEXT), read_file;
    /// Write text as the whole of a file: the text, then the path.
    WriteFile = "ရေး"(TEXT, TEXT) -> None, write_file;
    /// Write text to an output stream, with no line break: the text, then the
    /// stream.
    WriteStream = "ရေး"(TEXT, OUTPUT_STREAM) -> None, write_stream;
}

/// Run [`HostFn::ReadLine`].
fn read_line(args: &Args, e: &mut Effects<'_>) -> HostValue
{
    let line = e.read_line(args.stream(0));
    text_or_fault(
        line.and_then(|l| l.ok_or_else(|| io::ErrorKind::UnexpectedEof.into())),
    )
}

/// Run [`HostFn::ReadFile`].
fn read_file(args: &Args, e: &mut Effects<'_>) -> HostValue
{
    text_or_fault(e.read_file(args.text(0)))
}

/// Run [`HostFn::WriteFile`].
fn write_file(args: &Args, e: &mut Effects<'_>) -> HostValue
{
    nothing_or_fault(e.write_file(args.text(1), args.text(0)))
}

/// Run [`HostFn::WriteStream`].
fn write_stream(args: &Args, e: &mut Effects<'_>) -> HostValue
{
    nothing_or_fault(e.write(args.stream(1), args.text(0)))
}

impl HostFn
{
    /// The row of the function.
    ///
    /// # Returns
    ///
    /// The row.
    pub fn def(self) -> &'static HostDef { &HOST_FNS[self as usize] }

    /// Run the function.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments, in the order of the parameters.
    /// * `effects` - What the function reaches.
    ///
    /// # Returns
    ///
    /// The result: a good result, or a failure with a member of [`IO_ERROR`].
    pub fn serve(self, args: &Args, effects: &mut Effects<'_>) -> HostValue
    {
        (self.def().run)(args, effects)
    }

    /// The host function an external declaration names: the same name, the
    /// same parameter kinds, and a result that can fail with [`IO_ERROR`]
    /// with the good result of the row.
    ///
    /// # Arguments
    ///
    /// * `name` - The declared name.
    /// * `params` - The kind name of each declared parameter.
    /// * `ok` - The kind name of the declared good result, `None` with no
    ///   value.
    /// * `err` - The kind name of the declared error.
    ///
    /// # Returns
    ///
    /// The host function, or `None` when no row matches.
    pub fn find(
        name: &str, params: &[String], ok: Option<&str>, err: Option<&str>,
    ) -> Option<HostFn>
    {
        let fits = |d: &&HostDef| {
            let kinds = params.iter().map(String::as_str).eq(d.params.iter().copied());
            d.name == name && kinds && ok == d.ok && err == Some(IO_ERROR)
        };
        HOST_FNS.iter().find(fits).map(|d| d.func)
    }
}

/// Each host function, as a message lists it: `name(kind, kind)`.
///
/// # Returns
///
/// The list.
pub fn listing() -> String
{
    let each = HOST_FNS.iter().map(|d| format!("{}({})", d.name, d.params.join(", ")));
    each.collect::<Vec<_>>().join(" / ")
}

/// A good result that holds text, or a failure.
///
/// # Arguments
///
/// * `read` - What the effect gave.
///
/// # Returns
///
/// The result.
fn text_or_fault(read: io::Result<String>) -> HostValue
{
    read.map_or_else(fault, |t| HostValue::Good(Some(Box::new(HostValue::Text(t)))))
}

/// A good result that holds nothing, or a failure.
///
/// # Arguments
///
/// * `done` - What the effect gave.
///
/// # Returns
///
/// The result.
fn nothing_or_fault(done: io::Result<()>) -> HostValue
{
    done.map_or_else(fault, |()| HostValue::Good(None))
}

/// The failure for an error of an effect: the member of [`IO_ERROR`] that
/// names its class.
///
/// # Arguments
///
/// * `e` - The error.
///
/// # Returns
///
/// The failure.
fn fault(e: io::Error) -> HostValue
{
    let member = match e.kind()
    {
        io::ErrorKind::NotFound => NOT_FOUND,
        io::ErrorKind::PermissionDenied => NO_PERMISSION,
        io::ErrorKind::UnexpectedEof => END_OF_INPUT,
        io::ErrorKind::InvalidData => BAD_TEXT,
        _ => OTHER,
    };
    HostValue::Fault(Box::new(HostValue::Member(member)))
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::kinds::STDIN;
    use crate::kinds::STDOUT;
    use crate::world::MemWorld;

    /// Run a host function in a world, and give back its result and what it
    /// wrote to standard output.
    fn run(
        func: HostFn, args: Vec<HostValue>, world: &mut MemWorld,
    ) -> (HostValue, String)
    {
        let (mut out, mut err) = (String::new(), String::new());
        let mut effects = Effects { world, out: &mut out, err: &mut err };
        (func.serve(&Args(args), &mut effects), out)
    }

    #[test]
    fn rows_follow_the_variants()
    {
        use strum::IntoEnumIterator;
        HostFn::iter().for_each(|f| assert_eq!(f.def().func, f));
    }

    #[test]
    fn functions_run_in_a_world_in_storage()
    {
        let text = |t: &str| HostValue::Text(t.to_string());
        let good = |v: Option<HostValue>| HostValue::Good(v.map(Box::new));
        let missing = HostValue::Fault(Box::new(HostValue::Member(NOT_FOUND)));
        let end = HostValue::Fault(Box::new(HostValue::Member(END_OF_INPUT)));
        let mut world = MemWorld::builder().input("one\r\ntwo").build();
        let line = vec![HostValue::Stream(STDIN)];
        assert_eq!(
            run(HostFn::ReadLine, line.clone(), &mut world).0,
            good(Some(text("one")))
        );
        assert_eq!(
            run(HostFn::ReadLine, line.clone(), &mut world).0,
            good(Some(text("two")))
        );
        assert_eq!(run(HostFn::ReadLine, line, &mut world).0, end);
        let path = vec![text("a.txt")];
        assert_eq!(run(HostFn::ReadFile, path.clone(), &mut world).0, missing);
        let write = vec![text("hi"), text("a.txt")];
        assert_eq!(run(HostFn::WriteFile, write, &mut world).0, good(None));
        assert_eq!(run(HostFn::ReadFile, path, &mut world).0, good(Some(text("hi"))));
        let out = run(
            HostFn::WriteStream,
            vec![text("x"), HostValue::Stream(STDOUT)],
            &mut world,
        );
        assert_eq!(out, (good(None), "x".to_string()));
    }
}
