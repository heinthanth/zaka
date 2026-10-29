use zaka_website::components::glance::source;
use zaka_website::components::sandbox::run::FenceOptions;
use zaka_website::components::sandbox::run::run_to_end;

// The labels of the glance promise this warning and this output; a change to
// the compiler that drops either one has to change the labels too.
#[test]
fn the_glance_program_warns_once_and_prints_its_total()
{
    let (codes, printed) = run_to_end(&source(), &FenceOptions::default());
    assert_eq!(codes, ["ZK0506"]);
    assert_eq!(printed, "၅၅\nများသည်\n");
}
