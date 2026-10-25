use strum::IntoEnumIterator;
use zaka_web_ui_common::font::HAS_CODE_FONT;
use zaka_web_ui_common::font::face::BURMESE;
use zaka_web_ui_common::font::face::Face;
use zaka_web_ui_common::font::face::Fallback;
use zaka_web_ui_common::font::face::INTERFACE;
use zaka_web_ui_common::font::face::Pair;
use zakac_rust_sugars::tern;

#[test]
fn faces_round_trip_through_their_names()
{
    for face in Face::iter()
    {
        assert_eq!(face.to_string().parse::<Face>().unwrap(), face);
    }
    // Names stored before the font lists merged still parse.
    for name in ["pda10", "fira-code", "jetbrains-mono", "native", "inter", "noto"]
    {
        assert!(name.parse::<Face>().is_ok(), "{name}");
    }
}

#[test]
fn fallbacks_round_trip_through_their_names()
{
    assert_eq!(Fallback::Inherit.to_string(), "inherit");
    assert_eq!("inherit".parse::<Fallback>().unwrap(), Fallback::Inherit);
    for face in Face::iter()
    {
        let fallback = Fallback::Face(face);
        assert_eq!(fallback.to_string().parse::<Fallback>().unwrap(), fallback);
    }
    assert!("nope".parse::<Fallback>().is_err());
}

#[test]
fn the_defaults_are_sabae_with_space_mono_and_sabae_with_sauce()
{
    let ui = Pair::default_ui();
    assert_eq!(
        (ui.burmese, ui.fallback),
        (Face::Sabae, Fallback::Face(Face::SpaceMono))
    );
    let code = Pair::default_code();
    assert_eq!(
        (code.burmese, code.fallback),
        (Face::Sabae, Fallback::Face(Face::SauceHtmm))
    );
}

#[test]
fn stored_fonts_the_pickers_lack_read_as_the_default()
{
    assert_eq!(Face::burmese_or(Face::Padauk, Face::Sabae), Face::Padauk);
    assert_eq!(Face::burmese_or(Face::FiraCode, Face::Sabae), Face::Sabae);
    let pda = Face::burmese_or(Face::Pda10, Face::Sabae);
    assert_eq!(pda, tern!(HAS_CODE_FONT, Face::Pda10, Face::Sabae));
    let inter = Fallback::Face(Face::Inter);
    assert_eq!(inter.or(Face::code_faces(), Fallback::Inherit), Fallback::Inherit);
    assert_eq!(inter.or(Face::ui_faces(), Fallback::Inherit), inter);
    assert_eq!(Fallback::Inherit.or(Face::code_faces(), inter), Fallback::Inherit);
}

#[test]
fn the_code_picker_leaves_out_interface_fonts()
{
    assert!(Face::code_faces().all(|f| f.group() != INTERFACE));
    assert_eq!(Face::code_faces().any(|f| f == Face::Pda10), HAS_CODE_FONT);
    for face in [Face::SauceHtmm, Face::SourceCodePro, Face::FiraCode, Face::SpaceMono]
    {
        assert!(Face::code_faces().any(|f| f == face), "{face:?}");
    }
}

#[test]
fn the_interface_fallbacks_hold_all_code_fonts_and_more()
{
    let ui: Vec<Face> = Face::ui_faces().collect();
    assert!(Face::code_faces().all(|f| ui.contains(&f)));
    for face in [Face::Native, Face::Inter, Face::Noto, Face::Padauk, Face::SpaceMono]
    {
        assert!(ui.contains(&face), "{face:?}");
    }
}

#[test]
fn burmese_fonts_are_grouped()
{
    let burmese: Vec<Face> = Face::iter().filter(|f| f.group() == BURMESE).collect();
    assert_eq!(burmese, [Face::Pda10, Face::Sabae, Face::Padauk, Face::Noto]);
    let picked: Vec<Face> = Face::burmese_faces().collect();
    assert_eq!(picked.contains(&Face::Pda10), HAS_CODE_FONT);
    assert!(picked.iter().all(|f| f.group() == BURMESE));
    assert_eq!(Face::Sabae.family(), "Myanmar Sabae");
    assert_eq!(Face::Pda10.burmese_alias(), "PDA10 Burmese Mono (Burmese)");
    assert_eq!(Face::Sabae.burmese_alias(), "Myanmar Sabae (Burmese)");
}

/// A pair of fonts.
fn pair(burmese: Face, fallback: Fallback) -> Pair
{
    Pair::builder().burmese(burmese).fallback(fallback).build()
}

#[test]
fn inherit_leaves_all_text_to_the_whole_burmese_font()
{
    let padauk = pair(Face::Padauk, Fallback::Inherit);
    assert_eq!(padauk.rest(), None);
    assert_eq!(padauk.code_stack(), "\"Padauk\", ui-monospace, monospace");
    assert!(padauk.ui_stack().starts_with("\"Padauk\", system-ui"));
    // A fallback that is the Burmese font acts as Inherit.
    let same = pair(Face::Padauk, Fallback::Face(Face::Padauk));
    assert_eq!(same.code_stack(), padauk.code_stack());
}

#[test]
fn a_fallback_follows_the_burmese_range_of_the_burmese_font()
{
    let code = pair(Face::Sabae, Fallback::Face(Face::FiraCode));
    assert_eq!(code.rest(), Some(Face::FiraCode));
    assert_eq!(
        code.code_stack(),
        "\"Myanmar Sabae (Burmese)\", \"Fira Code\", ui-monospace, monospace"
    );
    let ui = Pair::default_ui().ui_stack();
    assert!(
        ui.starts_with("\"Myanmar Sabae (Burmese)\", \"Space Mono\", system-ui"),
        "{ui}"
    );
    assert!(ui.ends_with("sans-serif"), "{ui}");
    let native = pair(Face::Noto, Fallback::Face(Face::Native)).ui_stack();
    assert!(
        native.starts_with("\"Noto Sans Myanmar (Burmese)\", system-ui"),
        "{native}"
    );
}

#[test]
fn labels_come_from_the_table()
{
    assert_eq!(Face::Native.label(), "OS default");
    assert_eq!(Face::Iosevka.label(), Face::Iosevka.family());
    assert_eq!(Face::Noto.group_label(), "Burmese");
}
