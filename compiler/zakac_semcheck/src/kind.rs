use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use serde::Deserialize;
use serde::Serialize;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;

use crate::enums::EnumKind;
use crate::fn_kind::FnKind;
use crate::fn_kind::FnParam;
use crate::fn_kind::bracket_step;
use crate::fn_kind::is_fn_text;
use crate::fn_kind::parse_fn_with;
use crate::fn_kind::unwrap_parens;
use crate::structs::StructKind;
use crate::structs::StructTable;
use crate::structs::parse_sig;

/// The name of the struct a function that can fail gives back; no written
/// name starts with `%`.
pub const FALLIBLE: &str = "%fallible";

/// The name of the struct kind of a closure record.
pub const CLOSURE: &str = "%closure";

/// The name of the struct that holds a ဇယား. Its signature lists its fields
/// in full, as a result's does, so it never takes a name of its own in a
/// signature, and two ဇယား kinds written out in one signature never clash.
pub const TABLE: &str = "%map";

/// The name of the struct of one row of a ဇယား, written out in full as
/// [`TABLE`] is.
pub const ROW: &str = "%row";

/// The fields of the struct of a ဇယား, in order: its rows in add order, its
/// slots, and its count of live rows.
pub const TABLE_FIELDS: [&str; 3] = ["%rows", "%slots", "%live"];

/// The fields of the struct of a row, in order: the hash of its index,
/// whether it is live, its index and its value.
pub const ROW_FIELDS: [&str; 4] = ["%hash", "%live", "%index", "%value"];

/// Bit counts an integer kind can have.
const INT_BITS: &[u16] = &[8, 16, 32, 64];
/// Bit counts a floating-point kind can have.
const FLOAT_BITS: &[u16] = &[32, 64];

/// Width of an integer kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Width
{
    /// Platform size; the check options give its bit count.
    Platform,
    /// A fixed bit count.
    Bits(u16),
}

/// Sign and width of an integer kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntKind
{
    /// `true` for a signed integer.
    pub signed: bool,
    /// The width.
    pub width: Width,
}

impl IntKind
{
    /// Make an integer kind.
    ///
    /// # Arguments
    ///
    /// * `signed` - `true` for a signed integer.
    /// * `width` - The width.
    ///
    /// # Returns
    ///
    /// The kind.
    pub const fn new(signed: bool, width: Width) -> Self { Self { signed, width } }

    /// The bit count of the kind.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The bit count.
    pub fn bits(self, platform: u16) -> u16
    {
        match self.width
        {
            Width::Platform => platform,
            Width::Bits(n) => n,
        }
    }
}

