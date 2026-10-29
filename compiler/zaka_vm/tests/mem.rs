use zaka_vm::mem::Mem;
use zaka_vm::mem::Origin;
use zaka_vm::trap::TrapKind;
use zakac_ir_builder::value::Value;
use zakac_rust_sugars::do_for;
use zakac_semcheck::kind::DataKind;

fn mem() -> Mem { Mem::new(64, 1 << 20) }
fn k(n: &str) -> DataKind { DataKind::parse(n).unwrap() }
fn v(t: &str) -> Value { t.parse().unwrap() }

#[test]
fn blocks_start_past_the_null_page_and_keep_a_red_zone()
{
    let mut m = mem();
    let a = m.alloc(10, Origin::Heap, 0).unwrap();
    let b = m.alloc(1, Origin::Heap, 0).unwrap();
    assert!(a >= 4096 && a.is_multiple_of(16));
    assert!(b >= a + 10 + 16);
    assert!(m.alloc(0, Origin::Heap, 0).is_some(), "a zero-size block is a real block");
}

#[test]
fn loads_and_stores_check_each_access()
{
    let mut m = mem();
    let a = m.alloc(8, Origin::Heap, 0).unwrap();
    assert_eq!(m.load(a, k("i32")), Ok(v("0:i32")), "fresh blocks read as zeros");
    assert_eq!(m.store(a, v("-2:i32")), Ok(()));
    assert_eq!(m.load(a, k("i32")), Ok(v("-2:i32")));
    assert_eq!(m.load(0, k("i32")), Err(TrapKind::NullDeref { addr: 0 }));
    assert_eq!(
        m.load(a + 6, k("i32")),
        Err(TrapKind::Misaligned { addr: a + 6, align: 4 })
    );
    assert_eq!(
        m.load(a + 8, k("i32")),
        Err(TrapKind::OutOfBounds { addr: a + 8, size: 4 })
    );
    m.store(a, v("2:u8")).unwrap();
    assert_eq!(
        m.load(a, k("bool")),
        Err(TrapKind::BadValue { addr: a, kind: "bool".into() })
    );
    m.free(a).unwrap();
    assert_eq!(m.load(a, k("i32")), Err(TrapKind::UseAfterFree { addr: a }));
    assert_eq!(m.free(a), Err(TrapKind::DoubleFree { addr: a }));
    assert_eq!(m.free(a + 4), Err(TrapKind::BadFree { addr: a + 4 }));
    assert_eq!(m.free(0), Ok(()));
}

#[test]
fn stack_blocks_go_back_with_their_call_and_their_room_is_taken_again()
{
    let mut m = mem();
    m.open_stack(1 << 10);
    let top = m.stack_top();
    let c = m.push_stack(8, 2).unwrap();
    assert_eq!(m.free(c), Err(TrapKind::BadFree { addr: c }));
    m.store(c, v("7:i64")).unwrap();
    m.pop_stack(&[c], top);
    assert_eq!(m.load(c, k("i64")), Err(TrapKind::UseAfterFree { addr: c }));
    assert_eq!(m.stack_top(), top);
    assert_eq!(m.push_stack(8, 2), Some(c), "the next call takes the same room");
    assert_eq!(m.load(c, k("i64")), Ok(v("0:i64")), "a new block starts at zero");
    assert_eq!(m.blocks().count(), 1, "the freed block left the table");
}

#[test]
fn the_stack_region_holds_blocks_up_to_its_cap_and_the_heap_starts_past_it()
{
    let mut m = mem();
    m.open_stack(64);
    let first = m.push_stack(8, 1).unwrap();
    assert!(m.push_stack(8, 1).is_some());
    assert_eq!(m.push_stack(8, 1), None, "two blocks with their red zones fill 64");
    let h = m.alloc(8, Origin::Heap, 0).unwrap();
    assert!(h >= first + 64, "the heap starts past the stack region");
}

#[test]
fn realloc_keeps_the_prefix_and_respects_the_cap()
{
    let mut m = Mem::new(64, 64);
    let a = m.alloc(8, Origin::Heap, 0).unwrap();
    m.store(a, v("7:i64")).unwrap();
    let b = m.realloc(a, 32).unwrap();
    assert_eq!(m.load(b, k("i64")), Ok(v("7:i64")));
    assert_eq!(m.load(b + 8, k("i64")), Ok(v("0:i64")));
    assert_eq!(m.realloc(b, 128), Err(TrapKind::OutOfMem { size: 128 }));
    assert_eq!(m.realloc(0, 16).map(|p| p != 0), Ok(true));
    assert!(m.alloc(64, Origin::Heap, 0).is_none(), "over the cap");
    assert_eq!(m.leaks().len(), 2);
}

