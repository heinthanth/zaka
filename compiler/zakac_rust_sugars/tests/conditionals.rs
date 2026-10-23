use zakac_rust_sugars::do_if;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_tern;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;

#[test]
fn do_if_plain()
{
    let mut n = 0;
    do_if!(true, n += 1);
    do_if!(false, n += 10);
    assert_eq!(n, 1);
}

#[test]
fn do_if_multiple_stmts()
{
    let mut n = 0;
    do_if!(true, n += 1, n += 2);
    do_if!(false, n += 10, n += 20);
    assert_eq!(n, 3);
}

#[test]
fn do_if_call_with_commas()
{
    let mut n = 0;
    do_if!(true, n = i32::saturating_add(n, 2));
    assert_eq!(n, 2);
}

#[test]
fn do_if_let()
{
    let mut n = 0;
    do_if!(let Some(a) = Some(3), n = a);
    do_if!(let Some(_a) = None::<i32>, n = 99);
    assert_eq!(n, 3);
}

#[test]
fn do_if_let_tuple()
{
    let mut n = 0;
    do_if!(let Some((a, b)) = Some((1, 2)), n = a + b);
    assert_eq!(n, 3);
}

#[test]
fn do_if_let_chain()
{
    let mut n = 0;
    do_if!(let Some(a) = Some(2) && a > 1 && let Some(b) = Some(5), n = a + b);
    do_if!(let Some(a) = Some(2) && a > 10, n = 99);
    assert_eq!(n, 7);
}

fn ret_if_unit(stop: bool, continued: &mut bool)
{
    ret_if!(stop);
    *continued = true;
}

fn ret_if_value(stop: bool, value: i32) -> i32
{
    ret_if!(stop, value);
    0
}

fn ret_if_from_opt(opt: Option<i32>) -> i32
{
    ret_if!(let Some(a) = opt, a);
    -1
}

fn ret_if_from_pair(opt: Option<(i32, i32)>) -> i32
{
    ret_if!(let Some((a, b)) = opt, a + b);
    -1
}

fn ret_if_from_chain(opt: Option<i32>, other: Option<i32>) -> i32
{
    ret_if!(let Some(a) = opt && a > 1 && let Some(b) = other, a + b);
    -1
}

#[test]
fn ret_if_plain()
{
    let mut continued = false;
    ret_if_unit(true, &mut continued);
    assert!(!continued);
    ret_if_unit(false, &mut continued);
    assert!(continued);
}

#[test]
fn ret_if_with_value()
{
    assert_eq!(ret_if_value(true, 7), 7);
    assert_eq!(ret_if_value(false, 7), 0);
}

#[test]
fn ret_if_call_with_commas()
{
    fn go(cond: bool) -> i32
    {
        ret_if!(cond, i32::saturating_add(1, 2));
        0
    }
    assert_eq!(go(true), 3);
    assert_eq!(go(false), 0);
}

#[test]
fn ret_if_stmts_before_value()
{
    let mut acc = 0;
    fn go(opt: Option<i32>, acc: &mut i32) -> i32
    {
        ret_if!(let Some(n) = opt, *acc += n, *acc);
        -1
    }
    assert_eq!(go(Some(3), &mut acc), 3);
    assert_eq!(acc, 3);
    assert_eq!(go(None, &mut acc), -1);
    assert_eq!(acc, 3);
}

#[test]
fn ret_if_let()
{
    assert_eq!(ret_if_from_opt(Some(3)), 3);
    assert_eq!(ret_if_from_opt(None), -1);
}

#[test]
fn ret_if_let_tuple()
{
    assert_eq!(ret_if_from_pair(Some((1, 2))), 3);
    assert_eq!(ret_if_from_pair(None), -1);
}

#[test]
fn ret_if_let_chain()
{
    assert_eq!(ret_if_from_chain(Some(2), Some(5)), 7);
    assert_eq!(ret_if_from_chain(Some(2), None), -1);
    assert_eq!(ret_if_from_chain(Some(1), Some(5)), -1);
}

fn flag_true(cond: bool) -> bool
{
    ret_true!(cond);
    false
}

fn flag_false(cond: bool) -> bool
{
    ret_false!(cond);
    true
}

#[test]
fn ret_true_plain()
{
    assert!(flag_true(true));
    assert!(!flag_true(false));
}

