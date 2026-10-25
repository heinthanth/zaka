use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use bon::Builder;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoEnumIterator;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::font::BURMESE_FONT;
use crate::font::CODE_FONT;
use crate::font::HAS_CODE_FONT;

/// Picker group of the fonts made for interface text.
pub const INTERFACE: &str = "Interface";

/// Picker group of the fonts that hold Burmese.
pub const BURMESE: &str = "Burmese";

/// Picker group of the Latin monospace fonts.
pub const LATIN: &str = "Latin";

/// Font name of Sauce Code Pro (HTMM's Custom); the app ships its file under
/// a hashed name and passes it to [`crate::font::set_face_file`].
pub const SAUCE_FONT: &str = "Sauce Code Pro (HTMM's Custom)";

/// Font name of Myanmar Sabae; the app ships its file under a hashed name
/// and passes it to [`crate::font::set_face_file`].
pub const SABAE_FONT: &str = "Myanmar Sabae";

/// Stored name of the fallback that leaves all text to the Burmese font.
const INHERIT: &str = "inherit";

/// Last fallbacks of code text, for ASCII.
const MONO_FALLBACK: &str = "ui-monospace, monospace";

/// The OS font, for interface text.
const SYSTEM_UI: &str = "system-ui, -apple-system, \"Segoe UI\"";

/// Last fallback of interface text.
const SANS_FALLBACK: &str = "sans-serif";

/// Weights of the @fontsource sheets most fonts load: regular, and the
/// semibold the editor draws reserved words in.
const REGULAR_SEMIBOLD: &[u16] = &[400, 600];

/// Weights of the @fontsource sheets of fonts that skip 600.
const REGULAR_BOLD: &[u16] = &[400, 700];

/// Version of the @fontsource packages, as `Trunk.toml` pins them; the files
/// of a package sit in a folder named after it, so an upgrade changes each
/// address.
pub const FONTSOURCE: &str = "5.3.0";

/// Version of @fontsource/source-code-pro, a step ahead of the rest.
pub const SOURCE_CODE_PRO: &str = "5.3.1";

/// Where the browser gets a face from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source
{
    /// Built into the wasm; the page adds it when it starts.
    Embedded,
    /// Installed on the OS.
    Native,
    /// A @fontsource package in `fonts/<dir>/<version>/`, one sheet per
    /// weight.
    Sheets
    {
        /// The folder under `fonts/`.
        dir: &'static str,
        /// The version of the package, the folder under `dir`.
        version: &'static str,
        /// The weights to load.
        weights: &'static [u16],
    },
    /// Font files under `fonts/`, each with its weight.
    Files(&'static [(u16, &'static str)]),
    /// One file at weight 400 that the app ships under a hashed name and
    /// passes to [`crate::font::set_face_file`].
    Shipped,
}

/// A font the pickers offer; its string form is what is stored. The Burmese
/// pickers offer the [`BURMESE`] group; the code fallback picker offers all
/// but the [`INTERFACE`] group; the interface fallback picker offers all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Face
{
    /// The OS font.
    Native,
    /// Inter, from @fontsource.
    Inter,
    /// PDA10 Burmese Mono, compiled into the pages when the build has the
    /// secret to open it.
    Pda10,
    /// Myanmar Sabae, a file the app ships.
    Sabae,
    /// Padauk, from @fontsource.
    Padauk,
    /// Noto Sans Myanmar, from @fontsource.
    Noto,
    /// Sauce Code Pro (HTMM's Custom), a file in `fonts/`.
    SauceHtmm,
    /// Source Code Pro, from @fontsource.
    SourceCodePro,
    /// Fira Code, from @fontsource.
    FiraCode,
    /// JetBrains Mono, from @fontsource.
    #[strum(serialize = "jetbrains-mono")]
    JetBrainsMono,
    /// Cascadia Code, from @fontsource.
    CascadiaCode,
    /// IBM Plex Mono, from @fontsource.
    IbmPlexMono,
    /// Roboto Mono, from @fontsource.
    RobotoMono,
    /// Inconsolata, from @fontsource.
    Inconsolata,
    /// Ubuntu Mono, from @fontsource.
    UbuntuMono,
    /// Victor Mono, from @fontsource.
    VictorMono,
    /// Geist Mono, from @fontsource.
    GeistMono,
    /// Iosevka, two files from @fontsource; the package is too big to ship
    /// whole.
    Iosevka,
    /// Commit Mono, from @fontsource.
    CommitMono,
    /// Monaspace Neon, from @fontsource.
    MonaspaceNeon,
    /// Red Hat Mono, from @fontsource.
    RedHatMono,
    /// Space Mono, from @fontsource.
    SpaceMono,
}

