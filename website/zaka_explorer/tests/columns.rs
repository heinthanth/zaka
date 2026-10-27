use zaka_explorer::state::columns::MIN_WIDTH;
use zaka_explorer::state::columns::drag_edge;
use zaka_explorer::state::columns::fit_widths;
use zaka_explorer::state::columns::is_hidden;
use zaka_explorer::state::columns::next_shown;
use zaka_explorer::state::columns::parse_widths;
use zaka_explorer::state::columns::shown_count;
use zaka_explorer::state::columns::shown_width;
use zaka_explorer::state::columns::toggle_column;
use zaka_explorer::state::columns::widths_list;

#[test]
fn hidden_list_names_columns_from_one()
{
    assert!(is_hidden("2 5", 2));
    assert!(is_hidden("2 5", 5));
    assert!(!is_hidden("2 5", 1));
    assert!(!is_hidden("12", 1));
    assert!(!is_hidden("", 1));
}

#[test]
fn toggle_hides_then_shows_again()
{
    assert_eq!(toggle_column("", 3, 5), "3");
    assert_eq!(toggle_column("3", 1, 5), "1 3");
    assert_eq!(toggle_column("1 3", 3, 5), "1");
    assert_eq!(toggle_column("1", 1, 5), "");
}

#[test]
fn toggle_keeps_the_last_shown_column()
{
    assert_eq!(toggle_column("1 2", 3, 3), "1 2");
    assert_eq!(toggle_column("1 2", 1, 3), "2");
}

#[test]
fn stale_entries_do_not_count()
{
    assert_eq!(shown_count("2 9 junk", 3), 2);
    assert_eq!(toggle_column("9 junk", 2, 3), "2");
}

#[test]
fn widths_need_one_per_column()
{
    assert_eq!(parse_widths("120 80 64", 3), Some(vec![120.0, 80.0, 64.0]));
    assert_eq!(parse_widths("120 80", 3), None);
    assert_eq!(parse_widths("120 junk 64", 3), None);
    assert_eq!(parse_widths("", 3), None);
}

#[test]
fn widths_keep_whole_px_and_never_add_up_to_more()
{
    assert_eq!(widths_list(&[120.4, 80.6, 64.0]), "120 80 64");
}

#[test]
fn the_shown_width_leaves_out_hidden_columns()
{
    assert_eq!(shown_width(&[120.0, 80.0, 64.0], "2"), 184.0);
    assert_eq!(shown_width(&[120.0, 80.0, 64.0], ""), 264.0);
}

#[test]
fn the_next_shown_column_skips_hidden_ones()
{
    assert_eq!(next_shown("", 2, 4), Some(3));
    assert_eq!(next_shown("3", 2, 4), Some(4));
    assert_eq!(next_shown("4", 3, 4), None);
    assert_eq!(next_shown("", 4, 4), None);
}

#[test]
fn an_edge_moves_width_between_two_columns()
{
    let widths = [120.0, 80.0, 64.0];
    assert_eq!(drag_edge(&widths, 1, 2, 30.0), vec![150.0, 50.0, 64.0]);
    assert_eq!(drag_edge(&widths, 2, 3, -20.0), vec![120.0, 60.0, 84.0]);
}

#[test]
fn an_edge_keeps_both_columns_over_the_floor()
{
    let widths = [120.0, 80.0, 64.0];
    let wide = drag_edge(&widths, 1, 2, 500.0);
    assert_eq!(wide, vec![120.0 + 80.0 - MIN_WIDTH, MIN_WIDTH, 64.0]);
    let narrow = drag_edge(&widths, 1, 2, -500.0);
    assert_eq!(narrow, vec![MIN_WIDTH, 80.0 + 120.0 - MIN_WIDTH, 64.0]);
}

#[test]
fn fit_scales_the_shown_columns_to_a_total()
{
    assert_eq!(fit_widths(&[100.0, 50.0, 50.0], "", 400.0), vec![200.0, 100.0, 100.0]);
    assert_eq!(fit_widths(&[100.0, 50.0, 50.0], "2", 300.0), vec![200.0, 50.0, 100.0]);
    assert_eq!(fit_widths(&[0.0, 0.0], "", 300.0), vec![0.0, 0.0]);
}
