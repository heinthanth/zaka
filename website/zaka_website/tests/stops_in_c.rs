//! Each program a page shows stopping stops alike as a native executable
//! from the C target: the same output, then the same reason.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::process::Stdio;

use zaka_website::book::Book;
use zaka_website::book::render::Block;
use zaka_website::book::render::Sample;
use zaka_website::book::render::render;
use zaka_website::components::sandbox::run::Fence;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_codegen_target_c::driver::translate;
use zakac_codegen_target_c::driver::write_cache;
use zakac_codegen_target_c::ffi::link_libs;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::options::TargetCOptions;
use zakac_codegen_target_c::toolchain;
use zakac_codegen_target_c::toolchain::BuildPlan;
use zakac_codegen_target_c::toolchain::Tool;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::Loaded;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The word of marked code. Mistakes in marked pointer code are allowed to
/// differ between the VM and native code, so such programs are left out.
const MARKED: &str = "စိတ်မချရ";

/// The words of a C function. The VM links a C function when the program
/// first calls it, a native build when it links, so a missing one stops the
/// VM and fails the build: such programs are left out.
const FOREIGN: &str = "မှလာသော";

/// The folder of the pages on the VM, whose stops show limits of the VM
/// itself, such as the size of its stack region, which native code does not
/// share.
const VM_PAGES: &str = "vm/";

/// Each program of each book that its page shows stopping, with its page.
fn stops() -> Vec<(&'static str, Sample)>
{
    let pages = Book::ALL.into_iter().flat_map(Book::pages);
    let samples = pages.flat_map(|p| {
        let path = p.path;
        render(p).blocks.into_iter().filter_map(move |b| match b
        {
            Block::Zaka(s) => Some((path, s)),
            Block::Prose(_) => None,
        })
    });
    let shared = |s: &Sample| ![MARKED, FOREIGN].iter().any(|w| s.source.contains(w));
    let kept = |(p, s): &(&str, Sample)| {
        s.fence == Fence::Stop && shared(s) && !p.starts_with(VM_PAGES)
    };
    samples.filter(kept).collect()
}

/// Text with each hex address written as `0x?`: a native address differs from
/// the VM's.
fn without_addresses(text: &str) -> String
{
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("0x")
    {
        out.push_str(&rest[..at + 2]);
        out.push('?');
        rest = rest[at + 2..].trim_start_matches(|c: char| c.is_ascii_hexdigit());
    }
    out.push_str(rest);
    out
}

/// Build a program with the C target, run it with its input, and give
/// what it printed and the reason it stopped, as its page writes them.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `sample` - The program.
///
/// # Returns
///
/// The output, or `None` when the C target cannot build the program.
fn native_output(cc: &Tool, sample: &Sample) -> Option<String>
{
    let mut sm = SourceMap::new();
    let name = FileName::Custom("main.zaka".into());
    let file = sm.new_source_file(name, sample.source.clone());
    let loaded = Loaded::single(&file, &LexerOptions::default());
    let fence = sample.options;
    let options = SemcheckOptions::builder().no_unsafe(fence.no_unsafe).build();
    #[rustfmt::skip]
    let mem = BorrowCheckOptions::builder().check(fence.check).strict(fence.strict)
        .overflow(fence.overflow).build();
    let fold = FoldOptions::default();
    let out =
        translate(loaded, sm.files(), &options, &fold, &mem, TargetCOptions::default());
    let files = out.files.as_ref()?.as_ref().ok()?;
    let folded = out.borrowchecked.lowered.folded.as_ref();
    let libs = folded.map(|m| link_libs(&m.foreign)).unwrap_or_default();
    let dir = tempfile::tempdir().unwrap();
    let sources = write_cache(dir.path(), files).unwrap();
    let exe = dir.path().join(format!("main{}", std::env::consts::EXE_SUFFIX));
    #[rustfmt::skip]
    let plan = BuildPlan::builder().cc(cc).sources(&sources).out(&exe)
        .checks(Checks::Default).sanitizers(false).libs(&libs).build();
    let built = toolchain::build(&plan).unwrap();
    assert!(built.failed.is_none(), "the C build failed for\n{}", sample.source);
    let mut cmd = std::process::Command::new(&exe);
    let cmd = cmd.current_dir(dir.path()).stdin(Stdio::piped());
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let input = sample.input.clone();
    let mut stdin = child.stdin.take().unwrap();
    std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let ran = child.wait_with_output().unwrap();
    let printed = String::from_utf8_lossy(&ran.stdout).into_owned();
    let err = String::from_utf8_lossy(&ran.stderr).into_owned();
    // The executable starts its message with its own name.
    // The stop line starts with the name of the executable and ends with the
    // place of the stop, ` (in ...)`, which the pages leave out.
    let marker = "the program stopped: ";
    let line = err.lines().find(|l| l.contains(marker)).unwrap_or_default();
    let from = &line[line.find(marker).unwrap_or(0)..];
    let reason = from.rfind(" (in ").map_or(from, |at| &from[..at]);
    Some(format!("{printed}{reason}\n"))
}

#[test]
fn each_stop_of_the_books_stops_the_same_way_in_c()
{
    let Ok(cc) = toolchain::find_cc(None)
    else
    {
        let required = std::env::var("ZAKA_REQUIRE_CC").is_ok_and(|v| v == "1");
        assert!(!required, "ZAKA_REQUIRE_CC=1, but no C compiler is on the PATH");
        eprintln!("no C compiler on the PATH: the native stops are not compared");
        return;
    };
    let all = stops();
    assert!(all.len() > 20, "the books hold their stops: {}", all.len());
    let differ = |(path, s): &(&str, Sample)| {
        let native = native_output(&cc, s)?;
        let same = without_addresses(&native) == without_addresses(&s.expected);
        let shown =
            format!("{path}: prints\n{native}not\n{}---\n{}", s.expected, s.source);
        (!same).then_some(shown)
    };
    let wrong: Vec<String> = all.iter().filter_map(differ).collect();
    assert!(
        wrong.is_empty(),
        "{} of {} stops differ:\n\n{}",
        wrong.len(),
        all.len(),
        wrong.join("\n")
    );
}