impl Default for Face
{
    /// The Burmese code font when none is stored.
    ///
    /// # Returns
    ///
    /// The face.
    fn default() -> Self { Pair::default_code().burmese }
}

impl Face
{
    /// Name in the pickers: the family, or a note for [`Face::Native`].
    ///
    /// # Returns
    ///
    /// The name.
    pub fn label(self) -> Cow<'static, str>
    {
        tern!(self == Face::Native, t!("web_ui.face.native"), self.family().into())
    }

    /// The CSS font family; [`Face::Native`] has none of its own.
    ///
    /// # Returns
    ///
    /// The family name.
    pub fn family(self) -> &'static str
    {
        match self
        {
            Face::Native => "",
            Face::Inter => "Inter",
            Face::Pda10 => CODE_FONT,
            Face::Sabae => SABAE_FONT,
            Face::Padauk => "Padauk",
            Face::Noto => "Noto Sans Myanmar",
            Face::SauceHtmm => SAUCE_FONT,
            Face::SourceCodePro => "Source Code Pro",
            Face::FiraCode => "Fira Code",
            Face::JetBrainsMono => "JetBrains Mono",
            Face::CascadiaCode => "Cascadia Code",
            Face::IbmPlexMono => "IBM Plex Mono",
            Face::RobotoMono => "Roboto Mono",
            Face::Inconsolata => "Inconsolata",
            Face::UbuntuMono => "Ubuntu Mono",
            Face::VictorMono => "Victor Mono",
            Face::GeistMono => "Geist Mono",
            Face::Iosevka => "Iosevka",
            Face::CommitMono => "Commit Mono",
            Face::MonaspaceNeon => "Monaspace Neon",
            Face::RedHatMono => "Red Hat Mono",
            Face::SpaceMono => "Space Mono",
        }
    }

    /// Group in the pickers.
    ///
    /// # Returns
    ///
    /// [`INTERFACE`], [`BURMESE`] or [`LATIN`].
    pub fn group(self) -> &'static str
    {
        match self
        {
            Face::Native | Face::Inter => INTERFACE,
            Face::Pda10 | Face::Sabae | Face::Padauk | Face::Noto => BURMESE,
            _ => LATIN,
        }
    }

    /// Name of the group in the pickers.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn group_label(self) -> Cow<'static, str>
    {
        match self.group()
        {
            INTERFACE => t!("web_ui.face.group.interface"),
            BURMESE => t!("web_ui.face.group.burmese"),
            _ => t!("web_ui.face.group.latin"),
        }
    }

