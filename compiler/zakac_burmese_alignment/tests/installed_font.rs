#![cfg(unix)]

use zakac_burmese_alignment::flow::Grid;
use zakac_burmese_alignment::flow::Part;
use zakac_burmese_alignment::flow::reach;
use zakac_burmese_alignment::fonts::Faces;
use zakac_burmese_alignment::ghostty::settings::BURMESE_FONTS;

/// Line 2 of `samples/diagnostics/lexer_errors.zaka`, as a flowing Ghostty
/// build draws it with Noto Sans Myanmar at 32 px in 19 px cells. A screenshot
/// of saturngod's fork shows the bad number from cell 46 of the row; its grid
/// column is 38.
#[test]
fn the_sample_line_matches_the_screenshot()
{
    let Some(faces) = Faces::installed(&BURMESE_FONTS, 32.0)
    else
    {
        eprintln!(
            "none of the Burmese fonts Ghostty looks for is installed; nothing to \
             check"
        );
        return;
    };
    let row = [
        Part::builder().text("2 | ").build(),
        Part::builder().text("ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ").build(),
        Part::builder().text("၁၈ဘစ်").bold(true).build(),
        Part::builder().text(" ဖြစ်ပါစေ။").build(),
    ];
    let grid = Grid::builder().cell(19.0).cols(120).build();
    let got = reach(&faces, grid, &row, 2).unwrap();
    assert_eq!((got.start, got.width), (46, 5));
}
