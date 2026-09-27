/// Run `$s` for each item matched by `for $($head)*`.
#[rustfmt::skip]
#[macro_export]
macro_rules! do_for {
(@split [$($head:tt)*] , $s:expr) => { for $($head)* { $s; } };
(@split [$($head:tt)*] $t:tt $($r:tt)*) => { $crate::do_for!(@split [$($head)* $t] $($r)*) };
($($t:tt)*) => { $crate::do_for!(@split [] $($t)*) }; }

/// Run `$($s)+` while the condition `$c` is met.
#[rustfmt::skip]
#[macro_export]
macro_rules! do_while {
(@split [$($head:tt)*] , $($s:stmt),+ $(,)?) => { while $($head)* { $($s)+ } };
(@split [$($head:tt)*] , $s:stmt) => { while $($head)* { $s; } };
(@split [$($head:tt)*] $t:tt $($r:tt)*) => { $crate::do_while!(@split [$($head)* $t] $($r)*) };
($($t:tt)*) => { $crate::do_while!(@split [] $($t)*) }; }
