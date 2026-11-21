//! Stops: the trap line with its place, and the backtrace of the calls in
//! progress, as the VM writes them.

use std::ffi::c_char;
use std::ffi::c_void;
use std::io::Write;
use std::mem::ManuallyDrop;
use std::sync::OnceLock;

use strum::EnumIter;
use strum::IntoEnumIterator;
use strum::IntoStaticStr;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::ctx::zk_ctx;
use crate::libs::TrapRecord;
use crate::out::flush_out;
use crate::program::ZkFrame;
use crate::program::prog;
use crate::program::row;
use crate::program::text;
use crate::program::tracks;

/// The traps a program can stop with; the compiler writes the text of each,
/// from its locale, into the table of the program.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, IntoStaticStr, EnumIter)]
#[strum(serialize_all = "snake_case")]
#[allow(missing_docs)]
pub enum ZkTrap
{
    ZeroDivisor,
    ShiftTooFar,
    StepNotPositive,
    Overflow,
    IndexOutOfRange,
    CharOutOfRange,
    EndBeforeStart,
    NullCall,
    NullObject,
    ClosedHandle,
    NullFromC,
    ClosureEscape,
    NoCellToRemove,
    NoCharToRemove,
    NoPlaceToInsert,
    SliceOutOfRange,
    TextCut,
    BadUtf8,
    NotGood,
    NotGoodBare,
    NotFault,
    NoData,
    NoRow,
    NullDeref,
    OutOfMem,
    Overlap,
    TooDeep,
    Permission,
    ThreadTrapped,
    Deadlock,
    PipeClosed,
    /// The head of the line of a trap that ended a thread alone.
    ThreadStopped,
    LockBroken,
    ThreadsNotPositive,
    ThreadWaited,
    WrongVariant,
    /// The stack of a light task ran full.
    StackFull,
    /// The count of ids, the length of a table of them.
    Count,
}

/// The number of each trap, as the runtime passes it around.
pub(crate) mod trap
{
    use super::ZkTrap;

    pub(crate) const ZERO_DIVISOR: i32 = ZkTrap::ZeroDivisor as i32;
    pub(crate) const SHIFT_TOO_FAR: i32 = ZkTrap::ShiftTooFar as i32;
    pub(crate) const STEP_NOT_POSITIVE: i32 = ZkTrap::StepNotPositive as i32;
    pub(crate) const OVERFLOW: i32 = ZkTrap::Overflow as i32;
    pub(crate) const INDEX_OUT_OF_RANGE: i32 = ZkTrap::IndexOutOfRange as i32;
    pub(crate) const CHAR_OUT_OF_RANGE: i32 = ZkTrap::CharOutOfRange as i32;
    pub(crate) const END_BEFORE_START: i32 = ZkTrap::EndBeforeStart as i32;
    pub(crate) const NULL_CALL: i32 = ZkTrap::NullCall as i32;
    pub(crate) const NULL_OBJECT: i32 = ZkTrap::NullObject as i32;
    pub(crate) const CLOSED_HANDLE: i32 = ZkTrap::ClosedHandle as i32;
    pub(crate) const NULL_FROM_C: i32 = ZkTrap::NullFromC as i32;
    pub(crate) const CLOSURE_ESCAPE: i32 = ZkTrap::ClosureEscape as i32;
    pub(crate) const NO_CELL_TO_REMOVE: i32 = ZkTrap::NoCellToRemove as i32;
    pub(crate) const NO_CHAR_TO_REMOVE: i32 = ZkTrap::NoCharToRemove as i32;
    pub(crate) const NO_PLACE_TO_INSERT: i32 = ZkTrap::NoPlaceToInsert as i32;
    pub(crate) const SLICE_OUT_OF_RANGE: i32 = ZkTrap::SliceOutOfRange as i32;
    pub(crate) const TEXT_CUT: i32 = ZkTrap::TextCut as i32;
    pub(crate) const BAD_UTF8: i32 = ZkTrap::BadUtf8 as i32;
    pub(crate) const NOT_GOOD: i32 = ZkTrap::NotGood as i32;
    pub(crate) const NOT_GOOD_BARE: i32 = ZkTrap::NotGoodBare as i32;
    pub(crate) const NOT_FAULT: i32 = ZkTrap::NotFault as i32;
    pub(crate) const NO_DATA: i32 = ZkTrap::NoData as i32;
    pub(crate) const NULL_DEREF: i32 = ZkTrap::NullDeref as i32;
    pub(crate) const OUT_OF_MEM: i32 = ZkTrap::OutOfMem as i32;
    pub(crate) const OVERLAP: i32 = ZkTrap::Overlap as i32;
    pub(crate) const TOO_DEEP: i32 = ZkTrap::TooDeep as i32;
    pub(crate) const PERMISSION: i32 = ZkTrap::Permission as i32;
    pub(crate) const THREAD_TRAPPED: i32 = ZkTrap::ThreadTrapped as i32;
    pub(crate) const DEADLOCK: i32 = ZkTrap::Deadlock as i32;
    pub(crate) const PIPE_CLOSED: i32 = ZkTrap::PipeClosed as i32;
    pub(crate) const THREAD_STOPPED: i32 = ZkTrap::ThreadStopped as i32;
    pub(crate) const LOCK_BROKEN: i32 = ZkTrap::LockBroken as i32;
    pub(crate) const THREADS_NOT_POSITIVE: i32 = ZkTrap::ThreadsNotPositive as i32;
    pub(crate) const THREAD_WAITED: i32 = ZkTrap::ThreadWaited as i32;
    #[cfg(zk_tasks)]
    pub(crate) const STACK_FULL: i32 = ZkTrap::StackFull as i32;
}

