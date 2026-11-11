use std::collections::HashSet;

pub use zakac_ir_builder::value::FALSE_TEXT;
pub use zakac_ir_builder::value::NULL_TEXT;
pub use zakac_ir_builder::value::TRUE_TEXT;
use zakac_ir_builder::value::Value;
pub use zakac_ir_builder::value::burmese_digits;
pub use zakac_ir_builder::value::value_text as text;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::enums::EnumKind;
use zakac_semcheck::enums::MemberValue;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::FALLIBLE;
use zakac_semcheck::structs::StructKind;

use crate::mem::Mem;
use crate::trap::TrapKind;

/// The text a print op writes for a list or a struct, without the line
/// break: the cells of a list between `[` and `]`, the fields of a struct as
/// `name: value` between `(` and `)` after the struct name, each joined with
/// `၊ `; a list or struct inside is written the same.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `addr` - The address of the value.
/// * `kind` - The list or struct kind; another kind gives the text of one
///   value.
///
/// # Returns
///
/// The text, or the trap a read of a cell or field hits.
pub fn list_text(mem: &Mem, addr: u64, kind: DataKind) -> Result<String, TrapKind>
{
    let mut out = String::new();
    let mut tasks = vec![Task::Show { addr, kind, quoted: false }];
    let mut inside = HashSet::new();
    while let Some(task) = tasks.pop()
    {
        match task
        {
            Task::Piece(piece) => out.push_str(&piece),
            Task::Leave(base) => _ = inside.remove(&base),
            Task::Show { addr, kind, quoted } =>
            {
                show(mem, (addr, kind, quoted), &mut tasks, &mut inside)?;
            }
        }
    }
    Ok(out)
}

/// One step of the walk [`list_text`] makes, kept in a list of its own so a
/// long chain of objects takes no stack: the tasks run last first.
enum Task
{
    /// Write this text.
    Piece(String),
    /// Write a value: a cell or a field is `quoted`, so its text reads back.
    Show
    {
        /// The address of the value.
        addr: u64,
        /// Its kind.
        kind: DataKind,
        /// Whether text is written in quotes.
        quoted: bool,
    },
    /// The print has left the object whose fields sit at this address.
    Leave(u64),
}

/// Write one value: a scalar, text, a member or a function value at once,
/// and a list, a struct or an object as tasks for its parts, pushed last
/// first. An object the print is still inside writes `…`.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `value` - The address of the value, its kind, and whether text is written
///   in quotes.
/// * `tasks` - The tasks still to run, added to.
/// * `inside` - The objects the print is inside, added to.
///
/// # Returns
///
/// `Ok`, or the trap a read hits.
fn show(
    mem: &Mem, value: (u64, DataKind, bool), tasks: &mut Vec<Task>,
    inside: &mut HashSet<u64>,
) -> Result<(), TrapKind>
{
    let (addr, kind, quoted) = value;
    let piece = |t: String| Task::Piece(t);
    ret_if!(let Some(e) = kind.as_enum(), Ok(tasks.push(piece(member_text(mem, addr, e)?))));
    if kind.is_fn()
    {
        let v = mem.load(addr, kind)?;
        return Ok(tasks.push(piece(fn_value_text(mem, v))));
    }
    if kind.is_text()
    {
        let t = text_at(mem, addr)?;
        return Ok(tasks.push(piece(tern!(quoted, quoted_text(&t), t))));
    }
    ret_if!(let Some(s) = kind.as_struct(), struct_tasks(mem, addr, s, tasks));
    if let Some(s) = kind.as_object()
    {
        let (base, _) = slice_parts(mem, addr)?;
        ret_if!(base == 0, Ok(tasks.push(piece(NULL_TEXT.to_string()))));
        ret_if!(!inside.insert(base), Ok(tasks.push(piece("…".to_string()))));
        tasks.push(Task::Leave(base));
        return struct_tasks(mem, base, s, tasks);
    }
    if let Some(cell) = kind.as_seq().map(|s| s.cell)
    {
        let (base, count) = slice_parts(mem, addr)?;
        return Ok(cell_tasks(mem, base, cell, count, tasks));
    }
    let Some(list) = kind.as_list()
    else
    {
        return Ok(tasks.push(piece(mem.load(addr, kind).map(text)?)));
    };
    Ok(cell_tasks(mem, addr, list.cell, u64::from(list.len), tasks))
}

