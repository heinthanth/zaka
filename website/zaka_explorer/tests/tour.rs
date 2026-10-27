use strum::IntoEnumIterator;
use zaka_explorer::compile::Options;
use zaka_explorer::compile::Phase;
use zaka_explorer::compile::compile;
use zaka_explorer::compile::samples::find_sample;
use zaka_explorer::state::tour::GAP;
use zaka_explorer::state::tour::Rect;
use zaka_explorer::state::tour::TOUR_SAMPLE;
use zaka_explorer::state::tour::place;
use zaka_explorer::state::tour::steps;

/// A window 1200 px wide and 800 px high.
const VIEW: (f64, f64) = (1200.0, 800.0);

/// A card 360 px wide and 200 px high.
const CARD: (f64, f64) = (360.0, 200.0);

/// A box from its left, top, width and height.
fn rect(left: f64, top: f64, width: f64, height: f64) -> Rect
{
    Rect::builder().left(left).top(top).width(width).height(height).build()
}

#[test]
fn each_phase_has_one_stage_step_in_pipeline_order()
{
    let staged: Vec<Phase> = steps().iter().filter_map(|s| s.stage).collect();
    assert_eq!(staged, Phase::iter().collect::<Vec<_>>());
}

#[test]
fn the_phases_and_the_timer_need_debug_mode()
{
    let all = steps();
    assert!(all.iter().any(|s| s.auto));
    assert!(all.iter().filter(|s| s.auto || s.stage.is_some()).all(|s| s.debug));
}

#[test]
fn debug_mode_turns_on_after_the_step_on_its_switch()
{
    let all = steps();
    let switch =
        all.iter().position(|s| s.target == Some("[data-tour=debug]")).unwrap();
    assert!(all[..=switch].iter().all(|s| !s.debug));
    assert!(all[switch + 1..].iter().all(|s| s.debug));
}

#[test]
fn the_compiler_options_come_before_debug_mode()
{
    let all = steps();
    let at = |target| all.iter().position(|s| s.target == Some(target)).unwrap();
    let options = at("[data-tour=options]");
    assert!(options < at("[data-tour=debug]"));
    assert!(!all[options].debug);
}

#[test]
fn each_debugger_control_has_a_step_in_bar_order()
{
    let controls = [
        "go",
        "stop",
        "reset",
        "step",
        "step-over",
        "step-out",
        "timer",
        "speed",
        "status",
    ];
    let targets: Vec<&str> = steps().iter().filter_map(|s| s.target).collect();
    let at = |name: &str| {
        let target = format!("[data-tour={name}]");
        targets.iter().position(|t| *t == target).expect(name)
    };
    let order: Vec<usize> = controls.iter().map(|c| at(c)).collect();
    assert!(order.is_sorted(), "{order:?}");
    assert_eq!(order.last().unwrap() - order[0], controls.len() - 1);
}

#[test]
fn the_tour_opens_and_ends_on_a_centered_card()
{
    let all = steps();
    assert_eq!(all.first().unwrap().target, None);
    assert_eq!(all.last().unwrap().target, None);
    assert!(all.iter().all(|s| s.title != s.text));
}

#[test]
fn the_sample_compiles_clean_and_emits_ops()
{
    let sample = find_sample(TOUR_SAMPLE).expect("the tour sample");
    let out = compile(sample.text, &Options::default());
    assert!(out.diagnostics.is_empty());
    assert!(out.emitted);
}

#[test]
fn a_card_goes_under_a_small_target()
{
    let (left, top) = place(Some(rect(100.0, 40.0, 200.0, 30.0)), CARD, VIEW);
    assert_eq!(top, 40.0 + 30.0 + GAP);
    assert_eq!(left, 100.0 + (200.0 - 360.0) / 2.0);
}

#[test]
fn a_card_goes_over_a_target_near_the_bottom()
{
    let (_, top) = place(Some(rect(400.0, 700.0, 200.0, 60.0)), CARD, VIEW);
    assert_eq!(top, 700.0 - GAP - 200.0);
}

#[test]
fn a_card_goes_beside_a_tall_target()
{
    let (left, top) = place(Some(rect(600.0, 50.0, 600.0, 750.0)), CARD, VIEW);
    assert_eq!(left, 600.0 - GAP - 360.0);
    assert_eq!(top, 50.0 + (750.0 - 200.0) / 2.0);
    let (left, _) = place(Some(rect(0.0, 50.0, 600.0, 750.0)), CARD, VIEW);
    assert_eq!(left, 600.0 + GAP);
}

#[test]
fn a_card_goes_inside_a_target_that_fills_the_window()
{
    let (_, top) = place(Some(rect(0.0, 0.0, 1200.0, 800.0)), CARD, VIEW);
    assert_eq!(top, VIEW.1 - GAP - 200.0);
}

#[test]
fn a_card_without_target_is_centered()
{
    assert_eq!(
        place(None, CARD, VIEW),
        ((1200.0 - 360.0) / 2.0, (800.0 - 200.0) / 2.0)
    );
}

#[test]
fn a_card_remains_in_the_window()
{
    let narrow = (320.0, 640.0);
    let (left, top) =
        place(Some(rect(-50.0, 10.0, 40.0, 20.0)), (296.0, 200.0), narrow);
    assert!(left >= GAP && left + 296.0 <= narrow.0 - GAP);
    assert!(top >= GAP && top + 200.0 <= narrow.1 - GAP);
}

#[test]
fn a_ring_grows_and_keeps_inside_the_window()
{
    let ring = rect(0.0, 100.0, 1200.0, 200.0).ring(6.0, VIEW);
    assert_eq!(ring.left, 1.0);
    assert_eq!(ring.top, 94.0);
    assert_eq!(ring.width, 1198.0);
    assert_eq!(ring.height, 212.0);
}