/// The most lines of calls a short backtrace shows, as in the VM.
const SHOWN_CALLS: u64 = 16;

/// The name of the executable, without its folder, which leads a trap line.
static NAME: OnceLock<Vec<u8>> = OnceLock::new();

/// Keep the name of the executable for trap lines.
///
/// # Arguments
///
/// * `path` - The path of the executable, as `main` got it.
pub fn set_name(path: &[u8])
{
    let base = path.rsplit(|c| *c == b'/' || *c == b'\\').next().unwrap_or(path);
    let _ = NAME.set(base.to_vec());
}

/// Fill a template: each `%{name}` slot with the value of that name; a slot
/// with no value stands as written.
///
/// # Arguments
///
/// * `tmpl` - The template.
/// * `pairs` - Each name and its value.
///
/// # Returns
///
/// The text.
pub fn fill(tmpl: &[u8], pairs: &[(&str, &[u8])]) -> Vec<u8>
{
    let mut out = Vec::with_capacity(tmpl.len());
    let mut rest = tmpl;
    while !rest.is_empty()
    {
        let slot = rest.strip_prefix(b"%{").and_then(|s| {
            let end = s.iter().position(|c| *c == b'}')?;
            let value = pairs.iter().find(|(n, _)| n.as_bytes() == &s[..end])?.1;
            Some((value, &s[end + 1..]))
        });
        if let Some((value, after)) = slot
        {
            out.extend_from_slice(value);
            rest = after;
            continue;
        }
        out.push(rest[0]);
        rest = &rest[1..];
    }
    out
}

/// The calls in progress that the frame table keeps, main first.
///
/// # Returns
///
/// The frames; blank when the build keeps none.
fn frames() -> &'static [ZkFrame]
{
    // SAFETY: `zk_ctx` made room for the depth limit plus main, and the
    // inline `ZK_ENTER` writes within it.
    unsafe {
        let ctx = &*zk_ctx();
        ret_if!(ctx.frames.is_null(), &[]);
        let top = ctx.depth.min(prog().max_depth);
        std::slice::from_raw_parts(ctx.frames, top as usize + 1)
    }
}

