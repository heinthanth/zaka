//! Arithmetic and conversions, with the checks and the wrapping of the inline
//! integer functions of `zaka_target_c_runtime.h`.

use cranelift_codegen::ir::InstBuilder;
use cranelift_codegen::ir::Type;
use cranelift_codegen::ir::Value;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_module::Module;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::RunCheck;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::PrefixOp;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::func::FnGen;

impl<M: Module> FnGen<'_, '_, M>
{
    /// Whether an overflow stops the program.
    ///
    /// # Returns
    ///
    /// `true` under `--overflow stop`.
    fn stop(&self) -> bool { self.s.options.overflow == OverflowMode::Stop }

    /// Whether an integer kind is signed.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// `true` for a signed integer.
    fn signed(kind: DataKind) -> bool
    {
        kind.repr().int_kind().is_some_and(|k| k.signed)
    }

    /// An integer as an `i64`, extended as its kind is.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn as_i64(&mut self, v: Value, kind: DataKind) -> Value
    {
        let signed = Self::signed(kind);
        self.int_cast(v, signed, types::I64)
    }

    /// An integer as a `u64`: the bits of its kind, extended as its kind is.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn as_u64(&mut self, v: Value, kind: DataKind) -> Value { self.as_i64(v, kind) }

    /// An integer cut or extended to another integer kind, as a C cast does.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    /// * `signed` - Whether its kind is signed.
    /// * `to` - The CLIF kind.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn int_cast(&mut self, v: Value, signed: bool, to: Type) -> Value
    {
        let from = self.b.func.dfg.value_type(v);
        ret_if!(from == to, v);
        ret_if!(from.bits() > to.bits(), self.b.ins().ireduce(to, v));
        tern!(signed, self.b.ins().sextend(to, v), self.b.ins().uextend(to, v))
    }

    /// Stop the run: the result of an operation does not fit its kind.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator text.
    /// * `kind` - The kind.
    fn overflow(&mut self, op: &str, kind: DataKind)
    {
        let ptr = self.s.ptr;
        let (o, k) =
            (self.text_ptr(op.as_bytes()), self.text_ptr(kind.name().as_bytes()));
        self.rt_stop("zk_overflow", &[ptr, ptr], &[o, k]);
    }

    /// Stop the run when a negation overflows: the least value of a signed
    /// kind, or each value but 0 of an unsigned one.
    ///
    /// # Arguments
    ///
    /// * `a` - The operand.
    /// * `kind` - Its kind.
    fn neg_check(&mut self, a: Value, kind: DataKind)
    {
        let t = self.s.clif_of(kind);
        let bad = match Self::signed(kind)
        {
            true => self.b.ins().icmp_imm_s(IntCC::Equal, a, signed_min(t)),
            false => self.b.ins().icmp_imm_s(IntCC::NotEqual, a, 0),
        };
        self.when(bad, |g| g.overflow("neg", kind));
    }

    /// Stop the run when a shift goes as far as the width of its value, or
    /// further.
    ///
    /// # Arguments
    ///
    /// * `b` - The distance.
    /// * `rhs_kind` - The kind of the distance.
    /// * `t` - The CLIF kind of the value shifted.
    fn shift_check(&mut self, b: Value, rhs_kind: DataKind, t: Type)
    {
        let d = self.as_u64(b, rhs_kind);
        let bits = i64::from(t.bits());
        let far = self.b.ins().icmp_imm_s(IntCC::UnsignedGreaterThanOrEqual, d, bits);
        self.when(far, |g| {
            let n = g.b.ins().iconst(types::I32, bits);
            g.rt_stop("zk_shift_too_far", &[types::I64, types::I32], &[d, n]);
        });
    }

    /// Lower a prefix operator.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `a` - The operand.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn prefix(&mut self, op: PrefixOp, a: Value, kind: DataKind) -> Value
    {
        match op
        {
            PrefixOp::Neg if kind.is_float() => self.b.ins().fneg(a),
            PrefixOp::Neg =>
            {
                do_if!(self.stop(), self.neg_check(a, kind));
                self.b.ins().ineg(a)
            }
            PrefixOp::BitNot => self.b.ins().bnot(a),
            PrefixOp::Not => self.b.ins().icmp_imm_s(IntCC::Equal, a, 0),
        }
    }