/// The kind at the bottom of a pointer kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Base
{
    /// An integer.
    Int(IntKind),
    /// A floating-point number with this bit count.
    Float(u16),
    /// A character.
    Char,
    /// A boolean.
    Bool,
    /// A bare `လိပ်စာ`, whose target kind is not known.
    Opaque,
    /// A list, for a pointer to a whole list.
    List(&'static ListKind),
    /// A struct, for a pointer to a whole struct.
    Struct(&'static StructKind),
    /// Text or a plain list, for a pointer to one.
    Seq(&'static SeqKind),
    /// An enumeration.
    Enum(&'static EnumKind),
    /// A function kind, for a pointer to a function value.
    Fn(&'static FnKind),
}

impl Base
{
    /// The plain kind of this base.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for [`Base::Opaque`], which is an address itself.
    pub fn scalar(self) -> Option<DataKind>
    {
        match self
        {
            Base::Int(n) => Some(DataKind::Int(n)),
            Base::Float(bits) => Some(DataKind::Float(bits)),
            Base::Char => Some(DataKind::Char),
            Base::Bool => Some(DataKind::Bool),
            Base::Opaque => None,
            Base::List(l) => Some(DataKind::List(l)),
            Base::Struct(s) => Some(DataKind::Struct(s)),
            Base::Seq(s) => Some(DataKind::Seq(s)),
            Base::Enum(e) => Some(DataKind::Enum(e)),
            Base::Fn(f) => Some(DataKind::Fn(f)),
        }
    }
}

/// A list kind: `len` cells of one kind, next to each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListKind
{
    /// The kind of each cell.
    pub cell: DataKind,
    /// The number of cells.
    pub len: u32,
}

/// Each list kind made so far, so a kind is made once and remains `Copy`.
static LISTS: OnceLock<Mutex<HashSet<&'static ListKind>>> = OnceLock::new();

/// A managed run of cells, whose count is known at run time: a plain list,
/// or text, whose cells are u8s that hold UTF-8. A value is three words: the
/// address of the first cell, the count, and the room of its heap block in
/// octets. A room of 0 means the value owns no block: its cells sit in
/// static storage, it holds no cells, or it reads cells it was lent. The
/// first edit gives it a block of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeqKind
{
    /// The kind of each cell.
    pub cell: DataKind,
    /// Whether the cells are u8s that hold UTF-8: `စာသား`, whose cells
    /// cannot be written one at a time and which prints as text.
    pub text: bool,
    /// Whether it is the box of a variable that a closure shares: one
    /// cell, shared among its duplicates and never lent or copied on a write.
    pub boxed: bool,
}

/// The kind of text: u8 cells that hold UTF-8.
static TEXT_SEQ: SeqKind = SeqKind { cell: DataKind::U8, text: true, boxed: false };

/// Each plain list kind made so far, so a kind is made once and remains
/// `Copy`.
static SEQS: OnceLock<Mutex<HashSet<&'static SeqKind>>> = OnceLock::new();

/// A pointer kind: `depth` levels of address over a base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PtrKind
{
    /// The number of address levels, at least 1. An opaque base counts
    /// itself, so `ptr` has depth 1 and `ptr*` depth 2.
    pub depth: u8,
    /// The kind at the bottom.
    pub base: Base,
    /// Whether the outer level is a borrow: it reads a block that an owner
    /// holds.
    pub borrowed: bool,
    /// The raw levels as a bitmask: bit 0 is the level nearest the base and
    /// bit `depth - 1` the outer one. A level without its bit is an owner.
    pub raw: u8,
}

impl PtrKind
{
    /// The owner kind `depth` levels over `base`.
    ///
    /// # Arguments
    ///
    /// * `depth` - The number of address levels, at least 1.
    /// * `base` - The kind at the bottom.
    ///
    /// # Returns
    ///
    /// The kind with no borrow mark and no raw level.
    pub const fn owner(depth: u8, base: Base) -> PtrKind
    {
        PtrKind { depth, base, borrowed: false, raw: 0 }
    }

    /// The mask of the outer level in [`PtrKind::raw`].
    ///
    /// # Returns
    ///
    /// The bit, or 0 when the depth is past the mask width.
    fn outer_bit(self) -> u8
    {
        let shift = u32::from(self.depth.saturating_sub(1));
        1u8.checked_shl(shift).unwrap_or(0)
    }

    /// Whether the outer level is a borrow.
    ///
    /// # Returns
    ///
    /// `true` for a borrow.
    pub fn is_borrowed(self) -> bool { self.borrowed }

    /// Whether the outer level is a raw address.
    ///
    /// # Returns
    ///
    /// `true` for a raw outer level.
    pub fn is_raw_outer(self) -> bool { self.raw & self.outer_bit() != 0 }

    /// Whether the outer level is an owner.
    ///
    /// # Returns
    ///
    /// `true` when the level is neither a borrow nor raw.
    pub fn is_owner(self) -> bool { !self.borrowed && !self.is_raw_outer() }

    /// The kind with its outer level turned into a borrow.
    ///
    /// # Returns
    ///
    /// The borrowed kind; a raw outer level cannot be borrowed, so it is
    /// returned as it is.
    pub fn as_borrowed(self) -> PtrKind
    {
        PtrKind { borrowed: !self.is_raw_outer(), ..self }
    }

    /// The kind with its outer level turned into an owner.
    ///
    /// # Returns
    ///
    /// The kind without a borrow mark.
    pub fn as_owner(self) -> PtrKind { PtrKind { borrowed: false, ..self } }

    /// The kind with its outer level turned into a raw address.
    ///
    /// # Returns
    ///
    /// The kind with the outer raw bit set and no borrow mark.
    pub fn with_outer_raw(self) -> PtrKind
    {
        PtrKind { borrowed: false, raw: self.raw | self.outer_bit(), ..self }
    }

    /// The kind without the borrow mark, which the shape of the block
    /// does not depend on.
    ///
    /// # Returns
    ///
    /// The owner flavour of the outer level, each raw level kept.
    pub fn shape(self) -> PtrKind { self.as_owner() }
}

/// A data kind as the checker sees it: a written kind with its defaults
/// filled in. Artifacts store it as its short name, such as `i8`, `usize`,
/// `i32*` or `ptr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum DataKind
{
    /// An integer.
    Int(IntKind),
    /// A floating-point number with this bit count.
    Float(u16),
    /// A character.
    Char,
    /// A boolean.
    Bool,
    /// A pointer.
    Ptr(PtrKind),
    /// The kind of `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, which converts to each pointer kind.
    Null,
    /// A list of cells.
    List(&'static ListKind),
    /// A struct of named fields.
    Struct(&'static StructKind),
    /// A plain list or text: three words, see [`SeqKind`].
    Seq(&'static SeqKind),
    /// An enumeration: a member, held as an `isize`.
    Enum(&'static EnumKind),
    /// A function value: one word, the function it calls or none.
    Fn(&'static FnKind),
}

/// The reason a kind has no initial value, `ကနဦးတန်ဖိုး`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitialGap
{
    /// A function value.
    Function,
    /// A handle, which C alone makes.
    Handle,
    /// An enumeration with no member marked `ကနဦးတန်ဖိုး`.
    Member(&'static EnumKind),
    /// A result, which must hold its good value or its failure.
    Result,
    /// A part inside the kind with none: the name of a field, or the kind of
    /// the cells of a list.
    Part(String),
}

impl DataKind
{
    /// `ကိန်းပြည့်`, the default integer kind.
    pub const ISIZE: DataKind = DataKind::Int(IntKind::new(true, Width::Platform));
    /// `လက္ခဏာမဲ့ ကိန်းပြည့်`
    pub const USIZE: DataKind = DataKind::Int(IntKind::new(false, Width::Platform));
    /// `ဒဿမကိန်း`, the default floating-point kind.
    pub const F64: DataKind = DataKind::Float(64);
    /// `လိပ်စာ`, the opaque pointer.
    pub const OPAQUE: DataKind = DataKind::Ptr(PtrKind::owner(1, Base::Opaque));
    /// Owned text.
    pub const TEXT: DataKind = DataKind::Seq(&TEXT_SEQ);
    /// `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`, the cell of a text.
    pub const U8: DataKind = DataKind::Int(IntKind::new(false, Width::Bits(8)));

    /// The data kind a written kind phrase names.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind phrase.
    /// * `structs` - The kind of each declared struct.
    ///
    /// # Returns
    ///
    /// The kind; a missing sign means signed, a missing bit count means
    /// platform size for integers and 64 for floats. Each pointer level of
    /// the phrase adds one address level. A struct name that is not declared
    /// stands for `isize`; the check reports it.
    pub fn from_ast(kind: &Kind, structs: &StructTable) -> DataKind
    {
        let width = kind.bits.map_or(Width::Platform, Width::Bits);
        let base = match kind.base
        {
            BaseKind::Int =>
            {
                Base::Int(IntKind::new(kind.signed.unwrap_or(true), width))
            }
            BaseKind::Float => Base::Float(kind.bits.unwrap_or(64)),
            BaseKind::Char => Base::Char,
            BaseKind::Bool => Base::Bool,
            BaseKind::Text => Base::Seq(&TEXT_SEQ),
            BaseKind::Opaque => Base::Opaque,
            BaseKind::Named | BaseKind::Result => named_base(kind, structs),
            BaseKind::Fn => Base::Fn(fn_base(kind, structs)),
            // A ဇယား is the struct `DataKind::table` makes, and its levels
            // wrap that struct, as a result kind's do.
            BaseKind::Table => Base::Int(IntKind::new(true, width)),
        };
        let skip = usize::from(kind.base == BaseKind::Opaque);
        // A result kind is the struct `DataKind::fallible` makes, and its
        // levels wrap that struct.
        let side =
            |k: &Option<Kind>| k.as_ref().map(|k| DataKind::from_ast(k, structs));
        let result = kind.parts.as_deref().map(|p| match (p.optional, side(&p.ok))
        {
            (true, Some(some)) => DataKind::optional(some),
            _ => DataKind::fallible(side(&p.ok), side(&p.err)),
        });
        let table = kind.table.as_deref().map(|t| {
            DataKind::table(
                DataKind::from_ast(&t.index, structs),
                DataKind::from_ast(&t.value, structs),
            )
        });
        let plain = result
            .or(table)
            .unwrap_or_else(|| DataKind::over(base, u8::try_from(skip).unwrap_or(1)));
        kind.wraps.iter().skip(skip).fold(plain, |inner, w| match w
        {
            Wrap::Ptr => inner.pointer_to(),
            Wrap::List(Cells::Fixed(n)) => DataKind::list_of(inner, *n),
            // The check writes each count back as digits; one still open
            // waits for a later run, or failed with a diagnostic, and stands
            // as 0 cells meanwhile, a count no written count has, so a read
            // of it knows to wait.
            Wrap::List(Cells::Expr(_)) => DataKind::list_of(inner, 0),
            Wrap::List(Cells::Inferred) => DataKind::seq_of(inner),
        })
    }

    /// The kind with its outer address level turned into a raw one.
    ///
    /// # Returns
    ///
    /// The kind for a pointer, the kind itself for each other kind.
    pub(crate) fn with_outer_raw(self) -> DataKind
    {
        tern!(let DataKind::Ptr(p) = self, DataKind::Ptr(p.with_outer_raw()), self)
    }

    /// The list kind of `len` cells of `cell`. Each list kind is made once
    /// and kept for the rest of the run.
    ///
    /// # Arguments
    ///
    /// * `cell` - The kind of each cell.
    /// * `len` - The number of cells.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn list_of(cell: DataKind, len: u32) -> DataKind
    {
        let wanted = ListKind { cell, len };
        let lists = LISTS.get_or_init(Mutex::default);
        let mut lists = lists.lock().unwrap_or_else(PoisonError::into_inner);
        ret_if!(let Some(found) = lists.get(&wanted), DataKind::List(found));
        let made: &'static ListKind = Box::leak(Box::new(wanted));
        lists.insert(made);
        DataKind::List(made)
    }

    /// The plain list kind of cells of `cell`, made once and kept for the
    /// rest of the run.
    ///
    /// # Arguments
    ///
    /// * `cell` - The kind of each cell.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn seq_of(cell: DataKind) -> DataKind
    {
        DataKind::seq_kind(SeqKind { cell, text: false, boxed: false })
    }

    /// The kind of the box of a variable that a closure shares: a plain list
    /// of one cell that each duplicate shares and nothing lends or copies.
    ///
    /// # Arguments
    ///
    /// * `cell` - The kind of the variable.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn box_of(cell: DataKind) -> DataKind
    {
        DataKind::seq_kind(SeqKind { cell, text: false, boxed: true })
    }

    /// The interned plain list kind with these parts.
    ///
    /// # Arguments
    ///
    /// * `wanted` - The parts.
    ///
    /// # Returns
    ///
    /// The kind.
    fn seq_kind(wanted: SeqKind) -> DataKind
    {
        let seqs = SEQS.get_or_init(Mutex::default);
        let mut seqs = seqs.lock().unwrap_or_else(PoisonError::into_inner);
        ret_if!(let Some(found) = seqs.get(&wanted), DataKind::Seq(found));
        let made: &'static SeqKind = Box::leak(Box::new(wanted));
        seqs.insert(made);
        DataKind::Seq(made)
    }

    /// The plain list part of the kind.
    ///
    /// # Returns
    ///
    /// The parts, or `None` for a kind that is no plain list.
    pub fn as_seq(self) -> Option<&'static SeqKind>
    {
        tern!(let DataKind::Seq(s) = self, Some(s).filter(|s| !s.text), None)
    }

    /// Whether the kind is a plain list.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn is_seq(self) -> bool { self.as_seq().is_some() }

    /// The struct of a struct kind, or of an object: the box of a struct
    /// marked `မျှဝေနိုင်သော`, whose fields are read through the box.
    ///
    /// # Returns
    ///
    /// The struct, or `None` for each other kind.
    pub fn object_struct(self) -> Option<&'static StructKind>
    {
        self.as_struct().or_else(|| self.as_object())
    }

    /// The struct of an object: the box of a struct marked `မျှဝေနိုင်သော`.
    ///
    /// # Returns
    ///
    /// The struct, or `None` for each other kind.
    pub fn as_object(self) -> Option<&'static StructKind>
    {
        self.as_seq().filter(|s| s.boxed).and_then(|s| s.cell.as_struct())
    }

    /// The struct of a handle: a value of a kind from outside the program,
    /// an object whose one field holds the address of the struct C keeps.
    ///
    /// # Returns
    ///
    /// The struct, or `None` for each other kind.
    pub fn as_external(self) -> Option<&'static StructKind>
    {
        self.as_object().filter(|s| s.is_external())
    }

    /// The kind of the address a handle holds: an opaque raw pointer, which
    /// the VM passes on as it is.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn handle_pointer() -> DataKind
    {
        DataKind::Ptr(PtrKind::owner(1, Base::Opaque).with_outer_raw())
    }

    /// Whether the kind is the box of a variable that a closure shares.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn is_box(self) -> bool { self.as_seq().is_some_and(|s| s.boxed) }

    /// The kind of each cell of a list of each form.
    ///
    /// # Returns
    ///
    /// The cell kind, or `None` for a kind with no cells.
    pub fn cell_kind(self) -> Option<DataKind>
    {
        ret_if!(let Some(l) = self.as_list(), Some(l.cell));
        self.as_seq().map(|s| s.cell)
    }

    /// Whether the kind is owned text.
    ///
    /// # Returns
    ///
    /// `true` for `စာသား`.
    pub fn is_text(self) -> bool { matches!(self, DataKind::Seq(s) if s.text) }

    /// Whether a value of the kind holds owned text, a plain list or a
    /// function value, which can own a heap block that its place frees as
    /// it ends: the text, the list or the closure record itself, or a
    /// counted list or a struct with one inside. A duplicate copies each such
    /// block.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_text(self) -> bool
    {
        ret_if!(let Some(e) = self.as_enum(), e.under().holds_text());
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_text());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_text()));
        self.is_text() || self.is_seq() || self.is_fn()
    }

    /// Whether a value of the kind holds a function value, which can be a
    /// closure with a record on the heap: the value itself, or a counted
    /// list or a struct with one inside.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_fn(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_fn());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_fn()));
        self.is_fn()
    }

    /// Whether a value of the kind holds a plain list: the list itself, or a
    /// counted list or a struct with one inside.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_seq(self) -> bool
    {
        ret_if!(let Some(e) = self.as_enum(), e.under().holds_seq());
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_seq());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_seq()));
        self.is_seq()
    }

    /// Whether a value of the kind holds an owner pointer, whose block the
    /// program must free or move on: the pointer itself, or a list or struct
    /// with one in it.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_owner_ptr(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_owner_ptr());
        // A box is shared, never moved, and can hold its own struct, so the
        // walk stops there.
        ret_if!(let Some(l) = self.as_seq().filter(|s| !s.boxed), l.cell.holds_owner_ptr());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_owner_ptr()));
        self.is_owner_ptr()
    }

    /// The list part of the kind.
    ///
    /// # Returns
    ///
    /// The cell kind and count, or `None` for a kind that is no list.
    pub fn as_list(self) -> Option<&'static ListKind>
    {
        tern!(let DataKind::List(l) = self, Some(l), None)
    }

    /// Whether the kind is a list.
    ///
    /// # Returns
    ///
    /// `true` for a list kind.
    pub fn is_list(self) -> bool { matches!(self, DataKind::List(_)) }

    /// The kind as storage holds it: an enumeration as its value kind,
    /// level after level; each other kind as it is.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn repr(self) -> DataKind
    {
        let mut kind = self;
        while let Some(e) = kind.as_enum()
        {
            kind = e.under();
        }
        kind
    }

    /// Whether a value of the kind holds an enumeration inline: the
    /// enumeration itself, or a counted list or a struct with one inside.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_enum(self) -> bool
    {
        ret_true!(self.as_enum().is_some());
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_enum());
        self.as_struct().is_some_and(|s| s.fields().iter().any(|(_, k)| k.holds_enum()))
    }

    /// The enumeration part of the kind.
    ///
    /// # Returns
    ///
    /// The enumeration, or `None` for a kind that is no enumeration.
    pub fn as_enum(self) -> Option<&'static EnumKind>
    {
        tern!(let DataKind::Enum(e) = self, Some(e), None)
    }

    /// The struct part of the kind.
    ///
    /// # Returns
    ///
    /// The struct, or `None` for a kind that is no struct.
    pub fn as_struct(self) -> Option<&'static StructKind>
    {
        tern!(let DataKind::Struct(s) = self, Some(s), None)
    }

    /// Whether the kind is a struct.
    ///
    /// # Returns
    ///
    /// `true` for a struct kind.
    pub fn is_struct(self) -> bool { matches!(self, DataKind::Struct(_)) }

    /// Whether a value of the kind is a block of other values: a list, a
    /// struct, or the three words of text or a plain list. Such a value lives
    /// in mem, and a duplicate takes the whole block.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn is_aggregate(self) -> bool
    {
        self.is_list() || self.is_struct() || self.is_text() || self.is_seq()
    }

    /// The kind `depth` address levels over `base`.
    ///
    /// # Arguments
    ///
    /// * `base` - The kind at the bottom.
    /// * `depth` - The number of address levels.
    ///
    /// # Returns
    ///
    /// The plain kind of `base` for depth 0, else a pointer kind; an opaque
    /// base has at least one level.
    pub fn over(base: Base, depth: u8) -> DataKind
    {
        let plain = base.scalar().filter(|_| depth == 0);
        plain.unwrap_or(DataKind::Ptr(PtrKind::owner(depth.max(1), base)))
    }

    /// Read a short name such as `i8`, `usize`, `char`, `i32*`, `i32&`,
    /// `i32^`, `ptr`, `ptr&`, `rawptr`, `isize[3]`, `isize[]`, `text` or
    /// `null`: a base, then one mark for each level in the order of the
    /// Burmese words.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for an unknown name, a borrowed raw level, or a
    /// level over a borrow.
    pub fn parse(name: &str) -> Option<DataKind>
    {
        ret_if!(name == "null", Some(DataKind::Null));
        let (base, marks) = split_marks(name);
        let base = parse_base(base)?;
        DataKind::wrap_marks(base, marks)
    }

    /// Wrap a kind in the levels its marks name, from left to right: `*` an
    /// owner, `^` a raw address, `&` a borrow, `[n]` a list of `n` cells,
    /// `[]` a plain list, and `@` an object. On `ptr`, which is an address
    /// itself, `&` borrows it and adds no level.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind under the marks.
    /// * `marks` - The marks.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for a mark that does not read.
    pub fn wrap_marks(kind: DataKind, marks: &str) -> Option<DataKind>
    {
        let mut kind = kind;
        let mut rest = marks;
        while let Some(ch) = rest.chars().next()
        {
            let len = tern!(ch == '[', rest.find(']')? + 1, ch.len_utf8());
            kind = wrap_mark(kind, &rest[..len])?;
            rest = &rest[len..];
        }
        Some(kind)
    }

    /// The short name, for messages.
    ///
    /// # Returns
    ///
    /// The name, such as `i8`, `usize`, `f64`, `char`, `bool`, `i32*`,
    /// `ptr` or `null`; a struct is its name alone.
    pub fn name(self) -> String { self.spell(false) }

    /// The name that reads back with [`DataKind::parse`], for artifacts.
    ///
    /// # Returns
    ///
    /// The short name, with each struct written as its signature.
    pub fn sig_name(self) -> String { self.spell(true) }

    /// The kind a function that can fail gives back: a struct with a `tag`
    /// (1 for a good result, 0 for a failure, so the blank struct, the
    /// default of each result, is a failure), the good result `ok` and the
    /// error value `err`, each when that side holds a value. Its name starts
    /// with `%`, which no written name can.
    ///
    /// # Arguments
    ///
    /// * `ok` - The result kind, when the good side holds a value.
    /// * `err` - The error kind, when the error side holds a value.
    ///
    /// # Returns
    ///
    /// The struct kind.
    pub fn fallible(ok: Option<DataKind>, err: Option<DataKind>) -> DataKind
    {
        let shape = parse_sig(&DataKind::fallible_sig(ok, err));
        DataKind::Struct(shape.expect("the signature of a fallible result reads back"))
    }

    /// The signature of the struct [`DataKind::fallible`] makes.
    ///
    /// # Arguments
    ///
    /// * `ok` - The kind of the good result, or `None` for none.
    /// * `err` - The error kind, or `None` for none.
    ///
    /// # Returns
    ///
    /// The signature.
    pub(crate) fn fallible_sig(ok: Option<DataKind>, err: Option<DataKind>) -> String
    {
        let ok = ok.map_or(String::new(), |k| format!(",ok:{}", k.spell_inner(true)));
        let err =
            err.map_or(String::new(), |k| format!(",err:{}", k.spell_inner(true)));
        format!("{FALLIBLE}{{tag:u8{ok}{err}}}")
    }

    /// The optional kind of `some`: the struct `DataKind::fallible` makes, with
    /// the data in a field `some` in place of `ok`, so the two kinds differ.
    ///
    /// # Arguments
    ///
    /// * `some` - The kind of the data.
    ///
    /// # Returns
    ///
    /// The struct kind.
    pub fn optional(some: DataKind) -> DataKind
    {
        let shape = parse_sig(&DataKind::optional_sig(some));
        DataKind::Struct(shape.expect("the signature of an optional reads back"))
    }

    /// The signature of the struct [`DataKind::optional`] makes.
    ///
    /// # Arguments
    ///
    /// * `some` - The kind of the data.
    ///
    /// # Returns
    ///
    /// The signature.
    pub(crate) fn optional_sig(some: DataKind) -> String
    {
        format!("{FALLIBLE}{{tag:u8,some:{}}}", some.spell_inner(true))
    }

    /// The kind of a ဇယား, `<index> ဖြင့် ညွှန်းသော <value> ဇယား`: a struct of
    /// its rows (a plain list of row structs, in add order), its slots (a
    /// plain list of `u32`, 0 for a slot that was never used, 1 for one whose
    /// row was removed, and 2 more than the row number for each other), and
    /// its count of live rows. The blank struct is a ဇယား with no rows.
    ///
    /// # Arguments
    ///
    /// * `index` - The index kind.
    /// * `value` - The value kind.
    ///
    /// # Returns
    ///
    /// The struct kind.
    pub fn table(index: DataKind, value: DataKind) -> DataKind
    {
        let shape = parse_sig(&DataKind::table_sig(index, value));
        DataKind::Struct(shape.expect("the signature of a ဇယား reads back"))
    }

    /// The signature of the struct [`DataKind::table`] makes.
    ///
    /// # Arguments
    ///
    /// * `index` - The index kind.
    /// * `value` - The value kind.
    ///
    /// # Returns
    ///
    /// The signature.
    fn table_sig(index: DataKind, value: DataKind) -> String
    {
        table_text(&index.spell_inner(true), &value.spell_inner(true))
    }

    /// Whether the kind is a ဇယား.
    ///
    /// # Returns
    ///
    /// `true` for a ဇယား.
    pub fn is_table(self) -> bool { self.as_struct().is_some_and(|s| s.name == TABLE) }

    /// The kind of each row of a ဇယား.
    ///
    /// # Returns
    ///
    /// The row struct kind, or `None` for a kind that is no ဇယား.
    pub fn table_row(self) -> Option<DataKind>
    {
        let shape = self.as_struct().filter(|s| s.name == TABLE)?;
        shape.field(TABLE_FIELDS[0]).and_then(|(_, k)| k.cell_kind())
    }

    /// The index kind and the value kind of a ဇယား.
    ///
    /// # Returns
    ///
    /// The two kinds, or `None` for a kind that is no ဇယား.
    pub fn table_parts(self) -> Option<(DataKind, DataKind)>
    {
        let row = self.table_row()?.as_struct()?;
        let part = |name| row.field(name).map(|(_, k)| k);
        Some((part(ROW_FIELDS[2])?, part(ROW_FIELDS[3])?))
    }

    /// The reason a kind cannot index the rows of a ဇယား: a part that cannot
    /// be hashed or compared as a whole value.
    ///
    /// # Returns
    ///
    /// The name of the kind that fails, or `None` for an index kind.
    pub fn index_gap(self) -> Option<String> { self.index_gap_in(&mut Vec::new()) }

    /// [`DataKind::index_gap`], with the structs the walk is inside.
    ///
    /// # Arguments
    ///
    /// * `inside` - The structs the walk is inside; one met again adds nothing.
    ///
    /// # Returns
    ///
    /// The name of the kind that fails, or `None`.
    fn index_gap_in(self, inside: &mut Vec<&'static StructKind>) -> Option<String>
    {
        let kind = self.repr();
        let whole = kind.is_table() || kind.fallible_parts().is_some();
        let shared = kind.is_fn() || kind.is_ptr() || kind.is_box();
        ret_if!(whole || shared, Some(kind.name()));
        let cell = kind.cell_kind().filter(|_| !kind.is_text());
        ret_if!(let Some(cell) = cell, cell.index_gap_in(inside));
        let shape = kind.as_struct()?;
        ret_if!(inside.iter().any(|s| std::ptr::eq(*s, shape)), None);
        inside.push(shape);
        let gap = shape.fields().iter().find_map(|(_, k)| k.index_gap_in(inside));
        inside.pop();
        gap
    }

    /// Whether a kind that can index a ဇယား holds a decimal number, whose
    /// rounding can make two written values one row or two.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_float(self) -> bool { self.holds_float_in(&mut Vec::new()) }

    /// [`DataKind::holds_float`], with the structs the walk is inside.
    ///
    /// # Arguments
    ///
    /// * `inside` - The structs the walk is inside; one met again adds nothing.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    fn holds_float_in(self, inside: &mut Vec<&'static StructKind>) -> bool
    {
        let kind = self.repr();
        ret_true!(matches!(kind, DataKind::Float(_)));
        let cell = kind.cell_kind().filter(|_| !kind.is_text());
        ret_if!(let Some(cell) = cell, cell.holds_float_in(inside));
        let Some(shape) = kind.as_struct()
        else
        {
            return false;
        };
        ret_false!(inside.iter().any(|s| std::ptr::eq(*s, shape)));
        inside.push(shape);
        let found = shape.fields().iter().any(|(_, k)| k.holds_float_in(inside));
        inside.pop();
        found
    }

    /// The name of a ဇယား kind, for a message: `map<text, isize>`.
    ///
    /// # Returns
    ///
    /// The name, or `None` for a kind that is no ဇယား.
    fn table_name(self) -> Option<String>
    {
        let (index, value) = self.table_parts()?;
        Some(format!("map<{}, {}>", index.name(), value.name()))
    }

    /// The reason the kind has no initial value, `ကနဦးတန်ဖိုး`: a kind has one
    /// when each of its parts has one. An optional stops the walk with its
    /// blank value, a plain list with no cells; a shared struct met again
    /// is left to the check of kinds that can never be built.
    ///
    /// # Returns
    ///
    /// The gap, or `None` when the kind has an initial value.
    pub fn initial_gap(self) -> Option<InitialGap> { self.gap_in(&mut Vec::new()) }

    /// [`DataKind::initial_gap`], with the structs the walk is inside.
    ///
    /// # Arguments
    ///
    /// * `inside` - The structs the walk is inside.
    ///
    /// # Returns
    ///
    /// The gap, or `None`.
    fn gap_in(self, inside: &mut Vec<&'static StructKind>) -> Option<InitialGap>
    {
        ret_if!(self.is_fn(), Some(InitialGap::Function));
        ret_if!(self.as_external().is_some(), Some(InitialGap::Handle));
        ret_if!(self.is_optional(), None);
        if let Some(e) = self.as_enum()
        {
            return tern!(e.initial().is_some(), None, Some(InitialGap::Member(e)));
        }
        if let Some(l) = self.as_list()
        {
            return l.cell.gap_in(inside).map(|_| InitialGap::Part(l.cell.name()));
        }
        if let Some(s) = self.as_seq()
        {
            return s.boxed.then(|| s.cell.gap_in(inside)).flatten();
        }
        let shape = self.as_struct()?;
        ret_if!(shape.name == FALLIBLE, Some(InitialGap::Result));
        ret_if!(inside.iter().any(|s| std::ptr::eq(*s, shape)), None);
        inside.push(shape);
        let gap = shape.fields().iter().find(|(_, k)| k.gap_in(inside).is_some());
        inside.pop();
        gap.map(|(name, _)| InitialGap::Part(name.clone()))
    }

    /// Whether the kind is an optional, `T ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`.
    ///
    /// # Returns
    ///
    /// `true` for an optional.
    pub fn is_optional(self) -> bool
    {
        let shape = self.as_struct().filter(|s| s.name == FALLIBLE);
        shape.is_some_and(|s| s.field("some").is_some())
    }

    /// The kind of the data an optional holds.
    ///
    /// # Returns
    ///
    /// The kind of its data, or `None` for a kind that is no optional.
    pub fn optional_data(self) -> Option<DataKind>
    {
        self.fallible_parts().filter(|_| self.is_optional()).and_then(|(some, _)| some)
    }

    /// The kind of the record of a closure: a struct of the code word, an
    /// info word that a native build points at the kind of the record, then
    /// one field for each capture, in the order of the list. Its name starts
    /// with `%`, which no written name can.
    ///
    /// # Arguments
    ///
    /// * `fields` - The kind of each capture field.
    ///
    /// # Returns
    ///
    /// The struct kind.
    pub fn closure_record(fields: &[DataKind]) -> DataKind
    {
        let each = fields.iter().enumerate();
        let fields: String =
            each.map(|(i, k)| format!(",c{i}:{}", k.spell_inner(true))).collect();
        let code = DataKind::Ptr(PtrKind::owner(1, Base::Opaque).with_outer_raw());
        let word = code.spell_inner(true);
        let sig = format!("{CLOSURE}{{code:{word},info:{word}{fields}}}");
        let shape = parse_sig(&sig);
        DataKind::Struct(shape.expect("the signature of a closure record reads back"))
    }

    /// The name of a result of a function that can fail, for a message:
    /// `isize result or text error`; `success` stands for a good side with
    /// no value and `failure` for an error side with no value.
    ///
    /// # Returns
    ///
    /// The name.
    fn fallible_name(self) -> String
    {
        let Some((ok, err)) = self.fallible_parts()
        else
        {
            return String::new();
        };
        let optional = ok.filter(|_| self.is_optional());
        ret_if!(let Some(some) = optional, format!("optional {}", some.name()));
        let ok = ok.map_or("success".to_string(), |k| format!("{} result", k.name()));
        let err = err.map_or("failure".to_string(), |k| format!("{} error", k.name()));
        format!("{ok} or {err}")
    }

    /// The parts of the kind a function that can fail gives back.
    ///
    /// # Returns
    ///
    /// The kind of the good result and the error kind, each `None` when
    /// that side holds no value; `None` for another kind.
    pub fn fallible_parts(self) -> Option<(Option<DataKind>, Option<DataKind>)>
    {
        let shape = self.as_struct().filter(|s| s.name == FALLIBLE)?;
        let side = |name| shape.field(name).map(|(_, k)| k);
        // An optional holds its data where a result holds its good value.
        Some((side("ok").or_else(|| side("some")), side("err")))
    }

    /// The name of the kind.
    ///
    /// # Arguments
    ///
    /// * `full` - Whether a struct is written as its signature.
    ///
    /// # Returns
    ///
    /// The name.
    fn spell(self, full: bool) -> String
    {
        match self
        {
            DataKind::Int(n) => int_name(n),
            DataKind::Float(bits) => format!("f{bits}"),
            DataKind::Char => "char".to_string(),
            DataKind::Bool => "bool".to_string(),
            DataKind::Ptr(p) => ptr_name(p, full),
            DataKind::Null => "null".to_string(),
            DataKind::List(l) => list_name(l, full),
            DataKind::Struct(s) if !full && s.name == FALLIBLE => self.fallible_name(),
            DataKind::Struct(s) if !full && s.name == TABLE =>
            {
                self.table_name().unwrap_or_else(|| s.name.clone())
            }
            DataKind::Struct(s) => tern!(full, s.sig(), &s.name).to_string(),
            DataKind::Seq(s) if s.text => "text".to_string(),
            DataKind::Seq(s) if s.boxed => format!("{}@", s.cell.spell_inner(full)),
            DataKind::Seq(s) => format!("{}[]", s.cell.spell_inner(full)),
            DataKind::Enum(e) => tern!(full, e.sig(), &e.name).to_string(),
            DataKind::Fn(f) => f.spell(full),
        }
    }

    /// The name of the kind inside another name: a function kind stands in
    /// parentheses, so the levels after it never read as levels of its
    /// result.
    ///
    /// # Arguments
    ///
    /// * `full` - Whether a struct is written as its signature.
    ///
    /// # Returns
    ///
    /// The name.
    pub(crate) fn spell_inner(self, full: bool) -> String
    {
        let name = self.spell(full);
        tern!(self.is_fn(), format!("({name})"), name)
    }

    /// The function part of the kind.
    ///
    /// # Returns
    ///
    /// The parameters and the result, or `None` for a kind that is no
    /// function kind.
    pub fn as_fn(self) -> Option<&'static FnKind>
    {
        tern!(let DataKind::Fn(f) = self, Some(f), None)
    }

    /// Whether the kind is a function kind.
    ///
    /// # Returns
    ///
    /// `true` for a function kind.
    pub fn is_fn(self) -> bool { matches!(self, DataKind::Fn(_)) }

    /// The address levels and the base of the kind.
    ///
    /// # Returns
    ///
    /// Depth 0 and the kind itself for a plain kind, the parts of a pointer
    /// kind, and the parts of `ptr` for `null`.
    pub fn levels(self) -> (u8, Base)
    {
        match self
        {
            DataKind::Int(n) => (0, Base::Int(n)),
            DataKind::Float(bits) => (0, Base::Float(bits)),
            DataKind::Char => (0, Base::Char),
            DataKind::Bool => (0, Base::Bool),
            DataKind::Ptr(p) => (p.depth, p.base),
            DataKind::Null => (1, Base::Opaque),
            DataKind::List(l) => (0, Base::List(l)),
            DataKind::Struct(s) => (0, Base::Struct(s)),
            DataKind::Seq(s) => (0, Base::Seq(s)),
            DataKind::Enum(e) => (0, Base::Enum(e)),
            DataKind::Fn(f) => (0, Base::Fn(f)),
        }
    }

    /// The kind of an address of a value of this kind.
    ///
    /// # Returns
    ///
    /// The pointer kind one level deeper; `null` counts as `ptr`.
    pub fn pointer_to(self) -> DataKind
    {
        let (depth, base) = self.levels();
        let raw = self.ptr_kind().map_or(0, |p| p.raw);
        let depth = depth.saturating_add(1);
        DataKind::Ptr(PtrKind { raw, ..PtrKind::owner(depth, base) })
    }

    /// The kind of a raw address of a value of this kind.
    ///
    /// # Returns
    ///
    /// The pointer kind one level deeper, with the new level raw.
    pub fn raw_pointer_to(self) -> DataKind { self.pointer_to().with_outer_raw() }

    /// The kind a pointer of this kind points at.
    ///
    /// # Returns
    ///
    /// The kind one level shallower, without a borrow mark, or `None` for a
    /// kind that is no pointer, for `ptr` and for `null`.
    pub fn pointee(self) -> Option<DataKind>
    {
        let DataKind::Ptr(p) = self
        else
        {
            return None;
        };
        ret_if!(self.is_opaque(), None);
        let depth = p.depth.saturating_sub(1);
        ret_if!(depth == 0, DataKind::over(p.base, 0).into());
        let inner =
            PtrKind { raw: p.raw & !p.outer_bit(), ..PtrKind::owner(depth, p.base) };
        Some(DataKind::Ptr(inner))
    }

    /// Whether the kind is a pointer.
    ///
    /// # Returns
    ///
    /// `true` for a pointer kind, `ptr` included, but not for `null`.
    pub fn is_ptr(self) -> bool { matches!(self, DataKind::Ptr(_)) }

    /// Whether the kind is the opaque pointer `ptr`.
    ///
    /// # Returns
    ///
    /// `true` for `ptr` in each flavour: `ptr`, `ptr&` and `rawptr`.
    pub fn is_opaque(self) -> bool
    {
        matches!(self, DataKind::Ptr(p) if p.depth == 1 && p.base == Base::Opaque)
    }

    /// The pointer part of the kind.
    ///
    /// # Returns
    ///
    /// The levels, flavours and base, or `None` for a kind that is no
    /// pointer.
    pub fn ptr_kind(self) -> Option<PtrKind>
    {
        tern!(let DataKind::Ptr(p) = self, Some(p), None)
    }

    /// Whether the kind is a pointer whose outer level is an owner.
    ///
    /// # Returns
    ///
    /// `true` for `i32*` and `ptr`, but not for a borrow or a raw address.
    pub fn is_owner_ptr(self) -> bool { self.ptr_kind().is_some_and(PtrKind::is_owner) }

    /// Whether the kind is a borrow: a pointer whose outer level is a
    /// borrow.
    ///
    /// # Returns
    ///
    /// `true` for `i32&` and `ptr&`.
    pub fn is_borrow(self) -> bool { self.ptr_kind().is_some_and(PtrKind::is_borrowed) }

    /// Whether the kind is a pointer whose outer level is a raw address.
    ///
    /// # Returns
    ///
    /// `true` for `i32^` and `rawptr`.
    pub fn is_raw_ptr(self) -> bool
    {
        self.ptr_kind().is_some_and(PtrKind::is_raw_outer)
    }

    /// Whether the kind is a pointer the checker follows: an owner or a
    /// borrow at its outer level.
    ///
    /// # Returns
    ///
    /// `true` for `i32*`, `i32&` and `ptr`, but not for `i32^`.
    pub fn is_checked_ptr(self) -> bool { self.is_ptr() && !self.is_raw_ptr() }

    /// Whether a value of the kind is a borrow or has a borrow in it.
    ///
    /// # Returns
    ///
    /// `true` for a borrow and for a list whose cells hold one.
    pub fn holds_borrow(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_borrow());
        ret_if!(let Some(l) = self.as_seq().filter(|s| !s.boxed), l.cell.holds_borrow());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_borrow()));
        self.is_borrow()
    }

    /// The kind with its outer address level turned into a borrow.
    ///
    /// # Returns
    ///
    /// The borrow of an owner; a raw address and each kind that is no
    /// pointer come back unchanged.
    pub fn as_borrowed(self) -> DataKind
    {
        tern!(let DataKind::Ptr(p) = self, DataKind::Ptr(p.as_borrowed()), self)
    }

    /// The kind with its outer address level turned into an owner.
    ///
    /// # Returns
    ///
    /// The kind without a borrow mark.
    pub fn as_owner(self) -> DataKind
    {
        tern!(let DataKind::Ptr(p) = self, DataKind::Ptr(p.as_owner()), self)
    }

    /// Whether two kinds describe blocks of the same shape, whether or not
    /// the borrow mark of their outer pointer level sets them apart.
    ///
    /// # Arguments
    ///
    /// * `other` - The kind to compare with.
    ///
    /// # Returns
    ///
    /// `true` when the kinds are equal up to the borrow mark of their
    /// outer pointer level; a raw level is part of the shape. A list, as a
    /// kind or as the base of a pointer, compares whole, the flavour of its
    /// cells included, so a list of borrows never has the shape of a list
    /// of owners.
    pub fn same_shape(self, other: DataKind) -> bool
    {
        let (Some(a), Some(b)) = (self.ptr_kind(), other.ptr_kind())
        else
        {
            return self == other;
        };
        a.shape() == b.shape()
    }

    /// Whether a value of the kind is an address.
    ///
    /// # Returns
    ///
    /// `true` for a pointer kind or `null`.
    pub fn is_address(self) -> bool { self.is_ptr() || self == DataKind::Null }

    /// The size of a value of the kind, in octets.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The bit count over 8: 4 for a character, 1 for a boolean, the
    /// platform size for an address, and the sum of the cells for a list. A
    /// list too large for a `u32` gives `u32::MAX`; see
    /// [`DataKind::checked_size`].
    pub fn size(self, platform: u16) -> u32
    {
        self.checked_size(platform).unwrap_or(u32::MAX)
    }

    /// The size of a value of the kind, in octets, when it fits a `u32`.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The size, or `None` for a list too large for the machine.
    pub fn checked_size(self, platform: u16) -> Option<u32>
    {
        ret_if!(let Some(e) = self.as_enum(), e.under().checked_size(platform));
        ret_if!(let Some(s) = self.as_struct(), s.size(platform));
        ret_if!(self.is_text() || self.is_seq(), Some(u32::from(platform) / 8 * 3));
        let Some(l) = self.as_list()
        else
        {
            return Some(u32::from(self.bits(platform)) / 8);
        };
        l.cell.checked_size(platform)?.checked_mul(l.len)
    }

    /// The alignment of a value of the kind, in octets.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The size, at most 16; for a list, the alignment of its cells.
    pub fn align(self, platform: u16) -> u32
    {
        ret_if!(let Some(e) = self.as_enum(), e.under().align(platform));
        ret_if!(let Some(l) = self.as_list(), l.cell.align(platform));
        ret_if!(let Some(s) = self.as_struct(), s.align(platform));
        let words = self.is_text() || self.is_seq();
        ret_if!(words, (u32::from(platform) / 8).min(16));
        self.size(platform).min(16)
    }

    /// The integer part of the kind.
    ///
    /// # Returns
    ///
    /// The sign and width, or `None` for a kind that is no integer.
    pub fn int_kind(self) -> Option<IntKind>
    {
        tern!(let DataKind::Int(n) = self, Some(n), None)
    }

    /// Whether the kind is an integer.
    ///
    /// # Returns
    ///
    /// `true` for an integer kind.
    pub fn is_int(self) -> bool { matches!(self, DataKind::Int(_)) }

    /// Whether the kind is an unsigned integer.
    ///
    /// # Returns
    ///
    /// `true` for an unsigned integer kind.
    pub fn is_unsigned(self) -> bool { self.int_kind().is_some_and(|n| !n.signed) }

    /// Whether the kind is a floating-point number.
    ///
    /// # Returns
    ///
    /// `true` for a floating-point kind.
    pub fn is_float(self) -> bool { matches!(self, DataKind::Float(_)) }

    /// Whether the kind is a number.
    ///
    /// # Returns
    ///
    /// `true` for an integer or a floating-point kind.
    pub fn is_numeric(self) -> bool { self.is_int() || self.is_float() }

    /// The bit count of the kind.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The bit count; 32 for a character, 8 for a boolean, the platform
    /// size for an address. A list counts the bits of its cells, at most
    /// `u16::MAX`; use [`DataKind::size`] for lists.
    pub fn bits(self, platform: u16) -> u16
    {
        match self
        {
            DataKind::Int(n) => n.bits(platform),
            DataKind::Float(bits) => bits,
            DataKind::Char => 32,
            DataKind::Bool => 8,
            DataKind::Ptr(_) | DataKind::Null | DataKind::Fn(_) => platform,
            DataKind::Enum(e) => e.under().bits(platform),
            DataKind::List(_) | DataKind::Struct(_) | DataKind::Seq(_) =>
            {
                let bits = u64::from(self.size(platform)) * 8;
                u16::try_from(bits).unwrap_or(u16::MAX)
            }
        }
    }

    /// Whether a value of this kind converts to `target` on its own, with no
    /// loss of data.
    ///
    /// # Arguments
    ///
    /// * `target` - The kind the place expects.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// `true` for the same kind, a wider integer of the same sign, a wider
    /// signed integer from an unsigned one, a float whose fraction holds the
    /// integer, a wider float, `null` to a pointer, and a pointer to or from
    /// `ptr` of the same flavour: owner, borrow or raw, and a value to an
    /// optional or a result whose good side is a kind it converts to. An owner
    /// or a borrow whose pointee holds an address needs a raw step to pass
    /// through `ptr`, as a cast does, see [`DataKind::reads_addresses_as`].
    pub fn widens_to(self, target: DataKind, platform: u16) -> bool
    {
        ret_true!(self == target);
        match (self, target)
        {
            (DataKind::Int(a), DataKind::Int(b)) => int_widens(a, b, platform),
            (DataKind::Int(a), DataKind::Float(b)) => a.bits(platform) <= precision(b),
            (DataKind::Float(a), DataKind::Float(b)) => a < b,
            (DataKind::Null, DataKind::Ptr(_)) => true,
            (DataKind::Fn(a), DataKind::Fn(b)) => a.fits(b),
            (_, t) if t.fallible_parts().is_some() =>
            {
                let some = t.fallible_parts().and_then(|(some, _)| some);
                some.is_some_and(|s| self.widens_to(s, platform))
            }
            (DataKind::List(a), DataKind::Seq(b)) => !b.text && a.cell == b.cell,
            (DataKind::Ptr(_), DataKind::Ptr(_)) =>
            {
                self.meets_pointer(target) && !self.reads_addresses_as(target)
            }
            _ => false,
        }
    }

    /// Whether two pointers of one flavour address blocks of one shape, or
    /// one of them is `ptr`. This is enough to compare the two addresses;
    /// a value that goes into a place also needs
    /// [`DataKind::reads_addresses_as`] to be false.
    ///
    /// # Arguments
    ///
    /// * `other` - The other kind.
    ///
    /// # Returns
    ///
    /// `true` for two such pointer kinds.
    pub fn meets_pointer(self, other: DataKind) -> bool
    {
        let (Some(a), Some(b)) = (self.ptr_kind(), other.ptr_kind())
        else
        {
            return false;
        };
        let same_flavour =
            a.is_borrowed() == b.is_borrowed() && a.is_raw_outer() == b.is_raw_outer();
        let opaque = self.is_opaque() || other.is_opaque();
        same_flavour && (opaque || a.shape() == b.shape())
    }

    /// Whether an integer literal value fits this kind.
    ///
    /// # Arguments
    ///
    /// * `value` - The literal value.
    /// * `negated` - Whether a negate operator stands before the literal.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// `true` when the kind is an integer whose range holds the value.
    pub fn fits_int(self, value: u128, negated: bool, platform: u16) -> bool
    {
        let Some(n) = self.int_kind()
        else
        {
            return false;
        };
        let bits = n.bits(platform);
        let max = u128::MAX >> (128 - bits);
        ret_if!(!n.signed, tern!(negated, value == 0, value <= max));
        let half = 1u128 << (bits - 1);
        tern!(negated, value <= half, value < half)
    }

    /// Whether a cast between two owners or borrows reads what the block
    /// holds as another kind, when one side holds an address there: it
    /// would read stored addresses as other data, or data as addresses,
    /// which the checker cannot follow. Such a cast goes through a raw
    /// address.
    ///
    /// # Arguments
    ///
    /// * `to` - The target kind.
    ///
    /// # Returns
    ///
    /// `true` when both kinds are owners or borrows, a pointee holds an
    /// address at some level, and the pointees differ in shape; an opaque
    /// pointee differs from each other one.
    pub fn reads_addresses_as(self, to: DataKind) -> bool
    {
        ret_false!(!self.is_checked_ptr() || !to.is_checked_ptr());
        let (a, b) = (self.pointee(), to.pointee());
        let holds = |k: Option<DataKind>| k.is_some_and(DataKind::holds_address);
        ret_false!(!holds(a) && !holds(b));
        let same = match (a, b)
        {
            (Some(a), Some(b)) => a.same_shape(b),
            (None, None) => true,
            _ => false,
        };
        !same
    }

    /// Whether a value of the kind holds an address at some level: a
    /// pointer, text, a plain list, or a list whose cells hold one.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_address(self) -> bool
    {
        ret_if!(let Some(e) = self.as_enum(), e.under().holds_address());
        ret_true!(self.is_text() || self.is_seq());
        ret_if!(let Some(list) = self.as_list(), list.cell.holds_address());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_address()));
        self.is_ptr()
    }

    /// Whether the conversion verb can turn this kind into `to`.
    ///
    /// # Arguments
    ///
    /// * `to` - The target kind.
    ///
    /// # Returns
    ///
    /// `true` between numbers, between characters and integers, from a
    /// plain list of u8s to text, between an object and a pointer, from a
    /// boolean or an enumeration to an integer, between addresses, between
    /// an address and an integer, and for the same kind. Each conversion that
    /// makes or reads an address needs unsafe code.
    pub fn cast_allowed(self, to: DataKind) -> bool
    {
        // a member turns into its value, and a scalar value on as such a
        // value turns
        let under = self.as_enum().map(|e| e.under());
        ret_if!(let Some(u) = under, u == to || (!u.is_aggregate() && u.cast_allowed(to)));
        // A plain list of u8s becomes text once its UTF-8 is checked.
        ret_true!(
            self.as_seq().is_some_and(|s| s.cell == DataKind::U8) && to.is_text()
        );
        // An object becomes a pointer that holds a count of it, and back.
        let object = |k: DataKind| k.as_object().is_some();
        ret_true!((object(self) && to.is_ptr()) || (self.is_ptr() && object(to)));
        ret_false!(self.is_aggregate() || to.is_aggregate());
        ret_true!(self == to);
        ret_true!(self.is_numeric() && to.is_numeric());
        ret_true!(self.is_address() && (to.is_address() || to.is_int()));
        ret_true!(self.is_int() && to.is_address());
        ret_false!(!to.is_int() && !self.is_int());
        matches!(
            (self, to),
            (DataKind::Char, DataKind::Int(_))
                | (DataKind::Int(_), DataKind::Char)
                | (DataKind::Bool, DataKind::Int(_))
        )
    }
}

