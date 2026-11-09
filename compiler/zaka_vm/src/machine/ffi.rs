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
    use zakac_semcheck::foreign::is_c_text_result;
    use zakac_semcheck::foreign::is_lib_path;
    use zakac_semcheck::kind::DataKind;
    use zakac_semcheck::kind::IntKind;
    use zakac_semcheck::kind::Width;

    use super::*;
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
    /// each function a call gave C, kept for the whole run, since a lib can
    /// keep the address and call it later. A trap inside a callback cannot
    /// cross C, so it waits here until C returns.
    #[derive(Default)]
    pub(crate) struct Callbacks
    {
        /// The closure of each function, under its index.
        made: HashMap<u32, Callback>,
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
            Callbacks { made: HashMap::new(), trap: self.trap.clone() }
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
        /// The libffi closure, which lives as long as its code.
        _closure: Closure<'static>,
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
            let pairs = values.iter().zip(&kinds);
            let cells = pairs.map(|(v, k)| self.arg_cell(*v, *k, &mut texts));
            let cells = cells.collect::<Result<Vec<Cell>, _>>()?;
            let types = kinds.iter().map(|k| ffi_kind(*k, platform));
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
                Cif::new_variadic(types, fixed, ret_kind),
                Cif::new(types, ret_kind)
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
        /// * `kind` - The result kind, text or a failure.
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
            if let Some(f) = kind.as_fn()
            {
                let native = self.callback(v, f)?;
                return Ok(Cell::of(&native.to_le_bytes()));
            }
            if kind.is_textual()
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
                self.to_native(kind, &mut raw, 0);
                return Ok(Cell::of(&raw));
            }
            let mut raw = to_raw(v, platform);
            self.to_native(kind, &mut raw, 0);
            Ok(Cell::of(&raw))
        }

        /// The native address C calls for a function value: none for `ဗလာ`,
        /// and a libffi closure that runs the function in this machine,
        /// made on the first call that gives it.
        ///
        /// # Arguments
        ///
        /// * `v` - The function value.
        /// * `f` - Its kind.
        ///
        /// # Returns
        ///
        /// The address, or `ClosureEscape` for a closure, which C cannot keep
        /// alive.
        fn callback(&mut self, v: Value, f: &FnKind) -> Result<usize, TrapKind>
        {
            let word =
                v.as_ptr().ok_or_else(|| malformed("foreign needs a function"))?;
            ret_if!(word == 0, Ok(0));
            ret_if!(
                self.mem.closure_record(word).is_some(),
                Err(TrapKind::ClosureEscape)
            );
            let func = u32::try_from(word - 1).map_err(|_| malformed("no function"))?;
            let me: *mut Machine = self;
            if let Some(made) = self.callbacks.made.get(&func)
            {
                made.data.machine.set(me);
                return Ok(made.code);
            }
            let platform = self.module.platform_bits;
            let (params, ret) = fn_shape(f);
            let shapes = params.iter().map(|k| ffi_kind(*k, platform));
            let cif = Cif::new(
                shapes,
                ret.map_or_else(Type::void, |k| ffi_kind(k, platform)),
            );
            #[rustfmt::skip]
            let data = CallbackData { machine: Shared::new(me), func, params, ret };
            // The data lives for the run; one for each function C gets.
            let data: &'static CallbackData = Box::leak(Box::new(data));
            let closure = Closure::new(cif, call_back, data);
            let code = *closure.code_ptr() as usize;
            let made = Callback { _closure: closure, data, code };
            self.callbacks.made.insert(func, made);
            Ok(code)
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

        /// Make each address in octets of a kind native.
        ///
        /// # Arguments
        ///
        /// * `kind` - The kind of the value.
        /// * `raw` - The octets of the value.
        /// * `at` - Where the value starts in them.
        fn to_native(&mut self, kind: DataKind, raw: &mut [u8], at: usize)
        {
            let mem = &mut self.mem;
            walk_ptrs(kind, at, self.module.platform_bits, &mut |at| {
                let word = read_word(raw, at);
                let native = mem.native_at(word as u64).unwrap_or(word);
                write_word(raw, at, native);
            });
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
        // SAFETY: the machine runs the call of C that runs this callback,
        // and does nothing else until C returns.
        let machine = unsafe { &mut *data.machine.get() };
        let platform = machine.module.platform_bits;
        let read = |(i, kind): (usize, &DataKind)| {
            let size = kind.size(platform) as usize;
            // SAFETY: libffi passes the address of each argument, of the kind
            // the closure was made with.
            let at = unsafe { *args.add(i) }.cast::<u8>();
            // SAFETY: as above.
            let mut raw = unsafe { std::slice::from_raw_parts(at, size) }.to_vec();
            machine.to_vm(*kind, &mut raw, 0);
            from_raw(*kind, &raw, platform).ok()
        };
        let values: Option<Vec<Value>> =
            data.params.iter().enumerate().map(read).collect();
        let Some(values) = values
        else
        {
            machine
                .callbacks
                .trap
                .get_or_insert(malformed("C gave a value of no kind"));
            return;
        };
        let out = machine.run_callback(data.func, values, data.ret.is_some());
        let Some((value, kind)) = out.zip(data.ret)
        else
        {
            return;
        };
        let mut raw = to_raw(value, platform);
        machine.to_native(kind, &mut raw, 0);
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
