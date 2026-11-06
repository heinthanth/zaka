//! Installed fonts: the Burmese faces a terminal draws with, shaped with
//! rustybuzz, and the cell width of a main font.

use fontdb::Database;
use fontdb::Family;
use fontdb::Query;
use fontdb::Weight;
use rustybuzz::UnicodeBuffer;
use rustybuzz::ttf_parser;
use zakac_rust_sugars::tern;

use crate::flow::Drawn;
use crate::flow::Shaper;

/// The octets of a font file, and the index of a face in it.
struct Face
{
    data: Vec<u8>,
    index: u32,
}

/// The regular and bold faces of a Burmese font, at a font size.
pub struct Faces
{
    regular: Face,
    bold: Face,
    /// The font size, in pixels.
    px: f64,
}

impl Faces
{
    /// Load the first of `names` that is installed on this machine.
    ///
    /// # Arguments
    ///
    /// * `names` - The fonts to look for, in order.
    /// * `px` - The font size, in pixels.
    ///
    /// # Returns
    ///
    /// The faces, or `None` when none of the fonts is installed.
    pub fn installed(names: &[&str], px: f64) -> Option<Self>
    {
        Self::load(&installed_fonts(), names, px)
    }

    /// Load the first of `names` that `db` holds.
    ///
    /// # Arguments
    ///
    /// * `db` - The installed fonts.
    /// * `names` - The fonts to look for, in order.
    /// * `px` - The font size, in pixels.
    ///
    /// # Returns
    ///
    /// The faces, or `None` when none of the fonts is installed.
    pub fn load(db: &Database, names: &[&str], px: f64) -> Option<Self>
    {
        let regular = names.iter().find_map(|name| find(db, name, Weight::NORMAL))?;
        let name = db.face(regular)?.families.first()?.0.clone();
        let bold = find(db, &name, Weight::BOLD).unwrap_or(regular);
        Some(Self { regular: read_face(db, regular)?, bold: read_face(db, bold)?, px })
    }
}

impl Shaper for Faces
{
    fn shape(&self, text: &str, bold: bool) -> Vec<Drawn>
    {
        let face = tern!(bold, &self.bold, &self.regular);
        let Some(font) = rustybuzz::Face::from_slice(&face.data, face.index)
        else
        {
            return Vec::new();
        };
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.guess_segment_properties();
        let shaped = rustybuzz::shape(&font, &[], buffer);
        let scale = self.px / f64::from(font.units_per_em());
        let drawn =
            |(info, pos): (&rustybuzz::GlyphInfo, &rustybuzz::GlyphPosition)| {
                Drawn::builder()
                    .at(info.cluster as usize)
                    .advance(f64::from(pos.x_advance) * scale)
                    .offset(f64::from(pos.x_offset) * scale)
                    .build()
            };
        shaped.glyph_infos().iter().zip(shaped.glyph_positions()).map(drawn).collect()
    }
}

/// The fonts installed on this machine.
///
/// # Returns
///
/// The font database.
pub fn installed_fonts() -> Database
{
    let mut db = Database::new();
    db.load_system_fonts();
    db
}

/// The cell width of a main font, in ems: the advance of its `0`.
///
/// # Arguments
///
/// * `db` - The installed fonts.
/// * `name` - The font name.
///
/// # Returns
///
/// The width, or `None` when the font is not installed.
pub fn cell_em(db: &Database, name: &str) -> Option<f64>
{
    let id = find(db, name, Weight::NORMAL)?;
    db.with_face_data(id, |data, index| {
        let face = ttf_parser::Face::parse(data, index).ok()?;
        let advance = face.glyph_hor_advance(face.glyph_index('0')?)?;
        Some(f64::from(advance) / f64::from(face.units_per_em()))
    })?
}

/// Find an installed face of a font.
///
/// # Arguments
///
/// * `db` - The installed fonts.
/// * `name` - The font name.
/// * `weight` - The weight wanted; the nearest one is taken.
///
/// # Returns
///
/// The face, or `None` when the font is not installed.
fn find(db: &Database, name: &str, weight: Weight) -> Option<fontdb::ID>
{
    let names = [Family::Name(name)];
    let wanted = Query { families: &names, weight, ..Query::default() };
    db.query(&wanted)
}

/// Read the font file of a face.
///
/// # Arguments
///
/// * `db` - The installed fonts.
/// * `id` - The face.
///
/// # Returns
///
/// The file and the index of the face in it.
fn read_face(db: &Database, id: fontdb::ID) -> Option<Face>
{
    db.with_face_data(id, |data, index| Face { data: data.to_vec(), index })
}
