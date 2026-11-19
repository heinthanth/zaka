//! The output of a program: a buffer in front of stdout, the sinks that an
//! interpolation writes into, and the print of each scalar.

use std::ffi::c_char;
use std::ffi::c_void;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering;

use zaka_host::numbers::burmese_digits;
use zaka_host::numbers::float_text;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::program::prog;
use crate::program::text;
use crate::seqs::zk_seq_new;
use crate::seqs::zk_seq_put;
use crate::traps::zk_out_of_mem;

/// The room of the output buffer: the VM writes the output of a run in order,
/// and so does this buffer; a stop writes it out before its own line.
const OUT_ROOM: usize = 1 << 16;

/// What waits to go to stdout.
static OUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// A growable run of octets that an interpolation builds; `zk_buf` in C.
#[repr(C)]
pub struct ZkBuf
{
    /// The octets.
    pub data: *mut u8,
    /// The count of octets.
    pub len: usize,
    /// The room.
    pub cap: usize,
    /// The sink before this one, which comes back when it closes.
    pub prev: *mut ZkBuf,
}

/// The sink of an interpolation; NULL while text goes to stdout.
static INTO: AtomicPtr<ZkBuf> = AtomicPtr::new(std::ptr::null_mut());

/// Write out what waits for stdout.
pub fn flush_out()
{
    let Ok(mut out) = OUT.lock()
    else
    {
        return;
    };
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(&out);
    let _ = stdout.flush();
    out.clear();
}

/// Write octets to the sink: stdout, or the buffer of an interpolation.
///
/// # Arguments
///
/// * `octets` - The octets.
pub fn emit(octets: &[u8])
{
    let into = INTO.load(Ordering::Relaxed);
    if !into.is_null()
    {
        // SAFETY: the generated code keeps the buffer alive between
        // `zk_sink_open` and `zk_sink_close`.
        return grow_into(unsafe { &mut *into }, octets);
    }
    let full = {
        let Ok(mut out) = OUT.lock()
        else
        {
            return;
        };
        out.extend_from_slice(octets);
        out.len() >= OUT_ROOM
    };
    do_if!(full, flush_out());
}

/// Add octets to the buffer of an interpolation, doubling its room.
///
/// # Arguments
///
/// * `buf` - The buffer.
/// * `octets` - The octets.
fn grow_into(buf: &mut ZkBuf, octets: &[u8])
{
    let need = buf.len + octets.len();
    if need > buf.cap
    {
        let cap = need.max(2 * buf.cap).max(1);
        // SAFETY: `data` is NULL or a block of the C allocator, which
        // `zk_sink_close` frees.
        let grown = unsafe { libc_realloc(buf.data.cast(), cap) }.cast::<u8>();
        do_if!(grown.is_null(), zk_out_of_mem(cap as u64));
        buf.data = grown;
        buf.cap = cap;
    }
    // SAFETY: the room holds `need` octets.
    unsafe {
        std::ptr::copy_nonoverlapping(
            octets.as_ptr(),
            buf.data.add(buf.len),
            octets.len(),
        )
    };
    buf.len = need;
}

unsafe extern "C" {
    #[link_name = "realloc"]
    fn libc_realloc(p: *mut c_void, size: usize) -> *mut c_void;
    #[link_name = "free"]
    fn libc_free(p: *mut c_void);
}

/// Write octets to the sink.
///
/// # Arguments
///
/// * `s` - The octets.
/// * `n` - Their count.
#[unsafe(export_name = stamped!("emit"))]
pub extern "C" fn zk_emit(s: *const c_void, n: usize)
{
    ret_if!(n == 0);
    // SAFETY: the caller hands `n` readable octets.
    emit(unsafe { std::slice::from_raw_parts(s.cast::<u8>(), n) });
}

/// Send what the print functions write into a buffer.
///
/// # Arguments
///
/// * `buf` - The buffer, made blank here.
#[unsafe(export_name = stamped!("sink_open"))]
pub extern "C" fn zk_sink_open(buf: *mut ZkBuf)
{
    // SAFETY: the generated code hands its own buffer.
    let b = unsafe { &mut *buf };
    b.data = std::ptr::null_mut();
    b.len = 0;
    b.cap = 0;
    b.prev = INTO.load(Ordering::Relaxed);
    INTO.store(buf, Ordering::Relaxed);
}

