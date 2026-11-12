//! The rows of a ဇယား: finding the row of an index, setting and removing a
//! row, and comparing two ဇယား values.
//!
//! A ဇယား is a struct of its rows, a plain list in add order, its slots, a
//! plain list of `u32` whose count is a power of two, and its count of live
//! rows. A slot holds 0 when no row ever used it, 1 when its row was
//! removed, and 2 more than the number of its row otherwise. An index goes to
//! the slot its hash names, or the first one after it that holds no other
//! row of the same index. At least a third of the slots hold 0, so each
//! search ends. A removed row is dropped and left as zero octets until a
//! rebuild moves the live rows together.

use super::*;

/// The first state of the 64-bit FNV-1a hash.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// The multiplier of the 64-bit FNV-1a hash.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
/// The fewest slots a ဇယား with rows holds.
const MIN_SLOTS: u64 = 8;

/// Where the parts of a ဇယား and of its rows sit, and their kinds.
struct Shape
{
    /// The offset of the rows in the ဇယား.
    rows: u64,
    /// The offset of the slots in the ဇယား.
    slots: u64,
    /// The offset of the count of live rows in the ဇယား.
    live: u64,
    /// The kind of the rows, a plain list.
    rows_kind: DataKind,
    /// The kind of the slots, a plain list.
    slots_kind: DataKind,
    /// The kind of a row.
    row: DataKind,
    /// The size of a row.
    stride: u64,
    /// The offset of the hash in a row.
    hash: u64,
    /// The offset of the live mark in a row.
    alive: u64,
    /// The offset of the index in a row.
    index: u64,
    /// The offset of the value in a row.
    value: u64,
    /// The index kind.
    index_kind: DataKind,
    /// The value kind.
    value_kind: DataKind,
}

/// The kind of a `u32`.
fn u32_kind() -> DataKind { DataKind::Int(IntKind::new(false, Width::Bits(32))) }

/// The kind of a `u64`.
fn u64_kind() -> DataKind { DataKind::Int(IntKind::new(false, Width::Bits(64))) }

/// Mix u8s into a 64-bit FNV-1a hash.
///
/// # Arguments
///
/// * `h` - The hash so far.
/// * `raw` - The u8s.
///
/// # Returns
///
/// The hash.
fn mix(h: u64, raw: &[u8]) -> u64
{
    raw.iter().fold(h, |h, b| (h ^ u64::from(*b)).wrapping_mul(FNV_PRIME))
}

/// The bits of a decimal number as an index compares and hashes them: each
/// NaN is one NaN, and -0.0 is 0.0.
///
/// # Arguments
///
/// * `bits` - The bit count of the kind.
/// * `raw` - The bit pattern.
///
/// # Returns
///
/// The folded pattern.
fn float_bits(bits: u16, raw: u128) -> u128
{
    let (exp, man) = match bits
    {
        16 => (5u32, 10u32),
        32 => (8, 23),
        64 => (11, 52),
        _ => (15, 112),
    };
    let sign = 1u128 << (exp + man);
    let magnitude = raw & (sign - 1);
    let exp_mask = ((1u128 << exp) - 1) << man;
    let nan = magnitude & exp_mask == exp_mask && magnitude & ((1u128 << man) - 1) != 0;
    ret_if!(nan, exp_mask | (1u128 << (man - 1)));
    tern!(magnitude == 0, 0, raw)
}

/// The bits of a scalar as an index compares and hashes them.
///
/// # Arguments
///
/// * `v` - The value.
///
/// # Returns
///
/// The bits.
fn index_bits(v: Value) -> u128
{
    match v
    {
        Value::Int { raw, .. } => raw,
        Value::Float { bits, raw } => float_bits(bits, raw),
        Value::Char(c) => u128::from(u32::from(c)),
        Value::Bool(b) => u128::from(b),
        Value::Ptr(p) => u128::from(p),
    }
}