#[test]
fn a_small_platform_reuses_freed_ranges_when_full()
{
    let mut m = Mem::new(8, 1 << 20); // 256 addresses, 16 reserved
    let a = m.alloc(32, Origin::Heap, 0).unwrap();
    let mut taken = vec![a];
    while let Some(p) = m.alloc(32, Origin::Heap, 0)
    {
        taken.push(p);
    }
    m.free(a).unwrap();
    assert_eq!(
        m.alloc(32, Origin::Heap, 0),
        Some(a),
        "the oldest freed range is reused"
    );
}

#[test]
fn same_block_reaches_the_end_of_a_live_block()
{
    let mut m = mem();
    let a = m.alloc(8, Origin::Heap, 0).unwrap();
    let z = m.alloc(0, Origin::Heap, 0).unwrap();
    let b = m.alloc(8, Origin::Heap, 0).unwrap();
    assert!(m.same_block(a, a + 8), "one past the end counts");
    assert!(m.same_block(a + 8, a));
    assert!(m.same_block(z, z), "a zero-size block holds its own start");
    assert!(!m.same_block(a, b), "two live blocks differ");
    assert!(!m.same_block(a, a + 9), "past the end and into the red zone");
    m.free(a).unwrap();
    assert!(!m.same_block(a, a + 4), "a freed block is not live");
}

#[test]
fn a_freed_block_drops_its_contents_but_keeps_its_length()
{
    let mut m = mem();
    let a = m.alloc(4096, Origin::Heap, 0).unwrap();
    m.store(a, v("7:i64")).unwrap();
    m.free(a).unwrap();
    let block = m.blocks().find(|b| b.start == a).unwrap();
    assert!(!block.live);
    assert_eq!(block.len, 4096);
    assert!(block.data.is_empty());
    assert_eq!(m.load(a, k("i64")), Err(TrapKind::UseAfterFree { addr: a }));
    let past = a + 4000;
    assert_eq!(m.load(past, k("i64")), Err(TrapKind::UseAfterFree { addr: past }));
    assert_eq!(m.block_near(a + 4096).map(|b| b.start), Some(a));
}

#[test]
fn realloc_counts_the_change_against_the_cap()
{
    let mut m = Mem::new(64, 64);
    let a = m.alloc(48, Origin::Heap, 0).unwrap();
    m.store(a, v("7:i64")).unwrap();
    let b = m.realloc(a, 32).expect("shrinking near the cap");
    assert_eq!(m.load(b, k("i64")), Ok(v("7:i64")));
    assert_eq!(m.live_heap(), 32);
    let c = m.realloc(b, 64).expect("growing up to the cap");
    assert_eq!(m.live_heap(), 64);
    assert_eq!(m.realloc(c, 65), Err(TrapKind::OutOfMem { size: 65 }));
    assert_eq!(m.live_heap(), 64);
    assert_eq!(m.load(c, k("i64")), Ok(v("7:i64")), "a failed resize keeps the block");
}

#[test]
fn a_list_copies_and_clears_as_one_checked_access()
{
    let mut m = mem();
    let a = m.alloc(24, Origin::Heap, 0).unwrap();
    let b = m.alloc(24, Origin::Heap, 0).unwrap();
    do_for!(i in 0..3u64, m.store(a + 8 * i, v(&format!("{}:isize", i + 1))).unwrap());
    assert_eq!(m.blit(b, a, k("[3]isize")), Ok(()));
    assert_eq!(m.load(b + 16, k("isize")), Ok(v("3:isize")));
    assert_eq!(m.zero(a, k("[3]isize")), Ok(()));
    assert_eq!(m.load(a + 8, k("isize")), Ok(v("0:isize")));
    assert_eq!(
        m.load(b + 8, k("isize")),
        Ok(v("2:isize")),
        "the duplicate stands apart"
    );
    let past = TrapKind::OutOfBounds { addr: b, size: 32 };
    assert_eq!(
        m.blit(b, a, k("[4]isize")),
        Err(TrapKind::OutOfBounds { addr: a, size: 32 })
    );
    assert_eq!(m.zero(b, k("[4]isize")), Err(past));
    assert_eq!(
        m.zero(b + 4, k("[2]isize")),
        Err(TrapKind::Misaligned { addr: b + 4, align: 8 })
    );
}

#[test]
fn a_step_lands_in_the_block_of_its_base()
{
    let mut m = mem();
    let a = m.alloc(8, Origin::Heap, 0).unwrap();
    let b = m.alloc(8, Origin::Heap, 0).unwrap();
    assert!(m.steps_within(a, a + 8), "one past the end counts");
    assert!(m.steps_within(a + 8, a), "back from one past the end");
    assert!(!m.steps_within(a, a + 9), "into the red zone");
    assert!(!m.steps_within(a, b), "into the next block");
    assert!(m.steps_within(a - 64, b), "a base in no block is not checked");
    m.free(a).unwrap();
    assert!(!m.steps_within(a, b), "a freed block still has its bounds");
}
