//! The names of the kinds the host functions take and give, as the check
//! names them, and the numbers the stdlib gives the streams and the error
//! members.

/// Text.
pub const TEXT: &str = "text";

/// A plain list of text.
pub const TEXT_LIST: &str = "text[]";

/// The enumeration of what a path holds, as the stdlib declares it: a file,
/// a folder, or another thing.
pub const PATH_KIND: &str = "ဖိုင်စနစ်ပစ္စည်း";

/// The member of [`PATH_KIND`] for a file, `ဖိုင်`.
pub const IS_FILE: u32 = 0;

/// The member of [`PATH_KIND`] for a folder, `ဖိုင်တွဲ`.
pub const IS_FOLDER: u32 = 1;

/// The member of [`PATH_KIND`] for another thing, `တခြားပစ္စည်း`.
pub const IS_OTHER: u32 = 2;

/// The struct of an input stream, as the stdlib declares it.
pub const INPUT_STREAM: &str = "အချက်အလက်အဝင်စီးကြောင်း";

/// The struct of an output stream, as the stdlib declares it.
pub const OUTPUT_STREAM: &str = "အချက်အလက်အထွက်စီးကြောင်း";

/// The enumeration each host function can fail with.
pub const IO_ERROR: &str = "အချက်အလက်အဝင်အထွက်အမှားအယွင်း";

/// The handle of standard input, `စံပြုအဝင်`.
pub const STDIN: u32 = 0;

/// The handle of standard output, `စံပြုအထွက်`.
pub const STDOUT: u32 = 1;

/// The handle of standard error, `စံပြုအမှား`.
pub const STDERR: u32 = 2;

/// The member of [`IO_ERROR`] for a file that is not there, `မတွေ့ခြင်း`.
pub const NOT_FOUND: u32 = 0;

/// The member for an effect the OS does not allow,
/// `ခွင့်ပြုချက်မရှိခြင်း`.
pub const NO_PERMISSION: u32 = 1;

/// The member for the end of the input, `အဝင်ဒေတာအဆုံးကိုရောက်ခြင်း`.
pub const END_OF_INPUT: u32 = 2;

/// The member for octets that are not UTF-8 text, `စာသားဖွဲ့စည်းပုံမမှန်ခြင်း`.
pub const BAD_TEXT: u32 = 3;

/// The member for each other failure, `တခြားအကြောင်းအရင်း`.
pub const OTHER: u32 = 4;
