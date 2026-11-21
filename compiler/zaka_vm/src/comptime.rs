//! The runner of comptime calls: it builds the small program the check
//! hands it to ops and runs them on the VM with the rights of the
//! `--comptime-allow-*` flags, where a denial stops the run and nothing is
//! asked. It reads the global of each call back as a value, keeps what each
//! call printed as a note, and keeps the results of each small program it
//! ran, so a check that meets the same calls again runs nothing.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

use zaka_host::Gate;
use zaka_host::Grant;
use zaka_host::OnDenial;
use zaka_host::TermWorld;
use zakac_ast::expr::ConstTree;
use zakac_ast::s_expr::ToSexpr;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_codegen_target_vm::artifact::run_loaded;
use zakac_codegen_target_vm::options::EmitOptions;
use zakac_diagnostics::fold as fd;
use zakac_i18n::t;
use zakac_ir_builder::ir::FoldOptions;
use zakac_parser::load::Loaded;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_semcheck::comptime_calls::ComptimeRunner;
use zakac_semcheck::comptime_calls::Ran;
use zakac_semcheck::comptime_calls::Site;
use zakac_semcheck::comptime_calls::SmallProgram;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::span::Span;

use crate::machine::Machine;
use crate::machine::RunOptions;
use crate::print::read_tree;
use crate::trap::Trap;
use crate::trap::TrapKind;
use crate::trap::denied_what;

/// The most ops a comptime run of `zakac` takes.
pub const MAX_STEPS: u64 = 50_000_000;

/// The most ops a comptime run takes in an editor or the explorer, which
/// check on each edit.
pub const EDITOR_STEPS: u64 = 1_000_000;

/// The most live heap octets of a comptime run.
const MAX_HEAP: u64 = 64 << 20;

/// What one call gave, with no spans, so a later check reuses it.
#[derive(Debug, Clone)]
enum Outcome
{
    /// The value.
    Value(ConstTree),
    /// Why the run stopped, and the calls that led there.
    Stopped(String, Vec<String>),
}

/// What one call gave, and what it printed.
type Kept = (Outcome, Option<String>);

/// The runner of comptime calls on the VM.
#[derive(Debug)]
pub struct VmRunner
{
    /// The most ops a run takes.
    max_steps: u64,
    /// Whether the runner serves an editor or the explorer: a run reads the
    /// files under the folder of the program at most, whatever is granted.
    editor: bool,
    /// What each small program gave, on its text.
    kept: Mutex<HashMap<String, Vec<Kept>>>,
}

/// Install the VM as the runner of comptime calls of `zakac`.
pub fn install() { install_with(MAX_STEPS, false); }

/// Install the VM as the runner of comptime calls, for each check in the
/// process; a later call keeps the first.
///
/// # Arguments
///
/// * `max_steps` - The most ops a run takes.
/// * `editor` - Whether the runner serves an editor or the explorer.
pub fn install_with(max_steps: u64, editor: bool)
{
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let kept = Mutex::new(HashMap::new());
        let runner = VmRunner { max_steps, editor, kept };
        zakac_semcheck::comptime_calls::install(Box::new(runner));
    });
}

impl ComptimeRunner for VmRunner
{
    fn run(&self, small: &SmallProgram, options: &SemcheckOptions) -> Vec<Ran>
    {
        let gate = self.gate(small);
        let key = key_of(small, options, &gate);
        let kept = self.kept.lock().ok().and_then(|k| k.get(&key).cloned());
        let kept = kept.unwrap_or_else(|| {
            let ran = self.run_fresh(small, options, gate);
            do_if!(let Ok(mut k) = self.kept.lock(), _ = k.insert(key, ran.clone()));
            ran
        });
        small.sites.iter().zip(kept).map(|(site, k)| ran_of(site, k)).collect()
    }
}

