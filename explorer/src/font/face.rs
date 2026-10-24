use std::borrow::Cow;

use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoEnumIterator;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::font::BURMESE_FONT;
use crate::font::CODE_FONT;
use crate::font::HAS_CODE_FONT;

/// Picker group of the fonts made for interface text.
pub const INTERFACE: &str = "Interface";

/// Picker group of the fonts that hold Burmese.
pub const BURMESE: &str = "Burmese";

/// Picker group of the Latin monospace fonts; Burmese text falls back to
/// Noto Sans Myanmar.
pub const LATIN: &str = "Latin";

/// Family name of Sauce Code Pro (HTMM's Custom), a file in `fonts/`.
pub const SAUCE_FONT: &str = "Sauce Code Pro (HTMM's Custom)";

/// Burmese fallback after the chosen font.
const BURMESE_FALLBACK: &str = "\"Noto Sans Myanmar\", ";

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

/// Where the browser gets a face from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source
{
    /// Built into the wasm; the editor adds it when it starts.
    Embedded,
    /// Installed on the OS.
    Native,
    /// A @fontsource package in `fonts/<dir>/`, one sheet per weight.
    Sheets
    {
        /// The folder under `fonts/`.
        dir: &'static str,
        /// The weights to load.
        weights: &'static [u16],
    },
    /// Font files under `fonts/`, each with its weight.
    Files(&'static [(u16, &'static str)]),
}

/// A font the pickers offer; its string form is what is stored. The code
/// picker offers all but the [`INTERFACE`] group; the interface picker
/// offers all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Face
{
    /// The OS font.
    Native,
    /// Inter, from @fontsource.
    Inter,
    /// PDA10 Burmese Mono, compiled into the explorer when the build has the
    /// secret to open it.
    Pda10,
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
    /// The default code font.
    ///
    /// # Returns
    ///
    /// The face.
    fn default() -> Self { Face::default_code() }
}

impl Face
{
    /// The code font when none is stored: PDA10 Burmese Mono when the build
    /// holds it, else Sauce Code Pro (HTMM's Custom).
    ///
    /// # Returns
    ///
    /// The face.
    pub fn default_code() -> Face { tern!(HAS_CODE_FONT, Face::Pda10, Face::SauceHtmm) }

    /// The interface font when none is stored.
    ///
    /// # Returns
    ///
    /// The face.
    pub fn default_ui() -> Face { Face::SpaceMono }

    /// Name in the pickers: the family, or a note for [`Face::Native`].
    ///
    /// # Returns
    ///
    /// The name.
    pub fn label(self) -> Cow<'static, str>
    {
        tern!(self == Face::Native, t!("explorer.face.native"), self.family().into())
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
            Face::Pda10 | Face::Padauk | Face::Noto => BURMESE,
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
            INTERFACE => t!("explorer.face.group.interface"),
            BURMESE => t!("explorer.face.group.burmese"),
            _ => t!("explorer.face.group.latin"),
        }
    }

    /// Where the browser gets the face from.
    ///
    /// # Returns
    ///
    /// The source.
    pub fn source(self) -> Source
    {
        let sheets = |dir, weights| Source::Sheets { dir, weights };
        match self
        {
            Face::Native => Source::Native,
            Face::Pda10 => Source::Embedded,
            Face::SauceHtmm =>
            {
                Source::Files(&[(400, "SauceCodeProHTMMsCustom-Regular.ttf")])
            }
            Face::Iosevka => Source::Files(&[
                (400, "iosevka/iosevka-latin-400-normal.woff2"),
                (700, "iosevka/iosevka-latin-700-normal.woff2"),
            ]),
            Face::Inter => sheets("inter", &[400, 500, 600]),
            Face::Padauk => sheets("padauk", REGULAR_BOLD),
            Face::Noto => sheets("noto-sans-myanmar", REGULAR_SEMIBOLD),
            Face::SourceCodePro => sheets("source-code-pro", REGULAR_SEMIBOLD),
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

    /// A stored code font, or the default when the picker lacks it.
    ///
    /// # Arguments
    ///
    /// * `face` - The stored face.
    ///
    /// # Returns
    ///
    /// The face to use.
    pub fn code_or_default(face: Face) -> Face
    {
        tern!(face.is_code(), face, Face::default_code())
    }

    /// A stored interface font, or the default when the picker lacks it.
    ///
    /// # Arguments
    ///
    /// * `face` - The stored face.
    ///
    /// # Returns
    ///
    /// The face to use.
    pub fn ui_or_default(face: Face) -> Face
    {
        tern!(face.usable(), face, Face::default_ui())
    }

    /// The CSS font stack of code text: the family, then fallbacks.
    ///
    /// # Arguments
    ///
    /// * `pda_burmese` - Draw Burmese text in PDA10 Burmese Mono: its
    ///   Burmese-range face goes first, so the chosen font keeps the rest.
    ///
    /// # Returns
    ///
    /// The stack.
    pub fn code_stack(self, pda_burmese: bool) -> String
    {
        let merged = HAS_CODE_FONT && pda_burmese && self != Face::Pda10;
        let lead = tern!(merged, format!("\"{BURMESE_FONT}\", "), String::new());
        let burmese = tern!(self == Face::Noto, "", BURMESE_FALLBACK);
        format!("{lead}\"{}\", {burmese}{MONO_FALLBACK}", self.family())
    }

    /// The CSS font stack of interface text: the family, then Noto Sans
    /// Myanmar for Burmese text, then the OS font.
    ///
    /// # Returns
    ///
    /// The stack.
    pub fn ui_stack(self) -> String
    {
        let native = self == Face::Native;
        let lead = tern!(native, String::new(), format!("\"{}\", ", self.family()));
        let burmese = tern!(self == Face::Noto, "", BURMESE_FALLBACK);
        format!("{lead}{burmese}{SYSTEM_UI}, {SANS_FALLBACK}")
    }
}