/// The base a struct name stands for.
///
/// # Arguments
///
/// * `kind` - The kind phrase, with a [`BaseKind::Named`] base.
/// * `structs` - The kind of each declared struct.
///
/// # Returns
///
/// The struct, or `isize` for a name that is not declared.
fn named_base(kind: &Kind, structs: &StructTable) -> Base
{
    let found = kind.name.as_ref().and_then(|n| structs.get(&n.name)).copied();
    found.map_or(Base::Int(IntKind::new(true, Width::Platform)), |k| k.levels().1)
}

/// Read the short name of a plain kind such as `i8`, `usize`, `f32`,
/// `char`.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The kind, or `None` for an unknown name.
fn parse_plain(name: &str) -> Option<DataKind>
{
    let named = [("isize", DataKind::ISIZE), ("usize", DataKind::USIZE)];
    let named = named.into_iter().chain([
        ("char", DataKind::Char),
        ("bool", DataKind::Bool),
        ("text", DataKind::TEXT),
    ]);
    let found = named.into_iter().find(|(n, _)| *n == name).map(|(_, k)| k);
    ret_if!(found.is_some(), found);
    let (head, digits) = name.split_at_checked(1)?;
    ret_if!(!digits.bytes().all(|c| c.is_ascii_digit()), None);
    let bits: u16 = digits.parse().ok()?;
    let int = |signed| DataKind::Int(IntKind::new(signed, Width::Bits(bits)));
    match head
    {
        "i" | "u" => INT_BITS.contains(&bits).then(|| int(head == "i")),
        "f" => FLOAT_BITS.contains(&bits).then_some(DataKind::Float(bits)),
        _ => None,
    }
}

