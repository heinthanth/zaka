use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::ops::Range;

use bon::Builder;
use strum::IntoStaticStr;
use zakac_ir_builder::value::Value;
use zakac_ir_builder::value::from_raw;
use zakac_ir_builder::value::to_raw;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::trap::TrapKind;

/// The most addresses kept back at the bottom of the address space.
const MAX_RESERVED: u64 = 4096;
/// Each block starts at a multiple of this.
const BLOCK_ALIGN: u64 = 16;
/// The gap kept free after each block, so an access just past a block
/// never lands in the next one.
const RED_ZONE: u64 = 16;

/// A run of zero octets, when the host has room for it.
///
/// # Arguments
///
/// * `size` - The octet count.
///
/// # Returns
///
/// The octets, or `None` when the host refuses the room, so the program
/// traps for want of heap rather than the host aborting.
fn zeros(size: u64) -> Option<Vec<u8>>
{
    let len = usize::try_from(size).ok()?;
    let mut data = Vec::new();
    data.try_reserve_exact(len).ok()?;
    data.resize(len, 0);
    Some(data)
}

/// Where a block came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Origin
{
    /// An allocation of the program.
    Heap,
    /// An addressed local of a call.
    Cell,
    /// Data that lives for the whole run.
    Static,
}

/// A run of octets at a fixed address.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Block
{
    /// The address of the first octet.
    pub start: u64,
    /// The octet count, kept once the block is freed.
    pub len: u64,
    /// The contents; empty once the block is freed.
    pub data: Vec<u8>,
    /// Where the block came from.
    pub origin: Origin,
    /// Whether the block is in use; a freed block stands in quarantine.
    pub live: bool,
    /// The call depth that made a cell; 0 for heap and static blocks.
    pub owner: usize,
}

impl Block
{
    /// The address just past the last octet.
    ///
    /// # Returns
    ///
    /// `start` plus the length.
    fn end(&self) -> u64 { self.start.saturating_add(self.len) }

    /// Whether an address lies inside the block or just past its end.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// `true` when `start <= addr <= end`.
    fn reaches(&self, addr: u64) -> bool { addr >= self.start && addr <= self.end() }

    /// Whether an address lies inside the block.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// `true` when `start <= addr < end`.
    fn holds(&self, addr: u64) -> bool { addr >= self.start && addr < self.end() }
}

/// The checked heap, cells and static data of the machine: blocks in a
/// flat address space, a reserved region at the bottom, red zones between
/// blocks and a quarantine of freed blocks.
#[derive(Debug, Clone, Builder)]
#[builder(start_fn(vis = ""))]
pub struct Mem
{
    /// Bit count of the platform-sized kinds and addresses.
    platform: u16,
    /// The end of the address space.
    space: u64,
    /// The addresses below this one are never handed out.
    reserved: u64,
    /// The lowest address a new block can take.
    next: u64,
    /// The cap on live heap octets.
    max_heap: u64,
    /// The live heap octets.
    #[builder(default)]
    live_heap: u64,
    /// Each block, live or freed, at its start.
    #[builder(default)]
    table: BTreeMap<u64, Block>,
    /// The starts of freed blocks, the oldest first.
    #[builder(default)]
    freed: VecDeque<u64>,
}

impl Mem
{
    /// Make an address space with no blocks.
    ///
    /// # Arguments
    ///
    /// * `platform_bits` - Bit count of addresses; the space holds
    ///   `2^platform_bits` addresses, at most `u64::MAX`.
    /// * `max_heap` - The cap on live heap octets.
    ///
    /// # Returns
    ///
    /// The address space.
    pub fn new(platform_bits: u16, max_heap: u64) -> Mem
    {
        let space = tern!(platform_bits >= 64, u64::MAX, 1u64 << platform_bits);
        let reserved = MAX_RESERVED.min(space / 16);
        Mem::builder()
            .platform(platform_bits)
            .space(space)
            .reserved(reserved)
            .next(reserved)
            .max_heap(max_heap)
            .build()
    }

    /// Make a block of zeros. A heap block counts against the cap; cells
    /// and static blocks do not. A zero-size block is a real block.
    ///
    /// # Arguments
    ///
    /// * `size` - The octet count.
    /// * `origin` - Where the block comes from.
    /// * `owner` - The call depth that makes a cell; 0 otherwise.
    ///
    /// # Returns
    ///
    /// The start of the block, or `None` when the heap cap or the address
    /// space has no room for it.
    pub fn alloc(&mut self, size: u64, origin: Origin, owner: usize) -> Option<u64>
    {
        let heap = origin == Origin::Heap;
        ret_if!(heap && self.live_heap.saturating_add(size) > self.max_heap, None);
        self.place(size, origin, owner)
    }