/// Send what the print functions write back to the sink before, and keep the
/// text of the buffer at a slot, in a heap block of its exact size, as the VM
/// does.
///
/// # Arguments
///
/// * `slot` - The slot of the text: three words.
#[unsafe(export_name = stamped!("sink_close"))]
pub extern "C" fn zk_sink_close(slot: *mut u8)
{
    let buf = INTO.load(Ordering::Relaxed);
    // SAFETY: a close follows its open, so the sink is the open buffer.
    let b = unsafe { &mut *buf };
    INTO.store(b.prev, Ordering::Relaxed);
    let block = match b.len
    {
        0 => std::ptr::null_mut(),
        len => zk_seq_new(len),
    };
    if !block.is_null()
    {
        // SAFETY: the block holds `len` octets.
        unsafe { std::ptr::copy_nonoverlapping(b.data, block, b.len) };
    }
    // SAFETY: `data` came from the C allocator in `grow_into`.
    unsafe { libc_free(b.data.cast()) };
    zk_seq_put(slot, block, b.len, b.len);
}

/// Write octets unchanged.
///
/// # Arguments
///
/// * `s` - The octets.
/// * `n` - Their count; a NUL among them is written too.
#[unsafe(export_name = stamped!("put_raw"))]
pub extern "C" fn zk_put_raw(s: *const c_char, n: usize) { zk_emit(s.cast(), n) }

/// Write the line break that ends each print.
#[unsafe(export_name = stamped!("put_line"))]
pub extern "C" fn zk_put_line() { emit(b"\n") }

/// Write number text, each Western digit swapped for the Burmese digit of the
/// same value.
///
/// # Arguments
///
/// * `s` - The text, NUL at its end.
#[unsafe(export_name = stamped!("put_burmese"))]
pub extern "C" fn zk_put_burmese(s: *const c_char)
{
    emit(burmese_digits(&String::from_utf8_lossy(text(s))).as_bytes())
}

/// Write an unsigned integer.
///
/// # Arguments
///
/// * `v` - The value.
#[unsafe(export_name = stamped!("put_u"))]
pub extern "C" fn zk_put_u(v: u64) { emit(burmese_digits(&v.to_string()).as_bytes()) }

/// Write a signed integer.
///
/// # Arguments
///
/// * `v` - The value.
#[unsafe(export_name = stamped!("put_i"))]
pub extern "C" fn zk_put_i(v: i64) { emit(burmese_digits(&v.to_string()).as_bytes()) }

/// Write a float from its bit pattern, as the VM writes it.
///
/// # Arguments
///
/// * `bits` - The width: 32 or 64.
/// * `raw` - The pattern, in the low `bits` bits.
#[unsafe(export_name = stamped!("put_fbits"))]
pub extern "C" fn zk_put_fbits(bits: i32, raw: u64)
{
    emit(float_text(bits as u16, raw).as_bytes())
}

/// Write an f32.
///
/// # Arguments
///
/// * `v` - The value.
#[unsafe(export_name = stamped!("put_f32"))]
pub extern "C" fn zk_put_f32(v: f32) { zk_put_fbits(32, u64::from(v.to_bits())) }

/// Write an f64.
///
/// # Arguments
///
/// * `v` - The value.
#[unsafe(export_name = stamped!("put_f64"))]
pub extern "C" fn zk_put_f64(v: f64) { zk_put_fbits(64, v.to_bits()) }

/// Write a truth value as its word.
///
/// # Arguments
///
/// * `v` - The value, 0 or 1.
#[unsafe(export_name = stamped!("put_bool"))]
pub extern "C" fn zk_put_bool(v: u8)
{
    let word = match v
    {
        0 => prog().false_text,
        _ => prog().true_text,
    };
    emit(text(word))
}

/// Write a character as its UTF-8 octets.
///
/// # Arguments
///
/// * `c` - The scalar value.
#[unsafe(export_name = stamped!("put_char"))]
pub extern "C" fn zk_put_char(c: u32)
{
    let ch = char::from_u32(c).unwrap_or(char::REPLACEMENT_CHARACTER);
    let mut octets = [0u8; 4];
    emit(ch.encode_utf8(&mut octets).as_bytes())
}

/// The character an integer names as its scalar value.
///
/// # Arguments
///
/// * `negative` - 1 when the integer is below zero.
/// * `v` - The integer, as its unsigned bits.
///
/// # Returns
///
/// The scalar value, or U+FFFD when the integer names none: below zero, past
/// U+10FFFF, or a surrogate.
#[unsafe(export_name = stamped!("char_of"))]
pub extern "C" fn zk_char_of(negative: i32, v: u64) -> u32
{
    let scalar =
        u32::try_from(v).ok().filter(|_| negative == 0).and_then(char::from_u32);
    scalar.map_or(0xfffd, u32::from)
}

/// Send the output to the standard output again, for a reset of a C lib: a
/// trap while text went to a buffer left it there.
pub(crate) fn reset_sink() { INTO.store(std::ptr::null_mut(), Ordering::Relaxed); }
