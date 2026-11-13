//! The op `host`, the one op of an external function: the VM reads each
//! argument through the address of its parameter as a value the host takes,
//! runs the host function against its world, and writes the result in the
//! result slot. Both turns follow the kinds alone (text, a stream struct, an
//! enumeration member, a result that can fail), so a new host function over
//! these kinds needs no code here.

use zaka_host::Args;
use zaka_host::Effects;
use zaka_host::HostFn;
use zaka_host::HostValue;
use zakac_ir_builder::value::Value;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;

use super::ISIZE;
use super::Machine;
use super::Step;
use super::malformed;
use crate::print::text_at;
use crate::trap::TrapKind;

impl Machine
{
    /// Run the op `host`: pop the address of each parameter, the last on top,
    /// then the address of the result slot; serve the host function, write
    /// its result in the slot, and push the address of the slot.
    ///
    /// # Arguments
    ///
    /// * `func` - The host function.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap a read or a write hits.
    pub(super) fn host(&mut self, func: HostFn) -> Step
    {
        let index = self.frames.last().map(|f| f.func);
        let def = index.and_then(|i| self.module.funcs.get(i as usize));
        let def = def.ok_or_else(|| malformed("host outside a function"))?;
        let ret = def.ret.ok_or_else(|| malformed("host needs a result"))?;
        let kinds: Vec<DataKind> =
            def.locals.iter().take(def.params as usize).map(|s| s.kind).collect();
        let mut addrs = kinds
            .iter()
            .map(|_| self.pop_ptr("host"))
            .collect::<Result<Vec<_>, _>>()?;
        addrs.reverse();
        let slot = self.pop_ptr("host")?;
        let read = addrs.iter().zip(&kinds).map(|(a, k)| self.host_value_at(*a, *k));
        let args = Args(read.collect::<Result<_, _>>()?);
        let world = &mut *self.world;
        #[rustfmt::skip]
        let mut effects = Effects { world, out: &mut self.out, err: &mut self.err, blocked: false };
        let result = func.serve(&args, &mut effects);
        ret_if!(effects.blocked, self.wait_for_input(slot, &addrs));
        self.put_host_value(slot, ret, result)?;
        self.push(Value::Ptr(slot))
    }

    /// Wait for standard input: put the operands of the op back and step
    /// back to it, so it runs again, uncounted, once input comes.
    ///
    /// # Arguments
    ///
    /// * `slot` - The address of the result slot.
    /// * `addrs` - The address of each parameter.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap a push hits.
    fn wait_for_input(&mut self, slot: u64, addrs: &[u64]) -> Step
    {
        self.push(Value::Ptr(slot))?;
        addrs.iter().try_for_each(|a| self.push(Value::Ptr(*a)))?;
        let frame = self.frames.last_mut().ok_or_else(|| malformed("no call"))?;
        frame.pc -= 1;
        self.steps -= 1;
        self.waiting = true;
        Ok(())
    }

    /// The value of a kind at an address, as the host takes it.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `kind` - The kind: text, an enumeration, or a stream struct.
    ///
    /// # Returns
    ///
    /// The value, or a trap a read hits.
    fn host_value_at(&self, addr: u64, kind: DataKind) -> Result<HostValue, TrapKind>
    {
        ret_if!(kind.is_text(), Ok(HostValue::Text(text_at(&self.mem, addr)?)));
        ret_if!(
            kind.as_enum().is_some(),
            Ok(HostValue::Member(self.int_at(addr, kind)?))
        );
        let handle = kind.as_struct().and_then(|s| s.fields().first().map(|(_, k)| *k));
        let handle =
            handle.ok_or_else(|| malformed("the host takes no value of this kind"))?;
        Ok(HostValue::Stream(self.int_at(addr, handle)?))
    }

    /// The integer of a kind at an address, cut to 32 bits.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `kind` - An integer or enumeration kind.
    ///
    /// # Returns
    ///
    /// The number, or a trap a read hits.
    fn int_at(&self, addr: u64, kind: DataKind) -> Result<u32, TrapKind>
    {
        match self.mem.load(addr, kind)?
        {
            Value::Int { raw, .. } => Ok(raw as u32),
            _ => Err(malformed("the host takes an integer here")),
        }
    }

    /// Write a value the host gives at an address, as a value of a kind.
    ///
    /// # Arguments
    ///
    /// * `at` - The address.
    /// * `kind` - The kind of the place.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// `Ok`, a trap a write hits, or `OutOfMem` for text with no room.
    fn put_host_value(&mut self, at: u64, kind: DataKind, value: HostValue) -> Step
    {
        let platform = self.module.platform_bits;
        match value
        {
            HostValue::Text(text) => self.place_text(at, &text),
            HostValue::Texts(texts) => self.place_text_list(at, &texts),
            HostValue::Member(n) =>
            {
                self.mem.store(at, Value::int(ISIZE, n.into(), platform))
            }
            HostValue::Stream(handle) =>
            {
                let field =
                    kind.as_struct().and_then(|s| s.fields().first().map(|(_, k)| *k));
                let int =
                    field.and_then(int_kind).ok_or_else(|| malformed("no handle"))?;
                self.mem.store(at, Value::int(int, handle.into(), platform))
            }
            HostValue::Good(inner) =>
            {
                self.put_side(at, kind, (1, "ok"), inner.map(|v| *v))
            }
            HostValue::Fault(inner) =>
            {
                self.put_side(at, kind, (0, "err"), Some(*inner))
            }
        }
    }

    /// Write a result that can fail: its tag, and the value of its side.
    ///
    /// # Arguments
    ///
    /// * `at` - The address of the result.
    /// * `kind` - The result kind.
    /// * `side` - The tag (1 for a good result, 0 for a failure, as the
    ///   compiler writes it) and the field of the side.
    /// * `inner` - The value of the side, when it holds one.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap a write hits.
    fn put_side(
        &mut self, at: u64, kind: DataKind, side: (u8, &str), inner: Option<HostValue>,
    ) -> Step
    {
        let platform = self.module.platform_bits;
        let shape =
            kind.as_struct().ok_or_else(|| malformed("the host gives a result"))?;
        let (tag_at, tag) = shape.field("tag").ok_or_else(|| malformed("no tag"))?;
        let tag = int_kind(tag).ok_or_else(|| malformed("no tag"))?;
        let tag_addr = at + u64::from(shape.offset(tag_at, platform));
        self.mem.store(tag_addr, Value::int(tag, side.0.into(), platform))?;
        let Some(inner) = inner
        else
        {
            return Ok(());
        };
        let (field_at, field) =
            shape.field(side.1).ok_or_else(|| malformed("no side"))?;
        let addr = at + u64::from(shape.offset(field_at, platform));
        self.put_host_value(addr, field, inner)
    }
}

/// The integer kind of a kind.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The integer kind, or `None` for another kind.
fn int_kind(kind: DataKind) -> Option<IntKind>
{
    match kind
    {
        DataKind::Int(int) => Some(int),
        _ => None,
    }
}