/// The short name of a pointer kind.
///
/// # Arguments
///
/// * `p` - The kind.
/// * `full` - Whether a struct base is written as its signature.
///
/// # Returns
///
/// The base name, then one mark per address level from the base out: `*`
/// for an owner and `^` for a raw level; a borrowed outer level is `&` in
/// place of its `*`. `ptr` is a level itself and reads `rawptr` when raw;
/// a borrow of it adds `&` after its marks.
fn ptr_name(p: PtrKind, full: bool) -> String
{
    let plain = p.base.scalar();
    let first = usize::from(plain.is_none());
    let raw = |level: usize| p.raw.checked_shr(level as u32).unwrap_or(0) & 1 == 1;
    let levels = first..usize::from(p.depth);
    let mut marks: String = levels.map(|l| tern!(raw(l), '^', '*')).collect();
    let replace = p.borrowed && plain.is_some();
    do_if!(replace, _ = marks.pop());
    do_if!(p.borrowed, marks.push('&'));
    let base = plain.map_or_else(
        || tern!(raw(0), "rawptr", "ptr").to_string(),
        |k| k.spell_inner(full),
    );
    format!("{base}{marks}")
}

/// Read the base of a short name: a plain kind, `ptr`, `rawptr` or a struct
/// signature.
///
/// # Arguments
///
/// * `name` - The base, with its marks taken off.
///
/// # Returns
///
/// The kind, or `None` for an unknown name.
fn parse_base(name: &str) -> Option<DataKind>
{
    ret_if!(is_fn_text(name), parse_fn_with(unwrap_parens(name), &mut DataKind::parse));
    ret_if!(name == "ptr", Some(DataKind::OPAQUE));
    ret_if!(name == "rawptr", Some(DataKind::OPAQUE.with_outer_raw()));
    ret_if!(name.ends_with('}'), parse_sig(name).map(DataKind::Struct));
    ret_if!(name.ends_with('>'), crate::enums::parse_sig(name).map(DataKind::Enum));
    parse_plain(name)
}

