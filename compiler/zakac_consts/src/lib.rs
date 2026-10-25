use shadow_rs::concatcp;
use shadow_rs::formatcp;
use shadow_rs::shadow;
use zakac_rust_sugars::tern;

shadow!(build);

const LOCAL_CHANGES_SUFFIX: &str = tern!(crate::build::GIT_CLEAN, "", ".local");

const COMMIT: &str = crate::build::SHORT_COMMIT;
const HEX_TIMESTAMP: &str = formatcp!("t{:x}", crate::build::BUILD_TIMESTAMP);
const COMMIT_TO_USE: &str = tern!(COMMIT.is_empty(), HEX_TIMESTAMP, COMMIT);

/// The standard build number for all components in the Zaka workspace.
pub const ZAKA_BUILD_NUMBER: &str = concatcp!(COMMIT_TO_USE, LOCAL_CHANGES_SUFFIX);

/// The standard license for all components in the Zaka workspace.
pub const ZAKA_LICENSE: &str = "MIT";

/// The author of the Zaka workspace.
pub const ZAKA_AUTHOR: &str = "Hein Thant Maung Maung";

/// When work on the Zaka workspace began, for the rights notice.
pub const ZAKA_FIRST_YEAR: u32 = 2023;
