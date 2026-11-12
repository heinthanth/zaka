use bon::Builder;
use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_diagnostics::has_errors;
use zakac_ir_builder::ir::Module;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::digest::FnDigest;
use crate::flow::run_flow;
use crate::flow::run_start_flow;
use crate::interval::CellProofs;
use crate::interval::mark_checked;
use crate::interval::uses_raw;
use crate::last_use::lend_arguments;
use crate::last_use::move_last_uses;
use crate::last_use::share_params;
use crate::options::BorrowCheckOptions;
use crate::options::MemCheck;
use crate::roots::Roots;
use crate::routine::Routine;

pub mod artifact;
mod callgraph;
mod cfg;
mod checks;
pub mod cli;
mod closures;
pub mod digest;
mod flow;
mod init;
mod interval;
mod last_use;
mod liveness;
mod narrow;
pub mod options;
mod roots;
mod routine;
mod state;
#[cfg(test)]
mod testing;
mod values;
mod walk;

pub use walk::each_stmt;

zakac_i18n::init!();

/// What the check found: the diagnostics and one digest per function.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct BorrowCheckOutput
{
    /// The diagnostics, in source order.
    pub diagnostics: Vec<Diagnostic>,
    /// The digest of each function, in the order of [`Module::funcs`].
    pub digests: Vec<FnDigest>,
}

/// Check the ownership and the loans of a folded module: the roots
/// top-level code gives the globals, then each function, callees first,
/// then the main program.
///
/// # Arguments
///
/// * `module` - The folded module.
/// * `options` - How to report.
/// * `platform_bits` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The diagnostics and the digests.
pub fn borrow_check(
    module: &mut Module, options: &BorrowCheckOptions, platform_bits: u16,
) -> BorrowCheckOutput
{
    // Moves at the last use come first, so the check follows the code that
    // runs.
    move_last_uses(module);
    lend_arguments(module);
    share_params(module);
    let read: &Module = module;
    let (global_roots, digests, diagnostics, mut proofs) =
        roots_and_digests(read, options, platform_bits);
    let main = Routine::main(read);
    let found = run_flow(read, main, &digests, &global_roots, options, platform_bits);
    proofs.add(&found.proofs, &found.doubts);
    let start = callgraph::start_block(read).and_then(|f| {
        run_start_flow(read, f, &digests, &global_roots, options, platform_bits)
    });
    let start_diags = start.map_or_else(Vec::new, |s| {
        proofs.add(&s.proofs, &s.doubts);
        s.diags
    });
    let all = [diagnostics, found.diags, start_diags].concat();
    // A value read before it is set is an error at each level of the check:
    // the kinds promise there is no hidden default.
    let unset = init::check_init(read);
    // A finding, or a raw address the flow cannot follow, means the facts
    // the proofs rest on need not hold.
    let trusted = !has_errors(&all) && !uses_raw(read);
    do_if!(trusted, mark_checked(module, &proofs.proven()));
    let mut diagnostics = all;
    // A pointer read before it is set holds the zero octets its declaration
    // gave it, so the null check sees it too; the report of the unset read
    // stands for both.
    let unset_at: Vec<Span> = unset.iter().map(|d| d.span).collect();
    let null =
        |d: &Diagnostic| matches!(d.code, Code::NullDeref | Code::NullDerefPossible);
    diagnostics.retain(|d| !(null(d) && unset_at.iter().any(|s| s.overlaps(&d.span))));
    diagnostics.extend(unset);
    let mut diagnostics = downgrade(diagnostics, options, &module.unsafe_spans);
    diagnostics.sort_by_key(|d| d.span.lo);
    BorrowCheckOutput::builder().diagnostics(diagnostics).digests(digests).build()
}

/// The roots of the globals and the digests of the functions, each of
/// which reads the other: a top-level set of a global borrow from a call
/// takes its roots from the digest, and a function reading that global
/// starts from them. The roots start from blank digests and grow, never
/// shrinking, until the digests of a round add none.
///
/// # Arguments
///
/// * `module` - The module.
/// * `options` - How to report.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The roots of each global, the digest of each function, the
/// diagnostics of the functions and the cell addresses their flows met.
fn roots_and_digests(
    module: &Module, options: &BorrowCheckOptions, platform: u16,
) -> (Vec<Roots>, Vec<FnDigest>, Vec<Diagnostic>, CellProofs)
{
    let blank: Vec<FnDigest> = module.funcs.iter().map(FnDigest::blank).collect();
    let mut global_roots = roots::global_roots(module, &blank);
    loop
    {
        let (digests, diags, proofs) =
            callgraph::digests(module, &global_roots, options, platform);
        let more = roots::global_roots(module, &digests);
        let next: Vec<Roots> = global_roots
            .iter()
            .zip(more)
            .map(|(old, new)| old.union(&new).cloned().collect())
            .collect();
        ret_if!(next == global_roots, (global_roots, digests, diags, proofs));
        global_roots = next;
    }
}

/// The diagnostics at the levels the options want.
///
/// # Arguments
///
/// * `diags` - What the check found.
/// * `options` - `--mem-check` and the named levels.
/// * `unsafe_spans` - The source regions of unsafe code.
///
/// # Returns
///
/// Each finding at its level: an error for [`MemCheck::Error`], a warning
/// for [`MemCheck::Warn`], none for [`MemCheck::Off`]. The pointer lint holds
/// each [`LINT`] finding, and each [`UNSAFE_LINT`] finding inside unsafe
/// code; `--mem-check` sets its level, and a named level wins. A
/// [`UNSAFE_LINT`] finding outside unsafe code remains an error, and a finding
/// that no name and no lint covers remains as it is.
fn downgrade(
    diags: Vec<Diagnostic>, options: &BorrowCheckOptions, unsafe_spans: &[Span],
) -> Vec<Diagnostic>
{
    let inside = |d: &Diagnostic| {
        unsafe_spans.iter().any(|m| m.lo <= d.span.lo && d.span.hi <= m.hi)
    };
    let lint = |d: &Diagnostic| {
        LINT.contains(&d.code) || (UNSAFE_LINT.contains(&d.code) && inside(d))
    };
    let named = |d: &Diagnostic| {
        let open = lint(d) || !UNSAFE_LINT.contains(&d.code);
        options.lint.get(d.code).filter(|_| open)
    };
    let level = |d: Diagnostic| {
        let set = named(&d);
        match set.or_else(|| lint(&d).then_some(options.check))
        {
            Some(MemCheck::Off) => None,
            Some(MemCheck::Warn) => Some(d.with_level(Level::Warning)),
            Some(MemCheck::Error) if set.is_some() => Some(d.with_level(Level::Error)),
            _ => Some(d),
        }
    };
    diags.into_iter().filter_map(level).collect()
}

/// The findings that are lint inside unsafe code and errors outside it,
/// where an object or a list gives them: a read of null, a step past the
/// end of a block, and a read that can meet null.
const UNSAFE_LINT: &[Code] =
    &[Code::NullDeref, Code::OffsetPastEnd, Code::NullDerefPossible];

/// The findings that come from pointer work, which unsafe code
/// alone does: a block given back twice or through the wrong pointer, a
/// block kept or freed while a pointer still reads it, a local's address
/// that escapes, an argument another one frees, a leaked block, pointers
/// into different blocks subtracted, and overlapping block copies.
const LINT: &[Code] = &[
    Code::BadFree,
    Code::BorrowStored,
    Code::InvalidatedLoan,
    Code::LocalBorrowReturned,
    Code::AliasedArgument,
    Code::BlockLeaked,
    Code::MixedBlocks,
    Code::BlitRangesOverlap,
];
