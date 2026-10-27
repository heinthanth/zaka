use std::collections::BTreeMap;
use std::fmt::Write;

use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::bc::Func;
use crate::bc::Module;
use crate::bc::Op;
use crate::bc::Slot;

/// The first line of a listing.
pub const LISTING_HEADER: &str = "; zakac asm listing v1";

/// Write a module as a readable asm listing: the globals, the pooled texts,
/// then each function with one op per line. Jump targets become labels
/// `L0`, `L1`, ... numbered per function in code order, and ops that name a
/// slot or a function show that name in a comment.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The listing.
pub fn listing(module: &Module) -> String
{
    let mut out = String::new();
    let _ = writeln!(out, "{LISTING_HEADER}, platform {} bits", module.platform_bits);
    do_if!(
        !module.globals.is_empty(),
        slots(&mut out, "globals", 'g', &module.globals)
    );
    do_if!(!module.texts.is_empty(), texts(&mut out, &module.texts));
    for (i, func) in module.funcs.iter().enumerate()
    {
        out.push('\n');
        function(&mut out, module, i, func);
    }
    out
}

/// Write a table of slots.
///
/// # Arguments
///
/// * `out` - The listing.
/// * `title` - The table title.
/// * `prefix` - The slot prefix, `g` or `l`.
/// * `list` - The slots.
fn slots(out: &mut String, title: &str, prefix: char, list: &[Slot])
{
    let _ = writeln!(out, "{title}");
    for (i, s) in list.iter().enumerate()
    {
        let _ = writeln!(out, "  {prefix}{i} {} {}", s.name, s.kind.name());
    }
}

/// Write the text pool.
///
/// # Arguments
///
/// * `out` - The listing.
/// * `list` - The texts.
fn texts(out: &mut String, list: &[String])
{
    let _ = writeln!(out, "texts");
    for (i, t) in list.iter().enumerate()
    {
        let _ = writeln!(out, "  t{i} {}", quote(t));
    }
}

/// Quote a text, escaping quotes, backslashes and control characters.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The quoted text.
pub fn quote(text: &str) -> String
{
    let mut out = String::from('"');
    for ch in text.chars()
    {
        match ch
        {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&c.escape_unicode().to_string()),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Write one function.
///
/// # Arguments
///
/// * `out` - The listing.
/// * `module` - The module, for the names in comments.
/// * `index` - The function index.
/// * `func` - The function.
fn function(out: &mut String, module: &Module, index: usize, func: &Func)
{
    let ret = func.ret.map(|k| format!(" -> {}", k.name())).unwrap_or_default();
    let params = func.params;
    let _ = writeln!(out, "fn f{index} {} (params {params}){ret}", func.name);
    do_if!(!func.locals.is_empty(), slots(out, "locals", 'l', &func.locals));
    let labels = labels(&func.code);
    for (pc, op) in func.code.iter().enumerate()
    {
        do_if!(let Some(n) = labels.get(&(pc as u32)), { let _ = writeln!(out, "L{n}:"); });
        let text = op_text(*op, &labels);
        let note = comment(*op, module, func);
        let line = note.map_or(text.clone(), |n| format!("{text:<20} ; {n}"));
        let _ = writeln!(out, "  {pc:>4}  {line}");
    }
    let end = func.code.len() as u32;
    do_if!(let Some(n) = labels.get(&end), { let _ = writeln!(out, "L{n}:"); });
}

/// Number the jump targets of a function in code order.
///
/// # Arguments
///
/// * `code` - The ops.
///
/// # Returns
///
/// The label number of each target.
pub fn labels(code: &[Op]) -> BTreeMap<u32, usize>
{
    let mut targets: BTreeMap<u32, usize> = BTreeMap::new();
    do_for!(t in code.iter().filter_map(|op| op.target()), targets.insert(t, 0));
    do_for!((n, v) in targets.values_mut().enumerate(), *v = n);
    targets
}

/// The text of an op, with a label in place of a jump target.
///
/// # Arguments
///
/// * `op` - The op.
/// * `labels` - The label numbers.
///
/// # Returns
///
/// The text.
pub fn op_text(op: Op, labels: &BTreeMap<u32, usize>) -> String
{
    let Some(target) = op.target()
    else
    {
        return op.to_string();
    };
    let label = labels.get(&target).map_or(format!("@{target}"), |n| format!("L{n}"));
    format!("{} {label}", op.name())
}

/// The name an op refers to, for a comment.
///
/// # Arguments
///
/// * `op` - The op.
/// * `module` - The module.
/// * `func` - The function the op is in.
///
/// # Returns
///
/// The name, or `None` for an op that names nothing.
pub fn comment(op: Op, module: &Module, func: &Func) -> Option<String>
{
    let local = |n: u32| func.locals.get(n as usize).map(|s| s.name.clone());
    let global = |n: u32| module.globals.get(n as usize).map(|s| s.name.clone());
    ret_if!(let Op::LocalAddr(n) = op, local(n));
    ret_if!(let Op::GlobalAddr(n) = op, global(n));
    ret_if!(let Op::Call(f) = op, module.funcs.get(f as usize).map(|f| f.name.clone()));
    None
}
