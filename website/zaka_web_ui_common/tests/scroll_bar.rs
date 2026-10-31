use zaka_web_ui_common::controls::scroll_bar::thumb_for;

#[test]
fn a_page_that_fits_the_window_has_no_thumb()
{
    assert_eq!(thumb_for(800.0, 800.0, 0.0), None);
}

#[test]
fn the_thumb_runs_from_the_top_to_the_foot_of_the_window()
{
    // A page twice the window: the thumb is half the window tall.
    assert_eq!(thumb_for(800.0, 1600.0, 0.0), Some((0.0, 400.0)));
    assert_eq!(thumb_for(800.0, 1600.0, 400.0), Some((200.0, 400.0)));
    assert_eq!(thumb_for(800.0, 1600.0, 800.0), Some((400.0, 400.0)));
}

#[test]
fn a_long_page_keeps_the_thumb_a_fair_target()
{
    let (_, height) = thumb_for(800.0, 1_000_000.0, 0.0).unwrap();
    assert_eq!(height, 40.0);
}

#[test]
fn an_overscroll_keeps_the_thumb_in_the_window()
{
    assert_eq!(thumb_for(800.0, 1600.0, -50.0), Some((0.0, 400.0)));
    assert_eq!(thumb_for(800.0, 1600.0, 900.0), Some((400.0, 400.0)));
}