impl VmRunner
{
    /// The rights of a run: the comptime grants of the command, with paths
    /// under the folder of the program, a denial that stops the run, and no
    /// question asked; an editor reads that folder alone.
    ///
    /// # Arguments
    ///
    /// * `small` - The program.
    ///
    /// # Returns
    ///
    /// The gate.
    fn gate(&self, small: &SmallProgram) -> Gate
    {
        let mut grants = zakac_semcheck::comptime_calls::grants();
        grants.prompt = false;
        grants.on_denial = OnDenial::Stop;
        let folder = small.folder.clone().unwrap_or_default();
        if self.editor
        {
            grants.read = Grant::Some(vec![folder.to_string_lossy().into_owned()]);
            grants.write = Grant::None;
            grants.ffi = false;
        }
        let mut gate = Gate::new(grants);
        gate.root = small.folder.clone();
        gate
    }

    /// Build a small program, run it, and read what each call gave.
    ///
    /// # Arguments
    ///
    /// * `small` - The program.
    /// * `options` - The check options of the build.
    /// * `gate` - The rights of the run.
    ///
    /// # Returns
    ///
    /// What each call gave, in order.
    fn run_fresh(
        &self, small: &SmallProgram, options: &SemcheckOptions, gate: Gate,
    ) -> Vec<Kept>
    {
        let options = SemcheckOptions { as_module: true, ..*options };
        let loaded = Loaded::builder().program(small.program.clone()).build();
        #[rustfmt::skip]
        let emitted = run_loaded(loaded, &options, &FoldOptions::default(),
            &BorrowCheckOptions::default(), EmitOptions::default());
        let Some(mut module) = emitted.module.clone()
        else
        {
            let first = emitted.diagnostics().iter().find(|d| d.is_error());
            let message = first.map(|d| d.message.clone()).unwrap_or_default();
            let stopped =
                Outcome::Stopped(fd::comptime_compile_reason(&message), Vec::new());
            return small.sites.iter().map(|_| (stopped.clone(), None)).collect();
        };
        module.built_for = None;
        let globals: Vec<_> = small
            .globals
            .iter()
            .map(|n| module.globals.iter().position(|g| g.name == n.as_str()))
            .collect();
        let kinds: Vec<_> = module.globals.iter().map(|g| g.kind).collect();
        let run = RunOptions::builder()
            .max_steps(self.max_steps)
            .max_heap(MAX_HEAP)
            .fixed_clock(true)
            .build();
        let mut machine = Machine::new(module, run);
        machine.set_world(Box::new(TermWorld));
        machine.set_gate(gate);
        let printed = drive(&mut machine, small);
        let trap = machine.state().trap().cloned();
        let read = |at: Option<usize>| {
            let at = at.ok_or(TrapKind::Malformed { reason: "no global".into() })?;
            read_tree(machine.mem(), machine.globals()[at], kinds[at])
        };
        let value = |at: &Option<usize>| {
            read(*at).map_or_else(
                |k| Outcome::Stopped(k.to_string(), Vec::new()),
                Outcome::Value,
            )
        };
        let Some(trap) = trap
        else
        {
            return globals.iter().map(value).zip(printed).collect();
        };
        // The calls before the one that trapped set their globals.
        let (mut kept, owner) = trapped(small, &trap, printed);
        let done = owner.unwrap_or(0);
        do_for!((k, at) in kept.iter_mut().zip(&globals).take(done), k.0 = value(at));
        kept
    }
}

/// Run a machine to its end, and give each site what it printed: the
/// output that grew while main ran the statement of its global.
///
/// # Arguments
///
/// * `machine` - The machine.
/// * `small` - The program.
///
/// # Returns
///
/// What each site printed, in order.
fn drive(machine: &mut Machine, small: &SmallProgram) -> Vec<Option<String>>
{
    let mut printed: Vec<Option<String>> = vec![None; small.sites.len()];
    while machine.running()
    {
        let before = machine.output().len();
        let at = main_span(machine);
        machine.step();
        let grew = machine
            .output()
            .get(before..)
            .filter(|t| !t.is_empty())
            .map(str::to_string);
        let Some(text) = grew
        else
        {
            continue;
        };
        let owner =
            at.and_then(|at| small.sites.iter().position(|s| s.span.contains(&at)));
        do_if!(let Some(i) = owner, printed[i].get_or_insert_with(String::new).push_str(&text));
    }
    printed
}

