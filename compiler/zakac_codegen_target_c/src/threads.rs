//! Threads and channels in C: a spawn fills the data block of the thread with a
//! duplicate of each argument and hands it to the runtime, which runs the
//! step of the thread on an OS thread of its own under the guard of this file.
//! The guard keeps the place a trap of the thread jumps back to.

use std::fmt::Write;

use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Par;
use zakac_ir_builder::ir::ParMap;
use zakac_ir_builder::ir::ParRounds;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::emit::FnGen;
use crate::emit::KINDS_HEADER;
use crate::emit::Shared;
use crate::emit::func_name;
use crate::kinds::is_blob;

/// The file of the guard and of the step of each thread.
pub const THREADS_SOURCE: &str = "zaka_threads.c";

/// The octets each part of a data block starts on.
const PART_ALIGN: u32 = 16;

/// The guard of a thread and the function that jumps back to it.
pub const GUARD: &str = "\
/* A longjmp of 64-bit Windows unwinds each frame it leaves; clearing the frame
   of the buffer makes it restore the registers alone, as on each other OS. */
#if defined(_WIN64) && defined(ZK_LIB_PLAIN_JUMP)
#define ZK_THREAD_NO_UNWIND(buf) (((_JUMP_BUFFER *)(buf))->Frame = 0)
#else
#define ZK_THREAD_NO_UNWIND(buf) ((void)(buf))
#endif

/**
 * Jump back to the guard of the thread of this OS thread, on a trap that ends the
 * thread alone.
 *
 * @param trap The id of the trap, which the runtime keeps.
 */
_Noreturn void zk_thread_leave(int32_t trap)
{
    (void)trap;
    longjmp(*(jmp_buf *)zk_lib_jump(), 1);
}

/**
 * Run the step of a thread, keeping the place a trap of the thread jumps back to.
 *
 * @param step The step.
 * @param data The data block of the thread.
 * @return 0, or 1 once a trap ended the thread.
 */
int32_t zk_thread_guard(void (*step)(void *), void *data)
{
    jmp_buf zk_here;
    if (setjmp(zk_here) != 0) return 1;
    ZK_THREAD_NO_UNWIND(zk_here);
    zk_lib_jump_set(&zk_here);
    step(data);
    zk_flush();
    return 0;
}
";

/// The offset of each part of the data block of a thread that calls a function:
/// the result first, then each argument.
///
/// # Arguments
///
/// * `kinds` - The kind of the result, or none, then of each parameter.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The offset of each part, and the size of the block.
pub fn part_offsets(kinds: &[Option<DataKind>], platform: u16) -> (Vec<u32>, u32)
{
    let mut at = 0;
    let offsets = kinds
        .iter()
        .map(|k| {
            let here = at;
            at = (at + k.map_or(0, |k| k.size(platform))).next_multiple_of(PART_ALIGN);
            here
        })
        .collect();
    (offsets, at.max(PART_ALIGN))
}

/// The result kind and the kind of each parameter of a function.
///
/// # Arguments
///
/// * `module` - The program.
/// * `f` - The function.
///
/// # Returns
///
/// The kind of the result, or none, then of each parameter.
pub fn parts(module: &Module, f: u32) -> Vec<Option<DataKind>>
{
    let func = &module.funcs[f as usize];
    let params = func.locals.iter().take(func.params as usize).map(|s| Some(s.kind));
    std::iter::once(func.ret).chain(params).collect()
}

