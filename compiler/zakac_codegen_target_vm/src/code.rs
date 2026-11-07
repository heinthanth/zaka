use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::bc::Op;

/// A place in the code that jumps can name before it is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(u32);

/// A label that a jump names but that was never bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unbound(pub u32);

/// Collects the ops of one function and patches jump targets once their
/// labels are bound.
#[derive(Debug, Clone, Default)]
pub struct CodeBuilder
{
    /// The ops so far; jumps to labels hold a placeholder target.
    ops: Vec<Op>,
    /// The source region of each op, parallel to `ops`.
    spans: Vec<Span>,
    /// The region the next ops get.
    span: Span,
    /// The op index of each label, once bound.
    bound: Vec<Option<u32>>,
    /// Jumps to patch: the op index and the label it names.
    fixups: Vec<(usize, Label)>,
}

impl CodeBuilder
{
    /// Make a builder with no ops.
    ///
    /// # Returns
    ///
    /// The builder.
    pub fn new() -> Self { Self::default() }

    /// The index the next op will get.
    ///
    /// # Returns
    ///
    /// The index.
    pub fn pc(&self) -> u32 { self.ops.len() as u32 }

    /// The ops so far, with placeholder targets in unpatched jumps.
    ///
    /// # Returns
    ///
    /// The ops.
    pub fn ops(&self) -> &[Op] { &self.ops }

    /// Make a new label that is not bound.
    ///
    /// # Returns
    ///
    /// The label.
    pub fn label(&mut self) -> Label
    {
        self.bound.push(None);
        Label(self.bound.len() as u32 - 1)
    }

    /// Bind a label to the index of the next op.
    ///
    /// # Arguments
    ///
    /// * `label` - The label; it must not be bound twice.
    pub fn bind(&mut self, label: Label)
    {
        let pc = self.pc();
        let slot = &mut self.bound[label.0 as usize];
        debug_assert!(slot.is_none(), "label {} bound twice", label.0);
        *slot = Some(pc);
    }

    /// Set the source region the next ops get.
    ///
    /// # Arguments
    ///
    /// * `span` - The region.
    ///
    /// # Returns
    ///
    /// The region in place before.
    pub fn at(&mut self, span: Span) -> Span { std::mem::replace(&mut self.span, span) }

    /// Add an op; it gets the current source region.
    ///
    /// # Arguments
    ///
    /// * `op` - The op.
    pub fn emit(&mut self, op: Op)
    {
        self.ops.push(op);
        self.spans.push(self.span);
    }

    /// Add a jump to a label.
    ///
    /// # Arguments
    ///
    /// * `label` - The target.
    pub fn jump(&mut self, label: Label) { self.fixup(Op::Jump(u32::MAX), label); }

    /// Add a jump to a label taken when the popped boolean is false.
    ///
    /// # Arguments
    ///
    /// * `label` - The target.
    pub fn jump_if_false(&mut self, label: Label)
    {
        self.fixup(Op::JumpIfFalse(u32::MAX), label);
    }

    /// Add a jump to a label taken when the popped boolean is true.
    ///
    /// # Arguments
    ///
    /// * `label` - The target.
    pub fn jump_if_true(&mut self, label: Label)
    {
        self.fixup(Op::JumpIfTrue(u32::MAX), label);
    }

    /// Add a jump op and note it for patching.
    ///
    /// # Arguments
    ///
    /// * `op` - The jump with a placeholder target.
    /// * `label` - The label it names.
    fn fixup(&mut self, op: Op, label: Label)
    {
        self.fixups.push((self.ops.len(), label));
        self.emit(op);
    }

    /// Patch each jump with the index of its label.
    ///
    /// # Returns
    ///
    /// The ops and their source regions, or the first label a jump names that
    /// was never bound.
    pub fn finish(self) -> Result<(Vec<Op>, Vec<Span>), Unbound>
    {
        let CodeBuilder { mut ops, spans, bound, fixups, .. } = self;
        for (at, label) in fixups
        {
            let target = bound[label.0 as usize];
            ret_if!(target.is_none(), Err(Unbound(label.0)));
            ops[at] = ops[at].retarget(target.unwrap_or_default());
        }
        Ok((ops, spans))
    }
}
