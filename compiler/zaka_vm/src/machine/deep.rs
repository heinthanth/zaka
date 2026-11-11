//! The deep duplicate: a value with a duplicate of its own of each object it
//! reaches, at each depth. A table from each old object to its new one makes
//! each object duplicated once, so a ring ends and two holders of one object
//! hold one new object. Text, plain lists of plain cells and function values
//! are shared, as a plain duplicate shares them; a closure keeps its captures.
//!
//! A list of the cells still to visit takes the place of a deep call stack.

use super::collector::can_ring;
use super::*;

/// A value still to visit: its address and its kind.
type Visit = (u64, DataKind);

impl Machine
{
    /// Give the value at an address a duplicate of its own of each object it
    /// reaches.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the value, a blit of the one duplicated.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// `Ok`, a trap a read hits, or `OutOfMem`.
    pub(super) fn deep_value(&mut self, addr: u64, kind: DataKind) -> Step
    {
        let mut copies = HashMap::new();
        let mut todo = vec![(addr, kind)];
        while let Some((at, k)) = todo.pop()
        {
            for (slot, sk) in self.blocks_in(at, k)
            {
                self.deep_slot(slot, sk, &mut copies, &mut todo)?;
            }
        }
        Ok(())
    }

    /// Duplicate one text, list or function value inside a deep duplicate:
    /// an object gets its new block, made once; a list that can reach an
    /// object gets a new block whose cells are visited in turn; the rest are
    /// shared.
    ///
    /// # Arguments
    ///
    /// * `at` - The address of the slot.
    /// * `kind` - Its kind.
    /// * `copies` - Each old object block with its new one.
    /// * `todo` - The cells still to visit.
    ///
    /// # Returns
    ///
    /// `Ok`, a trap a read hits, or `OutOfMem`.
    fn deep_slot(
        &mut self, at: u64, kind: DataKind, copies: &mut HashMap<u64, u64>,
        todo: &mut Vec<Visit>,
    ) -> Step
    {
        ret_if!(kind.is_fn() || !can_ring(kind), self.dup_value(at, kind));
        let (base, count) = slice_parts(&self.mem, at)?;
        ret_if!(base == 0 || self.mem.is_frozen(base), Ok(()));
        if let Some(&dup) = copies.get(&base)
        {
            self.mem.retain(dup)?;
            return self.mem.store(at, Value::Ptr(dup));
        }
        let stride = self.stride_of(kind);
        let size = count * stride;
        ret_if!(size == 0, self.write_text(at, 0, 0, 0));
        let content = self.mem.octets(base, size)?;
        let block = self.mem.alloc(size, Origin::Heap, 0);
        let block = block.ok_or(TrapKind::OutOfMem { size })?;
        let cell = kind.as_seq().map(|s| s.cell);
        do_if!(let Some(c) = cell, self.mem.set_kind(block, c));
        self.mem.fill_block(block, &content)?;
        self.write_text(at, block, count, size)?;
        do_if!(kind.is_box(), {
            copies.insert(base, block);
        });
        do_if!(let Some(c) = cell, todo.extend((0..count).map(|i| (block + i * stride, c))));
        Ok(())
    }
}