    /// Where the browser gets the face from.
    ///
    /// # Returns
    ///
    /// The source.
    pub fn source(self) -> Source
    {
        let sheets =
            |dir, weights| Source::Sheets { dir, version: FONTSOURCE, weights };
        match self
        {
            Face::Native => Source::Native,
            Face::Pda10 => Source::Embedded,
            Face::Sabae | Face::SauceHtmm => Source::Shipped,
            Face::Iosevka => Source::Files(&[
                (400, "iosevka/5.3.0/iosevka-latin-400-normal.woff2"),
                (700, "iosevka/5.3.0/iosevka-latin-700-normal.woff2"),
            ]),
            Face::Inter => sheets("inter", &[400, 500, 600]),
            Face::Padauk => sheets("padauk", REGULAR_BOLD),
            Face::Noto => sheets("noto-sans-myanmar", REGULAR_SEMIBOLD),
            Face::SourceCodePro => Source::Sheets {
                dir: "source-code-pro",
                version: SOURCE_CODE_PRO,
                weights: REGULAR_SEMIBOLD,
            },
            Face::FiraCode => sheets("fira-code", REGULAR_SEMIBOLD),
            Face::JetBrainsMono => sheets("jetbrains-mono", REGULAR_SEMIBOLD),
            Face::CascadiaCode => sheets("cascadia-code", REGULAR_SEMIBOLD),
            Face::IbmPlexMono => sheets("ibm-plex-mono", REGULAR_SEMIBOLD),
            Face::RobotoMono => sheets("roboto-mono", REGULAR_SEMIBOLD),
            Face::Inconsolata => sheets("inconsolata", REGULAR_SEMIBOLD),
            Face::UbuntuMono => sheets("ubuntu-mono", REGULAR_BOLD),
            Face::VictorMono => sheets("victor-mono", REGULAR_SEMIBOLD),
            Face::GeistMono => sheets("geist-mono", REGULAR_SEMIBOLD),
            Face::CommitMono => sheets("commit-mono", REGULAR_SEMIBOLD),
            Face::MonaspaceNeon => sheets("monaspace-neon", REGULAR_SEMIBOLD),
            Face::RedHatMono => sheets("red-hat-mono", REGULAR_SEMIBOLD),
            Face::SpaceMono => sheets("space-mono", REGULAR_BOLD),
        }
    }

    /// Path of the sheet of one weight under the font folder, for a face from
    /// @fontsource.
    ///
    /// # Arguments
    ///
    /// * `weight` - The weight.
    ///
    /// # Returns
    ///
    /// `<dir>/<version>/<weight>.css`, or `None` for other faces.
    pub fn sheet_path(self, weight: u16) -> Option<String>
    {
        let Source::Sheets { dir, version, .. } = self.source()
        else
        {
            return None;
        };
        Some(format!("{dir}/{version}/{weight}.css"))
    }

    /// Whether the build holds the face; PDA10 Burmese Mono needs the secret.
    ///
    /// # Returns
    ///
    /// `true` when the pickers can offer it.
    pub fn usable(self) -> bool { HAS_CODE_FONT || self != Face::Pda10 }

    /// Whether the code picker offers the face.
    ///
    /// # Returns
    ///
    /// `true` for a usable face outside the [`INTERFACE`] group.
    pub fn is_code(self) -> bool { self.usable() && self.group() != INTERFACE }

    /// The fonts of the code picker.
    ///
    /// # Returns
    ///
    /// The faces, in picker order.
    pub fn code_faces() -> impl Iterator<Item = Face>
    {
        Face::iter().filter(|f| f.is_code())
    }

    /// The fonts of the interface picker.
    ///
    /// # Returns
    ///
    /// The faces, in picker order.
    pub fn ui_faces() -> impl Iterator<Item = Face>
    {
        Face::iter().filter(|f| f.usable())
    }

    /// The fonts of the Burmese pickers.
    ///
    /// # Returns
    ///
    /// The faces, in picker order.
    pub fn burmese_faces() -> impl Iterator<Item = Face>
    {
        Face::iter().filter(|f| f.usable() && f.group() == BURMESE)
    }

    /// Font name of the face limited to the Burmese range; first in a stack,
    /// it draws Burmese text and leaves the rest to the fallback. It needs a
    /// name of its own: under the font name the whole face would win.
    ///
    /// # Returns
    ///
    /// The font name, then ` (Burmese)`.
    pub fn burmese_alias(self) -> String
    {
        ret_if!(self == Face::Pda10, BURMESE_FONT.to_string());
        format!("{} (Burmese)", self.family())
    }

