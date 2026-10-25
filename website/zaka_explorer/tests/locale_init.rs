use zaka_explorer::compile::Options;
use zaka_explorer::compile::compile;
use zaka_explorer::compile::samples::DIAGNOSTICS;
use zaka_explorer::compile::samples::SAMPLES;

/// A locale set after the first detection stands, even when a compiler crate
/// translates for the first time later on; the explorer and the compiler
/// share one locale.
#[test]
fn a_set_locale_outlives_the_first_lookup_of_each_crate()
{
    zakac_i18n::init_locale();
    zakac_i18n::set_locale("xx");
    for sample in SAMPLES.iter().filter(|s| s.group == DIAGNOSTICS)
    {
        let out = compile(sample.text, &Options::default());
        assert!(out.diagnostics.iter().all(|d| !d.message.is_empty()));
    }
    assert_eq!(&*zakac_i18n::locale(), "xx");
}