/// The C file of the guard and of the step of each function a thread runs, or
/// `None` for a program that starts no thread.
///
/// # Arguments
///
/// * `shared` - The state shared across functions.
///
/// # Returns
///
/// The C text.
pub(crate) fn threads_source(shared: &mut Shared<'_>) -> Option<String>
{
    ret_if!(no_threads(shared), None);
    let mut out =
        format!("#include \"{KINDS_HEADER}\"\n#include <setjmp.h>\n\n{GUARD}");
    let spawned: Vec<u32> = shared.spawned.iter().copied().collect();
    spawned.into_iter().for_each(|f| out.push_str(&step(shared, f)));
    let mapped: Vec<u32> = shared.mapped.iter().copied().collect();
    mapped.into_iter().for_each(|f| out.push_str(&map_step(shared, f)));
    let rounds: Vec<u32> = shared.rounds.iter().copied().collect();
    rounds.into_iter().for_each(|f| out.push_str(&round_step(shared, f)));
    Some(out)
}

/// Whether a program starts no thread, maps nothing at once and runs no
/// parallel loop.
///
/// # Arguments
///
/// * `shared` - The state shared across functions.
///
/// # Returns
///
/// `true` when it needs no guard.
fn no_threads(shared: &Shared<'_>) -> bool
{
    shared.spawned.is_empty() && shared.mapped.is_empty() && shared.rounds.is_empty()
}

/// The offset of each argument of a round in the data block of its loop,
/// after the number of the round, which the block leaves out.
///
/// # Arguments
///
/// * `module` - The program.
/// * `f` - The function of a round.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The kind of each argument with its offset, and the size of the block.
pub fn round_parts(
    module: &Module, f: u32, platform: u16,
) -> (Vec<(DataKind, u32)>, u32)
{
    let kinds = parts(module, f);
    let rest: Vec<Option<DataKind>> =
        std::iter::once(None).chain(kinds.iter().skip(2).copied()).collect();
    let (offsets, size) = part_offsets(&rest, platform);
    let each = rest.iter().zip(offsets).skip(1);
    let each = each.map(|(k, at)| (k.expect("a parameter has a kind"), at));
    (each.collect(), size)
}

/// The step of a round of a parallel loop: it calls the function of the round
/// with the number of the round and the arguments of the data block, which
/// are words the loop shares.
///
/// # Arguments
///
/// * `shared` - The state shared across functions.
/// * `f` - The function of a round.
///
/// # Returns
///
/// The C text.
fn round_step(shared: &mut Shared<'_>, f: u32) -> String
{
    let (each, _) = round_parts(shared.module, f, shared.options.platform_bits);
    let loads = each
        .iter()
        .map(|(k, at)| format!("zk_ld_{}(zk_b + {at}, ZK_NO_SITE)", shared.mem(*k)));
    let args: Vec<String> = std::iter::once("zk_i".to_string()).chain(loads).collect();
    let name = shared.module.funcs[f as usize].name.replace("*/", "* /");
    format!(
        "\n/** The step of a round that runs `{name}`. */\nvoid \
         zk_round_step_{f}(uint8_t *zk_b, size_t zk_i)\n{{\n    {}({});\n}}\n",
        func_name(f),
        args.join(", ")
    )
}

/// The step of a parallel map through a function: it lends the cell to the
/// function, as a call does, and writes the result.
///
/// # Arguments
///
/// * `shared` - The state shared across functions.
/// * `f` - The function.
///
/// # Returns
///
/// The C text.
fn map_step(shared: &mut Shared<'_>, f: u32) -> String
{
    let kinds = parts(shared.module, f);
    let (ret, param) = (kinds[0], kinds[1].expect("a map takes one parameter"));
    let size = shared.size(param);
    let put = format!(
        "    _Alignas(16) uint8_t zk_l[{size}];\n    zk_put_block(zk_l, zk_c, \
         {size});\n"
    );
    let lend = match (is_blob(param), param.holds_text())
    {
        (true, true) =>
        {
            format!("{put}    zk_lend_seqs(zk_l, {});\n", shared.descr.of(param))
        }
        (true, false) => put,
        _ => String::new(),
    };
    let arg = tern!(
        is_blob(param),
        "zk_l".to_string(),
        format!("zk_ld_{}(zk_c, ZK_NO_SITE)", shared.mem(param))
    );
    let call = match ret
    {
        Some(k) if is_blob(k) => format!("    {}(zk_o, {arg});\n", func_name(f)),
        Some(k) =>
        {
            format!(
                "    zk_st_{}(zk_o, {}({arg}), ZK_NO_SITE);\n",
                shared.mem(k),
                func_name(f)
            )
        }
        None => format!("    {}({arg});\n", func_name(f)),
    };
    let name = shared.module.funcs[f as usize].name.replace("*/", "* /");
    format!(
        "\n/** The step of a parallel map through `{name}`. */\nvoid \
         zk_map_step_{f}(uint8_t *zk_c, uint8_t *zk_o)\n{{\n{lend}{call}}}\n"
    )
}

/// The step of a thread that calls a function: it calls the function with the
/// arguments of the data block, which the function drops as it returns, and
/// writes the result at its start.
///
/// # Arguments
///
/// * `shared` - The state shared across functions.
/// * `f` - The function.
///
/// # Returns
///
/// The C text.
fn step(shared: &mut Shared<'_>, f: u32) -> String
{
    let platform = shared.options.platform_bits;
    let kinds = parts(shared.module, f);
    let (offsets, _) = part_offsets(&kinds, platform);
    let ret = kinds[0];
    let mut args: Vec<String> = Vec::new();
    for (k, at) in kinds.iter().zip(&offsets).skip(1)
    {
        let k = k.expect("a parameter has a kind");
        let place = format!("(zk_b + {at})");
        let arg = match is_blob(k)
        {
            true => place.clone(),
            false => format!("zk_ld_{}({place}, ZK_NO_SITE)", shared.mem(k)),
        };
        args.push(arg);
    }
    let call = |into: Option<&str>| {
        let all = into.map(str::to_string).into_iter().chain(args.iter().cloned());
        format!("{}({})", func_name(f), all.collect::<Vec<_>>().join(", "))
    };
    let call_line = match ret
    {
        Some(k) if is_blob(k) => format!("    {};\n", call(Some("zk_b"))),
        Some(k) =>
        {
            format!("    zk_st_{}(zk_b, {}, ZK_NO_SITE);\n", shared.mem(k), call(None))
        }
        None => format!("    {};\n", call(None)),
    };
    format!(
        "\n/** The step of a thread that runs `{}`. */\nvoid zk_thread_step_{f}(void \
         *zk_d)\n{{\n    uint8_t *zk_b = (uint8_t *)zk_d;\n{call_line}}}\n",
        shared.module.funcs[f as usize].name.replace("*/", "* /")
    )
}

/// The C file of the guard of a native build, whose steps Cranelift writes.
///
/// # Returns
///
/// The C text.
pub fn native_threads_glue() -> String
{
    format!(
        "#include <stdint.h>\n#include <setjmp.h>\n\n#include \
         \"zaka_runtime.h\"\n\n{GUARD}"
    )
}

/// The declarations of the guard and of each step, for the header.
///
/// # Arguments
///
/// * `shared` - The state shared across functions.
///
/// # Returns
///
/// The C text, blank for a program that starts no thread.
pub(crate) fn declarations(shared: &Shared<'_>) -> String
{
    ret_if!(no_threads(shared), String::new());
    let mut out = String::from(
        "int32_t zk_thread_guard(void (*step)(void *), void *data);\n_Noreturn void \
         zk_thread_leave(int32_t trap);\n",
    );
    shared
        .spawned
        .iter()
        .for_each(|f| _ = writeln!(out, "void zk_thread_step_{f}(void *zk_d);"));
    let steps = shared.mapped.iter();
    steps.for_each(|f| {
        _ = writeln!(out, "void zk_map_step_{f}(uint8_t *zk_c, uint8_t *zk_o);")
    });
    let rounds = shared.rounds.iter();
    rounds.for_each(|f| {
        _ = writeln!(out, "void zk_round_step_{f}(uint8_t *zk_b, size_t zk_i);")
    });
    out
}

impl FnGen<'_, '_>
{
    /// Write what a native build runs at once in place of a loop: a parallel
    /// map or the rounds of a parallel loop.
    ///
    /// # Arguments
    ///
    /// * `par` - The map or the rounds.
    pub(crate) fn par(&mut self, par: &Par)
    {
        match par
        {
            Par::Map(p) => self.par_map(p),
            Par::Rounds(p) => self.par_rounds(p),
        }
    }

    /// Write a parallel map that runs its rounds at once, in place of its
    /// loop.
    ///
    /// # Arguments
    ///
    /// * `p` - The map.
    fn par_map(&mut self, p: &ParMap)
    {
        self.shared.mapped.insert(p.func);
        let (src, out) = (self.place(p.src), self.place(p.out));
        let (sk, ok) =
            (self.shared.descr.of(p.src_kind), self.shared.descr.of(p.out_kind));
        let abort = i32::from(self.shared.options.trap_abort);
        let threads = p.threads.map_or_else(|| "0".to_string(), |t| self.place(t));
        let site = self.check_site();
        self.line(&format!(
            "zk_par_map({src}, {sk}, {out}, {ok}, zk_map_step_{}, zk_thread_guard, \
             zk_thread_leave, {abort}, {threads}, {site});",
            p.func
        ));
    }

    /// Write the rounds of a parallel loop that run at once, in place of its
    /// loop: a data block of the arguments the rounds share, handed to the
    /// runtime with the step of the round.
    ///
    /// # Arguments
    ///
    /// * `p` - The rounds.
    fn par_rounds(&mut self, p: &ParRounds)
    {
        self.shared.rounds.insert(p.func);
        let platform = self.shared.options.platform_bits;
        let (each, size) = round_parts(self.shared.module, p.func, platform);
        let values = self.operands(&p.args.iter().collect::<Vec<_>>());
        let data = self.spill(DataKind::OPAQUE, format!("zk_thread_data({size})"));
        for (v, (k, at)) in values.iter().zip(&each)
        {
            self.put_at(&format!("({data} + {at})"), v, *k);
        }
        let count = self.place(p.count);
        let abort = i32::from(self.shared.options.trap_abort);
        let threads = p.threads.map_or_else(|| "0".to_string(), |t| self.place(t));
        self.line(&format!(
            "zk_par_for({count}, zk_round_step_{}, {data}, zk_thread_guard, \
             zk_thread_leave, {abort}, {threads});",
            p.func
        ));
    }

    /// The C text of a spawn: a data block that each argument moves into,
    /// handed to the runtime with the step of the function.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    /// * `args` - The arguments.
    /// * `apart` - Whether the call asks for an OS thread of its own.
    ///
    /// # Returns
    ///
    /// The C text of the number of the thread.
    pub(crate) fn spawn(&mut self, f: u32, args: &[Expr], apart: bool) -> String
    {
        self.shared.spawned.insert(f);
        let platform = self.shared.options.platform_bits;
        let kinds = parts(self.shared.module, f);
        let (offsets, size) = part_offsets(&kinds, platform);
        let values = self.operands(&args.iter().collect::<Vec<_>>());
        let data = self.spill(DataKind::OPAQUE, format!("zk_thread_data({size})"));
        // The arguments come owned, as the value of a set does: each moves
        // into the block.
        for ((v, arg), at) in values.iter().zip(args).zip(offsets.iter().skip(1))
        {
            self.put_at(&format!("({data} + {at})"), v, arg.kind);
        }
        let ret =
            kinds[0].map_or_else(|| "NULL".to_string(), |k| self.shared.descr.of(k));
        let abort = i32::from(self.shared.options.trap_abort);
        let start = self.start_fn(f, apart);
        format!(
            "{start}(zk_thread_guard, zk_thread_leave, zk_thread_step_{f}, {data}, \
             {ret}, {abort})"
        )
    }

    /// The runtime function that starts a thread: `zk_task_start` for a light
    /// task, when the function of the thread can run as one and the call does
    /// not ask for an OS thread of its own; `zk_thread_start` otherwise.
    ///
    /// # Arguments
    ///
    /// * `f` - The function of the thread.
    /// * `apart` - Whether the call asks for an OS thread of its own.
    ///
    /// # Returns
    ///
    /// The name.
    fn start_fn(&self, f: u32, apart: bool) -> &'static str
    {
        let light = !apart && self.shared.light.contains(&f);
        tern!(light, "zk_task_start", "zk_thread_start")
    }

    /// The C text of a wait that takes the result of a thread: the result moves
    /// out of the data block of the thread, which the runtime then frees.
    ///
    /// # Arguments
    ///
    /// * `thread` - The thread.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// The C text.
    pub(crate) fn wait(&mut self, thread: &Expr, kind: DataKind) -> String
    {
        let j = self.expr(thread);
        let j = self.spill(thread.kind, j);
        let site = self.check_site();
        let at = self.spill(DataKind::OPAQUE, format!("zk_thread_wait({j}, {site})"));
        if !is_blob(kind)
        {
            let load = format!("zk_ld_{}({at}, {site})", self.shared.mem(kind));
            let v = self.spill(kind, load);
            self.line(&format!("zk_thread_taken({j});"));
            return v;
        }
        let t = self.temp(kind);
        let size = self.shared.size(kind);
        self.line(&format!("zk_put_block({t}, {at}, {size});"));
        self.line(&format!("zk_thread_taken({j});"));
        t
    }

    /// The C text of a receive: the optional, in a local of its own.
    ///
    /// # Arguments
    ///
    /// * `pipe` - The channel.
    /// * `kind` - The optional kind.
    ///
    /// # Returns
    ///
    /// The C text.
    pub(crate) fn receive(&mut self, pipe: &Expr, kind: DataKind) -> String
    {
        let p = self.expr(pipe);
        let t = self.temp(kind);
        let d = self.shared.descr.of(kind);
        let site = self.check_site();
        self.line(&format!("zk_pipe_receive({p}, {t}, {d}, {site});"));
        t
    }

    /// Write a wait on several channels: three words for each channel (its
    /// number, the address of its optional and the kind of the optional), then
    /// the call that writes the place of the chosen one.
    ///
    /// # Arguments
    ///
    /// * `pipes` - The channels.
    /// * `slots` - The optional of each.
    /// * `ms` - The time limit in milliseconds, or `None`.
    /// * `idle` - Whether the wait does not wait.
    /// * `chosen` - The slot of the chosen place.
    pub(crate) fn select(
        &mut self, pipes: &[Expr], slots: &[Place], ms: Option<&Expr>, idle: bool,
        chosen: Place,
    )
    {
        let exprs: Vec<&Expr> = pipes.iter().chain(ms).collect();
        let texts = self.operands(&exprs);
        let words: Vec<String> = texts
            .iter()
            .zip(slots)
            .flat_map(|(p, s)| {
                let d = self.shared.descr.of(self.place_kind(*s));
                let at = self.addr_of(*s);
                [
                    format!("(size_t)({p})"),
                    format!("(size_t){at}"),
                    format!("(size_t){d}"),
                ]
            })
            .collect();
        let arms = format!("zk_t{}", self.fresh());
        self.line(&format!("size_t {arms}[] = {{{}}};", words.join(", ")));
        let mode = tern!(idle, "ZK_SELECT_IDLE", "ZK_SELECT_WAITS");
        let (m, mode) = match ms
        {
            Some(_) => (texts[pipes.len()].clone(), "ZK_SELECT_TIMED"),
            None => ("0".to_string(), mode),
        };
        let (c, n) = (self.place(chosen), pipes.len());
        self.line(&format!("{c} = zk_select({arms}, {n}, {m}, {mode});"));
    }

    /// Write a send: the value moves into the channel, and zeros take its
    /// place.
    ///
    /// # Arguments
    ///
    /// * `pipe` - The channel.
    /// * `value` - The value, owned.
    pub(crate) fn send(&mut self, pipe: &Expr, value: &Expr)
    {
        let texts = self.operands(&[pipe, value]);
        let (p, v) = (&texts[0], &texts[1]);
        let d = self.shared.descr.of(value.kind);
        let at = match is_blob(value.kind)
        {
            true => v.clone(),
            false =>
            {
                let held = self.spill(value.kind, v.clone());
                format!("((uint8_t *)&{held})")
            }
        };
        self.line(&format!("zk_pipe_send({p}, {at}, {d});"));
    }
}