/// The name of the function of a call.
///
/// # Arguments
///
/// * `func` - Its number: 0 for main, n + 1 for row n.
///
/// # Returns
///
/// The name.
fn frame_name(func: u32) -> &'static [u8]
{
    let p = prog();
    ret_if!(func == 0 || func as usize > p.fn_count, b"main");
    // SAFETY: the table holds `fn_count` rows.
    text(unsafe { (*p.fn_names.add(func as usize - 1)).name })
}

/// A site as `<file>:<line>:<column>`, when the build keeps it.
///
/// # Arguments
///
/// * `site` - The index of the site, 0 for none.
///
/// # Returns
///
/// The text, or `None`.
fn place(site: u32) -> Option<Vec<u8>>
{
    let (file, line, col) = site_of(site)?;
    Some([file, format!(":{line}:{col}").into_bytes()].concat())
}

/// The file, the line and the column of a site, when the build keeps them.
///
/// # Arguments
///
/// * `site` - The index of the site, 0 for none.
///
/// # Returns
///
/// The file, the line and the column, or `None`.
fn site_of(site: u32) -> Option<(Vec<u8>, u32, u32)>
{
    let p = prog();
    ret_if!(p.debug_info == 0 || site == 0 || p.sites.is_null(), None);
    // SAFETY: the compiler writes a row for each site it marks.
    let s = unsafe { *p.sites.add(site as usize) };
    Some((row(p.site_files, s.file as usize).to_vec(), s.line, s.col))
}

/// The text of a call with its function, and its site when the build keeps
/// it.
///
/// # Arguments
///
/// * `f` - The call.
/// * `with_site` - The template with a site.
/// * `without` - The template with none.
///
/// # Returns
///
/// The text.
fn call_text(f: ZkFrame, with_site: *const c_char, without: *const c_char) -> Vec<u8>
{
    let name = frame_name(f.func);
    match place(f.site)
    {
        Some(at) => fill(text(with_site), &[("func", name), ("place", &at)]),
        None => fill(text(without), &[("func", name)]),
    }
}

/// Whether a short backtrace leaves a call out: the main function, unless the
/// stop is in it.
///
/// # Arguments
///
/// * `all` - The calls.
/// * `i` - The index of the call.
///
/// # Returns
///
/// `true` when it is left out.
fn hidden(all: &[ZkFrame], i: usize) -> bool { i + 1 != all.len() && all[i].func == 0 }

/// The backtrace of a stop, as much of it as `ZAKA_BACKTRACE` asks: `full`
/// each call, `0` none; else a run of one call once, the first lines, and a
/// last line that tells how to see the rest. A backtrace of one call is left
/// out.
///
/// # Returns
///
/// The text.
fn backtrace() -> Vec<u8>
{
    let mode = std::env::var("ZAKA_BACKTRACE").ok();
    let all = frames();
    ret_if!(!tracks() || mode.as_deref() == Some("0"), Vec::new());
    let full = mode.as_deref() == Some("full");
    let shown = (0..all.len()).filter(|&i| full || !hidden(all, i)).count();
    ret_if!(shown < 2, Vec::new());
    let p = prog();
    let mut out = [text(p.backtrace_text), b"\n"].concat();
    let (mut runs, mut more, mut left_out) = (0u64, 0u64, false);
    let mut i = all.len();
    while i > 0
    {
        i -= 1;
        let f = all[i];
        let skip = !full && hidden(all, i);
        left_out |= skip;
        do_if!(skip, continue);
        let mut n = 1u64;
        while !full && i > 0 && all[i - 1].func == f.func && all[i - 1].site == f.site
        {
            i -= 1;
            n += 1;
        }
        if !full && runs >= SHOWN_CALLS
        {
            more += n;
            continue;
        }
        runs += 1;
        out.extend(call_text(f, p.call_at_text, p.call_text));
        out.push(b'\n');
        do_if!(n > 1, out.extend(count_line(p.call_again_text, n - 1)));
        left_out |= n > 1;
    }
    do_if!(more > 0, out.extend(count_line(p.calls_more_text, more)));
    left_out |= more > 0;
    if !full && left_out
    {
        out.extend(fill(text(p.full_hint_text), &[("var", b"ZAKA_BACKTRACE")]));
        out.push(b'\n');
    }
    out
}

