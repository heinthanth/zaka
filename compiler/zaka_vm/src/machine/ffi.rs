//! The op `foreign`, the call of a C function of a shared lib. The first call
//! opens each lib the program names and finds each C function in it, or in
//! the process itself, which holds libc. A call passes each value in the
//! octets C reads; an address of the VM that lies in a live block becomes the
//! native address of that octet, so C reads and writes the block in place, and
//! the reverse holds for an address C gives back. Another address passes as it
//! is: one C gave earlier, which the VM cannot read through.
//!
//! A build with no feature `ffi`, such as the one in the browser, stops the
//! program at the call instead.

#[cfg(all(feature = "ffi", not(target_arch = "wasm32")))]
pub(crate) use with_ffi::Callbacks;
#[cfg(all(feature = "ffi", not(target_arch = "wasm32")))]
pub(crate) use with_ffi::Linked;
#[cfg(not(all(feature = "ffi", not(target_arch = "wasm32"))))]
pub(crate) use without_ffi::Callbacks;
#[cfg(not(all(feature = "ffi", not(target_arch = "wasm32"))))]
pub(crate) use without_ffi::Linked;
use zakac_codegen_target_vm::bc::ForeignCall;
use zakac_codegen_target_vm::bc::ForeignFn;

use super::Machine;
use super::Step;
use super::malformed;

impl Machine
{
    /// Run the op `foreign`: pop the value of each parameter and other value,
    /// the last on top, and the address of the slot for a struct result under
    /// them; call the C function, and push its result, or the address of the
    /// slot.
    ///
    /// # Arguments
    ///
    /// * `call` - Index into the C calls of the module.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap the call hits.
    pub(super) fn foreign(&mut self, call: u32) -> Step
    {
        let call = self.module.foreign_calls.get(call as usize).cloned();
        let call = call.ok_or_else(|| malformed("foreign names no C call"))?;
        let def = self.module.foreign.get(call.func as usize).cloned();
        let def = def.ok_or_else(|| malformed("foreign names no C function"))?;
        self.call_c(&def, &call)
    }
}

/// The calls, in a build with no feature `ffi`.
#[cfg(not(all(feature = "ffi", not(target_arch = "wasm32"))))]
mod without_ffi
{
    use zakac_semcheck::kind::DataKind;

    use super::*;
    use crate::trap::TrapKind;

    /// Nothing: this build opens no lib.
    #[derive(Debug)]
    pub(crate) struct Linked;

    /// Nothing: this build gives C no function to call back.
    #[derive(Debug, Clone, Default)]
    pub(crate) struct Callbacks;

    impl Machine
    {
        /// Stop the program: this build cannot call C.
        ///
        /// # Arguments
        ///
        /// * `def` - The C function.
        /// * `_call` - The call.
        ///
        /// # Returns
        ///
        /// The trap `ForeignCall`.
        pub(super) fn call_c(&mut self, def: &ForeignFn, _call: &ForeignCall) -> Step
        {
            Err(TrapKind::ForeignCall { name: def.name.clone() })
        }

        /// Pop the function value of a leave: this build gives C nothing to
        /// call, so the leave changes nothing.
        ///
        /// # Arguments
        ///
        /// * `_kind` - The function kind.
        ///
        /// # Returns
        ///
        /// `Ok`, or the trap the pop hits.
        pub(in crate::machine) fn c_leave(&mut self, _kind: DataKind) -> Step
        {
            self.pop().map(drop)
        }

        /// Pop the function value of the end of a leave, which changes
        /// nothing in this build.
        ///
        /// # Returns
        ///
        /// `Ok`, or the trap the pop hits.
        pub(in crate::machine) fn c_revoke(&mut self) -> Step { self.pop().map(drop) }

        /// Stop the program: this build holds no struct of C to read.
        ///
        /// # Arguments
        ///
        /// * `_kind` - The kind of the field.
        /// * `_offset` - Its offset.
        ///
        /// # Returns
        ///
        /// The trap.
        pub(in crate::machine) fn c_load(
            &mut self, _kind: DataKind, _offset: u32,
        ) -> Step
        {
            Err(malformed("this build reads no struct of C"))
        }

        /// Stop the program: this build holds no struct of C to write.
        ///
        /// # Arguments
        ///
        /// * `_kind` - The kind of the field.
        /// * `_offset` - Its offset.
        ///
        /// # Returns
        ///
        /// The trap.
        pub(in crate::machine) fn c_store(
            &mut self, _kind: DataKind, _offset: u32,
        ) -> Step
        {
            Err(malformed("this build writes no struct of C"))
        }

        /// End each leave at the end of the run: this build gives none.
        ///
        /// # Returns
        ///
        /// `Ok`.
        pub(in crate::machine) fn end_leaves(&mut self) -> Step { Ok(()) }
    }
}

/// The calls, through `libloading` and `libffi`.
#[cfg(all(feature = "ffi", not(target_arch = "wasm32")))]
mod with_ffi
{
    use std::cell::Cell as Shared;
    use std::collections::HashMap;
    use std::ffi::c_char;
    use std::ffi::c_void;
    use std::fmt;
    use std::sync::Arc;
    use std::thread::ThreadId;

    use libffi::low::ffi_cif;
    use libffi::middle::Arg;
    use libffi::middle::Cif;
    use libffi::middle::Closure;
    use libffi::middle::CodePtr;
    use libffi::middle::Ret;
    use libffi::middle::Type;
    use libloading::Library;
    use zakac_codegen_target_vm::bc::Module;
    use zakac_i18n::t;
    use zakac_ir_builder::ir::fn_shape;
    use zakac_ir_builder::value::Value;
    use zakac_ir_builder::value::from_raw;
    use zakac_ir_builder::value::to_raw;
    use zakac_rust_sugars::do_for;
    use zakac_rust_sugars::do_if;
    use zakac_rust_sugars::ret_if;
    use zakac_rust_sugars::tern;
    use zakac_semcheck::fn_kind::FnKind;
    use zakac_semcheck::foreign::is_c_handle_result;
    use zakac_semcheck::foreign::is_c_text_result;
    use zakac_semcheck::foreign::is_lib_path;
    use zakac_semcheck::kind::DataKind;
    use zakac_semcheck::kind::IntKind;
    use zakac_semcheck::kind::Width;

    use super::*;
    use crate::mem::Origin;
    use crate::trap::TrapKind;

