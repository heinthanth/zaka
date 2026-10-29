mod common;

use std::collections::BTreeSet;
use std::path::PathBuf;

use common::codes;
use zakac_diagnostics::Code;

/// The dir of the VM's heap trap inputs.
fn heap_inputs() -> PathBuf
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../zaka_vm/tests/inputs/heap")
}

/// The stems of the `.zaka` files in the heap inputs dir.
fn heap_stems() -> BTreeSet<String>
{
    std::fs::read_dir(heap_inputs())
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "zaka"))
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect()
}

/// Each VM heap trap input with the codes the checker gives it. The
/// inputs the checker cannot judge before the run (a misaligned read, a
/// bad octet read as a truth value, a heap cap, a step off a `ဗလာ`
/// parameter) give none, and the VM stops them.
const TABLE: [(&str, &[Code]); 12] = [
    ("use_after_free", &[Code::InvalidatedLoan]),
    ("double_free", &[Code::InvalidatedLoan, Code::FreedBorrow]),
    ("bad_free", &[Code::FreedBorrow]),
    ("local_escape", &[Code::LocalBorrowReturned]),
    ("null_deref", &[Code::NullDeref]),
    ("null_step", &[]),
    ("out_of_bounds", &[Code::OffsetPastEnd]),
    ("mixed_blocks", &[Code::MixedBlocks]),
    ("index_out_of_range", &[Code::OffsetPastEnd]),
    ("misaligned", &[]),
    ("bad_value", &[]),
    ("out_of_mem", &[]),
];

#[test]
fn each_heap_input_has_its_code()
{
    let want: BTreeSet<String> = TABLE.iter().map(|(s, _)| s.to_string()).collect();
    assert_eq!(heap_stems(), want, "each input needs a row");
    for (stem, want) in TABLE
    {
        let text = std::fs::read_to_string(heap_inputs().join(format!("{stem}.zaka")))
            .unwrap();
        assert_eq!(codes(&text), want, "{stem}");
    }
}