/// A line of a template with one count in it.
///
/// # Arguments
///
/// * `tmpl` - The template, which names the count `n`.
/// * `n` - The count.
///
/// # Returns
///
/// The line.
fn count_line(tmpl: *const c_char, n: u64) -> Vec<u8>
{
    let mut line = fill(text(tmpl), &[("n", n.to_string().as_bytes())]);
    line.push(b'\n');
    line
}

/// Stop the run, as the VM does: write what is buffered, then the trap line
/// `<executable>: the program stopped: <trap>` with its place, then the
/// backtrace, and leave with status 1. In a call of a C lib,
/// leave the call in progress instead.
///
/// # Arguments
///
/// * `trap` - The trap.
/// * `pairs` - Each name of its template and its value.
pub fn stop(trap: i32, pairs: &[(&str, &[u8])]) -> !
{
    flush_out();
    if let Some((jump, thread)) = crate::threads::leave()
    {
        thread_stopped(thread, &reason_text(trap, pairs));
        crate::threads::keep_trap(trap);
        jump(trap);
    }
    let said = stop_text(trap, pairs);
    let Some(leave) = crate::libs::leave()
    else
    {
        exit_with(said)
    };
    leave_call(leave, trap, said)
}

/// Stop the run, as [`stop`] does, with the values its trap line names
/// handed over: a jump back to the wrapper of a C lib skips the drops of each
/// frame it leaves, so the caller keeps nothing that needs one, and these
/// values are left to leak.
///
/// # Arguments
///
/// * `trap` - The trap.
/// * `pairs` - Each name of its template and its value.
pub fn stop_owned(trap: i32, pairs: Vec<(String, Vec<u8>)>) -> !
{
    let pairs = ManuallyDrop::new(pairs);
    let each = pairs.iter().map(|(n, v)| (n.as_str(), v.as_slice()));
    let borrowed: ManuallyDrop<Vec<(&str, &[u8])>> = ManuallyDrop::new(each.collect());
    stop(trap, &borrowed)
}

/// A name of a trap template and its value as text.
///
/// # Arguments
///
/// * `name` - The name.
/// * `value` - The value.
///
/// # Returns
///
/// The pair, owned.
pub fn named(name: &str, value: impl ToString) -> (String, Vec<u8>)
{
    (name.to_string(), value.to_string().into_bytes())
}

/// End the run on a trap: write the trap line after the name of the
/// executable, then the backtrace, and leave with status 1.
///
/// # Arguments
///
/// * `said` - The trap line, with no name before it.
fn exit_with(said: Vec<u8>) -> !
{
    let name = NAME.get().map_or(&b"zaka"[..], Vec::as_slice);
    let mut line = [name, b": "].concat();
    line.extend(said);
    line.push(b'\n');
    line.extend(backtrace());
    let mut err = std::io::stderr().lock();
    let _ = err.write_all(&line);
    let _ = err.flush();
    std::process::exit(1)
}

/// Stop the program with the trap `stack full`, from the handler of a fault on
/// the guard page of a light task: the line is written with no allocation and
/// no lock, since the fault can come inside either, and the program ends.
///
/// # Arguments
///
/// * `size` - The octets of the stack.
#[cfg(zk_tasks)]
pub(crate) fn stack_full_from_signal(size: usize) -> !
{
    let mut digits = [0u8; 20];
    let digits = decimal(size, &mut digits);
    let p = prog();
    let name = NAME.get().map_or(&b"zaka"[..], Vec::as_slice);
    raw_write(name);
    raw_write(b": ");
    let reason = row(p.trap_text, trap::STACK_FULL as usize);
    // The head of the line holds the reason at `%{e}`.
    let stopped = text(p.stopped_text);
    let at = stopped.windows(4).position(|w| w == b"%{e}");
    let (head, tail) =
        at.map_or((stopped, &b""[..]), |i| (&stopped[..i], &stopped[i + 4..]));
    raw_write(head);
    fill_raw(reason, &[(b"size", digits)], &mut |piece| raw_write(piece));
    raw_write(tail);
    raw_write(b"\n");
    end_now()
}

