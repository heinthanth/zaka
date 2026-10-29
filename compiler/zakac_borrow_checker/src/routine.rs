use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Slot;
use zakac_ir_builder::ir::Stmt;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

/// The name the main program goes under.
pub(crate) const MAIN_NAME: &str = "main";

/// A function or the main program, as the flow sees it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Routine<'m>
{
    /// The name.
    pub(crate) name: &'m str,
    /// The local slots; none for the main program, whose variables are
    /// the globals.
    pub(crate) locals: &'m [Slot],
    /// The number of parameters; `l0` to the one before this are the
    /// parameters.
    pub(crate) params: u32,
    /// The local slot of the result variable, for a function with a
    /// result.
    pub(crate) result: Option<u32>,
    /// The result kind, for a function with a result.
    pub(crate) ret: Option<DataKind>,
    /// The statements.
    pub(crate) block: &'m [Stmt],
    /// Source region.
    pub(crate) span: Span,
    /// Whether this is the main program.
    pub(crate) is_main: bool,
}

impl<'m> Routine<'m>
{
    /// The routine of a function.
    ///
    /// # Arguments
    ///
    /// * `func` - The function.
    ///
    /// # Returns
    ///
    /// The routine.
    pub(crate) fn func(func: &'m Func) -> Self
    {
        Self {
            name: &func.name,
            locals: &func.locals,
            params: func.params,
            result: func.result,
            ret: func.ret,
            block: &func.block,
            span: func.span,
            is_main: false,
        }
    }

    /// The routine of the top-level statements.
    ///
    /// # Arguments
    ///
    /// * `module` - The module.
    ///
    /// # Returns
    ///
    /// The routine.
    pub(crate) fn main(module: &'m Module) -> Self
    {
        Self {
            name: MAIN_NAME,
            locals: &[],
            params: 0,
            result: None,
            ret: None,
            block: &module.main,
            span: module.span,
            is_main: true,
        }
    }
}
