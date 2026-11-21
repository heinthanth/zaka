//! A wait on several channels at once: the opener, the arms of the channels,
//! and the arm of a time limit or of no data.

use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::stmt;

/// The opener of the wait.
const OPENER: &str = "အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။";

/// The arms of the channels `က` and `ခ`.
const ARMS: &str = "က ထဲတွင် ဒေတာ ရှိလျှင် \"က\" ဟု ပြသမည်။
ခ ထဲတွင် ဒေတာ ရှိလျှင် \"ခ\" ဟု ပြသမည်။";

/// The arm of a time limit of 500 milliseconds.
const MILLIS: &str = "၅၀၀ မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် \"ကုန်\" ဟု ပြသမည်။";

/// The closed arm, glued as written.
const CLOSED: &str = "မည်သည့်လမ်းကြောင်းထံမှ ဒေတာမရောက်လာနိုင်တော့လျှင် \"ပိတ်\" ဟု ပြသမည်။";

/// The arm of no data.
const IDLE: &str = "မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင် \"မရှိ\" ဟု ပြသမည်။";

#[test]
fn the_arms_of_the_channels_follow_the_opener()
{
    let tree = stmt(&format!("{OPENER}\n{ARMS}"));
    assert!(tree.starts_with("(select "), "{tree}");
    assert!(!tree.contains("after-") && !tree.contains("idle"), "{tree}");
}

#[test]
fn a_time_limit_counts_milliseconds_or_seconds()
{
    let tree = stmt(&format!("{OPENER}\n{ARMS}\n{MILLIS}"));
    assert!(tree.contains("after-ms"), "{tree}");
    let seconds = "(၁ + ၁) စက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် \"ကုန်\" ဟု ပြသမည်။";
    let tree = stmt(&format!("{OPENER}\n{ARMS}\n{seconds}"));
    assert!(tree.contains("after-s"), "{tree}");
}

#[test]
fn the_arm_of_no_data_ends_the_wait()
{
    let tree = stmt(&format!("{OPENER}\n{ARMS}\n{IDLE}"));
    assert!(tree.contains("idle"), "{tree}");
}

#[test]
fn an_opener_with_no_arm_is_zk0264()
{
    assert_eq!(codes(&format!("{OPENER}\n\"က\" ဟု ပြသမည်။")), [Code::SelectNoArm]);
}

#[test]
fn a_time_limit_and_an_arm_of_no_data_together_is_zk0265()
{
    assert_eq!(
        codes(&format!("{OPENER}\n{ARMS}\n{MILLIS}\n{IDLE}")),
        [Code::SelectTwoEnds]
    );
}

#[test]
fn the_closed_arm_reads_glued_or_apart()
{
    let tree = stmt(&format!("{OPENER}\n{ARMS}\n{CLOSED}"));
    assert!(tree.contains("closed"), "{tree}");
    let apart = "မည်သည့်လမ်းကြောင်းထံမှ ဒေတာ မရောက်လာနိုင်တော့လျှင် \"ပိတ်\" ဟု ပြသမည်။";
    let tree = stmt(&format!("{OPENER}\n{ARMS}\n{apart}"));
    assert!(tree.contains("closed"), "{tree}");
}

#[test]
fn the_closed_arm_goes_with_a_time_limit_or_the_arm_of_no_data()
{
    let tree = stmt(&format!("{OPENER}\n{ARMS}\n{CLOSED}\n{MILLIS}"));
    assert!(tree.contains("closed") && tree.contains("after-ms"), "{tree}");
    let tree = stmt(&format!("{OPENER}\n{ARMS}\n{CLOSED}\n{IDLE}"));
    assert!(tree.contains("closed") && tree.contains("idle"), "{tree}");
}