    /// A stored Burmese font, or a default when the picker lacks it.
    ///
    /// # Arguments
    ///
    /// * `face` - The stored face.
    /// * `default` - The face to use in its place.
    ///
    /// # Returns
    ///
    /// The face to use.
    pub fn burmese_or(face: Face, default: Face) -> Face
    {
        tern!(Face::burmese_faces().any(|f| f == face), face, default)
    }
}

/// The font of the text outside the Burmese range; its string form is what
/// is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Fallback
{
    /// The Burmese font draws all text.
    #[default]
    Inherit,
    /// A font of its own.
    Face(Face),
}

impl fmt::Display for Fallback
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Fallback::Inherit => f.write_str(INHERIT),
            Fallback::Face(face) => write!(f, "{face}"),
        }
    }
}

impl FromStr for Fallback
{
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        ret_if!(s == INHERIT, Ok(Fallback::Inherit));
        s.parse().map(Fallback::Face)
    }
}

impl Fallback
{
    /// A stored fallback, or a default when the picker lacks its face.
    ///
    /// # Arguments
    ///
    /// * `faces` - The faces of the picker.
    /// * `default` - The fallback to use in its place.
    ///
    /// # Returns
    ///
    /// The fallback to use.
    pub fn or(
        self, mut faces: impl Iterator<Item = Face>, default: Fallback,
    ) -> Fallback
    {
        let Fallback::Face(face) = self
        else
        {
            return self;
        };
        tern!(faces.any(|f| f == face), self, default)
    }
}

/// A Burmese font and the font of the rest of the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct Pair
{
    /// The font of Burmese text.
    pub burmese: Face,
    /// The font of the rest.
    pub fallback: Fallback,
}

impl Pair
{
    /// The interface fonts when none are stored: Myanmar Sabae and Space
    /// Mono.
    ///
    /// # Returns
    ///
    /// The pair.
    pub fn default_ui() -> Pair
    {
        Pair::builder()
            .burmese(Face::Sabae)
            .fallback(Fallback::Face(Face::SpaceMono))
            .build()
    }

    /// The code fonts when none are stored: Myanmar Sabae and Sauce Code Pro
    /// (HTMM's Custom). PDA10 Burmese Mono remains in the picker when the build
    /// holds it.
    ///
    /// # Returns
    ///
    /// The pair.
    pub fn default_code() -> Pair
    {
        Pair::builder()
            .burmese(Face::Sabae)
            .fallback(Fallback::Face(Face::SauceHtmm))
            .build()
    }

    /// The font of the text outside the Burmese range, when it is not the
    /// Burmese font.
    ///
    /// # Returns
    ///
    /// The face, or `None` when the Burmese font draws all text.
    pub fn rest(self) -> Option<Face>
    {
        let Fallback::Face(face) = self.fallback
        else
        {
            return None;
        };
        (face != self.burmese).then_some(face)
    }

    /// The lead of a stack: the Burmese font whole, or limited to the
    /// Burmese range when another font draws the rest.
    ///
    /// # Returns
    ///
    /// The quoted name, then a comma.
    fn lead(self) -> String
    {
        let name = tern!(
            self.rest().is_some(),
            self.burmese.burmese_alias(),
            self.burmese.family().to_string()
        );
        format!("\"{name}\", ")
    }

    /// The CSS font stack of code text: the Burmese font, the fallback, then
    /// the OS monospace font.
    ///
    /// # Returns
    ///
    /// The stack.
    pub fn code_stack(self) -> String
    {
        let rest = self.rest().map(|f| format!("\"{}\", ", f.family()));
        format!("{}{}{MONO_FALLBACK}", self.lead(), rest.unwrap_or_default())
    }

    /// The CSS font stack of interface text: the Burmese font, the fallback,
    /// then the OS font.
    ///
    /// # Returns
    ///
    /// The stack.
    pub fn ui_stack(self) -> String
    {
        let rest = self.rest().filter(|f| *f != Face::Native);
        let rest = rest.map(|f| format!("\"{}\", ", f.family()));
        format!(
            "{}{}{SYSTEM_UI}, {SANS_FALLBACK}",
            self.lead(),
            rest.unwrap_or_default()
        )
    }
}