/// End the process at once, with status 1, as a trap does: no handler runs
/// and nothing is freed.
#[cfg(unix)]
fn end_now() -> !
{
    // SAFETY: `_exit` ends the process.
    unsafe { libc::_exit(1) }
}

/// End the process at once, with status 1, as a trap does: no handler runs
/// and nothing is freed.
#[cfg(windows)]
fn end_now() -> !
{
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    use windows_sys::Win32::System::Threading::TerminateProcess;
    // SAFETY: the process ends itself.
    unsafe { TerminateProcess(GetCurrentProcess(), 1) };
    unreachable!("the process ended")
}

/// Write octets to standard error with no allocation.
///
/// # Arguments
///
/// * `octets` - The octets.
#[cfg(unix)]
fn raw_write(octets: &[u8])
{
    // SAFETY: `write` reads `len` octets of the slice.
    let _ = unsafe { libc::write(2, octets.as_ptr().cast(), octets.len()) };
}

/// Write octets to standard error with no allocation.
///
/// # Arguments
///
/// * `octets` - The octets.
#[cfg(windows)]
fn raw_write(octets: &[u8])
{
    use windows_sys::Win32::Storage::FileSystem::WriteFile;
    use windows_sys::Win32::System::Console::GetStdHandle;
    use windows_sys::Win32::System::Console::STD_ERROR_HANDLE;
    let mut wrote = 0u32;
    let len = u32::try_from(octets.len()).unwrap_or(u32::MAX);
    // SAFETY: `WriteFile` reads `len` octets of the slice.
    unsafe {
        let err = GetStdHandle(STD_ERROR_HANDLE);
        WriteFile(err, octets.as_ptr(), len, &mut wrote, std::ptr::null_mut());
    }
}

/// The decimal digits of a number, in a buffer of the caller.
///
/// # Arguments
///
/// * `n` - The number.
/// * `buf` - Room for the digits.
///
/// # Returns
///
/// The digits, at the end of `buf`.
#[cfg(zk_tasks)]
fn decimal(mut n: usize, buf: &mut [u8; 20]) -> &[u8]
{
    let mut at = buf.len();
    loop
    {
        at -= 1;
        buf[at] = b'0' + (n % 10) as u8;
        n /= 10;
        ret_if!(n == 0, &buf[at..]);
    }
}

/// [`fill`] with no allocation: each piece goes to `out` in turn.
///
/// # Arguments
///
/// * `tmpl` - The template.
/// * `pairs` - Each name of its slots and its value.
/// * `out` - Where each piece goes.
#[cfg(zk_tasks)]
fn fill_raw(tmpl: &[u8], pairs: &[(&[u8], &[u8])], out: &mut dyn FnMut(&[u8]))
{
    let mut rest = tmpl;
    while !rest.is_empty()
    {
        let slot = rest.strip_prefix(b"%{").and_then(|s| {
            let end = s.iter().position(|c| *c == b'}')?;
            let value = pairs.iter().find(|(n, _)| *n == &s[..end])?.1;
            Some((value, &s[end + 1..]))
        });
        if let Some((value, after)) = slot
        {
            out(value);
            rest = after;
            continue;
        }
        let next =
            rest[1..].iter().position(|c| *c == b'%').map_or(rest.len(), |i| i + 1);
        out(&rest[..next]);
        rest = &rest[next..];
    }
}

/// The trap line with its place, with no name of an executable before it.
///
/// # Arguments
///
/// * `trap` - The trap.
/// * `pairs` - Each name of its template and its value.
///
/// # Returns
///
/// The line, with no line end.
fn stop_text(trap: i32, pairs: &[(&str, &[u8])]) -> Vec<u8>
{
    let p = prog();
    let reason = fill(row(p.trap_text, trap as usize), pairs);
    let mut line = fill(text(p.stopped_text), &[("e", &reason)]);
    line.extend(place_text());
    line
}

