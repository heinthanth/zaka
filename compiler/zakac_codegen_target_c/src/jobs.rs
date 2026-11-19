//! Jobs and channels in C: a spawn fills the data block of the job with a
//! duplicate of each argument and hands it to the runtime, which runs the
//! step of the job on a thread of its own under the guard of this file. The
//! guard keeps the place a trap of the job jumps back to.

use std::fmt::Write;

use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::ParMap;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::emit::FnGen;
use crate::emit::KINDS_HEADER;
use crate::emit::Shared;
use crate::emit::func_name;
use crate::kinds::is_blob;

/// The file of the guard and of the step of each job.
pub const JOBS_SOURCE: &str = "zaka_jobs.c";

/// The octets each part of a data block starts on.
const PART_ALIGN: u32 = 16;

/// The guard of a job and the function that jumps back to it.
pub const GUARD: &str = "\
/* A longjmp of 64-bit Windows unwinds each frame it leaves; clearing the frame
   of the buffer makes it restore the registers alone, as on each other OS. */
#if defined(_WIN64) && defined(ZK_LIB_PLAIN_JUMP)
#define ZK_JOB_NO_UNWIND(buf) (((_JUMP_BUFFER *)(buf))->Frame = 0)
#else
#define ZK_JOB_NO_UNWIND(buf) ((void)(buf))
#endif

/**
 * Jump back to the guard of the job of this thread, on a trap that ends the
 * job alone.
 *
 * @param trap The id of the trap, which the runtime keeps.
 */
_Noreturn void zk_job_leave(int32_t trap)
{
    (void)trap;
    longjmp(*(jmp_buf *)zk_lib_jump(), 1);
}

/**
 * Run the step of a job, keeping the place a trap of the job jumps back to.
 *
 * @param step The step.
 * @param data The data block of the job.
 * @return 0, or 1 once a trap ended the job.
 */
int32_t zk_job_guard(void (*step)(void *), void *data)
{
    jmp_buf zk_here;
    if (setjmp(zk_here) != 0) return 1;
    ZK_JOB_NO_UNWIND(zk_here);
    zk_lib_jump_set(&zk_here);
    step(data);
    zk_flush();
    return 0;
}
";

/// The offset of each part of the data block of a job that calls a function:
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

/// The C file of the guard and of the step of each function a job runs, or
/// `None` for a program that starts no job.
///
/// # Arguments
///
/// * `shared` - The state shared across functions.
///
/// # Returns
///
/// The C text.
pub(crate) fn jobs_source(shared: &mut Shared<'_>) -> Option<String>
{
    if shared.spawned.is_empty() && shared.mapped.is_empty()
    {
        return None;
    }
    let mut out =
        format!("#include \"{KINDS_HEADER}\"\n#include <setjmp.h>\n\n{GUARD}");
    let spawned: Vec<u32> = shared.spawned.iter().copied().collect();
    spawned.into_iter().for_each(|f| out.push_str(&step(shared, f)));
    let mapped: Vec<u32> = shared.mapped.iter().copied().collect();
    mapped.into_iter().for_each(|f| out.push_str(&map_step(shared, f)));
    Some(out)
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
        format!("zk_ld_{}(zk_c)", shared.mem(param))
    );
    let call = match ret
    {
        Some(k) if is_blob(k) => format!("    {}(zk_o, {arg});\n", func_name(f)),
        Some(k) =>
        {
            format!("    zk_st_{}(zk_o, {}({arg}));\n", shared.mem(k), func_name(f))
        }
        None => format!("    {}({arg});\n", func_name(f)),
    };
    let name = shared.module.funcs[f as usize].name.replace("*/", "* /");
    format!(
        "\n/** The step of a parallel map through `{name}`. */\nvoid \
         zk_map_step_{f}(uint8_t *zk_c, uint8_t *zk_o)\n{{\n{lend}{call}}}\n"
    )
}

