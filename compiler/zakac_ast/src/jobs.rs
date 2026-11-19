//! The phrase that marks a shared object as one jobs on other threads can
//! use, and names the trait of each kind such jobs can use. Neither word is
//! reserved: the parser reads them through their text where the phrase can
//! stand, so each stays free to name a variable.

/// The two words of the phrase, in order.
pub const JOBS_WORDS: [&str; 2] = ["အပြိုင်လုပ်ငန်းများတွင်", "အသုံးပြုနိုင်သော"];

/// The name of the channel kind the core declares, which the room of a
/// bounded channel, `<kind> <n> ခု ပို့နိုင်သော ဒေတာလမ်းကြောင်း`, goes before.
pub const PIPE_KIND: &str = "ဒေတာလမ်းကြောင်း";

/// The words before the integer kind of an atomic number, `အက်တမ်မစ် ဖြစ်သော
/// <kind>`; the first names the kind.
pub const ATOMIC_WORDS: [&str; 2] = ["အက်တမ်မစ်", "ဖြစ်သော"];

/// The name of the built-in trait of the kinds jobs can use: the two words
/// with a space between them.
pub const USABLE_IN_JOBS: &str = "အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော";
