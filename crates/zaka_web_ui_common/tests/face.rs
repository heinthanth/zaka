use strum::IntoEnumIterator;
use zaka_web_ui_common::font::HAS_CODE_FONT;
use zaka_web_ui_common::font::face::BURMESE;
use zaka_web_ui_common::font::face::Face;
use zaka_web_ui_common::font::face::INTERFACE;
use zakac_rust_sugars::ret_if;
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
fn the_defaults_are_space_mono_and_pda10_or_sauce()
{
    assert_eq!(Face::default_ui(), Face::SpaceMono);
    let code = tern!(HAS_CODE_FONT, Face::Pda10, Face::SauceHtmm);
    assert_eq!(Face::default_code(), code);
    assert_eq!(Face::code_or_default(Face::Pda10), code);
    assert_eq!(Face::code_or_default(Face::Inter), code);
    assert_eq!(Face::code_or_default(Face::FiraCode), Face::FiraCode);
    assert_eq!(Face::ui_or_default(Face::Native), Face::Native);
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
fn the_interface_picker_holds_all_code_fonts_and_more()
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
    assert_eq!(burmese, [Face::Pda10, Face::Padauk, Face::Noto]);
}

#[test]
fn code_stacks_lead_with_the_face_and_fall_back_to_burmese()
{
    for face in Face::code_faces()
    {
        let stack = face.code_stack(false);
        assert!(stack.starts_with(&format!("\"{}\"", face.family())), "{stack}");
        assert_eq!(stack.matches("Noto Sans Myanmar").count(), 1, "{stack}");
        assert!(stack.ends_with("monospace"), "{stack}");
    }
    assert_eq!(Face::Pda10.family(), "PDA10 Burmese Mono");
    assert_eq!(Face::SauceHtmm.family(), "Sauce Code Pro (HTMM's Custom)");
}

#[test]
fn pda_burmese_leads_the_stack_of_other_fonts()
{
    assert_eq!(Face::Pda10.code_stack(true), Face::Pda10.code_stack(false));
    let fira = Face::FiraCode.code_stack(true);
    // Without the font in the build, the switch changes nothing.
    ret_if!(!HAS_CODE_FONT, assert_eq!(fira, Face::FiraCode.code_stack(false)));
    let lead = "\"PDA10 Burmese Mono (Burmese)\"";
    assert!(fira.starts_with(&format!("{lead}, \"Fira Code\"")), "{fira}");
}

#[test]
fn ui_stacks_fall_back_to_burmese_and_the_os_font()
{
    for face in Face::ui_faces()
    {
        let stack = face.ui_stack();
        assert_eq!(stack.matches("Noto Sans Myanmar").count(), 1, "{stack}");
        assert!(stack.ends_with("sans-serif"), "{stack}");
    }
    assert!(
        Face::SpaceMono.ui_stack().starts_with("\"Space Mono\", \"Noto Sans Myanmar\"")
    );
    assert!(Face::Native.ui_stack().starts_with("\"Noto Sans Myanmar\", system-ui"));
}

#[test]
fn labels_come_from_the_table()
{
    assert_eq!(Face::Native.label(), "OS default");
    assert_eq!(Face::Iosevka.label(), Face::Iosevka.family());
    assert_eq!(Face::Noto.group_label(), "Burmese");
}