    /// Make a block of zeros with no check of the heap cap; a heap block
    /// still adds to the live heap octets.
    ///
    /// # Arguments
    ///
    /// * `size` - The octet count.
    /// * `origin` - Where the block comes from.
    /// * `owner` - The call depth that makes a cell; 0 otherwise.
    ///
    /// # Returns
    ///
    /// The start of the block, or `None` when the address space has no room
    /// for it.
    fn place(&mut self, size: u64, origin: Origin, owner: usize) -> Option<u64>
    {
        let heap = origin == Origin::Heap;
        let data = zeros(size)?;
        let start = self.fresh_start(size).or_else(|| self.reuse(size))?;
        let block = Block::builder()
            .start(start)
            .len(size)
            .data(data)
            .origin(origin)
            .live(true)
            .owner(owner)
            .build();
        self.table.insert(start, block);
        do_if!(heap, self.live_heap += size);
        Some(start)
    }

    /// Resize a heap block: make a new block, keep the shorter of the two
    /// lengths of contents, zero the rest and free the old block.
    ///
    /// # Arguments
    ///
    /// * `addr` - The start of a live heap block, or 0 for a new block.
    /// * `size` - The new octet count.
    ///
    /// # Returns
    ///
    /// The start of the new block, or the trap: `BadFree` or `DoubleFree`
    /// as for [`Mem::free`], `OutOfMem` when the live heap with the old
    /// length swapped for the new one passes the cap, or when the address
    /// space has no room for the new block.
    pub fn realloc(&mut self, addr: u64, size: u64) -> Result<u64, TrapKind>
    {
        let full = TrapKind::OutOfMem { size };
        ret_if!(addr == 0, self.alloc(size, Origin::Heap, 0).ok_or(full));
        self.check_free(addr)?;
        let old = self.table.get(&addr).map_or(0, |b| b.len);
        let after = (self.live_heap - old).saturating_add(size);
        ret_if!(after > self.max_heap, Err(full));
        let start = self.place(size, Origin::Heap, 0).ok_or(full)?;
        self.keep_prefix(addr, start);
        self.retire(addr);
        Ok(start)
    }

    /// Free a heap block and put it in quarantine.
    ///
    /// # Arguments
    ///
    /// * `addr` - The start of a live heap block; 0 does nothing.
    ///
    /// # Returns
    ///
    /// `Ok`, `DoubleFree` for a freed heap block, or `BadFree` for an
    /// address that is not the start of a heap block.
    pub fn free(&mut self, addr: u64) -> Result<(), TrapKind>
    {
        ret_if!(addr == 0, Ok(()));
        self.check_free(addr)?;
        self.retire(addr);
        Ok(())
    }

    /// Free the cells a call made, at the end of the call, and put them in
    /// quarantine.
    ///
    /// # Arguments
    ///
    /// * `starts` - The start of each cell.
    pub fn release(&mut self, starts: &[u64])
    {
        do_for!(s in starts, self.retire(*s));
    }

    /// Read a value.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `kind` - The kind to read.
    ///
    /// # Returns
    ///
    /// The value, or the trap the access hits.
    pub fn load(&self, addr: u64, kind: DataKind) -> Result<Value, TrapKind>
    {
        let (start, span) = self.check(addr, kind)?;
        let raw = self.table.get(&start).and_then(|b| b.data.get(span));
        from_raw(kind, raw.unwrap_or_default(), self.platform)
            .map_err(|_| TrapKind::BadValue { addr, kind: kind.name() })
    }

    /// Write a value; an address takes `platform / 8` octets.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `v` - The value.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap the access hits.
    pub fn store(&mut self, addr: u64, v: Value) -> Result<(), TrapKind>
    {
        let (start, span) = self.check(addr, v.kind())?;
        let size = span.len() as u32;
        let raw = to_raw(v, self.platform);
        let slot = self.table.get_mut(&start).and_then(|b| b.data.get_mut(span));
        let slot = slot.ok_or(TrapKind::OutOfBounds { addr, size })?;
        slot.copy_from_slice(&raw);
        Ok(())
    }

    /// Whether two addresses lie in `[start, start + len]` of one live
    /// block, the one-past-the-end address included, so a pointer formed at
    /// the end of a block can be subtracted from its start and `p - p` on a
    /// zero-size block holds. The red zone keeps the end of a block from
    /// being the start of the next one. `load` and `store` keep the
    /// half-open bounds.
    ///
    /// # Arguments
    ///
    /// * `a` - The first address.
    /// * `b` - The second address.
    ///
    /// # Returns
    ///
    /// `true` when one live block reaches both.
    pub fn same_block(&self, a: u64, b: u64) -> bool
    {
        let Some((_, block)) = self.table.range(..=a).next_back()
        else
        {
            return false;
        };
        block.live && block.reaches(a) && block.reaches(b)
    }

