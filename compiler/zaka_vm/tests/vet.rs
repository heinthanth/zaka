use std::path::Path;
use std::path::PathBuf;

use strum::IntoEnumIterator;
use zaka_vm::machine::RunOptions;
use zaka_vm::vet::Malformed;
use zaka_vm::vet::vet_module;
use zakac_codegen_target_vm::bc::Func;
use zakac_codegen_target_vm::bc::Module;
use zakac_codegen_target_vm::bc::Op;
use zakac_codegen_target_vm::options::EmitOptions;
use zakac_codegen_target_vm::options::ResultMode;
use zakac_codegen_target_vm::options::ScratchMode;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::value::Value;

use crate::common::compile_file;

/// Sample groups that hold programs with errors on purpose.
const SKIPPED_GROUPS: &[&str] = &["diagnostics", "mistakes"];

/// The samples dir.
fn samples() -> PathBuf
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples")
}

/// The size limit the command line gives with its default limits.
fn limit() -> u64
{
    let options = RunOptions::default();
    options.max_heap.max(options.max_stack)
}

/// The fault of a module that must fail the check.
fn rejected(module: &Module) -> Malformed
{
    vet_module(module, limit()).expect_err("the check passes a corrupt module")
}

/// A sample compiled with the default choices.
fn sample(name: &str) -> Module
{
    compile_file(&samples().join(name), EmitOptions::default())
}

/// Put an op in a function and move each jump past it one op on.
fn insert_op(func: &mut Func, at: usize, op: Op)
{
    let moved = |t: u32| if t as usize >= at { t + 1 } else { t };
    func.code
        .iter_mut()
        .for_each(|o| *o = o.target().map_or(*o, |t| o.retarget(moved(t))));
    func.code.insert(at, op);
    func.spans.insert(at, func.spans[at]);
}

/// Take an op out of a function and move each jump past it one op back.
fn remove_op(func: &mut Func, at: usize)
{
    let moved = |t: u32| if t as usize > at { t - 1 } else { t };
    func.code.remove(at);
    func.spans.remove(at);
    func.code
        .iter_mut()
        .for_each(|o| *o = o.target().map_or(*o, |t| o.retarget(moved(t))));
}

/// The sample files of the groups the check covers, sorted.
fn sample_files(root: &Path) -> Vec<PathBuf>
{
    let groups = std::fs::read_dir(root).unwrap().flatten().map(|g| g.path());
    let groups = groups.filter(|p| p.is_dir());
    let groups = groups.filter(|p| !SKIPPED_GROUPS.iter().any(|s| p.ends_with(s)));
    let files =
        groups.flat_map(|g| std::fs::read_dir(g).unwrap().flatten().map(|f| f.path()));
    let mut files: Vec<PathBuf> =
        files.filter(|p| p.extension().is_some_and(|e| e == "zaka")).collect();
    files.sort();
    files
}

/// Each repo sample, with each result and scratch choice, passes the check
/// a loaded artifact passes: the check rejects nothing the compiler makes.
#[test]
fn each_sample_passes_the_check()
{
    let files = sample_files(&samples());
    assert!(files.len() >= 30, "found {} samples", files.len());
    let emits = ResultMode::iter().flat_map(|r| {
        ScratchMode::iter().map(move |s| EmitOptions { results: r, scratch: s })
    });
    let emits: Vec<EmitOptions> = emits.collect();
    for file in &files
    {
        for emit in &emits
        {
            let module = compile_file(file, *emit);
            let vetted = vet_module(&module, limit());
            assert_eq!(vetted, Ok(()), "{} ({emit:?})", file.display());
        }
    }
}

/// A jump past the last op fails the check.
#[test]
fn a_jump_past_the_end_is_rejected()
{
    let mut module = sample("programs/gcd.zaka");
    let func =
        module.funcs.iter_mut().find(|f| f.code.iter().any(|o| o.target().is_some()));
    let func = func.expect("a function with a jump");
    let at = func.code.iter().position(|o| o.target().is_some()).unwrap();
    let past = func.code.len() as u32 + 3;
    func.code[at] = func.code[at].retarget(past);
    let fault = rejected(&module);
    assert_eq!(fault.pc, Some(at as u32));
    assert!(fault.reason.contains("past the end"), "{fault}");
}

/// A function that returns the address of its result, its load cut out,
/// fails the check.
#[test]
fn a_removed_load_is_rejected()
{
    let mut module = sample("programs/gcd.zaka");
    let found = module.funcs.iter_mut().find_map(|f| {
        let at = f.code.windows(2).position(|w| matches!(w, [Op::Load(_), Op::Ret]));
        at.map(|at| (f, at))
    });
    let (func, at) = found.expect("a load before a ret");
    remove_op(func, at);
    let fault = rejected(&module);
    assert!(fault.reason.contains("reads Int"), "{fault}");
}

/// A value pushed that nothing pops leaves the main function with one value
/// on the stack, where it gives none.
#[test]
fn an_extra_push_is_rejected()
{
    let mut module = sample("programs/gcd.zaka");
    let main = module.main as usize;
    insert_op(&mut module.funcs[main], 0, Op::Push(Value::Bool(true)));
    let fault = rejected(&module);
    assert!(fault.reason.contains("ret leaves 1 values"), "{fault}");
}

/// A C function whose signature lost a parameter pops one value too few.
#[test]
fn a_foreign_call_with_a_lost_parameter_is_rejected()
{
    let mut module = sample("ffi/libc.zaka");
    let def = module.foreign.iter_mut().find(|f| f.params.len() >= 2);
    def.expect("a C function with two parameters").params.pop();
    rejected(&module);
}

/// A C function whose integer parameter became an address reads an integer
/// where it wants an address.
#[test]
fn a_foreign_parameter_of_another_sort_is_rejected()
{
    let mut module = sample("ffi/libc.zaka");
    let mut params = module.foreign.iter_mut().flat_map(|f| f.params.iter_mut());
    let param = params.find(|k| k.is_int()).expect("an integer parameter");
    *param = DataKind::OPAQUE;
    let fault = rejected(&module);
    assert!(fault.reason.contains("reads Addr"), "{fault}");
}

/// A list kind too large to size, or larger than the storage limits, fails
/// the check before the VM claims storage for it.
#[test]
fn a_huge_list_kind_is_rejected()
{
    let kinds = [
        (DataKind::list_of(DataKind::ISIZE, i32::MAX as u32), "has no size"),
        (DataKind::list_of(DataKind::ISIZE, 1 << 28), "pass the limit"),
    ];
    for (kind, reason) in kinds
    {
        let mut module = sample("programs/gcd.zaka");
        let mut load = module.funcs.iter_mut().flat_map(|f| f.code.iter_mut());
        *load.find(|o| matches!(o, Op::Load(_))).expect("a load") = Op::Load(kind);
        let fault = rejected(&module);
        assert!(fault.reason.contains(reason), "{fault}");
    }
}