/// Push the tasks of a run of cells, between `[` and `]` and joined with
/// `၊ `, last first.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the cells.
/// * `addr` - The address of the first cell.
/// * `cell` - The kind of each cell.
/// * `count` - The number of cells.
/// * `tasks` - The tasks, added to.
fn cell_tasks(mem: &Mem, addr: u64, cell: DataKind, count: u64, tasks: &mut Vec<Task>)
{
    let stride = u64::from(cell.size(mem.platform()));
    tasks.push(Task::Piece("]".to_string()));
    for i in (0..count).rev()
    {
        tasks.push(Task::Show { addr: addr + i * stride, kind: cell, quoted: true });
        do_if!(i > 0, tasks.push(Task::Piece("၊ ".to_string())));
    }
    tasks.push(Task::Piece("[".to_string()));
}

/// The text a print writes for a value: a member of an enumeration as
/// `<stem>များထဲမှ <member>` and a function value as
/// `<name> ဟူသော လုပ်ဆောင်ချက်`, the forms that read back; a list, a struct or
/// text as [`list_text`] writes it; each other value as [`text`] writes it.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `v` - The value, or the address of one that storage holds as a list, a
///   struct or text.
/// * `kind` - Its kind.
///
/// # Returns
///
/// The text, or the trap a read hits.
pub fn shown_text(mem: &Mem, v: Value, kind: DataKind) -> Result<String, TrapKind>
{
    let stored = kind.repr();
    if stored.is_aggregate()
    {
        let reason = "a printed list, struct or text is no address".to_string();
        let addr = v.as_ptr().ok_or(TrapKind::Malformed { reason })?;
        return list_text(mem, addr, kind);
    }
    ret_if!(kind.is_fn(), Ok(fn_value_text(mem, v)));
    let Some(e) = kind.as_enum()
    else
    {
        return Ok(text(v));
    };
    let at = e.find(|m| m.scalar() == Some(v));
    Ok(at.map_or_else(|| text(v), |at| e.text_at(at)))
}

/// The text of a function value: the function it holds, in the form that
/// reads back, or the zero word.
///
/// # Arguments
///
/// * `mem` - The RAM, which knows the function names.
/// * `v` - The value.
///
/// # Returns
///
/// The text.
fn fn_value_text(mem: &Mem, v: Value) -> String
{
    v.as_ptr().map_or_else(|| text(v), |word| mem.fn_text(word))
}

/// The text of a member of an enumeration held at an address: its name, or
/// the value as its value kind prints when no member has it.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `addr` - The address of the value.
/// * `e` - The enumeration.
///
/// # Returns
///
/// The text, or the trap a read hits.
fn member_text(mem: &Mem, addr: u64, e: &EnumKind) -> Result<String, TrapKind>
{
    for (at, (_, v)) in e.members().iter().enumerate()
    {
        ret_if!(holds_value(mem, addr, e.under(), v)?, Ok(e.text_at(at)));
    }
    list_text(mem, addr, e.under())
}

/// Whether the value at an address is a member value.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `addr` - The address of the value.
/// * `kind` - The value kind.
/// * `v` - The member value.
///
/// # Returns
///
/// `true` if it is, or the trap a read hits.
fn holds_value(
    mem: &Mem, addr: u64, kind: DataKind, v: &MemberValue,
) -> Result<bool, TrapKind>
{
    let kind = kind.repr();
    match v
    {
        MemberValue::Scalar(x) => Ok(mem.load(addr, kind)? == *x),
        MemberValue::Text(t) => Ok(text_at(mem, addr)? == *t),
        MemberValue::Items(items) => items_held(mem, addr, kind, items),
    }
}

/// Whether the cells or fields at an address are the items of a member
/// value.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `addr` - The address of the list or the struct.
/// * `kind` - Its kind.
/// * `items` - The cells or fields of the member value.
///
/// # Returns
///
/// `true` when so, or the trap a read hits.
fn items_held(
    mem: &Mem, addr: u64, kind: DataKind, items: &[MemberValue],
) -> Result<bool, TrapKind>
{
    let platform = mem.platform();
    if let Some(s) = kind.as_struct()
    {
        for (at, (v, (_, k))) in items.iter().zip(s.fields()).enumerate()
        {
            let field = addr + u64::from(s.offset(at, platform));
            ret_if!(!holds_value(mem, field, *k, v)?, Ok(false));
        }
        return Ok(true);
    }
    let Some(cell) = kind.cell_kind()
    else
    {
        return Ok(false);
    };
    let (base, count) =
        tern!(kind.is_seq(), slice_parts(mem, addr)?, (addr, items.len() as u64));
    ret_if!(count != items.len() as u64, Ok(false));
    let stride = u64::from(cell.size(platform));
    for (i, v) in items.iter().enumerate()
    {
        ret_if!(!holds_value(mem, base + i as u64 * stride, cell, v)?, Ok(false));
    }
    Ok(true)
}

