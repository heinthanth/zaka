use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_while;

#[test]
fn do_for_ident()
{
    let mut out = Vec::new();
    do_for!(x in [1, 2, 3], out.push(x));
    assert_eq!(out, [1, 2, 3]);
}

#[test]
fn do_for_pattern()
{
    let mut out = String::new();
    let pairs = [(1u8, 'a'), (0u8, 'b')];
    do_for!((_, ch) in pairs, out.push(ch));
    assert_eq!(out, "ab");
}

#[test]
fn do_for_with_commas()
{
    let mut out = Vec::new();
    do_for!(x in [1, 2], out.push(i32::saturating_add(x, 1)));
    assert_eq!(out, [2, 3]);
}

#[test]
fn do_while_ident()
{
    let mut out = Vec::new();
    let mut i = 0;
    do_while!(i < 3, out.push(i), i += 1);
    assert_eq!(out, [0, 1, 2]);
}

#[test]
fn do_while_pattern()
{
    let mut out = String::new();
    let pairs = [(1u8, 'a'), (0u8, 'b')];
    let mut i = 0;
    do_while!(i < pairs.len(), out.push(pairs[i].1), i += 1);
    assert_eq!(out, "ab");
}
