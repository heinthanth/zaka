use std::collections::HashMap;

use bon::Builder;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::kind::DataKind;

/// What a name allows besides reading it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Access
{
    /// A variable, a parameter or a generic-loop variable.
    Var,
    /// A constant.
    Const,
    /// A counted-loop variable.
    LoopVar,
}

/// What a declared name stands for.
#[derive(Debug, Clone, Copy, PartialEq, Builder)]
pub(crate) struct Binding
{
    /// The kind; `None` when an error left it unknown.
    pub kind: Option<DataKind>,
    /// What the name allows.
    pub access: Access,
    /// The declared name.
    pub span: Span,
    /// `true` for a top-level declaration that other files can import.
    #[builder(default)]
    pub exported: bool,
}

/// Block scopes from the outermost to the innermost.
#[derive(Debug, Default)]
pub(crate) struct Scopes(Vec<HashMap<String, Binding>>);

impl Scopes
{
    /// Open a scope.
    pub fn push(&mut self) { self.0.push(HashMap::new()) }

    /// Close the innermost scope.
    pub fn pop(&mut self) { self.0.pop(); }

    /// Declare a name in the innermost scope.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `binding` - What it stands for.
    ///
    /// # Returns
    ///
    /// `Ok`, or the earlier binding when the scope holds the name.
    pub fn declare(&mut self, name: &str, binding: Binding) -> Result<(), Binding>
    {
        let Some(inner) = self.0.last_mut()
        else
        {
            return Ok(());
        };
        ret_if!(let Some(prior) = inner.get(name), Err(*prior));
        inner.insert(name.to_string(), binding);
        Ok(())
    }

    /// Find a name, innermost scope first.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// What the name stands for, or `None` when it is not declared.
    pub fn lookup(&self, name: &str) -> Option<Binding>
    {
        self.0.iter().rev().find_map(|scope| scope.get(name)).copied()
    }

    /// Mark a name of the innermost scope as exported.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    pub fn mark_exported(&mut self, name: &str)
    {
        let found = self.0.last_mut().and_then(|scope| scope.get_mut(name));
        do_if!(let Some(binding) = found, binding.exported = true);
    }

    /// Close each scope and open one blank top-level scope, as a new file
    /// begins.
    ///
    /// # Returns
    ///
    /// The top-level names of the file that ended.
    pub fn restart(&mut self) -> HashMap<String, Binding>
    {
        let top = std::mem::take(&mut self.0).into_iter().next();
        self.push();
        top.unwrap_or_default()
    }
}

/// Parameter and result kinds of a function.
#[derive(Debug, Clone, PartialEq, Builder)]
pub(crate) struct FnSig
{
    /// The parameter kinds, in order.
    #[builder(default)]
    pub params: Vec<DataKind>,
    /// The result kind; `None` for a function without a result.
    pub ret: Option<DataKind>,
    /// `true` when other files can import the function.
    #[builder(default)]
    pub exported: bool,
}

/// Functions, indexed on name.
pub(crate) type FnSigs = HashMap<String, FnSig>;