impl Machine
{
    /// Where the parts of a ဇယား of a kind sit.
    ///
    /// # Arguments
    ///
    /// * `kind` - The ဇယား kind.
    ///
    /// # Returns
    ///
    /// The shape, or a malformed module for a kind that is no ဇယား.
    fn table_shape(&self, kind: DataKind) -> Result<Shape, TrapKind>
    {
        let platform = self.module.platform_bits;
        let bad = || malformed("a row op takes a ဇယား");
        let table = kind.as_struct().filter(|_| kind.is_table()).ok_or_else(bad)?;
        let row = kind.table_row().ok_or_else(bad)?;
        let fields = row.as_struct().ok_or_else(bad)?;
        let (index_kind, value_kind) = kind.table_parts().ok_or_else(bad)?;
        let at = |s: &zakac_semcheck::structs::StructKind, i| {
            u64::from(s.offset(i, platform))
        };
        Ok(Shape {
            rows: at(table, 0),
            slots: at(table, 1),
            live: at(table, 2),
            rows_kind: table.fields()[0].1,
            slots_kind: table.fields()[1].1,
            row,
            stride: u64::from(row.size(platform)),
            hash: at(fields, 0),
            alive: at(fields, 1),
            index: at(fields, 2),
            value: at(fields, 3),
            index_kind,
            value_kind,
        })
    }

    /// Read an unsigned integer of a kind.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `kind` - The integer kind.
    ///
    /// # Returns
    ///
    /// The value, or the trap the read hits.
    fn load_count(&self, addr: u64, kind: DataKind) -> Result<u64, TrapKind>
    {
        let v = self.mem.load(addr, kind)?;
        Ok(tern!(let Value::Int { raw, .. } = v, raw as u64, 0))
    }

    /// Write an unsigned integer of a kind.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `kind` - The integer kind.
    /// * `n` - The value.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap the write hits.
    fn store_count(&mut self, addr: u64, kind: DataKind, n: u64) -> Step
    {
        let int = kind.int_kind().ok_or_else(|| malformed("a count is an integer"))?;
        let platform = self.module.platform_bits;
        self.mem.store(addr, Value::int(int, u128::from(n), platform))
    }

    /// Whether the row at an address is live.
    ///
    /// # Arguments
    ///
    /// * `row` - The address of the row.
    /// * `sh` - The shape.
    ///
    /// # Returns
    ///
    /// `true` for a live row, or the trap the read hits.
    fn row_live(&self, row: u64, sh: &Shape) -> Result<bool, TrapKind>
    {
        Ok(self.mem.load(row + sh.alive, DataKind::Bool)? == Value::Bool(true))
    }

    /// The hash of an index: its parts in order, decimals folded, text and
    /// plain lists through their content.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the index.
    /// * `kind` - The index kind.
    ///
    /// # Returns
    ///
    /// The hash, or the trap a read hits.
    fn index_hash(&self, addr: u64, kind: DataKind) -> Result<u64, TrapKind>
    {
        let platform = self.module.platform_bits;
        let mut h = FNV_OFFSET;
        let mut todo = vec![(addr, kind)];
        while let Some((at, k)) = todo.pop()
        {
            let k = k.repr();
            if k.is_text()
            {
                h = mix(mix(h, text_at(&self.mem, at)?.as_bytes()), &[0xff]);
                continue;
            }
            if let Some(cell) = k.cell_kind()
            {
                let (base, count) = match k.as_list()
                {
                    Some(l) => (at, u64::from(l.len)),
                    None => slice_parts(&self.mem, at)?,
                };
                h = mix(h, &count.to_le_bytes());
                let stride = u64::from(cell.size(platform));
                todo.extend((0..count).rev().map(|i| (base + i * stride, cell)));
                continue;
            }
            if let Some(s) = k.as_struct()
            {
                let at = |i: usize| at + u64::from(s.offset(i, platform));
                todo.extend(
                    s.fields().iter().enumerate().rev().map(|(i, (_, f))| (at(i), *f)),
                );
                continue;
            }
            h = mix(h, &index_bits(self.mem.load(at, k)?).to_le_bytes());
        }
        Ok(h)
    }