    /// The block, live or freed, whose range holds an address or ends just
    /// before it, so an access one past the end still names its block.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// The block, or `None` when the address is outside each block.
    pub fn block_near(&self, addr: u64) -> Option<&Block>
    {
        let (_, block) = self.table.range(..=addr).next_back()?;
        tern!(block.reaches(addr), Some(block), None)
    }

    /// Each block, live or freed, in address order.
    ///
    /// # Returns
    ///
    /// The blocks.
    pub fn blocks(&self) -> impl Iterator<Item = &Block> { self.table.values() }

    /// The heap blocks never freed.
    ///
    /// # Returns
    ///
    /// The start and length of each live heap block, in address order.
    pub fn leaks(&self) -> Vec<(u64, u64)>
    {
        self.table
            .values()
            .filter(|b| b.live && b.origin == Origin::Heap)
            .map(|b| (b.start, b.len))
            .collect()
    }

    /// The live heap octets.
    ///
    /// # Returns
    ///
    /// The sum of the lengths of the live heap blocks.
    pub fn live_heap(&self) -> u64 { self.live_heap }

    /// Place a new block above each block made so far.
    ///
    /// # Arguments
    ///
    /// * `size` - The octet count.
    ///
    /// # Returns
    ///
    /// The start, or `None` when the block would pass the end of the
    /// address space.
    fn fresh_start(&mut self, size: u64) -> Option<u64>
    {
        let start =
            self.next.max(self.reserved).checked_next_multiple_of(BLOCK_ALIGN)?;
        let end = start.checked_add(size).filter(|e| *e <= self.space)?;
        self.next = end.saturating_add(RED_ZONE);
        Some(start)
    }

    /// Take the oldest freed block long enough out of quarantine.
    ///
    /// # Arguments
    ///
    /// * `size` - The octet count.
    ///
    /// # Returns
    ///
    /// The start of the freed block, or `None` when none fits.
    fn reuse(&mut self, size: u64) -> Option<u64>
    {
        let fits = |s: &u64| self.table.get(s).is_some_and(|b| b.len >= size);
        let at = self.freed.iter().position(fits)?;
        self.freed.remove(at)
    }

    /// Check that an address is the start of a live heap block.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// `Ok`, `DoubleFree` for a freed heap block, or `BadFree`.
    fn check_free(&self, addr: u64) -> Result<(), TrapKind>
    {
        let block = self.table.get(&addr).filter(|b| b.origin == Origin::Heap);
        let block = block.ok_or(TrapKind::BadFree { addr })?;
        tern!(block.live, Ok(()), Err(TrapKind::DoubleFree { addr }))
    }

    /// Mark a block freed, drop its contents and put it in quarantine.
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the block.
    fn retire(&mut self, start: u64)
    {
        let Some(block) = self.table.get_mut(&start)
        else
        {
            return;
        };
        block.live = false;
        let heap = block.origin == Origin::Heap;
        do_if!(heap, self.live_heap -= block.len);
        block.data = Vec::new();
        self.freed.push_back(start);
    }

    /// Write the head of one block into another, as far as both reach.
    ///
    /// # Arguments
    ///
    /// * `from` - The start of the source block.
    /// * `to` - The start of the target block.
    fn keep_prefix(&mut self, from: u64, to: u64)
    {
        let head = self.table.get(&from).map(|b| b.data.clone()).unwrap_or_default();
        let Some(target) = self.table.get_mut(&to)
        else
        {
            return;
        };
        let n = head.len().min(target.data.len());
        target.data[..n].copy_from_slice(&head[..n]);
    }

    /// The block whose range holds an address, live or freed.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// The block, or `None` when the address is outside each block.
    fn block_at(&self, addr: u64) -> Option<&Block>
    {
        let (_, block) = self.table.range(..=addr).next_back()?;
        tern!(block.holds(addr), Some(block), None)
    }

    /// Check an access of a kind at an address: the reserved region, the
    /// alignment, the bounds of the block that holds it, then its state.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `kind` - The kind of the access.
    ///
    /// # Returns
    ///
    /// The start of the block and the range of its contents the access
    /// covers, or the trap it hits.
    fn check(&self, addr: u64, kind: DataKind)
    -> Result<(u64, Range<usize>), TrapKind>
    {
        let size = kind.size(self.platform);
        let align = kind.align(self.platform).max(1);
        ret_if!(addr < self.reserved, Err(TrapKind::NullDeref { addr }));
        let aligned = addr.is_multiple_of(u64::from(align));
        ret_if!(!aligned, Err(TrapKind::Misaligned { addr, align }));
        let outside = TrapKind::OutOfBounds { addr, size };
        let block = self.block_at(addr).ok_or_else(|| outside.clone())?;
        ret_if!(addr.saturating_add(u64::from(size)) > block.end(), Err(outside));
        ret_if!(!block.live, Err(TrapKind::UseAfterFree { addr }));
        let from = (addr - block.start) as usize;
        Ok((block.start, from..from + size as usize))
    }
}
