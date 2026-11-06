//! The functions the host serves: each external function
//! (`ပြင်ပမှလာသော`) a program declares must be one of them, with the same
//! name, parameter kinds and result kind. The VM serves them while the
//! program runs; a native build would map them to libc.

use serde::Deserialize;
use serde::Serialize;
use strum::EnumIter;
use strum::IntoEnumIterator;
use strum::IntoStaticStr;

use crate::kind::DataKind;

/// The error kind of each host function: an enumeration of failure classes.
pub const IO_ERROR: &str = "အချက်အလက်အဝင်အထွက်အမှားအယွင်း";

/// The kind of an input stream.
pub const INPUT_STREAM: &str = "အချက်အလက်အဝင်စီးကြောင်း";

/// The kind of an output stream.
pub const OUTPUT_STREAM: &str = "အချက်အလက်အထွက်စီးကြောင်း";

/// The kind name of text.
const TEXT: &str = "text";

/// A function the host serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(EnumIter, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum HostFn
{
    /// Read one line of an input stream, without its line break.
    ReadLine,
    /// Read a whole file as text.
    ReadFile,
    /// Write text as the whole of a file.
    WriteFile,
    /// Write text to an output stream, with no line break.
    WriteStream,
}

/// What a host function takes and gives, as kind names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostSig
{
    /// The name a program declares it with.
    pub name: &'static str,
    /// The kind name of each parameter.
    pub params: &'static [&'static str],
    /// The kind name of the good result; `None` for a good result that holds
    /// no value.
    pub ok: Option<&'static str>,
}

impl HostFn
{
    /// What the function takes and gives. Each one can fail with
    /// [`IO_ERROR`].
    ///
    /// # Returns
    ///
    /// The signature.
    pub const fn sig(self) -> HostSig
    {
        match self
        {
            HostFn::ReadLine => HostSig {
                name: "စာကြောင်းတစ်ကြောင်းဖတ်",
                params: &[INPUT_STREAM],
                ok: Some(TEXT),
            },
            HostFn::ReadFile =>
            {
                HostSig {
                    name: "ဖိုင်ကိုဖတ်", params: &[TEXT], ok: Some(TEXT)
                }
            }
            HostFn::WriteFile =>
            {
                HostSig { name: "ရေး", params: &[TEXT, TEXT], ok: None }
            }
            HostFn::WriteStream =>
            {
                HostSig { name: "ရေး", params: &[TEXT, OUTPUT_STREAM], ok: None }
            }
        }
    }

    /// The host function an external declaration names: the same name, the
    /// same parameter kinds, and a result kind that can fail with
    /// [`IO_ERROR`] and gives the good result of the host function.
    ///
    /// # Arguments
    ///
    /// * `name` - The declared name.
    /// * `params` - The declared parameter kinds.
    /// * `ret` - The declared result kind.
    ///
    /// # Returns
    ///
    /// The host function, or `None` when none matches.
    pub fn find(
        name: &str, params: &[DataKind], ret: Option<DataKind>,
    ) -> Option<HostFn>
    {
        let (ok, err) = ret.and_then(DataKind::fallible_parts)?;
        let named = |k: Option<DataKind>| k.map(DataKind::name);
        let fits = |f: &HostFn| {
            let sig = f.sig();
            let kinds = params.iter().map(|k| k.name()).eq(sig.params.iter().copied());
            let gives = named(ok).as_deref() == sig.ok
                && named(err).as_deref() == Some(IO_ERROR);
            sig.name == name && kinds && gives
        };
        HostFn::iter().find(fits)
    }

    /// Each host function, as a message lists it: `name(kind, kind)`.
    ///
    /// # Returns
    ///
    /// The list.
    pub fn listing() -> String
    {
        let each = HostFn::iter().map(|f| {
            let sig = f.sig();
            format!("{}({})", sig.name, sig.params.join(", "))
        });
        each.collect::<Vec<_>>().join(" / ")
    }
}
