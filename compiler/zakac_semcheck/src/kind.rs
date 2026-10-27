use serde::Deserialize;
use serde::Serialize;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;

/// Bit counts an integer kind can have.
const INT_BITS: &[u16] = &[8, 16, 32, 64, 128];
/// Bit counts a floating-point kind can have.
const FLOAT_BITS: &[u16] = &[16, 32, 64, 128];

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
        }
    }
}

/// A pointer kind: `depth` levels of address over a base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PtrKind
{
    /// The number of address levels, at least 1. An opaque base counts
    /// itself, so `ptr` has depth 1 and `*ptr` depth 2.
    pub depth: u8,
    /// The kind at the bottom.
    pub base: Base,
}

/// A data kind as the checker sees it: a written kind with its defaults
/// filled in. Artifacts store it as its short name, such as `i8`, `usize`,
/// `*i32` or `ptr`.
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
    /// The kind of `ဗလာ`, which converts to each pointer kind.
    Null,
}

impl DataKind
{
    /// `ကိန်းပြည့်`, the default integer kind.
    pub const ISIZE: DataKind = DataKind::Int(IntKind::new(true, Width::Platform));
    /// `လက္ခဏာမဲ့ ကိန်းပြည့်`
    pub const USIZE: DataKind = DataKind::Int(IntKind::new(false, Width::Platform));
    /// `ဒဿမကိန်း`, the default floating-point kind.
    pub const F64: DataKind = DataKind::Float(64);
    /// `လိပ်စာ`, the opaque pointer.
    pub const OPAQUE: DataKind =
        DataKind::Ptr(PtrKind { depth: 1, base: Base::Opaque });

    /// The data kind a written kind phrase names.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind phrase.
    ///
    /// # Returns
    ///
    /// The kind; a missing sign means signed, a missing bit count means
    /// platform size for integers and 64 for floats. Each pointer level of
    /// the phrase adds one address level.
    pub fn from_ast(kind: &Kind) -> DataKind
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
            BaseKind::Opaque => Base::Opaque,
        };
        DataKind::over(base, kind.depth)
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
        plain.unwrap_or(DataKind::Ptr(PtrKind { depth: depth.max(1), base }))
    }

    /// Read a short name such as `i8`, `usize`, `f32`, `char`, `*i32`, `ptr`
    /// or `null`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for an unknown name.
    pub fn parse(name: &str) -> Option<DataKind>
    {
        ret_if!(name == "null", Some(DataKind::Null));
        let rest = name.trim_start_matches('*');
        let stars = u8::try_from(name.len() - rest.len()).ok()?;
        ret_if!(
            rest == "ptr",
            Some(DataKind::over(Base::Opaque, stars.checked_add(1)?))
        );
        let (_, base) = parse_plain(rest)?.levels();
        Some(DataKind::over(base, stars))
    }

    /// The short name, for messages and artifacts.
    ///
    /// # Returns
    ///
    /// The name, such as `i8`, `usize`, `f64`, `char`, `bool`, `*i32`,
    /// `ptr` or `null`.
    pub fn name(self) -> String
    {
        match self
        {
            DataKind::Int(n) => int_name(n),
            DataKind::Float(bits) => format!("f{bits}"),
            DataKind::Char => "char".to_string(),
            DataKind::Bool => "bool".to_string(),
            DataKind::Ptr(p) => ptr_name(p),
            DataKind::Null => "null".to_string(),
        }
    }

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
        DataKind::over(base, depth.saturating_add(1))
    }

    /// The kind a pointer of this kind points at.
    ///
    /// # Returns
    ///
    /// The kind one level shallower, or `None` for a kind that is no
    /// pointer, for `ptr` and for `null`.
    pub fn pointee(self) -> Option<DataKind>
    {
        let DataKind::Ptr(p) = self
        else
        {
            return None;
        };
        ret_if!(self.is_opaque(), None);
        Some(DataKind::over(p.base, p.depth.saturating_sub(1)))
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
    /// `true` for `ptr`.
    pub fn is_opaque(self) -> bool { self == DataKind::OPAQUE }

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
    /// platform size for an address.
    pub fn size(self, platform: u16) -> u32 { u32::from(self.bits(platform)) / 8 }

    /// The alignment of a value of the kind, in octets.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The size, at most 16.
    pub fn align(self, platform: u16) -> u32 { self.size(platform).min(16) }

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
    /// size for an address.
    pub fn bits(self, platform: u16) -> u16
    {
        match self
        {
            DataKind::Int(n) => n.bits(platform),
            DataKind::Float(bits) => bits,
            DataKind::Char => 32,
            DataKind::Bool => 8,
            DataKind::Ptr(_) | DataKind::Null => platform,
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
    /// `ptr`.
    pub fn widens_to(self, target: DataKind, platform: u16) -> bool
    {
        ret_true!(self == target);
        match (self, target)
        {
            (DataKind::Int(a), DataKind::Int(b)) => int_widens(a, b, platform),
            (DataKind::Int(a), DataKind::Float(b)) => a.bits(platform) <= precision(b),
            (DataKind::Float(a), DataKind::Float(b)) => a < b,
            (DataKind::Null, DataKind::Ptr(_)) => true,
            (DataKind::Ptr(_), DataKind::Ptr(_)) =>
            {
                self.is_opaque() || target.is_opaque()
            }
            _ => false,
        }
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

    /// Whether the conversion verb can turn this kind into `to`.
    ///
    /// # Arguments
    ///
    /// * `to` - The target kind.
    ///
    /// # Returns
    ///
    /// `true` between numbers, between characters and integers, from a
    /// boolean to an integer, between addresses, between addresses and
    /// integers, and for the same kind.
    pub fn cast_allowed(self, to: DataKind) -> bool
    {
        ret_true!(self == to);
        ret_true!(self.is_numeric() && to.is_numeric());
        ret_true!(self.is_address() && (to.is_address() || to.is_int()));
        ret_true!(to.is_address() && self.is_int());
        ret_false!(!to.is_int() && !self.is_int());
        matches!(
            (self, to),
            (DataKind::Char, DataKind::Int(_))
                | (DataKind::Int(_), DataKind::Char)
                | (DataKind::Bool, DataKind::Int(_))
        )
    }
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
    let named =
        named.into_iter().chain([("char", DataKind::Char), ("bool", DataKind::Bool)]);
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
///
/// # Returns
///
/// One `*` per address level, then the base name; `ptr` counts as one
/// level itself.
fn ptr_name(p: PtrKind) -> String
{
    let Some(plain) = p.base.scalar()
    else
    {
        return format!("{}ptr", "*".repeat(usize::from(p.depth.saturating_sub(1))));
    };
    format!("{}{}", "*".repeat(usize::from(p.depth)), plain.name())
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
        16 => 11,
        32 => 24,
        64 => 53,
        128 => 113,
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
    fn from(kind: DataKind) -> String { kind.name() }
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