/// Wrap a kind in the level one mark names.
///
/// # Arguments
///
/// * `kind` - The kind under the mark.
/// * `mark` - `*`, `^`, `&`, `[n]`, `[]` or `@`.
///
/// # Returns
///
/// The kind, or `None` for a mark that does not read here.
fn wrap_mark(kind: DataKind, mark: &str) -> Option<DataKind>
{
    // An address level over a borrow would lose the borrow, so no name
    // writes one; a list of borrows keeps them.
    let level = matches!(mark, "*" | "^" | "&");
    ret_if!(level && kind.is_borrow() && kind.is_ptr(), None);
    let opaque = kind.ptr_kind().is_some_and(|p| p.base == Base::Opaque);
    match mark
    {
        "*" => Some(kind.pointer_to()),
        "^" => Some(kind.raw_pointer_to()),
        "&" if opaque && !kind.is_raw_ptr() => Some(kind.as_borrowed()),
        "&" => Some(kind.pointer_to().as_borrowed()),
        "[]" => Some(DataKind::seq_of(kind)),
        "@" => Some(DataKind::box_of(kind)),
        _ =>
        {
            let count = mark.strip_prefix('[')?.strip_suffix(']')?;
            let len: u32 = count.parse().ok()?;
            ret_if!(len == 0, None);
            Some(DataKind::list_of(kind, len))
        }
    }
}