    /// Whether two indexes are one: equal parts, with each NaN equal to each
    /// other one and -0.0 equal to 0.0.
    ///
    /// # Arguments
    ///
    /// * `a` - The address of the first index.
    /// * `b` - The address of the second index.
    /// * `kind` - The index kind.
    ///
    /// # Returns
    ///
    /// `true` when the two are one, or the trap a read hits.
    fn index_eq(&self, a: u64, b: u64, kind: DataKind) -> Result<bool, TrapKind>
    {
        let platform = self.module.platform_bits;
        let mut todo = vec![(a, b, kind)];
        while let Some((l, r, k)) = todo.pop()
        {
            let k = k.repr();
            if k.is_text()
            {
                ret_if!(text_at(&self.mem, l)? != text_at(&self.mem, r)?, Ok(false));
                continue;
            }
            if let Some(cell) = k.cell_kind()
            {
                let ((lb, lc), (rb, rc)) = match k.as_list()
                {
                    Some(list) => ((l, u64::from(list.len)), (r, u64::from(list.len))),
                    None => (slice_parts(&self.mem, l)?, slice_parts(&self.mem, r)?),
                };
                ret_if!(lc != rc, Ok(false));
                let stride = u64::from(cell.size(platform));
                todo.extend((0..lc).map(|i| (lb + i * stride, rb + i * stride, cell)));
                continue;
            }
            if let Some(s) = k.as_struct()
            {
                let off = |i: usize| u64::from(s.offset(i, platform));
                todo.extend(
                    s.fields()
                        .iter()
                        .enumerate()
                        .map(|(i, (_, f))| (l + off(i), r + off(i), *f)),
                );
                continue;
            }
            let (a, b) = (self.mem.load(l, k)?, self.mem.load(r, k)?);
            ret_if!(index_bits(a) != index_bits(b), Ok(false));
        }
        Ok(true)
    }

    /// Find the row of a ဇယား that holds an index.
    ///
    /// # Arguments
    ///
    /// * `table` - The address of the ဇယား.
    /// * `index` - The address of the index.
    /// * `sh` - The shape.
    ///
    /// # Returns
    ///
    /// The number of the row and of its slot, `None` when no row holds the
    /// index, or the trap a read hits.
    fn find_row_at(
        &self, table: u64, index: u64, sh: &Shape,
    ) -> Result<Option<(u64, u64)>, TrapKind>
    {
        let (slots, count) = slice_parts(&self.mem, table + sh.slots)?;
        ret_if!(count == 0, Ok(None));
        let (rows, _) = slice_parts(&self.mem, table + sh.rows)?;
        let h = self.index_hash(index, sh.index_kind)?;
        let mask = count - 1;
        let mut at = h & mask;
        for _ in 0..count
        {
            let slot = self.load_count(slots + 4 * at, u32_kind())?;
            ret_if!(slot == 0, Ok(None));
            let row = rows + slot.saturating_sub(2) * sh.stride;
            let same = slot >= 2
                && self.load_count(row + sh.hash, u64_kind())? == h
                && self.index_eq(row + sh.index, index, sh.index_kind)?;
            ret_if!(same, Ok(Some((slot - 2, at))));
            at = (at + 1) & mask;
        }
        Ok(None)
    }

    /// Pop the address of an index, then the address of a ဇယား; push the
    /// number of the row that holds the index, or -1.
    ///
    /// # Arguments
    ///
    /// * `kind` - The ဇယား kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a read hits.
    pub(super) fn row_find(&mut self, kind: DataKind) -> Step
    {
        let index = self.pop_ptr("row_find")?;
        let table = self.pop_ptr("row_find")?;
        let sh = self.table_shape(kind)?;
        let found = self.find_row_at(table, index, &sh)?;
        let n = found.map_or(-1i128, |(row, _)| i128::from(row));
        let platform = self.module.platform_bits;
        self.push(Value::int(IntKind::new(true, Width::Platform), n as u128, platform))
    }

    /// Give the rows and the slots of a ဇယား blocks of their own, so a write
    /// reaches this ဇယား alone.
    ///
    /// # Arguments
    ///
    /// * `table` - The address of the ဇယား.
    /// * `sh` - The shape.
    ///
    /// # Returns
    ///
    /// `Ok`, a trap a read hits, or `OutOfMem`.
    fn own_table(&mut self, table: u64, sh: &Shape) -> Step
    {
        self.own_value(table + sh.rows, sh.rows_kind)?;
        self.own_value(table + sh.slots, sh.slots_kind)
    }