/// The reason of a trap with its place, with no head before it.
///
/// # Arguments
///
/// * `trap` - The trap.
/// * `pairs` - Each name of its template and its value.
///
/// # Returns
///
/// The text.
fn reason_text(trap: i32, pairs: &[(&str, &[u8])]) -> Vec<u8>
{
    let mut line = fill(row(prog().trap_text, trap as usize), pairs);
    line.extend(place_text());
    line
}

/// Write the line of a trap that ended a thread alone under `--trap unwind`:
/// `<executable>: thread <n> stopped: <reason>`; the rest of the program goes
/// on.
///
/// # Arguments
///
/// * `thread` - The number of the thread.
/// * `reason` - The reason with its place.
fn thread_stopped(thread: usize, reason: &[u8])
{
    let name = NAME.get().map_or(&b"zaka"[..], Vec::as_slice);
    let head = fill(
        row(prog().trap_text, trap::THREAD_STOPPED as usize),
        &[("thread", thread.to_string().as_bytes())],
    );
    let mut line = [name, b": ", &head, b": ", reason, b"\n"].concat();
    line.extend(backtrace());
    let mut err = std::io::stderr().lock();
    let _ = err.write_all(&line);
    let _ = err.flush();
}

/// The place of the call in progress, ` (in <f>, at <file>:<line>:<col>)`,
/// when the build keeps the calls.
///
/// # Returns
///
/// The text, blank when the build keeps none.
fn place_text() -> Vec<u8>
{
    let p = prog();
    let mut line = Vec::new();
    if tracks()
        && let Some(f) = frames().last()
    {
        line.push(b' ');
        line.extend(call_text(*f, p.where_text, p.in_text));
    }
    line
}

/// Leave the call of a C lib that stopped on a trap: keep its text, then
/// jump back to the wrapper of the call. Nothing of this frame needs a drop
/// when `leave` runs, since the jump skips the drops of each frame it leaves;
/// each function on the path here hands its values to [`stop_owned`] for the
/// same reason, and holds no lock.
///
/// # Arguments
///
/// * `leave` - The function of the lib that jumps back.
/// * `trap` - The trap.
/// * `said` - The trap line.
fn leave_call(leave: extern "C" fn(i32), trap: i32, said: Vec<u8>) -> !
{
    let kind = ZkTrap::iter().nth(trap as usize).map_or("", <&str>::from);
    // The calls of the lib: those past where its call began, the innermost
    // first; a lib that C called from inside another lib leaves that lib's
    // calls out, since their functions are not its own.
    let all = frames();
    let from = (crate::libs::base() as usize + 1).min(all.len());
    let calls = all[from..].iter().rev();
    let calls = calls.map(|f| (frame_name(f.func).to_vec(), site_of(f.site))).collect();
    let record = TrapRecord::new(trap, kind, said, crate::libs::name(), calls);
    crate::libs::keep_trap(record);
    leave(trap);
    std::process::abort()
}

/// Stop the run: an integer division or remainder had zero as divisor.
#[unsafe(export_name = stamped!("zero_divisor"))]
pub extern "C" fn zk_zero_divisor() -> ! { stop(trap::ZERO_DIVISOR, &[]) }

/// Stop the run: a shift went as far as the width of its value, or further.
///
/// # Arguments
///
/// * `distance` - The shift distance.
/// * `bits` - The bit width of the shifted value.
#[unsafe(export_name = stamped!("shift_too_far"))]
pub extern "C" fn zk_shift_too_far(distance: u64, bits: u32) -> !
{
    stop_owned(
        trap::SHIFT_TOO_FAR,
        vec![named("distance", distance), named("bits", bits)],
    )
}

/// Stop the run: the step of a counted loop was not positive.
#[unsafe(export_name = stamped!("step_not_positive"))]
pub extern "C" fn zk_step_not_positive() -> ! { stop(trap::STEP_NOT_POSITIVE, &[]) }