#[test]
fn ret_true_let()
{
    fn go(opt: Option<i32>) -> bool
    {
        ret_true!(let Some(a) = opt && a > 0);
        false
    }
    assert!(go(Some(1)));
    assert!(!go(Some(0)));
    assert!(!go(None));
}

#[test]
fn ret_false_plain()
{
    assert!(!flag_false(true));
    assert!(flag_false(false));
}

#[test]
fn ret_false_let()
{
    fn go(opt: Option<i32>) -> bool
    {
        ret_false!(let Some(a) = opt && a > 0);
        true
    }
    assert!(!go(Some(1)));
    assert!(go(Some(0)));
    assert!(go(None));
}

#[test]
fn tern_plain()
{
    assert_eq!(tern!(true, 1, 2), 1);
    assert_eq!(tern!(false, 1, 2), 2);
}

#[test]
fn tern_stmts_before_true_value()
{
    let mut acc = 0;
    assert_eq!(
        tern!(
            true,
            {
                acc += 1;
            },
            acc,
            -1
        ),
        1
    );
    assert_eq!(acc, 1);
    assert_eq!(
        tern!(
            false,
            {
                acc += 10;
            },
            acc,
            -1
        ),
        -1
    );
    assert_eq!(acc, 1);
}

#[test]
fn tern_call_with_commas()
{
    assert_eq!(tern!(true, i32::saturating_add(1, 2), i32::saturating_add(3, 4)), 3);
    assert_eq!(tern!(false, i32::saturating_add(1, 2), i32::saturating_add(3, 4)), 7);
}

#[test]
fn tern_let()
{
    assert_eq!(tern!(let Some(a) = Some(3), a, 0), 3);
    assert_eq!(tern!(let Some(a) = None::<i32>, a, 0), 0);
}

#[test]
fn tern_let_tuple()
{
    assert_eq!(tern!(let Some((a, b)) = Some((1, 2)), a + b, 0), 3);
    assert_eq!(tern!(let Some((a, b)) = None::<(i32, i32)>, a + b, 0), 0);
}

#[test]
fn tern_let_chain()
{
    assert_eq!(
        tern!(let Some(a) = Some(2) && a > 1 && let Some(b) = Some(5), a + b, 0),
        7
    );
    assert_eq!(tern!(let Some(a) = Some(2) && a > 10, a, 0), 0);
}

fn pick(cond: bool) -> i32
{
    ret_tern!(cond, 1, 2);
}

fn pick_from_opt(opt: Option<i32>) -> i32
{
    ret_tern!(let Some(a) = opt, a, -1);
}

#[test]
fn ret_tern_plain()
{
    assert_eq!(pick(true), 1);
    assert_eq!(pick(false), 2);
}

#[test]
fn ret_tern_call_with_commas()
{
    fn go(cond: bool) -> i32
    {
        ret_tern!(cond, i32::saturating_add(1, 2), i32::saturating_add(3, 4));
    }
    assert_eq!(go(true), 3);
    assert_eq!(go(false), 7);
}

#[test]
fn ret_tern_let()
{
    assert_eq!(pick_from_opt(Some(3)), 3);
    assert_eq!(pick_from_opt(None), -1);
}

#[test]
fn ret_tern_let_chain()
{
    fn go(opt: Option<i32>, other: Option<i32>) -> i32
    {
        ret_tern!(let Some(a) = opt && a > 1 && let Some(b) = other, a + b, 0);
    }
    assert_eq!(go(Some(2), Some(5)), 7);
    assert_eq!(go(Some(2), None), 0);
}

enum Kind
{
    A(i32),
    B,
}

fn kind_a(kind: &Kind) -> i32
{
    extract!(Kind::A(n) = kind);
    *n
}

#[test]
fn extract_binds()
{
    assert_eq!(kind_a(&Kind::A(3)), 3);
}

#[test]
fn extract_tuple()
{
    extract!(Some((a, b)) = Some((1, 2)));
    assert_eq!(a + b, 3);
}

#[test]
fn extract_with_message()
{
    extract!(Some(n) = Some(4), "expected Some");
    assert_eq!(n, 4);
}

#[test]
#[should_panic(expected = "internal error: entered unreachable code")]
fn extract_panics() { kind_a(&Kind::B); }

#[test]
#[should_panic(expected = "expected A")]
fn extract_panics_with_message()
{
    let kind = Kind::B;
    extract!(Kind::A(_n) = kind, "expected A");
}
