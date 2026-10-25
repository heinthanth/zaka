use zaka_explorer::state::split::MAX_SPLIT;
use zaka_explorer::state::split::MIN_SPLIT;
use zaka_explorer::state::split::clamp_split;
use zaka_explorer::state::split::share_fr;
use zaka_explorer::state::split::split_at;

#[test]
fn split_is_the_share_before_the_pointer()
{
    assert_eq!(split_at(150.0, 100.0, 200.0), 0.25);
    assert_eq!(split_at(200.0, 100.0, 200.0), 0.5);
}

#[test]
fn split_keeps_inside_its_bounds()
{
    assert_eq!(split_at(0.0, 100.0, 200.0), MIN_SPLIT);
    assert_eq!(split_at(900.0, 100.0, 200.0), MAX_SPLIT);
    assert_eq!(split_at(50.0, 0.0, 0.0), 0.5);
    assert_eq!(clamp_split(f64::NAN), 0.5);
    assert_eq!(clamp_split(0.4), 0.4);
}

#[test]
fn share_is_a_clamped_grid_track()
{
    assert_eq!(share_fr(0.5), "0.5fr");
    assert_eq!(share_fr(0.95), "0.8fr");
    assert_eq!(share_fr(f64::NAN), "0.5fr");
}
