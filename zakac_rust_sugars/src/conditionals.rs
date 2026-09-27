/// Run `$($s)+` if the condition `$c` is met.
#[rustfmt::skip]
#[macro_export]
macro_rules! do_if {
(@split [$($c:tt)*] , $($s:stmt),+ $(,)?) => { if $($c)* { $($s)+ } };
(@split [$($c:tt)*] , $s:stmt) => { if $($c)* { $s; } };
(@split [$($c:tt)*] $t:tt $($r:tt)*) => { $crate::do_if!(@split [$($c)* $t] $($r)*) };
($($t:tt)*) => { $crate::do_if!(@split [] $($t)*) }; }

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
macro_rules! ret_true {
($($t:tt)*) => { $crate::ret_if!($($t)*, true); }; }

/// Returns `false` if the condition `$c` is met.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_false {
($($t:tt)*) => { $crate::ret_if!($($t)*, false); }; }

/// Evaluates to `$vt` if the condition `$c` is met, otherwise `$vf`.
///
/// With `$($s)+ , $vt , $vf`, runs `$($s)+` in the true branch before `$vt`.
#[rustfmt::skip]
#[macro_export]
macro_rules! tern {
(@split [$($c:tt)*] , $($rest:tt)+) => { $crate::tern!(@tern_peel [$($c)*] [] $($rest)+) };
(@split [$($c:tt)*] $t:tt $($r:tt)*) => { $crate::tern!(@split [$($c)* $t] $($r)*) };
(@tern_peel [$($c:tt)*] [$($acc:expr)*] $vt:expr , $vf:expr) => {
    if $($c)* { $($acc;)* $vt } else { $vf } };
(@tern_peel [$($c:tt)*] [$($acc:expr)*] $e:expr , $($more:tt)+) => {
    $crate::tern!(@tern_peel [$($c)*] [$($acc)* $e] $($more)+) };
($($t:tt)*) => { $crate::tern!(@split [] $($t)*) }; }

/// Returns the value `$vt` if the condition `$c` is met,
/// otherwise returns `$vf`.
#[rustfmt::skip]
#[macro_export]
macro_rules! ret_tern {
($($t:tt)*) => { return $crate::tern!($($t)*); }; }

/// Extracts `$p` from `$e`, otherwise `unreachable!`.
#[rustfmt::skip]
#[macro_export]
macro_rules! extract {
($p:pat = $e:expr) => { let $p = $e else { unreachable!() }; };
($p:pat = $e:expr, $($t:tt)*) => { let $p = $e else { unreachable!($($t)*) }; }; }