    /// Lower an arithmetic, bitwise or shift operator.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `a` - The left operand.
    /// * `b` - The right operand.
    /// * `kind` - The result kind.
    /// * `rhs_kind` - The kind of the right operand: a shift distance has its
    ///   own.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn infix(
        &mut self, op: InfixOp, a: Value, b: Value, kind: DataKind, rhs_kind: DataKind,
    ) -> Value
    {
        ret_if!(kind.is_float(), self.float_op(op, a, b, kind));
        let signed = Self::signed(kind);
        match op
        {
            InfixOp::Add | InfixOp::Sub | InfixOp::Mul if self.stop() =>
            {
                self.checked(op, a, b, kind)
            }
            InfixOp::Add => self.b.ins().iadd(a, b),
            InfixOp::Sub => self.b.ins().isub(a, b),
            InfixOp::Mul => self.b.ins().imul(a, b),
            InfixOp::Div | InfixOp::Rem => self.divide(op, a, b, kind),
            InfixOp::Shl | InfixOp::Shr =>
            {
                let t = self.s.clif_of(kind);
                let checked = self.s.options.checks.has(RunCheck::Shift);
                do_if!(checked, self.shift_check(b, rhs_kind, t));
                match (op, signed)
                {
                    (InfixOp::Shl, _) => self.b.ins().ishl(a, b),
                    (_, true) => self.b.ins().sshr(a, b),
                    (_, false) => self.b.ins().ushr(a, b),
                }
            }
            InfixOp::BitAnd => self.b.ins().band(a, b),
            InfixOp::BitOr => self.b.ins().bor(a, b),
            InfixOp::BitXor => self.b.ins().bxor(a, b),
        }
    }

    /// `+`, `-` or `*` that stops the run when the result does not fit its
    /// kind: a narrow kind works in 64 bits and tests the range; a 64-bit kind
    /// tests the overflow flag.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `a` - The left operand.
    /// * `b` - The right operand.
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The value.
    fn checked(&mut self, op: InfixOp, a: Value, b: Value, kind: DataKind) -> Value
    {
        let t = self.s.clif_of(kind);
        let signed = Self::signed(kind);
        let mark = op.mnemonic_text();
        if t.bits() == 64
        {
            let (v, bad) = match (op, signed)
            {
                (InfixOp::Add, true) => self.b.ins().sadd_overflow(a, b),
                (InfixOp::Add, false) => self.b.ins().uadd_overflow(a, b),
                (InfixOp::Sub, true) => self.b.ins().ssub_overflow(a, b),
                (InfixOp::Sub, false) => self.b.ins().usub_overflow(a, b),
                (_, true) => self.b.ins().smul_overflow(a, b),
                (_, false) => self.b.ins().umul_overflow(a, b),
            };
            self.when(bad, |g| g.overflow(mark, kind));
            return v;
        }
        let (wa, wb) = (
            self.int_cast(a, signed, types::I64),
            self.int_cast(b, signed, types::I64),
        );
        let wide = match op
        {
            InfixOp::Add => self.b.ins().iadd(wa, wb),
            InfixOp::Sub => self.b.ins().isub(wa, wb),
            _ => self.b.ins().imul(wa, wb),
        };
        let v = self.b.ins().ireduce(t, wide);
        let back = self.int_cast(v, signed, types::I64);
        let bad = self.b.ins().icmp(IntCC::NotEqual, back, wide);
        self.when(bad, |g| g.overflow(mark, kind));
        v
    }

    /// `/` or `%` of integers, as `zk_div_N` and `zk_rem_N` work: a zero
    /// divisor stops the run under the divide check; the lowest signed value
    /// over -1 gives itself (or 0 for `%`), or stops under `--overflow stop`.
    ///
    /// # Arguments
    ///
    /// * `op` - `/` or `%`.
    /// * `a` - The dividend.
    /// * `b` - The divisor.
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The value.
    fn divide(&mut self, op: InfixOp, a: Value, b: Value, kind: DataKind) -> Value
    {
        let t = self.s.clif_of(kind);
        let div = op == InfixOp::Div;
        if self.s.options.checks.has(RunCheck::Divide)
        {
            let zero = self.b.ins().icmp_imm_s(IntCC::Equal, b, 0);
            self.when(zero, |g| g.rt_stop("zk_zero_divisor", &[], &[]));
        }
        if !Self::signed(kind)
        {
            return tern!(div, self.b.ins().udiv(a, b), self.b.ins().urem(a, b));
        }
        let held = self.temp(kind);
        let minus_one = self.b.ins().icmp_imm_s(IntCC::Equal, b, -1);
        let (odd, plain, after) =
            (self.b.create_block(), self.b.create_block(), self.b.create_block());
        self.branch(minus_one, odd, plain);
        self.enter_block(odd);
        let low = self.b.ins().icmp_imm_s(IntCC::Equal, a, signed_min(t));
        if self.stop()
        {
            let mark = tern!(div, "/", "%");
            self.when(low, |g| g.overflow(mark, kind));
        }
        // Over -1 the quotient is the negation, which wraps at the lowest
        // value; the remainder is 0.
        let v = tern!(div, self.b.ins().ineg(a), self.b.ins().iconst(t, 0));
        self.store(v, held);
        self.jump(after);
        self.enter_block(plain);
        let v = tern!(div, self.b.ins().sdiv(a, b), self.b.ins().srem(a, b));
        self.store(v, held);
        self.jump(after);
        self.enter_block(after);
        self.load(t, held)
    }

    /// A float operator; `%` is the remainder of C's `fmod`.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `a` - The left operand.
    /// * `b` - The right operand.
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The value.
    fn float_op(&mut self, op: InfixOp, a: Value, b: Value, kind: DataKind) -> Value
    {
        let t = self.s.clif_of(kind);
        match op
        {
            InfixOp::Add => self.b.ins().fadd(a, b),
            InfixOp::Sub => self.b.ins().fsub(a, b),
            InfixOp::Mul => self.b.ins().fmul(a, b),
            InfixOp::Div => self.b.ins().fdiv(a, b),
            _ =>
            {
                let name = tern!(t == types::F32, "zk_fmodf", "zk_fmod");
                self.rt(name, &[t, t], &[t], &[a, b]).unwrap_or(a)
            }
        }
    }

    /// A conversion of a scalar, as a C cast does: a float to an integer
    /// saturates and gives 0 for NaN, as `zk_f2i_N` does; an integer to a
    /// character is a scalar value or U+FFFD.
    ///
    /// # Arguments
    ///
    /// * `a` - The value.
    /// * `from` - Its kind.
    /// * `to` - The kind it converts to.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn scalar_convert(&mut self, a: Value, from: DataKind, to: DataKind) -> Value
    {
        let t = self.s.clif_of(to);
        let src = self.b.func.dfg.value_type(a);
        let address = |k: DataKind| k.is_ptr() || k == DataKind::Null || k.is_fn();
        match (from, to)
        {
            (DataKind::Float(_), DataKind::Int(k)) =>
            {
                let x =
                    tern!(src == types::F32, self.b.ins().fpromote(types::F64, a), a);
                self.float_to_int(x, k.signed, t)
            }
            (DataKind::Int(k), DataKind::Char) =>
            {
                let neg = match k.signed
                {
                    true => self.b.ins().icmp_imm_s(IntCC::SignedLessThan, a, 0),
                    false => self.b.ins().iconst(types::I8, 0),
                };
                let neg = self.b.ins().uextend(types::I32, neg);
                let v = self.int_cast(a, k.signed, types::I64);
                self.rt(
                    "zk_char_of",
                    &[types::I32, types::I64],
                    &[types::I32],
                    &[neg, v],
                )
                .unwrap_or(v)
            }
            (DataKind::Int(k), DataKind::Float(_)) =>
            {
                tern!(
                    k.signed,
                    self.b.ins().fcvt_from_sint(t, a),
                    self.b.ins().fcvt_from_uint(t, a)
                )
            }
            (DataKind::Char | DataKind::Bool, DataKind::Float(_)) =>
            {
                self.b.ins().fcvt_from_uint(t, a)
            }
            (DataKind::Float(_), DataKind::Float(_)) =>
            {
                tern!(
                    t.bits() > src.bits(),
                    self.b.ins().fpromote(t, a),
                    self.b.ins().fdemote(t, a)
                )
            }
            (DataKind::Int(k), DataKind::Int(_)) => self.int_cast(a, k.signed, t),
            (DataKind::Int(k), x) if address(x) => self.int_cast(a, k.signed, t),
            (DataKind::Char | DataKind::Bool, DataKind::Int(_)) =>
            {
                self.int_cast(a, false, t)
            }
            (f, DataKind::Int(_)) if address(f) => self.int_cast(a, false, t),
            (f, x) if address(f) && address(x) => a,
            _ =>
            {
                self.s.missing(format!(
                    "a conversion from {} to {}",
                    from.name(),
                    to.name()
                ));
                a
            }
        }
    }

    /// A float to an integer kind: saturated at the ends of the kind, 0 for
    /// NaN.
    ///
    /// # Arguments
    ///
    /// * `x` - The value, an `f64`.
    /// * `signed` - Whether the kind is signed.
    /// * `t` - The CLIF kind.
    ///
    /// # Returns
    ///
    /// The value.
    fn float_to_int(&mut self, x: Value, signed: bool, t: Type) -> Value
    {
        let wide = tern!(
            signed,
            self.b.ins().fcvt_to_sint_sat(types::I64, x),
            self.b.ins().fcvt_to_uint_sat(types::I64, x)
        );
        ret_if!(t.bits() == 64, wide);
        let bits = t.bits();
        let clamped = if signed
        {
            let (lo, hi) = (-(1i64 << (bits - 1)), (1i64 << (bits - 1)) - 1);
            let (lo, hi) = (
                self.b.ins().iconst(types::I64, lo),
                self.b.ins().iconst(types::I64, hi),
            );
            let v = self.b.ins().smax(wide, lo);
            self.b.ins().smin(v, hi)
        }
        else
        {
            let hi = self.b.ins().iconst(types::I64, ((1u64 << bits) - 1) as i64);
            self.b.ins().umin(wide, hi)
        };
        self.b.ins().ireduce(t, clamped)
    }
}

/// The lowest value of a signed CLIF kind, as an immediate.
///
/// # Arguments
///
/// * `t` - The kind.
///
/// # Returns
///
/// The value.
fn signed_min(t: Type) -> i64
{
    tern!(t.bits() >= 64, i64::MIN, -(1i64 << (t.bits() - 1)))
}

/// The text of an operator, as a stop names it.
trait Mark
{
    /// The text.
    ///
    /// # Returns
    ///
    /// The text, such as `+`.
    fn mnemonic_text(self) -> &'static str;
}

impl Mark for InfixOp
{
    fn mnemonic_text(self) -> &'static str { self.into() }
}
