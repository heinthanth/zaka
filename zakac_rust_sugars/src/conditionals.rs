/// Do `$s` if the condition `$c` is met.
#[rustfmt::skip]
#[macro_export]
macro_rules! do_if {
(@split [$($c:tt)*] , $s:expr) => { if $($c)* { $s; } };
(@split [$($c:tt)*] $t:tt $($r:tt)*) => { $crate::do_if!(@split [$($c)* $t] $($r)*) };
($($t:tt)*) => { $crate::do_if!(@split [] $($t)*) }; }

/// Returns the value `$v` if the condition `$c` is met.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_if {
(@split [$($c:tt)*] , $v:expr) => { if $($c)* { return $v; } };
(@split [$($c:tt)*]) => { if $($c)* { return; } };
(@split [$($c:tt)*] $t:tt $($r:tt)*) => { $crate::ret_if!(@split [$($c)* $t] $($r)*) };
($($t:tt)*) => { $crate::ret_if!(@split [] $($t)*) }; }

/// Returns `true` if the condition `$c` is met.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_true {
($($t:tt)*) => { $crate::ret_if!($($t)*, true); }; }

/// Returns `false` if the condition `$c` is met.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_false {
($($t:tt)*) => { $crate::ret_if!($($t)*, false); }; }

/// Evaluates to the value `$v` if the condition `$c` is met,
/// otherwise evaluates to the value `$vf`.
#[rustfmt::skip]
#[macro_export]
macro_rules! tern {
(@split [$($c:tt)*] , $v:expr, $vf:expr) => { if $($c)* { $v } else { $vf } };
(@split [$($c:tt)*] $t:tt $($r:tt)*) => { $crate::tern!(@split [$($c)* $t] $($r)*) };
($($t:tt)*) => { $crate::tern!(@split [] $($t)*) }; }

/// Returns the value `$vt` if the condition `$c` is met,
/// otherwise returns `$vf`.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_tern {
($($t:tt)*) => { return $crate::tern!($($t)*); }; }
