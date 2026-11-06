use std::collections::HashMap;

use bon::Builder;
use zakac_ast::name::Name;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::consts::ConstVal;
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
    /// The value of a constant, when the check works it out.
    #[builder(default)]
    pub value: ConstVal,
}

/// Block scopes from the outermost to the innermost.
#[derive(Debug, Default)]
pub(crate) struct Scopes(Vec<HashMap<Name, Binding>>);

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
    pub fn declare(&mut self, name: Name, binding: Binding) -> Result<(), Binding>
    {
        let Some(inner) = self.0.last_mut()
        else
        {
            return Ok(());
        };
        ret_if!(let Some(prior) = inner.get(&name), Err(*prior));
        inner.insert(name, binding);
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

    /// Give a name of the innermost scope its constant value.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `value` - The value.
    pub fn set_value(&mut self, name: &str, value: ConstVal)
    {
        let found = self.0.last_mut().and_then(|scope| scope.get_mut(name));
        do_if!(let Some(binding) = found, binding.value = value);
    }

    /// The number of open scopes; 1 at the top level of a file.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn depth(&self) -> usize { self.0.len() }

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
    pub fn restart(&mut self) -> HashMap<Name, Binding>
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
    /// Whether the function can change each parameter.
    #[builder(default)]
    pub edits: Vec<bool>,
    /// The result kind; `None` for a function without a result, and a
    /// result kind ([`DataKind::fallible`]) for a function that can fail.
    pub ret: Option<DataKind>,
    /// `true` when other files can import the function.
    #[builder(default)]
    pub exported: bool,
    /// The mark of each parameter, when the parameters have marks.
    pub marks: Option<Vec<Name>>,
    /// The name each later stage gives the function: the written name for
    /// the first function of a name in its file, `<name>#<n>` for the n-th.
    pub inner: Name,
}

impl FnSig
{
    /// Whether two functions take the same parameters with the same marks, so
    /// no call can tell them apart.
    ///
    /// # Arguments
    ///
    /// * `other` - The other function.
    ///
    /// # Returns
    ///
    /// `true` when both match.
    pub fn same_params(&self, other: &FnSig) -> bool
    {
        self.params == other.params && self.marks == other.marks
    }
}

/// Functions, indexed on name; each name holds its overloads in the order
/// of their declarations.
pub(crate) type FnSigs = HashMap<Name, Vec<FnSig>>;

/// The name the `index`-th function of a name, counted from 0, takes in the
/// later stages.
///
/// # Arguments
///
/// * `name` - The written name.
/// * `index` - Its place among the functions of that name in its file.
///
/// # Returns
///
/// The name, `<name>#<index + 1>` from the second function on.
pub(crate) fn overload_name(name: Name, index: usize) -> Name
{
    tern!(index == 0, name, Name::from(format!("{name}#{}", index + 1)))
}