/// Split a short name into its base and its marks. The base ends at the
/// first `*`, `^`, `&` or `[` outside the braces of a struct signature.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The base and the marks after it.
pub fn split_marks(name: &str) -> (&str, &str)
{
    // A function kind not in parentheses takes each mark after it as a mark
    // of its result.
    ret_if!(name.starts_with("fn("), (name, ""));
    let (mut depth, mut prev) = (0i32, None);
    for (at, ch) in name.char_indices()
    {
        depth += bracket_step(prev, ch);
        prev = Some(ch);
        let mark = matches!(ch, '*' | '^' | '&' | '[' | '@');
        ret_if!(mark && depth == 0, name.split_at(at));
    }
    (name, "")
}

/// The short name of a list kind.
///
/// # Arguments
///
/// * `l` - The kind.
/// * `full` - Whether a struct cell is written as its signature.
///
/// # Returns
///
/// The cell name then `[n]`, such as `isize[3]`; `[]` while the count is
/// open.
fn list_name(l: &ListKind, full: bool) -> String
{
    let count = tern!(l.len == 0, "_".to_string(), l.len.to_string());
    format!("{}[{count}]", l.cell.spell_inner(full))
}

/// The function kind a function kind phrase names.
///
/// # Arguments
///
/// * `kind` - The phrase, with a [`BaseKind::Fn`] base.
/// * `structs` - The kind of each declared struct.
///
/// # Returns
///
/// The kind; a phrase with no signature names a function with no
/// parameters and no result.
fn fn_base(kind: &Kind, structs: &StructTable) -> &'static FnKind
{
    let Some(sig) = kind.sig.as_deref()
    else
    {
        return FnKind::make(Vec::new(), None);
    };
    let param = |p: &zakac_ast::kind::SigParam| FnParam {
        kind: DataKind::from_ast(&p.kind, structs),
        edit: p.edit,
        preposition: p.preposition.as_ref().map(|m| m.name),
    };
    let ret = sig.ret.as_ref().map(|r| DataKind::from_ast(r, structs));
    FnKind::make(sig.params.iter().map(param).collect(), ret)
}