/// The source region of the statement main runs.
///
/// # Arguments
///
/// * `machine` - The machine.
///
/// # Returns
///
/// The region, or `None` before main starts or in a release build.
fn main_span(machine: &Machine) -> Option<Span>
{
    let frame = machine.frames().first()?;
    let func = machine.module().funcs.get(frame.func as usize)?;
    func.spans.get(frame.pc.saturating_sub(1) as usize).copied()
}

/// What each site of a run that trapped gave: the site whose global was
/// being set gets the trap, and each other site with it, as the run stopped
/// before their values.
///
/// # Arguments
///
/// * `small` - The program.
/// * `trap` - The trap.
/// * `printed` - What each site printed.
///
/// # Returns
///
/// What each site gave, and the site that hit the trap, when known.
fn trapped(
    small: &SmallProgram, trap: &Trap, printed: Vec<Option<String>>,
) -> (Vec<Kept>, Option<usize>)
{
    let reason = match &trap.kind
    {
        TrapKind::PermissionDenied { right, subject } =>
        {
            let what = denied_what(*right, subject);
            let flag = format!("--comptime-allow-{}", <&str>::from(*right));
            t!("vm.trap.permission_denied", what = what, flag = flag).to_string()
        }
        kind => kind.to_string(),
    };
    let mut chain: Vec<String> =
        trap.frames.iter().filter(|f| !f.added).map(|f| f.func.clone()).collect();
    chain.reverse();
    let at = trap.frames.iter().rev().find_map(|f| f.span);
    let owner = at.and_then(|at| small.sites.iter().position(|s| s.span.contains(&at)));
    let chain_of = |i: usize| tern_chain(owner == Some(i) || owner.is_none(), &chain);
    let each = printed.into_iter().enumerate();
    let kept = each.map(|(i, p)| (Outcome::Stopped(reason.clone(), chain_of(i)), p));
    (kept.collect(), owner)
}

/// The chain of calls a site reports: the calls that led to the trap for
/// the site that hit it, none for the others.
///
/// # Arguments
///
/// * `hit` - Whether the site hit the trap.
/// * `chain` - The calls.
///
/// # Returns
///
/// The chain to report.
fn tern_chain(hit: bool, chain: &[String]) -> Vec<String>
{
    if hit
    {
        return chain.to_vec();
    }
    Vec::new()
}

/// What a site gave, with the diagnostics placed at the site.
///
/// # Arguments
///
/// * `site` - The site.
/// * `kept` - What it gave.
///
/// # Returns
///
/// The value or the `ZK0611` diagnostic, and the `ZK0612` note.
fn ran_of(site: &Site, kept: Kept) -> Ran
{
    let name = site.callee.name.as_str();
    let (outcome, printed) = kept;
    let value = match outcome
    {
        Outcome::Value(tree) => Ok(tree),
        Outcome::Stopped(reason, chain) =>
        {
            Err(fd::comptime_run_stopped(site.span, name, &reason, &chain))
        }
    };
    let note = printed.map(|text| fd::comptime_printed(site.span, name, &text));
    Ran { value, note }
}

/// The key of a small program in the store of results: its statements and
/// calls with no spans, the platform, and the rights.
///
/// # Arguments
///
/// * `small` - The program.
/// * `options` - The check options.
/// * `gate` - The rights of the run.
///
/// # Returns
///
/// The key.
fn key_of(small: &SmallProgram, options: &SemcheckOptions, gate: &Gate) -> String
{
    let stmts: Vec<String> =
        small.program.stmts.iter().map(ToSexpr::to_sexpr).collect();
    let args: Vec<String> =
        small.sites.iter().map(|s| format!("{:?}", s.args)).collect();
    let bits = options.platform_bits;
    let units: Vec<&str> =
        small.program.units.iter().map(|u| u.name.as_str()).collect();
    format!("{stmts:?}{args:?}{bits}{units:?}{:?}{:?}", gate.grants, gate.root)
}