/// The text of text, owned or a view: its u8s read as UTF-8, a broken
/// sequence as `U+FFFD`.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the text.
/// * `addr` - The address of the text; it starts with the two words of a slice.
///
/// # Returns
///
/// The text, or the trap a read hits.
pub fn text_at(mem: &Mem, addr: u64) -> Result<String, TrapKind>
{
    let (base, count) = slice_parts(mem, addr)?;
    Ok(String::from_utf8_lossy(&mem.octets(base, count)?).into_owned())
}

/// Text in double quotes as a literal writes it: a backslash, a quote and
/// each brace escaped.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The literal.
fn quoted_text(text: &str) -> String
{
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars()
    {
        match c
        {
            '\\' | '"' => out.extend(['\\', c]),
            '{' | '}' => out.extend([c, c]),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The two words of a slice: the address of its first cell and its count.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the slice.
/// * `addr` - The address of the slice.
///
/// # Returns
///
/// The address and the count, or the trap a read hits.
pub fn slice_parts(mem: &Mem, addr: u64) -> Result<(u64, u64), TrapKind>
{
    let word = u64::from(mem.platform() / 8);
    let base = mem.load(addr, DataKind::OPAQUE)?.as_ptr().unwrap_or(0);
    let count = match mem.load(addr + word, DataKind::USIZE)?
    {
        Value::Int { raw, .. } => u64::try_from(raw).unwrap_or(u64::MAX),
        _ => 0,
    };
    Ok((base, count))
}

/// Push the tasks of a struct, last first: `<struct>(name: value၊ ...)`; a
/// result writes `ရလဒ်ကောင်း(<value>)` or `အမှားအယွင်း(<value>)`, and a side
/// with no value writes `အောင်မြင်သည့်သတင်း` or `မအောင်မြင်သည့်သတင်း`. An optional
/// writes `ဒေတာ(<value>)`, or `အလွတ်` when it holds no data.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the struct.
/// * `addr` - The address of the struct.
/// * `s` - The struct kind.
/// * `tasks` - The tasks, added to.
///
/// # Returns
///
/// `Ok`, or the trap a read of the tag of a result hits.
fn struct_tasks(
    mem: &Mem, addr: u64, s: &StructKind, tasks: &mut Vec<Task>,
) -> Result<(), TrapKind>
{
    let platform = mem.platform();
    let at = |i: usize| addr + u64::from(s.offset(i, platform));
    if s.name == FALLIBLE
    {
        let part = |name: &str| s.field(name).map(|(i, kind)| (at(i), kind));
        let (tag_at, tag_kind) =
            part("tag").ok_or(TrapKind::BadValue { addr, kind: s.name.clone() })?;
        // Tag 0, the blank of a result, is a failure.
        let failed = mem.load(tag_at, tag_kind)? == Value::default_of(tag_kind);
        let some = part("some");
        let (word, shown) = match (failed, some)
        {
            (true, None) => ("အမှားအယွင်း", part("err")),
            (false, None) => ("ရလဒ်ကောင်း", part("ok")),
            (true, Some(_)) => ("", None),
            (false, Some(_)) => ("ဒေတာ", some),
        };
        let bare = match (failed, some.is_some())
        {
            (true, true) => "အလွတ်",
            (true, false) => "မအောင်မြင်သည့်သတင်း",
            (false, _) => "အောင်မြင်သည့်သတင်း",
        };
        let Some((addr, kind)) = shown
        else
        {
            tasks.push(Task::Piece(bare.to_string()));
            return Ok(());
        };
        tasks.push(Task::Piece(")".to_string()));
        tasks.push(Task::Show { addr, kind, quoted: true });
        tasks.push(Task::Piece(format!("{word}(")));
        return Ok(());
    }
    tasks.push(Task::Piece(")".to_string()));
    for (i, (name, kind)) in s.fields().iter().enumerate().rev()
    {
        tasks.push(Task::Show { addr: at(i), kind: *kind, quoted: true });
        tasks.push(Task::Piece(tern!(
            i > 0,
            format!("၊ {name}: "),
            format!("{name}: ")
        )));
    }
    tasks.push(Task::Piece(format!("{}(", s.name)));
    Ok(())
}
