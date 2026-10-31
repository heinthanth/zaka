/// Where the landing page of the website lives.
pub const HOME_URL: &str = "/";

/// Where the Learn pages live on the website.
pub const LEARN_URL: &str = "/learn/";

/// Where the manual of the language, the compiler and the machine lives on
/// the website.
pub const DOCS_URL: &str = "/docs/";

/// Where the explorer lives on the website.
pub const EXPLORER_URL: &str = "/explorer/";

/// The window event the header of the explorer page sends to open the
/// settings of the explorer: the two are islands apart, with no signal in
/// common.
pub const EXPLORER_SETTINGS_EVENT: &str = "zaka-explorer-settings";

/// The window event the header sends right before it reloads the page for a
/// new language, so the explorer writes the link of its latest edits to the
/// URL first; the reload keeps it.
pub const EXPLORER_FLUSH_EVENT: &str = "zaka-explorer-flush";
