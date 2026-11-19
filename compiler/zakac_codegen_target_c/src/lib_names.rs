//! The names of a C lib: its own name, which the names C calls hold
//! (`zk_reset_<lib>`), and the prefix of each name it links under that C
//! never calls, `zk_<lib>_<hash>_`, so two libs in one program never clash.
//! The hash covers the lib name, its program and the runtime `zakac` carries.
//!
//! The C files of a lib include `zaka_lib_names.h` first, which maps each
//! such name to its linked form; a Cranelift object renames them in place.

use std::fmt::Write;

use bon::Builder;
use zakac_host_errors::Error;
use zakac_ir_builder::ir::Module;
use zakac_rust_sugars::ret_if;

use crate::RUNTIME_PREFIX;
use crate::emit::CFile;
use crate::emit::PROGRAM_TABLE;

/// The header that maps the names of a lib to their linked form.
pub const NAMES_HEADER: &str = "zaka_lib_names.h";

/// The start of each numbered name a lib links under that C never calls: its
/// functions, their records, its globals, its kind descriptors, and the
/// shapes and the bridges of its function kinds.
const NUMBERED: &[&str] = &["zk_f", "zk_r", "zk_g", "zk_k", "zk_cbs_", "zk_cbb_"];

/// The other names a lib links under that C never calls.
const FIXED: &[&str] = &[PROGRAM_TABLE, "zk_lib_init"];

/// The names of a C lib.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct LibNames
{
    /// The name of the lib, a C name.
    #[builder(into)]
    pub name: String,
    /// The prefix of its linked names, in place of `zk_`.
    #[builder(into)]
    pub prefix: String,
}

impl LibNames
{
    /// The names of a lib.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the lib.
    /// * `module` - Its program.
    ///
    /// # Returns
    ///
    /// The names, or the error of a name C cannot take.
    pub fn of(name: &str, module: &Module) -> Result<LibNames, Error>
    {
        let mut chars = name.chars();
        let first = chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
        let rest = chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
        ret_if!(!first || !rest, Err(Error::LibName { name: name.to_string() }));
        let seed = format!("{name}\n{RUNTIME_PREFIX}\n{module:?}");
        let hash = xxhash_rust::xxh3::xxh3_64(seed.as_bytes()) as u32;
        Ok(LibNames::builder()
            .name(name)
            .prefix(format!("zk_{name}_{hash:08x}_"))
            .build())
    }

    /// The linked form of a name of the lib that C never calls.
    ///
    /// # Arguments
    ///
    /// * `plain` - The name as the generated code writes it.
    ///
    /// # Returns
    ///
    /// The linked name, or `None` for a name of another kind.
    pub fn linked(&self, plain: &str) -> Option<String>
    {
        let rest = plain.strip_prefix("zk_")?;
        let numbered = |start: &&str| {
            plain
                .strip_prefix(*start)
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()))
        };
        let ours = FIXED.contains(&plain) || NUMBERED.iter().any(numbered);
        ours.then(|| format!("{}{rest}", self.prefix))
    }

    /// The names of the lib that C never calls and that an object holds:
    /// each fixed one it holds, and each numbered one from 0 up to the first
    /// it lacks.
    ///
    /// # Arguments
    ///
    /// * `holds` - Whether the object holds a name.
    ///
    /// # Returns
    ///
    /// The names, as the generated code writes them.
    pub fn held(holds: impl Fn(&str) -> bool) -> Vec<String>
    {
        let fixed = FIXED.iter().map(|n| n.to_string()).filter(|n| holds(n));
        let numbered = NUMBERED.iter().flat_map(|start| {
            let names = (0..).map(move |n| format!("{start}{n}"));
            names.take_while(|n| holds(n)).collect::<Vec<_>>()
        });
        fixed.chain(numbered).collect()
    }

    /// The name of the function that resets the lib.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn reset(&self) -> String { format!("zk_reset_{}", self.name) }

    /// The name of the function that frees what the lib gave back, with the C
    /// runtime the lib was built with.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn free(&self) -> String { format!("zk_free_from_{}", self.name) }

    /// The name of the function that frees what the lib gave back with each
    /// block in it, with the C runtime the lib was built with.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn free_all(&self) -> String { format!("zk_free_all_from_{}", self.name) }

    /// The name of the function that gives the last trap of the lib.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn last_trap(&self) -> String { format!("zk_last_trap_from_{}", self.name) }

    /// Add the header of the linked names to the C files of the lib, and
    /// include it first in each C source.
    ///
    /// # Arguments
    ///
    /// * `files` - The C files.
    pub fn add_to(&self, files: &mut Vec<CFile>)
    {
        // The glue header names the fixed ones, which the generated text can
        // leave out.
        let fixed = FIXED.iter().map(|n| n.to_string());
        let mut used: Vec<String> =
            files.iter().flat_map(|f| self.used(&f.text)).chain(fixed).collect();
        used.sort();
        used.dedup();
        let mut header = format!(
            "/* The names the lib {} links under that C never calls. */\n#ifndef \
             ZAKA_LIB_NAMES_H\n#define ZAKA_LIB_NAMES_H\n",
            self.name
        );
        let defines = used.iter().filter_map(|p| Some((p, self.linked(p)?)));
        defines.for_each(|(plain, linked)| {
            _ = writeln!(header, "#define {plain} {linked}")
        });
        // The glue names the functions C calls for the lib alone, and the lib.
        let (reset, last, name) = (self.reset(), self.last_trap(), &self.name);
        let (free, free_all) = (self.free(), self.free_all());
        let _ = write!(
            header,
            "#define zk_lib_reset {reset}\n#define zk_lib_last_trap {last}\n#define \
             zk_lib_free {free}\n#define zk_lib_free_all {free_all}\n#define \
             ZK_LIB_NAME \"{name}\"\n#define ZK_LIB 1\n#endif\n"
        );
        let include = format!("#include \"{NAMES_HEADER}\"\n");
        let sources = files.iter_mut().filter(|f| f.name.ends_with(".c"));
        sources.for_each(|f| f.text.insert_str(0, &include));
        files.push(CFile::builder().name(NAMES_HEADER).text(header).build());
    }

    /// The names of C text that link under the prefix of the lib.
    ///
    /// # Arguments
    ///
    /// * `text` - The C text.
    ///
    /// # Returns
    ///
    /// The names, as the text writes them.
    fn used<'a>(&self, text: &'a str) -> impl Iterator<Item = String> + 'a
    {
        let words = text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'));
        let mapped = self.clone();
        words.filter(move |w| mapped.linked(w).is_some()).map(str::to_string)
    }
}

/// The ids of the traps, as the runtime header names them, such as
/// `ZK_TRAP_OVERFLOW`, in the order of their numbers.
///
/// # Returns
///
/// The names.
pub fn trap_names() -> Vec<&'static str>
{
    let header = crate::driver::RUNTIME.iter().find(|(n, _)| *n == "zaka_runtime.h");
    let text = header.map_or("", |(_, t)| t);
    let rows = text.lines().map(str::trim).filter_map(|l| l.strip_suffix(','));
    let ids = rows.filter(|l| l.starts_with("ZK_TRAP_") && !l.contains(' '));
    ids.filter(|l| *l != "ZK_TRAP_COUNT").collect()
}