    /// Point a free slot at a row: the slot its hash names, or the first free
    /// one after it.
    ///
    /// # Arguments
    ///
    /// * `table` - The address of the ဇယား.
    /// * `sh` - The shape.
    /// * `h` - The hash of the index of the row.
    /// * `row` - The number of the row.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a read or a write hits.
    fn place_slot(&mut self, table: u64, sh: &Shape, h: u64, row: u64) -> Step
    {
        let (slots, count) = slice_parts(&self.mem, table + sh.slots)?;
        let mask = count.saturating_sub(1);
        let mut at = h & mask;
        for _ in 0..count
        {
            let slot = self.load_count(slots + 4 * at, u32_kind())?;
            ret_if!(slot <= 1, self.store_count(slots + 4 * at, u32_kind(), row + 2));
            at = (at + 1) & mask;
        }
        Err(malformed("a ဇယား has no free slot"))
    }

    /// Move the live rows of a ဇယား together, in their order, and give it a
    /// new block of slots, each pointing at its row again.
    ///
    /// # Arguments
    ///
    /// * `table` - The address of the ဇယား, which owns its blocks.
    /// * `sh` - The shape.
    /// * `slots` - The new count of slots, a power of two.
    ///
    /// # Returns
    ///
    /// `Ok`, a trap a read hits, or `OutOfMem`.
    fn rebuild_rows(&mut self, table: u64, sh: &Shape, slots: u64) -> Step
    {
        let at = table + sh.rows;
        let (base, used) = slice_parts(&self.mem, at)?;
        let room = self.room_of(at)?;
        let mut kept = 0;
        for n in 0..used
        {
            let row = base + n * sh.stride;
            do_if!(!self.row_live(row, sh)?, continue);
            if kept != n
            {
                self.mem.blit(base + kept * sh.stride, row, sh.row)?;
                self.mem.zero(row, sh.row)?;
            }
            kept += 1;
        }
        self.write_text(at, base, kept, room)?;
        self.drop_value(table + sh.slots, sh.slots_kind)?;
        let size = slots * 4;
        let block =
            self.mem.alloc(size, Origin::Heap, 0).ok_or(TrapKind::OutOfMem { size })?;
        self.mem.set_kind(block, u32_kind());
        self.mem.fill_block(block, &vec![0u8; size as usize])?;
        self.write_text(table + sh.slots, block, slots, size)?;
        for n in 0..kept
        {
            let h = self.load_count(base + n * sh.stride + sh.hash, u64_kind())?;
            self.place_slot(table, sh, h, n)?;
        }
        Ok(())
    }

    /// Pop the address of a value, then the address of an index, then the
    /// address of a ဇယား; give the row of the index the value, or add a new
    /// last row of both.
    ///
    /// # Arguments
    ///
    /// * `kind` - The ဇယား kind.
    ///
    /// # Returns
    ///
    /// `Ok`, a trap a read hits, or `OutOfMem`.
    pub(super) fn set_row(&mut self, kind: DataKind) -> Step
    {
        let src = self.pop_ptr("set_row")?;
        let index = self.pop_ptr("set_row")?;
        let table = self.pop_ptr("set_row")?;
        let sh = self.table_shape(kind)?;
        self.own_table(table, &sh)?;
        if let Some((row, _)) = self.find_row_at(table, index, &sh)?
        {
            let (base, _) = slice_parts(&self.mem, table + sh.rows)?;
            let at = base + row * sh.stride + sh.value;
            self.drop_value(at, sh.value_kind)?;
            self.mem.blit(at, src, sh.value_kind)?;
            self.mem.zero(src, sh.value_kind)?;
            return self.drop_value(index, sh.index_kind);
        }
        let h = self.index_hash(index, sh.index_kind)?;
        let (_, used) = slice_parts(&self.mem, table + sh.rows)?;
        let (_, slots) = slice_parts(&self.mem, table + sh.slots)?;
        let live = self.load_count(table + sh.live, DataKind::USIZE)?;
        // Each row, live or removed, holds a slot until a rebuild, and a
        // third of the slots remain free.
        let full = (used + 1) * 3 > slots * 2;
        let want = ((live + 1) * 2).max(MIN_SLOTS).next_power_of_two();
        do_if!(full, self.rebuild_rows(table, &sh, want)?);
        let row = self.push_row(table, &sh, h, index, src)?;
        self.place_slot(table, &sh, h, row)?;
        self.store_count(table + sh.live, DataKind::USIZE, live + 1)
    }