    /// The libs a run opened, and the native address of each C function of
    /// the module, in its order.
    pub(crate) struct Linked
    {
        /// The libs, kept open while the addresses are in use.
        _libs: Vec<Library>,
        /// The address of each C function.
        addrs: Vec<usize>,
        /// libc's `fflush`, when the process holds it.
        flush: Option<usize>,
        /// libc's `free`, for the strings C hands over, when the process
        /// holds it.
        free: Option<usize>,
    }

    impl fmt::Debug for Linked
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            f.debug_struct("Linked").field("addrs", &self.addrs).finish()
        }
    }

    /// The functions of the program C can call back: a libffi closure for
    /// each function and each closure a call gave C, kept for the whole run,
    /// since a lib can keep the address and call it later. The code of a
    /// closure remains in place after its last use, so a late call from C stops
    /// the program instead of running freed code. A trap inside a callback
    /// cannot cross C, so it waits here until C returns.
    #[derive(Default)]
    pub(crate) struct Callbacks
    {
        /// The libffi closure of each function, under its index.
        made: HashMap<u32, Callback>,
        /// The libffi closure of each closure, under the octets of its
        /// record: two values with the same function and the same captures
        /// are one closure.
        closures: HashMap<Vec<u8>, Callback>,
        /// The closures lent to the C calls that run now, the latest last.
        lent: Vec<&'static CallbackData>,
        /// The trap a callback hit, raised once C returns.
        trap: Option<TrapKind>,
    }

    // SAFETY: a closure runs inside a call of C that the machine makes, on
    // the thread that runs the machine, and the machine moves to another
    // thread between runs alone, when no call of C is in progress.
    unsafe impl Send for Callbacks {}
    // SAFETY: as above; nothing reads the closures through a shared machine.
    unsafe impl Sync for Callbacks {}

    impl Clone for Callbacks
    {
        /// A duplicate of a machine makes its own closures when it calls C,
        /// since each closure runs the machine that made it.
        fn clone(&self) -> Self
        {
            let trap = self.trap.clone();
            Callbacks { trap, ..Callbacks::default() }
        }
    }

    impl fmt::Debug for Callbacks
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            f.debug_struct("Callbacks").field("trap", &self.trap).finish()
        }
    }

    /// A function of the program that C can call.
    struct Callback
    {
        /// The libffi closure. It is never freed, as its data is leaked: a lib
        /// can keep the address past the run, and freeing libffi closures as
        /// the VM drops crashed on macOS now and then. The C target keeps its
        /// closures for the whole run too.
        _closure: std::mem::ManuallyDrop<Closure<'static>>,
        /// What the closure needs when C calls it.
        data: &'static CallbackData,
        /// The native address C calls.
        code: usize,
    }

    /// What a closure needs when C calls it.
    struct CallbackData
    {
        /// The machine that runs the function, set at each call of C that
        /// gives the function.
        machine: Shared<*mut Machine>,
        /// The index of the function.
        func: u32,
        /// The parameter kinds.
        params: Vec<DataKind>,
        /// The result kind.
        ret: Option<DataKind>,
        /// The record a closure passes its thunk last; 0 for a function.
        record: Shared<u64>,
        /// The C calls that run now and got the closure.
        calls: Shared<u32>,
        /// Whether C has leave to call the closure after those calls.
        kept: Shared<bool>,
        /// The thread that runs the VM.
        thread: ThreadId,
    }

    impl CallbackData
    {
        /// Whether C can call the function now: a function at all times, a
        /// closure while a C call that got it runs or while C has leave.
        ///
        /// # Returns
        ///
        /// `true` when it can.
        fn live(&self) -> bool
        {
            self.record.get() == 0 || self.kept.get() || self.calls.get() > 0
        }
    }

    /// Octets aligned for each kind C passes, the storage of an argument or a
    /// result.
    struct Cell(Vec<u128>);

    impl Cell
    {
        /// A cell of zeros with room for `len` octets, and at least 16, which
        /// libffi needs for a small result.
        fn zeros(len: usize) -> Cell { Cell(vec![0; len.max(16).div_ceil(16)]) }

        /// A cell that holds these octets.
        fn of(raw: &[u8]) -> Cell
        {
            let mut cell = Cell::zeros(raw.len());
            cell.octets_mut()[..raw.len()].copy_from_slice(raw);
            cell
        }

        /// The octets.
        fn octets(&self) -> &[u8]
        {
            // SAFETY: a u128 is 16 octets, with no padding.
            unsafe {
                std::slice::from_raw_parts(self.0.as_ptr().cast(), self.0.len() * 16)
            }
        }

        /// The octets, to write.
        fn octets_mut(&mut self) -> &mut [u8]
        {
            // SAFETY: a u128 is 16 octets, with no padding.
            unsafe {
                std::slice::from_raw_parts_mut(
                    self.0.as_mut_ptr().cast(),
                    self.0.len() * 16,
                )
            }
        }
    }

    impl Machine
    {
        /// Call a C function: link the C functions on the first call, read
        /// the arguments from the stack, call, and push the result.
        ///
        /// # Arguments
        ///
        /// * `def` - The C function.
        /// * `func` - Its index.
        ///
        /// # Returns
        ///
        /// `Ok`, or the trap the call hits.
        pub(super) fn call_c(&mut self, def: &ForeignFn, call: &ForeignCall) -> Step
        {
            let linked = self.linked()?;
            let platform = self.module.platform_bits;
            let kinds: Vec<DataKind> =
                def.params.iter().chain(&call.extra).copied().collect();
            let mut values =
                kinds.iter().map(|_| self.pop()).collect::<Result<Vec<_>, _>>()?;
            values.reverse();
            let blob = def.ret.filter(|k| k.is_aggregate());
            let slot = blob.map(|_| self.pop_ptr("foreign")).transpose()?;
            // The zero-ended duplicates of the texts, which live until the call
            // ends.
            let mut texts = Vec::new();
            let lent_from = self.callbacks.lent.len();
            let pairs = values.iter().zip(&kinds);
            let cells = pairs.map(|(v, k)| self.arg_cell(*v, *k, &mut texts));
            let cells = cells.collect::<Result<Vec<Cell>, _>>();
            let cells = cells.inspect_err(|_| self.end_lends(lent_from))?;
            let ffi_kinds = kinds.iter().map(|k| ffi_kind(*k, platform));
            let text_ret = def.ret.filter(|k| is_c_text_result(*k));
            let ret_kind = match def.ret
            {
                // C gives the string as an address.
                _ if text_ret.is_some() => Type::pointer(),
                Some(k) => ffi_kind(k, platform),
                None => Type::void(),
            };
            let fixed = def.params.len();
            let cif = tern!(
                def.variadic,
                Cif::new_variadic(ffi_kinds, fixed, ret_kind),
                Cif::new(ffi_kinds, ret_kind)
            );
            let size = def.ret.map_or(0, |k| k.size(platform) as usize);
            let mut ret = Cell::zeros(size);
            let args: Vec<Arg> =
                cells.iter().map(|c| Arg::new(c.0.as_slice())).collect();
            let code =
                CodePtr::from_ptr(linked.addrs[call.func as usize] as *const c_void);
            // SAFETY: the kinds of the call are the ones the program declares
            // the C function with; C itself must keep to them, as in C.
            unsafe {
                cif.call_return_into(code, &args, Ret::new(ret.0.as_mut_slice()))
            };
            self.end_lends(lent_from);
            flush_c(&linked);
            ret_if!(let Some(trap) = self.callbacks.trap.take(), Err(trap));
            if let (Some(kind), Some(slot)) = (text_ret, slot)
            {
                let native = read_word(ret.octets(), 0);
                let made = self.text_from_c(kind, slot, native);
                // A string C hands over is freed once its octets are copied.
                do_if!(def.owned_ret && native != 0, free_c(&linked, native));
                made?;
                return self.push(Value::Ptr(slot));
            }
            let Some(kind) = def.ret
            else
            {
                return Ok(());
            };
            let mut raw = ret.octets()[..size].to_vec();
            self.to_vm(kind, &mut raw, 0);
            let Some(slot) = slot
            else
            {
                let v = from_raw(kind, &raw, platform);
                return self
                    .push(v.map_err(|_| malformed("C gave a value of no kind"))?);
            };
            self.mem.write_octets(slot, &raw)?;
            self.push(Value::Ptr(slot))
        }

        /// Write the result of a C function that gives back text: a failure
        /// for NULL, else text of its own with a duplicate of the zero-ended
        /// octets C gave, which C keeps.
        ///
        /// # Arguments
        ///
        /// * `kind` - The result kind, an optional of text.
        /// * `slot` - The address of the result.
        /// * `native` - The address C gave back.
        ///
        /// # Returns
        ///
        /// `Ok`, or `BadUtf8` for octets that are not UTF-8, or the trap a
        /// write hits.
        fn text_from_c(&mut self, kind: DataKind, slot: u64, native: usize) -> Step
        {
            let platform = self.module.platform_bits;
            let blank = vec![0; kind.size(platform) as usize];
            self.mem.write_octets(slot, &blank)?;
            ret_if!(native == 0, Ok(()));
            // SAFETY: C gives back a string that ends in a zero octet, as the
            // program declares; C itself must keep to that, as in C.
            let octets = unsafe { std::ffi::CStr::from_ptr(native as *const c_char) };
            let text = std::str::from_utf8(octets.to_bytes()).map_err(|e| {
                TrapKind::BadUtf8 { addr: (native + e.valid_up_to()) as u64 }
            })?;
            let s =
                kind.as_struct().ok_or_else(|| malformed("a result is a struct"))?;
            let tag = IntKind::new(false, Width::Bits(8));
            self.mem.store(slot, Value::int(tag, 1, platform))?;
            self.place_text(slot + u64::from(s.offset(1, platform)), text)
        }

        /// The linked C functions, linked on the first call.
        ///
        /// # Returns
        ///
        /// The C functions, or the trap `ForeignLink`.
        fn linked(&mut self) -> Result<Arc<Linked>, TrapKind>
        {
            ret_if!(let Some(l) = &self.linked, Ok(l.clone()));
            let linked = Arc::new(link(&self.module)?);
            self.linked = Some(linked.clone());
            Ok(linked)
        }

        /// The octets of an argument: a scalar as it is, an address made
        /// native, the octets of a struct, read from the address on the
        /// stack, with each address in it made native, and for text the
        /// native address of a duplicate of its octets that ends in a zero
        /// octet.
        ///
        /// # Arguments
        ///
        /// * `v` - The value on the stack.
        /// * `kind` - The kind of the parameter.
        /// * `texts` - The duplicates of texts, kept until the call ends.
        ///
        /// # Returns
        ///
        /// The cell, or the trap a read hits.
        fn arg_cell(
            &mut self, v: Value, kind: DataKind, texts: &mut Vec<Vec<u8>>,
        ) -> Result<Cell, TrapKind>
        {
            let platform = self.module.platform_bits;
            ret_if!(
                !suits(v, kind),
                Err(malformed(&format!("foreign takes no {v:?} for {}", kind.name())))
            );
            if let Some(f) = kind.as_fn()
            {
                let native = self.callback(v, f)?;
                return Ok(Cell::of(&native.to_le_bytes()));
            }
            if kind.is_text()
            {
                let addr =
                    v.as_ptr().ok_or_else(|| malformed("foreign needs an address"))?;
                let mut dup = self.text_octets(addr)?;
                dup.push(0);
                let native = dup.as_ptr() as usize;
                texts.push(dup);
                return Ok(Cell::of(&native.to_le_bytes()));
            }
            if kind.is_aggregate()
            {
                let addr =
                    v.as_ptr().ok_or_else(|| malformed("foreign needs an address"))?;
                let mut raw = self.mem.octets(addr, u64::from(kind.size(platform)))?;
                self.write_native(kind, &mut raw, 0)?;
                return Ok(Cell::of(&raw));
            }
            let mut raw = to_raw(v, platform);
            self.write_native(kind, &mut raw, 0)?;
            Ok(Cell::of(&raw))
        }

        /// The native address C calls for a function value: none for the zero
        /// word, and a libffi closure that runs the function in this
        /// machine, made on the first call that gives it. A closure is
        /// lent to the call: C can call it until the call returns, and
        /// later while C has leave.
        ///
        /// # Arguments
        ///
        /// * `v` - The function value.
        /// * `f` - Its kind.
        ///
        /// # Returns
        ///
        /// The address, or the trap a read of the record hits.
        fn callback(&mut self, v: Value, f: &FnKind) -> Result<usize, TrapKind>
        {
            let word =
                v.as_ptr().ok_or_else(|| malformed("foreign needs a function"))?;
            ret_if!(word == 0, Ok(0));
            if let Some((record, kind)) = self.record_of(word)?
            {
                let (data, code) = self.closure_callback(record, kind, f)?;
                data.calls.set(data.calls.get() + 1);
                do_if!(!data.kept.get(), data.record.set(record));
                self.callbacks.lent.push(data);
                return Ok(code);
            }
            let func = u32::try_from(word - 1).map_err(|_| malformed("no function"))?;
            let me: *mut Machine = self;
            if let Some(made) = self.callbacks.made.get(&func)
            {
                made.data.machine.set(me);
                return Ok(made.code);
            }
            let made = self.new_callback(func, 0, f);
            let code = made.code;
            self.callbacks.made.insert(func, made);
            Ok(code)
        }

        /// The libffi closure of a closure, made on its first use: one for
        /// each function and set of captures.
        ///
        /// # Arguments
        ///
        /// * `record` - The record of the closure.
        /// * `kind` - The kind of the record.
        /// * `f` - The function kind.
        ///
        /// # Returns
        ///
        /// What the libffi closure reads and the native address, or the trap
        /// a read of the record hits.
        fn closure_callback(
            &mut self, record: u64, kind: DataKind, f: &FnKind,
        ) -> Result<(&'static CallbackData, usize), TrapKind>
        {
            let size = u64::from(kind.size(self.module.platform_bits));
            let raw = self.mem.octets(record, size)?;
            let me: *mut Machine = self;
            if let Some(made) = self.callbacks.closures.get(&raw)
            {
                made.data.machine.set(me);
                return Ok((made.data, made.code));
            }
            let thunk = self.mem.thunk_word(record)?.saturating_sub(1);
            let func = u32::try_from(thunk).map_err(|_| malformed("no thunk"))?;
            let made = self.new_callback(func, record, f);
            let found = (made.data, made.code);
            self.callbacks.closures.insert(raw, made);
            Ok(found)
        }

        /// A libffi closure that runs a function of the program in this
        /// machine; its data lives for the run.
        ///
        /// # Arguments
        ///
        /// * `func` - The function, or the thunk of a closure.
        /// * `record` - The record of a closure, 0 for a function.
        /// * `f` - The function kind.
        ///
        /// # Returns
        ///
        /// The closure.
        fn new_callback(&mut self, func: u32, record: u64, f: &FnKind) -> Callback
        {
            let me: *mut Machine = self;
            let platform = self.module.platform_bits;
            let (params, ret) = fn_shape(f);
            let shapes = params.iter().map(|k| ffi_kind(*k, platform));
            let cif = Cif::new(
                shapes,
                ret.map_or_else(Type::void, |k| ffi_kind(k, platform)),
            );
            #[rustfmt::skip]
            let data = CallbackData {
                machine: Shared::new(me), func, params, ret, record: Shared::new(record),
                calls: Shared::new(0), kept: Shared::new(false),
                thread: std::thread::current().id(),
            };
            let data: &'static CallbackData = Box::leak(Box::new(data));
            let closure = Closure::new(cif, call_back, data);
            let code = *closure.code_ptr() as usize;
            Callback { _closure: std::mem::ManuallyDrop::new(closure), data, code }
        }

        /// End the lends of the closures a C call got, once it returns.
        ///
        /// # Arguments
        ///
        /// * `from` - The count of lends that ran before the call.
        fn end_lends(&mut self, from: usize)
        {
            let ended = self.callbacks.lent.drain(from..);
            ended.for_each(|d| d.calls.set(d.calls.get().saturating_sub(1)));
        }

        /// Run the op `c_leave`: pop a function value and give C leave to
        /// call it after the call that gave it. The leave holds a count of
        /// the record, so its captures live while C can call it. A function
        /// with no captures, the zero word and a closure with leave are left
        /// alone.
        ///
        /// # Arguments
        ///
        /// * `kind` - The function kind.
        ///
        /// # Returns
        ///
        /// `Ok`, or the trap a read of the record hits.
        pub(in crate::machine) fn c_leave(&mut self, kind: DataKind) -> Step
        {
            let word = self.pop_ptr("c_leave")?;
            let Some((record, rkind)) = self.record_of(word)?
            else
            {
                return Ok(());
            };
            let f =
                kind.as_fn().ok_or_else(|| malformed("c_leave needs a function"))?;
            let (data, _) = self.closure_callback(record, rkind, f)?;
            ret_if!(data.kept.get(), Ok(()));
            self.mem.retain(record)?;
            data.kept.set(true);
            data.record.set(record);
            Ok(())
        }

        /// Run the op `c_load`: read a field of a struct C keeps, at the
        /// address a handle holds plus an offset. A number or a boolean is
        /// pushed; a counted list is copied into the block under the address,
        /// and so is the string a `char *` field points at, as text or a
        /// optional with no data for NULL.
        ///
        /// # Arguments
        ///
        /// * `kind` - The kind of the field.
        /// * `offset` - Its offset.
        ///
        /// # Returns
        ///
        /// `Ok`, or the trap a write of the block hits.
        pub(in crate::machine) fn c_load(&mut self, kind: DataKind, offset: u32)
        -> Step
        {
            let base = self.pop_ptr("c_load")?;
            let native = (base as usize).wrapping_add(offset as usize);
            if is_c_text_result(kind)
            {
                let slot = self.pop_ptr("c_load")?;
                // SAFETY: the declaration gives the field a `char *`, and the
                // handle is open, as the guard before the read checks.
                let string =
                    unsafe { std::ptr::read_unaligned(native as *const usize) };
                self.text_from_c(kind, slot, string)?;
                return self.push(Value::Ptr(slot));
            }
            let size = kind.size(self.module.platform_bits) as usize;
            // SAFETY: the declaration names the fields of the C struct from
            // its start, and the handle is open, as the guard checks.
            let raw = unsafe { std::slice::from_raw_parts(native as *const u8, size) }
                .to_vec();
            if kind.is_aggregate()
            {
                let slot = self.pop_ptr("c_load")?;
                self.mem.write_octets(slot, &raw)?;
                return self.push(Value::Ptr(slot));
            }
            let platform = self.module.platform_bits;
            let value = from_raw(kind, &raw, platform);
            self.push(value.map_err(|_| malformed("C gave a value of no kind"))?)
        }

        /// Run the op `c_store`: write a field of a struct C keeps, at the
        /// address a handle holds plus an offset, from a number or a boolean
        /// on the stack, or from the block a counted list holds.
        ///
        /// # Arguments
        ///
        /// * `kind` - The kind of the field.
        /// * `offset` - Its offset.
        ///
        /// # Returns
        ///
        /// `Ok`, or the trap a read of the list hits.
        pub(in crate::machine) fn c_store(
            &mut self, kind: DataKind, offset: u32,
        ) -> Step
        {
            let platform = self.module.platform_bits;
            let size = kind.size(platform) as usize;
            let value = self.pop()?;
            let raw = match kind.is_aggregate()
            {
                true =>
                {
                    let at = value
                        .as_ptr()
                        .ok_or_else(|| malformed("c_store needs a list"))?;
                    self.mem.octets(at, size as u64)?
                }
                false => to_raw(value, platform),
            };
            let base = self.pop_ptr("c_store")?;
            let native = (base as usize).wrapping_add(offset as usize);
            let n = raw.len().min(size);
            // SAFETY: the declaration names the fields of the C struct from
            // its start, the handle is open, as the guard checks, and unsafe
            // code writes the field.
            unsafe {
                std::ptr::copy_nonoverlapping(raw.as_ptr(), native as *mut u8, n)
            };
            Ok(())
        }

        /// End each leave left when the run ends, as `c_revoke` would: C can
        /// call nothing more, and the records the leaves hold go back.
        ///
        /// # Returns
        ///
        /// `Ok`, or the trap a read or a free hits.
        pub(in crate::machine) fn end_leaves(&mut self) -> Step
        {
            let held = self.callbacks.closures.values().map(|made| made.data);
            let kept: Vec<&'static CallbackData> =
                held.filter(|d| d.kept.get()).collect();
            for data in kept
            {
                data.kept.set(false);
                let record = data.record.get();
                let kind = self.record_of(record)?.map(|(_, k)| k);
                let kind = kind.ok_or_else(|| malformed("a leave holds no record"))?;
                self.release_record(record, kind)?;
            }
            Ok(())
        }

        /// Run the op `c_revoke`: pop a function value and end the leave
        /// `c_leave` gave it, letting go of the count of its record. A value
        /// with no leave is left alone.
        ///
        /// # Returns
        ///
        /// `Ok`, or the trap a read or a free hits.
        pub(in crate::machine) fn c_revoke(&mut self) -> Step
        {
            let word = self.pop_ptr("c_revoke")?;
            let Some((record, kind)) = self.record_of(word)?
            else
            {
                return Ok(());
            };
            let size = u64::from(kind.size(self.module.platform_bits));
            let raw = self.mem.octets(record, size)?;
            let data = self.callbacks.closures.get(&raw).map(|made| made.data);
            let Some(data) = data.filter(|d| d.kept.get())
            else
            {
                return Ok(());
            };
            data.kept.set(false);
            self.release_record(data.record.get(), kind)
        }

        /// Run a function of the program for C, to its return: push the
        /// arguments, enter the function, and run its ops until it leaves.
        /// A trap waits in [`Callbacks`] and the frames remain, so the trap
        /// names the function it hit; each later callback of the C call
        /// gives a zero.
        ///
        /// # Arguments
        ///
        /// * `func` - The index of the function.
        /// * `args` - The arguments.
        /// * `gives` - Whether it gives a result.
        ///
        /// # Returns
        ///
        /// The result, or `None`.
        fn run_callback(
            &mut self, func: u32, args: Vec<Value>, gives: bool,
        ) -> Option<Value>
        {
            ret_if!(self.callbacks.trap.is_some(), None);
            let depth = self.frames.len();
            do_for!(v in args, self.stack.push(v));
            let mut ran = self.enter(func);
            while ran.is_ok() && self.frames.len() > depth
            {
                ran = self.fetch();
            }
            if let Err(trap) = ran
            {
                self.callbacks.trap = Some(trap);
                return None;
            }
            tern!(gives, self.stack.pop(), None)
        }

        /// The value of an argument C passes to a callback, from its native
        /// octets. A number, a boolean or an address is read as it is. A
        /// struct, text and a handle become a value of the VM in a block of
        /// its own; the argument is the address of a lent duplicate of it, as
        /// each call passes, so a drop in the callback frees nothing. Both
        /// blocks are noted in `temps`, the duplicate as `ptr`, and dropped
        /// once the callback returns: a `char *` is
        /// copied into text, and a struct address C keeps becomes a new
        /// handle. NULL is an optional with no data, a handle that reads as
        /// closed, or, for plain text, `NullFromC`.
        ///
        /// # Arguments
        ///
        /// * `kind` - The parameter kind.
        /// * `at` - The address of the argument.
        /// * `temps` - The blocks made for the arguments, added to.
        ///
        /// # Returns
        ///
        /// The value, or the trap the read hits.
        fn callback_arg(
            &mut self, kind: DataKind, at: *const u8, temps: &mut Vec<(u64, DataKind)>,
        ) -> Result<Value, TrapKind>
        {
            let platform = self.module.platform_bits;
            let size = kind.size(platform) as usize;
            let word = size_of::<usize>();
            // A result and an optional are structs of Zaka, never C structs.
            let plain = kind.is_struct() && kind.fallible_parts().is_none();
            let made = plain || kind.is_text() || kind.as_external().is_some();
            // SAFETY: libffi passes the address of the argument, of its kind;
            // a pointer argument is one word.
            let read = |n: usize| unsafe { std::slice::from_raw_parts(at, n) }.to_vec();
            if !made && !is_c_text_result(kind) && !is_c_handle_result(kind)
            {
                let mut raw = read(size);
                self.to_vm(kind, &mut raw, 0);
                return from_raw(kind, &raw, platform)
                    .map_err(|_| malformed("C gave a value of no kind"));
            }
            let raw = read(tern!(plain, size, word));
            let slot = self.mem.alloc(size as u64, Origin::Heap, 0);
            let slot = slot.ok_or(TrapKind::OutOfMem { size: size as u64 })?;
            temps.push((slot, kind));
            let native = read_word(&raw, 0);
            match kind
            {
                _ if plain =>
                {
                    let mut raw = raw;
                    self.to_vm(kind, &mut raw, 0);
                    self.mem.write_octets(slot, &raw)?;
                }
                _ if is_c_text_result(kind) => self.text_from_c(kind, slot, native)?,
                _ if kind.is_text() => self.text_arg(slot, native)?,
                _ if kind.as_external().is_some() =>
                {
                    self.handle_arg(slot, kind, native)?
                }
                _ => self.optional_handle_arg(slot, kind, native)?,
            }
            // The callback gets a lent duplicate, as each call does, so its
            // drop frees nothing; the slot above keeps the blocks.
            let lent = self.mem.alloc(size as u64, Origin::Heap, 0);
            let lent = lent.ok_or(TrapKind::OutOfMem { size: size as u64 })?;
            temps.push((lent, DataKind::OPAQUE));
            self.mem.blit(lent, slot, kind)?;
            self.lend_value(lent, kind)?;
            Ok(Value::Ptr(lent))
        }

        /// Write the text a `char *` C passed into a slot; NULL stops the run.
        ///
        /// # Arguments
        ///
        /// * `slot` - Where the text goes.
        /// * `native` - The address C passed.
        ///
        /// # Returns
        ///
        /// `Ok`, `NullFromC`, or `BadUtf8` for octets that are not UTF-8.
        fn text_arg(&mut self, slot: u64, native: usize) -> Step
        {
            ret_if!(native == 0, Err(TrapKind::NullFromC));
            // SAFETY: C passes a string that ends in a zero octet, as the
            // function kind declares; C itself must keep to that, as in C.
            let octets = unsafe { std::ffi::CStr::from_ptr(native as *const c_char) };
            let text = std::str::from_utf8(octets.to_bytes()).map_err(|e| {
                TrapKind::BadUtf8 { addr: (native + e.valid_up_to()) as u64 }
            })?;
            self.place_text(slot, text)
        }

        /// Write a new handle around an address C passed into a slot: a box
        /// whose one field holds the address. NULL stops the run: the
        /// parameter promised a handle, and an optional takes NULL.
        ///
        /// # Arguments
        ///
        /// * `slot` - Where the handle goes.
        /// * `kind` - The handle kind.
        /// * `native` - The address C passed.
        ///
        /// # Returns
        ///
        /// `Ok`, `NullFromC` for NULL, or `OutOfMem`.
        fn handle_arg(&mut self, slot: u64, kind: DataKind, native: usize) -> Step
        {
            ret_if!(native == 0, Err(TrapKind::NullFromC));
            let shape = kind.as_external().ok_or_else(|| malformed("a handle"))?;
            let cell = DataKind::Struct(shape);
            let size = u64::from(cell.size(self.module.platform_bits));
            let block = self.mem.alloc(size, Origin::Heap, 0);
            let block = block.ok_or(TrapKind::OutOfMem { size })?;
            self.mem.set_kind(block, cell);
            self.mem.store(block, Value::Ptr(native as u64))?;
            self.write_text(slot, block, 1, 1)
        }

        /// Write an optional of a handle C passed into a slot: no data for
        /// NULL, else a new handle around the address.
        ///
        /// # Arguments
        ///
        /// * `slot` - Where the optional goes.
        /// * `kind` - The optional kind.
        /// * `native` - The address C passed.
        ///
        /// # Returns
        ///
        /// `Ok`, or `OutOfMem`.
        fn optional_handle_arg(
            &mut self, slot: u64, kind: DataKind, native: usize,
        ) -> Step
        {
            let platform = self.module.platform_bits;
            let blank = vec![0; kind.size(platform) as usize];
            self.mem.write_octets(slot, &blank)?;
            ret_if!(native == 0, Ok(()));
            let s =
                kind.as_struct().ok_or_else(|| malformed("an optional is a struct"))?;
            let handle = s
                .fields()
                .get(1)
                .map(|(_, k)| *k)
                .ok_or_else(|| malformed("no data field"))?;
            let tag = IntKind::new(false, Width::Bits(8));
            self.mem.store(slot, Value::int(tag, 1, platform))?;
            self.handle_arg(slot + u64::from(s.offset(1, platform)), handle, native)
        }

        /// Drop the blocks a callback got for its arguments, once it returns:
        /// a value the callback kept holds its own count, so the block it
        /// shares remains.
        ///
        /// # Arguments
        ///
        /// * `temps` - The blocks and their kinds.
        fn drop_temps(&mut self, temps: Vec<(u64, DataKind)>)
        {
            for (slot, kind) in temps
            {
                // A lent duplicate, noted as `ptr`, holds nothing to drop.
                let lent = kind == DataKind::OPAQUE;
                let held = tern!(lent, Ok(()), self.drop_value(slot, kind));
                let dropped = held.and_then(|()| self.mem.free(slot));
                do_if!(let Err(trap) = dropped, _ = self.callbacks.trap.get_or_insert(trap));
            }
        }

        /// The octets of text or a text view: its first word is the address of
        /// the octets, its second their count.
        ///
        /// # Arguments
        ///
        /// * `addr` - The address of the text.
        ///
        /// # Returns
        ///
        /// The octets, or the trap a read hits.
        fn text_octets(&self, addr: u64) -> Result<Vec<u8>, TrapKind>
        {
            let word = size_of::<usize>();
            let head = self.mem.octets(addr, 2 * word as u64)?;
            let (at, len) = (read_word(&head, 0) as u64, read_word(&head, word) as u64);
            ret_if!(len == 0, Ok(Vec::new()));
            self.mem.octets(at, len)
        }

        /// Make each address in octets of a kind native. Null, and an
        /// address in no block of the VM, such as one C gave, goes unchanged.
        ///
        /// # Arguments
        ///
        /// * `kind` - The kind of the value.
        /// * `raw` - The octets of the value.
        /// * `at` - Where the value starts in them.
        ///
        /// # Returns
        ///
        /// `Ok`, or the trap for an address in a freed block, which C would
        /// read as a number no block holds, or for octets too short for the
        /// kind.
        fn write_native(
            &mut self, kind: DataKind, raw: &mut [u8], at: usize,
        ) -> Result<(), TrapKind>
        {
            let mem = &mut self.mem;
            let mut trap = None;
            walk_ptrs(kind, at, self.module.platform_bits, &mut |at| {
                if at + size_of::<usize>() > raw.len()
                {
                    trap.get_or_insert(malformed("octets short of the kind"));
                    return;
                }
                let word = read_word(raw, at);
                let addr = word as u64;
                let freed = mem.block_near(addr).is_some_and(|b| !b.live);
                do_if!(freed, _ = trap.get_or_insert(TrapKind::UseAfterFree { addr }));
                let native = mem.native_at(addr).unwrap_or(word);
                write_word(raw, at, native);
            });
            trap.map_or(Ok(()), Err)
        }

        /// Make each native address in octets of a kind that lies in a live
        /// block an address of the VM.
        ///
        /// # Arguments
        ///
        /// * `kind` - The kind of the value.
        /// * `raw` - The octets of the value.
        /// * `at` - Where the value starts in them.
        fn to_vm(&self, kind: DataKind, raw: &mut [u8], at: usize)
        {
            walk_ptrs(kind, at, self.module.platform_bits, &mut |at| {
                let word = read_word(raw, at);
                let vm = self.mem.vm_addr(word).map_or(word, |a| a as usize);
                write_word(raw, at, vm);
            });
        }
    }

    /// The code C calls back: read each argument from its native octets, run
    /// the function in the machine, and write its result as C reads it, an
    /// integer narrower than a word widened with its sign.
    ///
    /// # Arguments
    ///
    /// * `_cif` - The shape of the call.
    /// * `result` - Where the result goes.
    /// * `args` - The address of each argument.
    /// * `data` - The function and its machine.
    unsafe extern "C" fn call_back(
        _cif: &ffi_cif, result: &mut u64, args: *const *const c_void,
        data: &CallbackData,
    )
    {
        *result = 0;
        if std::thread::current().id() != data.thread
        {
            eprintln!("{}", t!("vm.callback_thread"));
            std::process::abort();
        }
        // SAFETY: the machine runs the call of C that runs this callback,
        // and does nothing else until C returns.
        let machine = unsafe { &mut *data.machine.get() };
        if !data.live()
        {
            machine.callbacks.trap.get_or_insert(TrapKind::ClosureEscape);
            return;
        }
        let platform = machine.module.platform_bits;
        let mut temps = Vec::new();
        let mut read = |(i, kind): (usize, &DataKind)| {
            // SAFETY: libffi passes the address of each argument, of the kind
            // the closure was made with.
            let at = unsafe { *args.add(i) }.cast::<u8>();
            machine.callback_arg(*kind, at, &mut temps)
        };
        let values: Result<Vec<Value>, TrapKind> =
            data.params.iter().enumerate().map(&mut read).collect();
        // A closure's thunk takes its record after the declared parameters.
        let record = data.record.get();
        let values = values.map(|mut v| {
            do_if!(record != 0, v.push(Value::Ptr(record)));
            v
        });
        let values = match values
        {
            Ok(values) => values,
            Err(trap) =>
            {
                machine.callbacks.trap.get_or_insert(trap);
                machine.drop_temps(temps);
                return;
            }
        };
        let out = machine.run_callback(data.func, values, data.ret.is_some());
        machine.drop_temps(temps);
        let Some((value, kind)) = out.zip(data.ret)
        else
        {
            return;
        };
        let mut raw = to_raw(value, platform);
        if let Err(trap) = machine.write_native(kind, &mut raw, 0)
        {
            machine.callbacks.trap.get_or_insert(trap);
            return;
        }
        let negative = kind.int_kind().is_some_and(|k| k.signed)
            && raw.last().is_some_and(|b| *b >= 0x80);
        let mut word = [tern!(negative, 0xff, 0); 8];
        let n = raw.len().min(8);
        word[..n].copy_from_slice(&raw[..n]);
        *result = u64::from_le_bytes(word);
    }

    /// Call `each` with the offset of each address a value of the kind holds:
    /// the value itself, or the fields of a struct and the cells of a counted
    /// list, at each level.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `at` - The offset of the value.
    /// * `platform` - Bit count of the platform-sized kinds.
    /// * `each` - Called with each offset.
    fn walk_ptrs(kind: DataKind, at: usize, platform: u16, each: &mut dyn FnMut(usize))
    {
        ret_if!(kind.is_ptr(), each(at));
        if let Some(list) = kind.as_list()
        {
            let stride = list.cell.size(platform) as usize;
            let cells = (0..list.len as usize).map(|i| at + i * stride);
            return cells.for_each(|c| walk_ptrs(list.cell, c, platform, each));
        }
        let Some(s) = kind.as_struct()
        else
        {
            return;
        };
        let fields = s.fields().iter().enumerate();
        fields.for_each(|(i, (_, k))| {
            walk_ptrs(*k, at + s.offset(i, platform) as usize, platform, each)
        });
    }

    /// Whether a value of the stack suits a parameter of C: an address for an
    /// address, a function, text, a list or a struct, and a number, a
    /// character or a boolean of the same sort for each other kind. A value of
    /// another sort means the ops are malformed, and C must not read it.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    /// * `kind` - The kind of the parameter.
    ///
    /// # Returns
    ///
    /// `true` when the value suits the kind.
    fn suits(v: Value, kind: DataKind) -> bool
    {
        let kind = kind.repr();
        match v
        {
            Value::Ptr(_) => kind.is_address() || kind.is_fn() || kind.is_aggregate(),
            Value::Int { .. } => kind.is_int(),
            Value::Float { .. } => kind.is_float(),
            Value::Char(_) => kind == DataKind::Char,
            Value::Bool(_) => kind == DataKind::Bool,
        }
    }

    /// The native word at an offset of octets.
    fn read_word(raw: &[u8], at: usize) -> usize
    {
        let mut word = [0; size_of::<usize>()];
        word.copy_from_slice(&raw[at..at + size_of::<usize>()]);
        usize::from_le_bytes(word)
    }

    /// Write a native word at an offset of octets.
    fn write_word(raw: &mut [u8], at: usize, word: usize)
    {
        raw[at..at + size_of::<usize>()].copy_from_slice(&word.to_le_bytes());
    }

    /// The libffi kind of a kind C passes. A counted list in a struct is its
    /// cells, one after the other, as C holds them there.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The libffi kind.
    fn ffi_kind(kind: DataKind, platform: u16) -> Type
    {
        match kind
        {
            DataKind::Int(n) => match (n.signed, n.bits(platform))
            {
                (true, 8) => Type::i8(),
                (false, 8) => Type::u8(),
                (true, 16) => Type::i16(),
                (false, 16) => Type::u16(),
                (true, 32) => Type::i32(),
                (false, 32) => Type::u32(),
                (true, _) => Type::i64(),
                (false, _) => Type::u64(),
            },
            DataKind::Float(32) => Type::f32(),
            DataKind::Float(_) => Type::f64(),
            DataKind::Bool => Type::u8(),
            DataKind::Struct(s) =>
            {
                let mut fields = Vec::new();
                s.fields()
                    .iter()
                    .for_each(|(_, k)| push_field(*k, platform, &mut fields));
                Type::structure(fields)
            }
            _ => Type::pointer(),
        }
    }

    /// Add the libffi kinds of a field of a struct: a counted list adds one
    /// for each cell.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the field.
    /// * `platform` - Bit count of the platform-sized kinds.
    /// * `out` - The kinds so far.
    fn push_field(kind: DataKind, platform: u16, out: &mut Vec<Type>)
    {
        let Some(list) = kind.as_list()
        else
        {
            return out.push(ffi_kind(kind, platform));
        };
        (0..list.len).for_each(|_| push_field(list.cell, platform, out));
    }

    /// Open the libs of the C functions of a module and find each function.
    ///
    /// # Arguments
    ///
    /// * `module` - The module.
    ///
    /// # Returns
    ///
    /// The linked C functions, or the trap `ForeignLink`.
    fn link(module: &Module) -> Result<Linked, TrapKind>
    {
        let host = usize::BITS as u16;
        let bits = module.platform_bits;
        let reason = t!("vm.trap.foreign_bits", bits = bits, host = host).to_string();
        ret_if!(bits != host, Err(TrapKind::ForeignLink { reason }));
        let mut names: Vec<&str> = Vec::new();
        let mut libs = Vec::new();
        for def in &module.foreign
        {
            let known = names.contains(&def.lib.as_str());
            do_if!(!known, {
                names.push(&def.lib);
                libs.push(open(&def.lib));
            });
        }
        let this = this_process();
        let addrs = module.foreign.iter().map(|def| {
            let at = names.iter().position(|n| *n == def.lib).unwrap_or(0);
            find(&libs[at], &this, def)
        });
        let addrs = addrs.collect::<Result<Vec<_>, _>>()?;
        // SAFETY: the address is called as `int fflush(FILE *)`, its C kind.
        let flush = this
            .as_ref()
            .and_then(|l| unsafe { l.get::<*const c_void>(b"fflush") }.ok());
        let flush = flush.map(|s| *s as usize);
        // SAFETY: the address is called as `void free(void *)`, its C kind.
        let free =
            this.as_ref().and_then(|l| unsafe { l.get::<*const c_void>(b"free") }.ok());
        let free = free.map(|s| *s as usize);
        let libs = libs.into_iter().filter_map(Result::ok).chain(this).collect();
        Ok(Linked { _libs: libs, addrs, flush, free })
    }

    /// Write out what C holds back for its streams, so the text a C function
    /// writes comes before the text the program prints after the call, as in
    /// a native build, where both share one stream.
    ///
    /// # Arguments
    ///
    /// * `linked` - The linked C functions, with libc's `fflush`.
    fn flush_c(linked: &Linked)
    {
        let Some(addr) = linked.flush
        else
        {
            return;
        };
        // SAFETY: the address is libc's `int fflush(FILE *)`; a null stream
        // writes out each stream.
        let flush: unsafe extern "C" fn(*mut c_void) -> i32 =
            unsafe { std::mem::transmute(addr) };
        // SAFETY: as above.
        unsafe { flush(std::ptr::null_mut()) };
    }

    /// Free a string C handed over, with libc's `free`.
    ///
    /// # Arguments
    ///
    /// * `linked` - The linked C functions, with libc's `free`.
    /// * `native` - The address of the string.
    fn free_c(linked: &Linked, native: usize)
    {
        let Some(addr) = linked.free
        else
        {
            return;
        };
        // SAFETY: the address is libc's `void free(void *)`, and the program
        // declares that C handed the string over.
        let free: unsafe extern "C" fn(*mut c_void) =
            unsafe { std::mem::transmute(addr) };
        // SAFETY: as above.
        unsafe { free(native as *mut c_void) };
    }

    /// The address of a C function: in its lib, else in the process.
    ///
    /// # Arguments
    ///
    /// * `lib` - The lib, or the reason it did not open.
    /// * `this` - The process, when it can be opened.
    /// * `def` - The C function.
    ///
    /// # Returns
    ///
    /// The address, or the trap `ForeignLink`.
    fn find(
        lib: &Result<Library, String>, this: &Option<Library>, def: &ForeignFn,
    ) -> Result<usize, TrapKind>
    {
        let name = def.name.as_bytes();
        let get = |l: &Library| {
            // SAFETY: the address is read as a word and called through libffi
            // alone, with the kinds the program declares.
            unsafe { l.get::<*const c_void>(name) }.ok().map(|s| *s as usize)
        };
        let found =
            lib.as_ref().ok().and_then(get).or_else(|| this.as_ref().and_then(get));
        ret_if!(let Some(addr) = found, Ok(addr));
        let reason = match lib
        {
            Err(reason) => t!("vm.trap.foreign_no_lib", lib = def.lib, reason = reason),
            Ok(_) => t!("vm.trap.foreign_no_name", lib = def.lib, name = def.name),
        };
        Err(TrapKind::ForeignLink { reason: reason.to_string() })
    }

    /// Open a lib from the name a program gives it: `libX` is the file
    /// `libX.dylib`, `libX.so` or `X.dll`, as the host names it, and libc on
    /// Windows is the UCRT. A path opens as it is; the loader made a
    /// relative one start at the folder of its file.
    ///
    /// # Arguments
    ///
    /// * `lib` - The name.
    ///
    /// # Returns
    ///
    /// The lib, or the reason it did not open.
    fn open(lib: &str) -> Result<Library, String>
    {
        // SAFETY: as below.
        ret_if!(
            is_lib_path(lib),
            unsafe { Library::new(lib) }.map_err(|e| e.to_string())
        );
        let stem = lib.strip_prefix("lib").unwrap_or(lib);
        let file = windows_libc(stem).unwrap_or_else(|| {
            libloading::library_filename(stem).to_string_lossy().into_owned()
        });
        // SAFETY: opening a lib runs its initializers, as linking it in C
        // would.
        let first = unsafe { Library::new(&file) };
        // glibc keeps libc and libm as `.so.6`; `libm.so` there is a script for
        // the linker, which does not open.
        let numbered = || unsafe { Library::new(format!("{file}.6")) };
        let opened =
            first.or_else(|e| tern!(cfg!(target_os = "linux"), numbered(), Err(e)));
        opened.map_err(|e| e.to_string())
    }

    /// The UCRT for libc on Windows.
    ///
    /// # Arguments
    ///
    /// * `stem` - The name of the lib with no `lib` in front.
    ///
    /// # Returns
    ///
    /// `ucrtbase.dll` for libc on Windows, else `None`.
    fn windows_libc(stem: &str) -> Option<String>
    {
        (cfg!(windows) && stem == "c").then(|| "ucrtbase.dll".to_string())
    }

    /// The process itself, which holds libc on each host but Windows.
    #[cfg(unix)]
    fn this_process() -> Option<Library>
    {
        Some(libloading::os::unix::Library::this().into())
    }

    /// The process itself, which holds libc on each host but Windows.
    #[cfg(windows)]
    fn this_process() -> Option<Library>
    {
        libloading::os::windows::Library::this().ok().map(Into::into)
    }
}