/// Stop the run: the count of threads of a parallel loop or map is zero or
/// below.
#[unsafe(export_name = stamped!("threads_not_positive"))]
pub extern "C" fn zk_threads_not_positive() -> !
{
    stop(trap::THREADS_NOT_POSITIVE, &[])
}

/// Stop the run: the result of an operation does not fit its kind.
///
/// # Arguments
///
/// * `op` - The operator, such as `+` or `neg`.
/// * `kind` - The Zaka name of the kind.
#[unsafe(export_name = stamped!("overflow"))]
pub extern "C" fn zk_overflow(op: *const c_char, kind: *const c_char) -> !
{
    stop(trap::OVERFLOW, &[("op", text(op)), ("kind", text(kind))])
}

/// Stop the run: more nested calls than the depth limit of the build.
#[unsafe(export_name = stamped!("too_deep"))]
pub extern "C" fn zk_too_deep() -> !
{
    stop_owned(trap::TOO_DEEP, vec![named("depth", prog().max_depth)])
}

/// Stop the run with a trap that names a number and its range.
///
/// # Arguments
///
/// * `trap` - The trap.
/// * `index` - The number as written.
/// * `lo` - The first number of the range.
/// * `hi` - The last number of the range.
#[unsafe(export_name = stamped!("range_trap"))]
pub extern "C" fn zk_range_trap(trap: i32, index: i64, lo: i64, hi: i64) -> !
{
    stop_owned(trap, vec![named("index", index), named("lo", lo), named("hi", hi)])
}

/// Stop the run with a trap that names nothing.
///
/// # Arguments
///
/// * `trap` - The trap.
#[unsafe(export_name = stamped!("plain_trap"))]
pub extern "C" fn zk_plain_trap(trap: i32) -> ! { stop(trap, &[]) }

/// Stop the run with a trap that names numbers: a range of text (start, end)
/// or of cells (len, start, limit).
///
/// # Arguments
///
/// * `trap` - The trap.
/// * `n` - The count of names.
/// * `names` - The names.
/// * `nums` - The numbers.
#[unsafe(export_name = stamped!("nums_trap"))]
pub extern "C" fn zk_nums_trap(
    trap: i32, n: i32, names: *const *const c_char, nums: *const i64,
) -> !
{
    let count = n.max(0) as usize;
    let label = |i: usize| String::from_utf8_lossy(row(names, i)).into_owned();
    // SAFETY: the caller hands `n` names and `n` numbers.
    let pair = |i: usize| named(&label(i), unsafe { *nums.add(i) });
    stop_owned(trap, (0..count).map(pair).collect())
}

/// Stop the run with a trap that names an address.
///
/// # Arguments
///
/// * `trap` - The trap.
/// * `addr` - The address.
#[unsafe(export_name = stamped!("addr_trap"))]
pub extern "C" fn zk_addr_trap(trap: i32, addr: *const c_void) -> !
{
    stop_owned(trap, vec![named("addr", format!("{:#x}", addr as usize))])
}

/// Stop the run with a trap that names a text.
///
/// # Arguments
///
/// * `trap` - The trap.
/// * `e` - The text of the `%{e}` slot.
#[unsafe(export_name = stamped!("text_trap"))]
pub extern "C" fn zk_text_trap(trap: i32, e: *const c_char) -> !
{
    stop(trap, &[("e", text(e))])
}

/// Stop the run: an access through the null address.
///
/// # Arguments
///
/// * `addr` - The address.
#[unsafe(export_name = stamped!("null_deref"))]
pub extern "C" fn zk_null_deref(addr: *const c_void) -> !
{
    zk_addr_trap(trap::NULL_DEREF, addr)
}

/// Stop the run: the heap could not give a block.
///
/// # Arguments
///
/// * `size` - The octets asked for.
#[unsafe(export_name = stamped!("out_of_mem"))]
pub extern "C" fn zk_out_of_mem(size: u64) -> !
{
    stop_owned(trap::OUT_OF_MEM, vec![named("size", size)])
}