/// The short name of an integer kind.
///
/// # Arguments
///
/// * `n` - The kind.
///
/// # Returns
///
/// The name, such as `i8` or `usize`.
fn int_name(n: IntKind) -> String
{
    let sign = tern!(n.signed, "i", "u");
    match n.width
    {
        Width::Platform => format!("{sign}size"),
        Width::Bits(bits) => format!("{sign}{bits}"),
    }
}

/// Whether integer kind `a` converts to integer kind `b` on its own.
///
/// # Arguments
///
/// * `a` - The source kind.
/// * `b` - The target kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// `true` when `b` is wider and no sign is lost.
fn int_widens(a: IntKind, b: IntKind, platform: u16) -> bool
{
    let wider = a.bits(platform) < b.bits(platform);
    wider && !(a.signed && !b.signed)
}

/// Significand bits of a floating-point kind, the implicit bit included.
///
/// # Arguments
///
/// * `bits` - The bit count of the float kind.
///
/// # Returns
///
/// The number of integer bits the kind holds without loss.
fn precision(bits: u16) -> u16
{
    match bits
    {
        32 => 24,
        64 => 53,
        _ => 0,
    }
}

impl From<DataKind> for String
{
    /// The short name of the kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The name.
    fn from(kind: DataKind) -> String { kind.sig_name() }
}

impl TryFrom<String> for DataKind
{
    type Error = String;

    /// Read a short name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The kind, or an error for an unknown name.
    fn try_from(name: String) -> Result<Self, Self::Error>
    {
        DataKind::parse(&name).ok_or_else(|| format!("unknown data kind {name}"))
    }
}

/// The signature of the struct of a ဇယား, from the text of its index kind
/// and its value kind.
///
/// # Arguments
///
/// * `index` - The index kind in text form.
/// * `value` - The value kind in text form.
///
/// # Returns
///
/// The signature.
pub(crate) fn table_text(index: &str, value: &str) -> String
{
    let [hash, live, at, data] = ROW_FIELDS;
    let row = format!("{ROW}{{{hash}:u64,{live}:bool,{at}:{index},{data}:{value}}}");
    let [rows, slots, count] = TABLE_FIELDS;
    format!("{TABLE}{{{rows}:{row}[],{slots}:u32[],{count}:usize}}")
}
