use bon::Builder;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_diagnostics::has_errors;
use zakac_ir_builder::ir::Module;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::digest::FnDigest;
use crate::flow::run_flow;
use crate::interval::CellProofs;
use crate::interval::mark_checked;
use crate::interval::uses_raw;
use crate::options::BorrowCheckOptions;
use crate::options::MemCheck;
use crate::roots::Roots;
use crate::routine::Routine;

pub mod artifact;
mod callgraph;
mod cfg;
mod checks;
pub mod cli;
pub mod digest;
mod flow;
mod interval;
mod liveness;
mod narrow;
pub mod options;
mod roots;
mod routine;
mod state;
#[cfg(test)]
mod testing;
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
    let read: &Module = module;
    let (global_roots, digests, diagnostics, mut proofs) =
        roots_and_digests(read, options, platform_bits);
    let main = Routine::main(read);
    let found = run_flow(read, main, &digests, &global_roots, options, platform_bits);
    proofs.add(&found.proofs, &found.doubts);
    let all = [diagnostics, found.diags].concat();
    // A finding, or a raw address the flow cannot follow, means the facts
    // the proofs rest on need not hold.
    let trusted = !has_errors(&all) && !uses_raw(read);
    do_if!(trusted, mark_checked(module, &proofs.proven()));
    let mut diagnostics = downgrade(all, options.check);
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

/// The diagnostics as the mode wants them.
///
/// # Arguments
///
/// * `diags` - What the check found.
/// * `mode` - How to report.
///
/// # Returns
///
/// Each diagnostic for [`MemCheck::Error`], each as a warning for
/// [`MemCheck::Warn`], none for [`MemCheck::Off`].
fn downgrade(diags: Vec<Diagnostic>, mode: MemCheck) -> Vec<Diagnostic>
{
    match mode
    {
        MemCheck::Error => diags,
        MemCheck::Warn =>
        {
            diags.into_iter().map(|d| d.with_level(Level::Warning)).collect()
        }
        MemCheck::Off => Vec::new(),
    }
}