    /// Add a new last row to a ဇယား, moving its index and its value in.
    ///
    /// # Arguments
    ///
    /// * `table` - The address of the ဇယား, which owns its blocks.
    /// * `sh` - The shape.
    /// * `h` - The hash of the index.
    /// * `index` - The address of the index, left at its default.
    /// * `src` - The address of the value, left at its default.
    ///
    /// # Returns
    ///
    /// The number of the new row, a trap a read hits, or `OutOfMem`.
    fn push_row(
        &mut self, table: u64, sh: &Shape, h: u64, index: u64, src: u64,
    ) -> Result<u64, TrapKind>
    {
        let at = table + sh.rows;
        let (base, count) = slice_parts(&self.mem, at)?;
        let held = self.room_of(at)?;
        let need = (count + 1).saturating_mul(sh.stride);
        let (mut block, mut room) = (base, held);
        if need > held
        {
            room = need.max(held.saturating_mul(2)).max(4 * sh.stride);
            // A ဇယား with no rows owns no block to move.
            block = self.mem.realloc(tern!(held == 0, 0, base), room)?;
            self.mem.set_kind(block, sh.row);
        }
        let row = block + count * sh.stride;
        self.mem.zero(row, sh.row)?;
        self.store_count(row + sh.hash, u64_kind(), h)?;
        self.mem.store(row + sh.alive, Value::Bool(true))?;
        self.mem.blit(row + sh.index, index, sh.index_kind)?;
        self.mem.zero(index, sh.index_kind)?;
        self.mem.blit(row + sh.value, src, sh.value_kind)?;
        self.mem.zero(src, sh.value_kind)?;
        self.write_text(at, block, count + 1, room)?;
        Ok(count)
    }

    /// Pop the address of an index, then the address of a ဇယား; remove the row
    /// that holds the index, when one does. A ဇယား left with no live row
    /// gives back its blocks.
    ///
    /// # Arguments
    ///
    /// * `kind` - The ဇယား kind.
    ///
    /// # Returns
    ///
    /// `Ok`, a trap a read hits, or `OutOfMem`.
    pub(super) fn remove_row(&mut self, kind: DataKind) -> Step
    {
        let index = self.pop_ptr("remove_row")?;
        let table = self.pop_ptr("remove_row")?;
        let sh = self.table_shape(kind)?;
        let Some((row, slot)) = self.find_row_at(table, index, &sh)?
        else
        {
            return Ok(());
        };
        self.own_table(table, &sh)?;
        let (base, _) = slice_parts(&self.mem, table + sh.rows)?;
        let at = base + row * sh.stride;
        self.drop_value(at, sh.row)?;
        self.mem.zero(at, sh.row)?;
        let (slots, _) = slice_parts(&self.mem, table + sh.slots)?;
        self.store_count(slots + 4 * slot, u32_kind(), 1)?;
        let live = self.load_count(table + sh.live, DataKind::USIZE)?.saturating_sub(1);
        ret_if!(live == 0, self.drop_value(table, kind));
        self.store_count(table + sh.live, DataKind::USIZE, live)
    }

    /// Compare two ဇယား values: equal counts of live rows, and for each live
    /// row of the first, a row of the second with one index whose value is
    /// equal, which the work list compares.
    ///
    /// # Arguments
    ///
    /// * `l` - The address of the first.
    /// * `r` - The address of the second.
    /// * `kind` - The ဇယား kind.
    /// * `work` - The pairs still to compare, added to.
    ///
    /// # Returns
    ///
    /// `false` when the two differ so far, else `true`, or the trap a read
    /// hits.
    pub(super) fn table_equal(
        &self, l: u64, r: u64, kind: DataKind, work: &mut Vec<(u64, u64, DataKind)>,
    ) -> Result<bool, TrapKind>
    {
        let sh = self.table_shape(kind)?;
        let count = |at: u64| self.load_count(at + sh.live, DataKind::USIZE);
        ret_if!(count(l)? != count(r)?, Ok(false));
        let (lrows, used) = slice_parts(&self.mem, l + sh.rows)?;
        let (rrows, _) = slice_parts(&self.mem, r + sh.rows)?;
        for n in 0..used
        {
            let row = lrows + n * sh.stride;
            do_if!(!self.row_live(row, &sh)?, continue);
            let Some((other, _)) = self.find_row_at(r, row + sh.index, &sh)?
            else
            {
                return Ok(false);
            };
            let there = rrows + other * sh.stride + sh.value;
            work.push((row + sh.value, there, sh.value_kind));
        }
        Ok(true)
    }
}