/// The step of a job that calls a function: it calls the function with the
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
            false => format!("zk_ld_{}({place})", shared.mem(k)),
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
        Some(k) => format!("    zk_st_{}(zk_b, {});\n", shared.mem(k), call(None)),
        None => format!("    {};\n", call(None)),
    };
    format!(
        "\n/** The step of a job that runs `{}`. */\nvoid zk_job_step_{f}(void \
         *zk_d)\n{{\n    uint8_t *zk_b = (uint8_t *)zk_d;\n{call_line}}}\n",
        shared.module.funcs[f as usize].name.replace("*/", "* /")
    )
}

/// The C file of the guard of a native build, whose steps Cranelift writes.
///
/// # Returns
///
/// The C text.
pub fn native_jobs_glue() -> String
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
/// The C text, blank for a program that starts no job.
pub(crate) fn declarations(shared: &Shared<'_>) -> String
{
    if shared.spawned.is_empty() && shared.mapped.is_empty()
    {
        return String::new();
    }
    let mut out = String::from(
        "int32_t zk_job_guard(void (*step)(void *), void *data);\n_Noreturn void \
         zk_job_leave(int32_t trap);\n",
    );
    shared
        .spawned
        .iter()
        .for_each(|f| _ = writeln!(out, "void zk_job_step_{f}(void *zk_d);"));
    let steps = shared.mapped.iter();
    steps.for_each(|f| {
        _ = writeln!(out, "void zk_map_step_{f}(uint8_t *zk_c, uint8_t *zk_o);")
    });
    out
}

impl FnGen<'_, '_>
{
    /// Write a parallel map that runs its rounds at once, in place of its
    /// loop.
    ///
    /// # Arguments
    ///
    /// * `p` - The map.
    pub(crate) fn par_map(&mut self, p: &ParMap)
    {
        self.shared.mapped.insert(p.func);
        let (src, out) = (self.place(p.src), self.place(p.out));
        let (sk, ok) =
            (self.shared.descr.of(p.src_kind), self.shared.descr.of(p.out_kind));
        self.line(&format!(
            "zk_par_map({src}, {sk}, {out}, {ok}, zk_map_step_{});",
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
    ///
    /// # Returns
    ///
    /// The C text of the number of the job.
    pub(crate) fn spawn(&mut self, f: u32, args: &[Expr]) -> String
    {
        self.shared.spawned.insert(f);
        let platform = self.shared.options.platform_bits;
        let kinds = parts(self.shared.module, f);
        let (offsets, size) = part_offsets(&kinds, platform);
        let values = self.operands(&args.iter().collect::<Vec<_>>());
        let data = self.spill(DataKind::OPAQUE, format!("zk_job_data({size})"));
        // The arguments come owned, as the value of a set does: each moves
        // into the block.
        for ((v, arg), at) in values.iter().zip(args).zip(offsets.iter().skip(1))
        {
            self.put_at(&format!("({data} + {at})"), v, arg.kind);
        }
        let ret =
            kinds[0].map_or_else(|| "NULL".to_string(), |k| self.shared.descr.of(k));
        let abort = i32::from(self.shared.options.trap_abort);
        format!(
            "zk_job_start(zk_job_guard, zk_job_leave, zk_job_step_{f}, {data}, {ret}, \
             {abort})"
        )
    }

    /// The C text of a wait that takes the result of a job: a duplicate of
    /// it, which the job keeps to the end of the program.
    ///
    /// # Arguments
    ///
    /// * `job` - The job.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// The C text.
    pub(crate) fn wait(&mut self, job: &Expr, kind: DataKind) -> String
    {
        let j = self.expr(job);
        let at = self.spill(DataKind::OPAQUE, format!("zk_job_wait({j})"));
        if !is_blob(kind)
        {
            return format!("zk_ld_{}({at})", self.shared.mem(kind));
        }
        let t = self.temp(kind);
        let size = self.shared.size(kind);
        self.line(&format!("zk_put_block({t}, {at}, {size});"));
        if kind.holds_text()
        {
            let d = self.shared.descr.of(kind);
            self.line(&format!("zk_dup_seqs({t}, {d});"));
        }
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
        self.line(&format!("zk_pipe_receive({p}, {t}, {d});"));
        t
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
