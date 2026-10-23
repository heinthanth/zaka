/// Returns `$v` if the condition `$c` is met.
///
/// With `$($s)+ , $v`, runs comma-separated `$($s)+` first, then returns `$v`.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_if {
(@split [$($c:tt)*] , $($rest:tt)+) => { $crate::ret_if!(@peel [$($c)*] [] $($rest)+) };
(@split [$($c:tt)*]) => { if $($c)* { return; } };
(@split [$($c:tt)*] $t:tt $($r:tt)*) => { $crate::ret_if!(@split [$($c)* $t] $($r)*) };
(@peel [$($c:tt)*] [$($acc:expr)*] $v:expr) => { if $($c)* { $($acc;)* return $v; } };
(@peel [$($c:tt)*] [$($acc:expr)*] $e:expr , $($more:tt)+) => {
    $crate::ret_if!(@peel [$($c)*] [$($acc)* $e] $($more)+) };
($($t:tt)*) => { $crate::ret_if!(@split [] $($t)*) }; }

/// Returns `true` if the condition `$c` is met.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_true { ($($t:tt)*) => { $crate::ret_if!($($t)*, true); }; }

/// Returns `false` if the condition `$c` is met.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_false { ($($t:tt)*) => { $crate::ret_if!($($t)*, false); }; }

/// Returns the value `$vt` if the condition `$c` is met,
/// otherwise returns `$vf`.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_tern { ($($t:tt)*) => { return $crate::tern!($($t)*); }; }
