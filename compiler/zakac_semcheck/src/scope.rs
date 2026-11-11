use std::cell::RefCell;
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
    /// The name in the file that declares it, for a name imported from
    /// another file; `None` when `span` is that name.
    pub from: Option<Span>,
    /// `true` for a top-level declaration that other files can import.
    #[builder(default)]
    pub exported: bool,
    /// The value of a constant, when the check works it out.
    #[builder(default)]
    pub value: ConstVal,
}

impl Binding
{
    /// The declared name in the file that declares it.
    ///
    /// # Returns
    ///
    /// The span of that name.
    pub fn home(&self) -> Span { self.from.unwrap_or(self.span) }
}

/// A local of an enclosing function that a nested function names, or a
/// nested function with captures that it calls or names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capture
{
    /// The name.
    pub name: Name,
    /// The declared name of the local or the function.
    pub decl: Span,
    /// Whether it names a function declared inside a function, which is a
    /// capture when it turns out to be a closure.
    pub func: bool,
}

/// Block scopes from the outermost to the innermost, with the functions
/// declared in each and the functions whose blocks are open.
#[derive(Debug, Default)]
pub(crate) struct Scopes
{
    /// The names of each scope.
    names: Vec<HashMap<Name, Binding>>,
    /// The functions each scope declares, beside its names: the functions
    /// declared inside a function.
    fns: Vec<HashMap<Name, FnSig>>,
    /// The index of the first scope of each function whose block is open,
    /// the outermost first.
    bases: Vec<usize>,
    /// The locals of enclosing functions each open function names so far,
    /// in the order of [`Self::bases`]. A read records them, so it sits in a
    /// cell.
    captures: RefCell<Vec<Vec<Capture>>>,
}

impl Scopes
{
    /// Open a scope.
    pub fn push(&mut self)
    {
        self.names.push(HashMap::new());
        self.fns.push(HashMap::new());
    }

    /// Close the innermost scope.
    pub fn pop(&mut self)
    {
        self.names.pop();
        self.fns.pop();
    }

    /// Open the first scope of a function.
    pub fn enter_fn(&mut self)
    {
        self.bases.push(self.names.len());
        self.captures.get_mut().push(Vec::new());
        self.push();
    }

    /// Close the first scope of a function.
    ///
    /// # Returns
    ///
    /// The locals of enclosing functions it names, in the order of their
    /// first use.
    pub fn leave_fn(&mut self) -> Vec<Capture>
    {
        self.pop();
        self.bases.pop();
        self.captures.get_mut().pop().unwrap_or_default()
    }

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
        let Some(inner) = self.names.last_mut()
        else
        {
            return Ok(());
        };
        ret_if!(let Some(prior) = inner.get(&name), Err(*prior));
        inner.insert(name, binding);
        Ok(())
    }

    /// Find a name, innermost scope first. A local of an enclosing function
    /// counts as a borrow of each open function inside that one.
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
        let mut each = self.names.iter().enumerate().rev();
        let (at, stored, binding) =
            each.find_map(|(i, s)| s.get_key_value(name).map(|(k, b)| (i, *k, *b)))?;
        let capture = Capture { name: stored, decl: binding.span, func: false };
        self.note_capture(at, capture);
        Some(binding)
    }

    /// Record a name found in scope `at` as a borrow of each open function
    /// whose first scope lies past it; a name of the top level of the file
    /// is no local.
    ///
    /// # Arguments
    ///
    /// * `at` - The index of the scope that declares the name.
    /// * `capture` - The name.
    fn note_capture(&self, at: usize, capture: Capture)
    {
        ret_if!(at == 0);
        let mut captures = self.captures.borrow_mut();
        let open = self.bases.iter().zip(captures.iter_mut());
        let inside = open.filter(|(base, _)| at < **base).map(|(_, list)| list);
        inside.for_each(|list| tern!(list.contains(&capture), (), list.push(capture)));
    }

    /// Declare a function in the innermost scope.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `sig` - The function.
    ///
    /// # Returns
    ///
    /// `Ok`, or the earlier function when the scope holds the name.
    pub fn declare_fn(&mut self, name: Name, sig: FnSig) -> Result<(), FnSig>
    {
        let Some(inner) = self.fns.last_mut()
        else
        {
            return Ok(());
        };
        ret_if!(let Some(prior) = inner.get(&name), Err(prior.clone()));
        inner.insert(name, sig);
        Ok(())
    }

    /// Find a function declared inside a function, innermost scope first. A
    /// closure declared in an enclosing function counts as a use of each open
    /// function inside that one, as a local does.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The function, or `None`.
    pub fn lookup_fn(&self, name: &str) -> Option<FnSig>
    {
        let mut each = self.fns.iter().enumerate().rev();
        let (at, stored, sig) =
            each.find_map(|(i, s)| s.get_key_value(name).map(|(k, f)| (i, *k, f)))?;
        // Whether the function is a closure is known once each function is
        // checked, so each use is noted and the check settles it.
        let capture = Capture { name: stored, decl: sig.decl, func: true };
        self.note_capture(at, capture);
        Some(sig.clone())
    }

    /// Mark a function declared inside a function as a closure, once its
    /// statements turn out to name locals around it.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function, innermost scope first.
    pub fn mark_closure(&mut self, name: &str)
    {
        let found = self.fns.iter_mut().rev().find_map(|s| s.get_mut(name));
        do_if!(let Some(sig) = found, sig.closure = true);
    }

    /// Give a name of the innermost scope its constant value.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `value` - The value.
    pub fn set_value(&mut self, name: &str, value: ConstVal)
    {
        let found = self.names.last_mut().and_then(|scope| scope.get_mut(name));
        do_if!(let Some(binding) = found, binding.value = value);
    }

    /// The number of open scopes; 1 at the top level of a file.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn depth(&self) -> usize { self.names.len() }

    /// Mark a name of the innermost scope as exported.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    pub fn mark_exported(&mut self, name: &str)
    {
        let found = self.names.last_mut().and_then(|scope| scope.get_mut(name));
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
        let top = std::mem::take(&mut self.names).into_iter().next();
        self.fns.clear();
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
    /// `true` for a function that comes from outside the program, which no
    /// function value can hold.
    #[builder(default)]
    pub external: bool,
    /// `true` for a function marked `စိတ်မချရသော`, which marked code alone
    /// can call.
    #[builder(default)]
    pub unchecked: bool,
    /// `true` for a C function that takes more arguments than it names,
    /// `နှင့် အခြားတန်ဖိုးများ`.
    #[builder(default)]
    pub variadic: bool,
    /// The mark of each parameter, when the parameters have marks.
    pub marks: Option<Vec<Name>>,
    /// Whether the function takes each parameter over and closes it: a
    /// parameter of a C function marked `အပိုင်အနေဖြင့်`.
    #[builder(default)]
    pub closes: Vec<bool>,
    /// `true` for a function declared inside a function with a capture
    /// list: its calls and its address go through its closure.
    #[builder(default)]
    pub closure: bool,
    /// `true` for a closure that borrows: a capture with `ငှား၍`, one that
    /// holds a borrow, or a borrowing closure among its captures.
    #[builder(default)]
    pub borrows: bool,
    /// The declared name; a placeholder for a function value's own
    /// signature.
    #[builder(default)]
    pub decl: Span,
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
