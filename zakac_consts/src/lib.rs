use shadow_rs::concatcp;
use shadow_rs::shadow;
use zakac_rust_sugars::tern;

shadow!(build);

const _COMMIT: &str = crate::build::SHORT_COMMIT;
const _PFXED_COMMIT: &str = tern!(_COMMIT.is_empty(), "", concatcp!(".", _COMMIT));
const LOCAL_CHANGES_SUFFIX: &str = tern!(crate::build::GIT_CLEAN, "", ".local");

/// The standard build number for all components in the Zaka workspace.
pub const ZAKA_BUILD_NUMBER: &str =
    concatcp!(crate::build::BUILD_TIMESTAMP, _PFXED_COMMIT, LOCAL_CHANGES_SUFFIX);

/// The standard license for all components in the Zaka workspace.
pub const ZAKA_LICENSE: &str = "MIT";
