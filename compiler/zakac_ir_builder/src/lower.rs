use std::collections::HashMap;
use std::collections::HashSet;

use bon::Builder;
use zakac_ast::expr as ast;
use zakac_ast::kind::Kind;
use zakac_ast::name::Name;
use zakac_ast::stmt as st;
use zakac_ast::stmt::START_NAME;
use zakac_ast::verb::Dir;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_block;
use zakac_ast::visit::walk_fn;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::enums::EnumKind;
use zakac_semcheck::enums::MemberValue;
use zakac_semcheck::fn_kind::FnKind;
use zakac_semcheck::foreign;
use zakac_semcheck::host;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;
use zakac_semcheck::kind::Width;
use zakac_semcheck::semcheck::KindTable;
use zakac_semcheck::semcheck::RESULT_NAME;
use zakac_semcheck::semcheck::each_list;
use zakac_semcheck::structs::StructTable;
use zakac_span::span::Span;

use crate::fault::Fault;
use crate::fault::has_call;
use crate::fault::has_pass;
use crate::fault::stmt_passes;
use crate::fault::wrap_returns;
use crate::ir::Access;
use crate::ir::CmpOp;
use crate::ir::Dest;
use crate::ir::Expr;
use crate::ir::ExprNode;
use crate::ir::FmtPiece;
use crate::ir::ForeignFn;
use crate::ir::Func;
use crate::ir::HaltCause;
use crate::ir::Host;
use crate::ir::InfixOp;
use crate::ir::LogicOp;
use crate::ir::LowerOptions;
use crate::ir::Module;
use crate::ir::Place;
use crate::ir::PrefixOp;
use crate::ir::PrintValue;
use crate::ir::SliceFrom;
use crate::ir::Slot;
use crate::ir::Stmt;
use crate::ir::fn_shape;
use crate::ir::fresh_text;
use crate::value::Value;

mod nested;
mod table;

use nested::Nested;
use table::row_place;

/// Lower a checked program to the IR. Names become slots, blocks are
/// flattened, operators get one spelling, and widening conversions become
/// explicit. A negated integer literal becomes a negative constant.
///
/// # Arguments
///
/// * `program` - The tree; the check must have reported no errors.
/// * `kinds` - The kind of each expression, from the check.
/// * `structs` - The kind of each declared struct, from the check.
/// * `captures` - The locals each function declared inside a function borrows,
///   from the check.
/// * `options` - The options.
///
/// # Returns
///
/// The module.
///
/// # Panics
///
/// When an expression has no kind, which a clean check rules out.
pub fn lower(
    program: &st::Program, kinds: &KindTable, structs: &StructTable,
    options: &LowerOptions,
) -> Module
{
    let unit_sigs = collect_sigs(program, structs, options.platform_bits);
    let top_count = program
        .stmts
        .iter()
        .filter(|s| matches!(s, st::Stmt::Fn(f) if !is_generic(f)))
        .count();
    #[rustfmt::skip]
    let mut lowerer = Lowerer::builder().kinds(kinds).structs(structs)
        .top_count(top_count as u32).platform(options.platform_bits)
        .sigs(unit_sigs[0].clone()).unit_sigs(unit_sigs).build();
    lowerer.scopes.push(HashMap::new());
    lowerer.nested.push(HashMap::new());
    lowerer.drops.push(Vec::new());
    let mut main = Vec::new();
    let mut imported_inits = 0;
    for (i, s) in program.stmts.iter().enumerate()
    {
        let unit = program.unit_of(i);
        lowerer.enter_unit(unit);
        lowerer.suffix = unit_suffix(program, unit);
        lowerer.top_stmt(s, &mut main);
        do_if!(!program.is_root(unit), imported_inits = main.len() as u32);
    }
    let host = lowerer.start_call(&mut main);
    // The functions declared inside functions follow the top-level ones.
    let lifted = std::mem::take(&mut lowerer.lifted).into_iter();
    lowerer.funcs.extend(lifted.map(|f| f.expect("each lifted function is lowered")));
    let ends_in_return = matches!(main.last(), Some(Stmt::Return { .. }));
    do_if!(!ends_in_return, main.extend(lowerer.global_drops(program.span)));
    #[rustfmt::skip]
    let module = Module::builder().globals(lowerer.globals).funcs(lowerer.funcs)
        .foreign(lowerer.foreign).main(main).imported_inits(imported_inits).host(host)
        .unsafe_spans(unsafe_spans(program)).span(program.span).build();
    module
}

/// The source regions of unsafe code in a program.
///
/// # Arguments
///
/// * `program` - The tree.
///
/// # Returns
///
/// The region of each unsafe function and each unsafe block, outer ones
/// included with the ones inside them.
fn unsafe_spans(program: &st::Program) -> Vec<Span>
{
    /// Gathers the regions as it walks the tree.
    struct Marks(Vec<Span>);
    impl Visitor for Marks
    {
        fn visit_fn(&mut self, n: &st::FnDecl)
        {
            do_if!(n.unchecked, self.0.push(n.span));
            walk_fn(self, n);
        }

        fn visit_block(&mut self, n: &st::BlockStmt)
        {
            do_if!(n.unchecked, self.0.push(n.span));
            walk_block(self, n);
        }
    }
    let mut marks = Marks(Vec::new());
    marks.visit_program(program);
    marks.0
}

/// What the function names of a unit end with: nothing for the file that
/// is run, and `@` with the file stem for an imported file, so two files
/// can each have a private function of one name.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `unit` - The unit.
///
/// # Returns
///
/// The suffix.
fn unit_suffix(program: &st::Program, unit: u32) -> String
{
    ret_if!(program.is_root(unit), String::new());
    format!("@{}", unit_stem(program, unit))
}

/// The stem of the file of a unit: its name with no folder and no extension.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `unit` - The unit.
///
/// # Returns
///
/// The stem; a program not read from a file has a blank one.
fn unit_stem(program: &st::Program, unit: u32) -> String
{
    let name = program.units.get(unit as usize).map_or("", |u| u.name.as_str());
    let stem = std::path::Path::new(name).file_stem().and_then(|s| s.to_str());
    stem.unwrap_or(name).to_string()
}

/// What a call needs to know about a function.
#[derive(Debug, Clone)]
struct Sig
{
    /// Index into the module functions.
    index: u32,
    /// The parameter kinds; a parameter the function can change holds a
    /// borrow of the caller's place.
    params: Vec<DataKind>,
    /// Whether the function can change each parameter.
    edits: Vec<bool>,
    /// Whether the function fills each parameter, `ဒေတာ ထည့်ပေးရန်`: it reads
    /// nothing the caller left there.
    fills: Vec<bool>,
    /// Whether each parameter takes over the block of its argument,
    /// `အပိုင်အနေဖြင့်`: the caller's place holds the zero word after the call.
    owned: Vec<bool>,
    /// The result kind, when the function has one; a result kind when it
    /// can fail.
    ret: Option<DataKind>,
    /// The C function a call names itself, for a C function that takes other
    /// values; `None` for each other function, which a call runs.
    foreign: Option<ForeignFn>,
    /// The locals a function declared inside a function borrows, whose
    /// borrows a call passes after the arguments.
    captures: Vec<Name>,
    /// The name of the closure of a function declared inside a function with
    /// captures, whose value a call passes after the arguments, before the
    /// borrows.
    own: Option<Name>,
}

/// What a call runs: a function of the module, or the function a value
/// holds.
enum Target
{
    /// A function of the module.
    Fn
    {
        /// Index into the module functions.
        func: u32,
        /// The function name.
        name: String,
    },
    /// The function value, of a function kind.
    Value(Expr),
    /// A C function that takes other values, at this index of the C
    /// functions of the module.
    Foreign(u32),
}

impl Target
{
    /// The call node with these arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - The lowered arguments.
    ///
    /// # Returns
    ///
    /// The node.
    fn node(self, args: Vec<Expr>) -> ExprNode
    {
        match self
        {
            Target::Fn { func, name } => ExprNode::Call { func, name, args },
            Target::Value(callee) =>
            {
                ExprNode::CallValue { callee: Box::new(callee), args }
            }
            Target::Foreign(func) => ExprNode::Foreign { func, args },
        }
    }

    /// The call statement with these arguments, its result dropped.
    ///
    /// # Arguments
    ///
    /// * `args` - The lowered arguments.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The statement.
    fn stmt(self, args: Vec<Expr>, span: Span) -> Stmt
    {
        match self
        {
            Target::Fn { func, name } => Stmt::Call { func, name, args, span },
            Target::Value(callee) => Stmt::CallValue { callee, args, span },
            Target::Foreign(func) => Stmt::Foreign { func, args, span },
        }
    }
}

/// The signature a call through a value of a function kind sees, laid out
/// as [`param_kind`] places a parameter.
///
/// # Arguments
///
/// * `f` - The function kind.
///
/// # Returns
///
/// The signature; its index names no function.
fn value_sig(f: &FnKind) -> Sig
{
    let (params, ret) = fn_shape(f);
    Sig {
        index: u32::MAX,
        params,
        edits: f.edits(),
        fills: Vec::new(),
        owned: Vec::new(),
        ret,
        foreign: None,
        captures: Vec::new(),
        own: None,
    }
}

/// Whether a function is generic: a template the check made instances of.
///
/// # Arguments
///
/// * `f` - The function.
///
/// # Returns
///
/// `true` when it has kind parameters.
fn is_generic(f: &st::FnDecl) -> bool { !f.kind_params.is_empty() }

/// The signatures of the top-level functions of each file, numbered in
/// source order across the files; the first of two with one name in a file
/// wins, as in the check. The start block takes its number and no name.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `structs` - The kind of each declared struct.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The signatures of each unit, one per name.
fn collect_sigs(
    program: &st::Program, structs: &StructTable, platform: u16,
) -> Vec<HashMap<Name, Sig>>
{
    let mut sigs = vec![HashMap::new(); program.units.len().max(1)];
    let decls = program.stmts.iter().enumerate().filter_map(|(i, s)| {
        let f = tern!(let st::Stmt::Fn(f) = s, Some(f), None)?;
        tern!(is_generic(f), None, Some((program.unit_of(i) as usize, f)))
    });
    for (index, (unit, f)) in decls.enumerate()
    {
        let params: Vec<DataKind> =
            f.params.iter().map(|p| param_kind(p, structs)).collect();
        let edits = f.params.iter().map(|p| p.edit).collect();
        let fills = f.params.iter().map(|p| p.fill).collect();
        let owned = f.params.iter().map(|p| p.owned).collect();
        let ret = f.ret.as_ref().map(|k| DataKind::from_ast(k, structs).repr());
        let foreign = f.lib.as_ref().filter(|_| f.variadic).map(|lib| {
            #[rustfmt::skip]
            return ForeignFn::builder().lib(lib.lit.value.clone()).name(f.name.name.to_string())
                .params(params.iter().map(|k| c_param_kind(*k, platform)).collect())
                .maybe_ret(ret).variadic(true).owned_ret(f.owned_ret).build();
        });
        let sig = Sig {
            index: index as u32,
            params,
            edits,
            fills,
            owned,
            ret,
            foreign,
            captures: Vec::new(),
            own: None,
        };
        do_if!(f.start, continue);
        // An instance of a generic is a function of each file, and each
        // function is known in each file as its name, `@` and the stem of its
        // file, as an instance calls a function of another file.
        let shared =
            Name::from(format!("{}@{}", f.name.name, unit_stem(program, unit as u32)));
        do_for!(table in sigs.iter_mut(), _ = table.entry(shared).or_insert(sig.clone()));
        let instance = f.name.name.contains('<');
        do_if!(
            instance,
            do_for!(table in sigs.iter_mut(), _ = table.entry(f.name.name).or_insert(sig.clone()))
        );
        sigs[unit].entry(f.name.name).or_insert(sig);
    }
    sigs
}

/// The kind of a parameter slot: the written kind, or for a parameter the
/// function can change, a borrow of it.
///
/// # Arguments
///
/// * `p` - The parameter.
/// * `structs` - The kind of each declared struct.
///
/// # Returns
///
/// The kind.
fn param_kind(p: &st::Param, structs: &StructTable) -> DataKind
{
    let kind = DataKind::from_ast(&p.kind, structs).repr();
    tern!(p.edit, kind.pointer_to().as_borrowed(), kind)
}

/// The list of numbers a parameter of a C function passes, as
/// [`param_kind`] sets the parameter out: the list itself, or the address of
/// the caller's list for one marked `ဒေတာ ပြောင်းလဲရန်`.
///
/// # Arguments
///
/// * `kind` - The parameter kind.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The list kind and whether the parameter holds its address, or `None`
/// for another parameter.
fn c_list(kind: DataKind, platform: u16) -> Option<(DataKind, bool)>
{
    let list = |k: DataKind| foreign::number_cells(k, platform).map(|_| k);
    ret_if!(let Some(k) = list(kind), Some((k, false)));
    kind.pointee().and_then(list).map(|k| (k, true))
}

/// The kind C takes a parameter as: the address of the first cell of a list
/// of numbers, else the parameter kind.
///
/// # Arguments
///
/// * `kind` - The parameter kind, as [`param_kind`] sets it out.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The kind.
fn c_param_kind(kind: DataKind, platform: u16) -> DataKind
{
    ret_if!(kind.as_external().is_some(), DataKind::handle_pointer());
    let cells =
        c_list(kind, platform).and_then(|(k, _)| foreign::number_cells(k, platform));
    cells.map_or(kind, DataKind::pointer_to)
}

/// The argument C takes for a list of numbers: the address of its first
/// cell. A list C can write is made to own its cells first, so no duplicate
/// of it sees the writes; the cells remain in place for the call.
///
/// # Arguments
///
/// * `arg` - The lowered argument, of the parameter kind.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The address, or the argument as it is for another parameter.
fn c_list_arg(arg: Expr, platform: u16) -> Expr
{
    let Some((list, edit)) = c_list(arg.kind, platform)
    else
    {
        return arg;
    };
    let (kind, span) = (c_param_kind(arg.kind, platform), arg.span);
    let len = list.as_list().map(|l| l.len);
    let list = match (edit, list.is_seq())
    {
        (true, true) => own(through(arg)),
        _ => arg,
    };
    let usize = DataKind::USIZE.int_kind().expect("usize is an integer");
    let zero = ExprNode::Const(Value::int(usize, 0, platform));
    let index = node_expr(DataKind::USIZE, zero, span);
    #[rustfmt::skip]
    let node = ExprNode::CellAddr { list: Box::new(list), index: Box::new(index), len, first: 0, checked: true };
    node_expr(kind, node, span)
}

/// The function being lowered.
#[derive(Debug, Default)]
struct Frame
{
    /// Its slots so far.
    locals: Vec<Slot>,
    /// Its result kind: the kind of the result variable.
    ret: Option<DataKind>,
    /// What it gives back when it can fail.
    fault: Option<Fault>,
    /// The number of scopes open outside it, so its own scopes start at
    /// this index.
    depth: usize,
    /// The parameter slots the function can change: each holds a borrow of
    /// the caller's place, and each use of the name goes through it.
    edits: HashSet<u32>,
    /// The slots that hold the box of a variable a closure shares: each use
    /// of the name reads or writes the cell of the box.
    boxes: HashSet<u32>,
    /// The names of the locals that live in a box: a function declared
    /// inside this one names them.
    boxed: HashSet<Name>,
    /// Each parameter marked `ဒေတာ ပြောင်းလဲရန်` that a closure shares, as the
    /// slot of the caller's address and the box: the value in the box goes
    /// back to the caller at each exit.
    write_backs: Vec<(Place, Place)>,
    /// Whether the result is marked `အပိုင်အနေဖြင့်`: a return hands the block
    /// over to the caller.
    owned_ret: bool,
}

/// The lowering walk.
#[derive(Builder)]
struct Lowerer<'k>
{
    /// Kinds from the check.
    kinds: &'k KindTable,
    /// The functions declared inside functions whose blocks are being
    /// lowered, the outermost first.
    #[builder(default)]
    own_fns: Vec<Name>,
    /// The number of top-level functions, which take the first numbers.
    top_count: u32,
    /// The functions declared inside functions, and their thunks, numbered
    /// after the top-level ones; each is `None` while it is lowered.
    #[builder(default)]
    lifted: Vec<Option<Func>>,
    /// The functions declared inside functions in each open scope.
    #[builder(default)]
    nested: Vec<HashMap<Name, Nested>>,
    /// The kind of each declared struct, from the check.
    structs: &'k StructTable,
    /// Bit count of the platform-sized kinds.
    platform: u16,
    /// The signatures of the functions the current file declares or
    /// imports, one per name.
    sigs: HashMap<Name, Sig>,
    /// The signatures of the functions each file declares.
    unit_sigs: Vec<HashMap<Name, Sig>>,
    /// The top-level places of each file lowered so far.
    #[builder(default)]
    saved: Vec<HashMap<Name, Place>>,
    /// The unit of the file being lowered.
    #[builder(default)]
    unit: u32,
    /// What the names of the functions of the current file end with.
    #[builder(default)]
    suffix: String,
    /// Global slots so far.
    #[builder(default)]
    globals: Vec<Slot>,
    /// Lowered functions so far.
    #[builder(default)]
    funcs: Vec<Func>,
    /// The C functions met so far, one for each C name.
    #[builder(default)]
    foreign: Vec<ForeignFn>,
    /// The function being lowered; `None` at the top level.
    frame: Option<Frame>,
    /// Names in scope, innermost last.
    #[builder(default)]
    scopes: Vec<HashMap<Name, Place>>,
    /// The places of each open scope whose kind holds text, in the order
    /// of their declarations, innermost scope last; each is dropped as its
    /// scope ends.
    #[builder(default)]
    drops: Vec<Vec<(Place, DataKind)>>,
    /// The index of the scope of the block of each open loop, innermost
    /// last; `break` and `continue` leave the scopes from there on.
    #[builder(default)]
    loops: Vec<usize>,
    /// The start block, once lowered: its index, its parameter kinds, its
    /// result kind and the span of its header word.
    start: Option<(u32, Vec<DataKind>, Option<DataKind>, Span)>,
    /// Statements that run before the statement being lowered: the calls
    /// that pass a failure on, and the values an operand left of such a call
    /// computes first.
    #[builder(default)]
    pre: Vec<Stmt>,
    /// Statements that run before [`Self::pre`]: each hidden slot that holds
    /// the text of a call that can fail starts blank, so a drop of it at the
    /// end of its scope is sound when the call did not run.
    #[builder(default)]
    pre_init: Vec<Stmt>,
    /// Statements that run right after the call being lowered: each value a
    /// parameter marked `ဒေတာ ပြောင်းလဲရန်` changed, copied back to its place.
    #[builder(default)]
    post: Vec<Stmt>,
    /// The calls that pass a failure on in the statement being lowered that
    /// are still to lower.
    #[builder(default)]
    pending: usize,
    /// Whether an operand is being lowered as a whole into a hidden slot, so
    /// the parts inside it are left in place.
    #[builder(default)]
    spilling: bool,
}

impl Lowerer<'_>
{
    /// The kind the check recorded for an expression.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The kind.
    fn kind_of(&self, e: &ast::Expr) -> DataKind { self.shown_kind(e).repr() }

    /// The kind the check recorded for an expression, with an enumeration
    /// kept as it is, for a print that writes the name of a member.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The kind.
    fn shown_kind(&self, e: &ast::Expr) -> DataKind
    {
        let kind = self.kinds.get(&e.id()).copied();
        kind.unwrap_or_else(|| panic!("expression {} has no kind", e.id().0))
    }

    /// Make a slot for a declared name and bring the name into scope.
    ///
    /// # Arguments
    ///
    /// * `ident` - The name.
    /// * `kind` - Its kind.
    /// * `access` - What it allows.
    ///
    /// # Returns
    ///
    /// The place of the slot.
    fn declare(&mut self, ident: &ast::Ident, kind: DataKind, access: Access) -> Place
    {
        let kind = kind.repr();
        let slot = Slot::builder()
            .name(ident.name.as_str())
            .kind(kind)
            .access(access)
            .span(ident.span)
            .build();
        let place = match &mut self.frame
        {
            Some(frame) =>
            {
                frame.locals.push(slot);
                Place::Local(frame.locals.len() as u32 - 1)
            }
            None =>
            {
                self.globals.push(slot);
                Place::Global(self.globals.len() as u32 - 1)
            }
        };
        let scope = self.scopes.last_mut().expect("a scope is open");
        scope.insert(ident.name, place);
        let dropped = kind.holds_text() && ident.name != RESULT_NAME;
        let drops = self.drops.last_mut().expect("a scope is open");
        do_if!(dropped, drops.push((place, kind)));
        place
    }

    /// Make a slot that no name refers to, for a value headed into a
    /// place.
    ///
    /// # Arguments
    ///
    /// * `kind` - Its kind.
    /// * `span` - The source region it stands for.
    ///
    /// # Returns
    ///
    /// The place of the slot.
    fn hidden(&mut self, kind: DataKind, span: Span) -> Place
    {
        let kind = kind.repr();
        #[rustfmt::skip]
        let slot = Slot::builder().name("%new").kind(kind).access(Access::Var).span(span).build();
        let Some(frame) = &mut self.frame
        else
        {
            self.globals.push(slot);
            return Place::Global(self.globals.len() as u32 - 1);
        };
        frame.locals.push(slot);
        Place::Local(frame.locals.len() as u32 - 1)
    }

    /// The drops of the places of the open scopes from `depth` on, the
    /// innermost scope first and the latest place of each scope first.
    ///
    /// # Arguments
    ///
    /// * `depth` - The index of the outermost scope to leave.
    /// * `span` - The source region of the exit.
    ///
    /// # Returns
    ///
    /// The drop statements.
    fn drops_from(&self, depth: usize, span: Span) -> Vec<Stmt>
    {
        let places = self.drops.iter().skip(depth).rev().flat_map(|s| s.iter().rev());
        places.map(|&(place, kind)| drop_stmt(Dest::Slot(place), kind, span)).collect()
    }

    /// The statements that leave a function from scope `depth` on: each
    /// shared parameter marked `ဒေတာ ပြောင်းလဲရန်` copied back to the caller, then
    /// the drops.
    ///
    /// # Arguments
    ///
    /// * `depth` - The first scope of the function.
    /// * `span` - The source region of the exit.
    ///
    /// # Returns
    ///
    /// The statements.
    fn exit_drops(&mut self, depth: usize, span: Span) -> Vec<Stmt>
    {
        let backs =
            self.frame.as_ref().map(|f| f.write_backs.clone()).unwrap_or_default();
        let mut out = Vec::new();
        for (addr, boxed) in backs
        {
            let cell = self.edit_addr(boxed, span);
            let kind = cell.kind.pointee().expect("a cell address points");
            let value = node_expr(kind, ExprNode::Load { addr: Box::new(cell) }, span);
            let value = tern!(kind.holds_text(), dup(value), value);
            let caller = read(addr, self.slot_kind(addr), span);
            let dest = Dest::Mem(caller.clone());
            let value =
                tern!(kind.holds_text(), self.replace(&dest, value, &mut out), value);
            out.push(Stmt::Store { addr: caller, value, span });
        }
        out.extend(self.drops_from(depth, span));
        out
    }

    /// The drops of each global whose kind holds text, the latest first, for
    /// the end of the program.
    ///
    /// # Arguments
    ///
    /// * `span` - The source region of the end.
    ///
    /// # Returns
    ///
    /// The drop statements.
    fn global_drops(&self, span: Span) -> Vec<Stmt>
    {
        let globals = self.globals.iter().enumerate().rev();
        let held = globals.filter(|(_, s)| s.kind.holds_text() && s.name != "%new");
        held.map(|(i, s)| drop_stmt(Dest::Slot(Place::Global(i as u32)), s.kind, span))
            .collect()
    }

    /// The place a name refers to.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The place of the innermost declaration.
    ///
    /// # Panics
    ///
    /// When the name is not declared, which a clean check rules out.
    fn lookup(&self, name: &str) -> Place
    {
        let found = self.scopes.iter().rev().find_map(|s| s.get(name).copied());
        found.unwrap_or_else(|| panic!("name {name} is not declared"))
    }

    /// Run `f` inside a new scope.
    ///
    /// # Arguments
    ///
    /// * `f` - The work.
    fn scoped(&mut self, f: impl FnOnce(&mut Self))
    {
        self.scopes.push(HashMap::new());
        self.nested.push(HashMap::new());
        self.drops.push(Vec::new());
        f(self);
        self.drops.pop();
        self.nested.pop();
        self.scopes.pop();
    }

    /// Lower statements into `out` inside a new scope, then drop the places
    /// of that scope unless the last statement leaves it.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    /// * `out` - The list the statements go to.
    fn scoped_stmts(&mut self, stmts: &[st::Stmt], out: &mut Vec<Stmt>)
    {
        self.scoped(|l| {
            let start = out.len();
            do_for!(s in stmts, l.stmt(s, out));
            let leaves = out.len() > start && out.last().is_some_and(leaves_scope);
            let end = stmts.last().map_or(Span::default(), |s| s.span());
            do_if!(!leaves, out.extend(l.drops_from(l.drops.len() - 1, end)));
        });
    }

    /// Lower the block of a loop in its own scope.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    ///
    /// # Returns
    ///
    /// The lowered statements.
    fn loop_block(&mut self, stmts: &[st::Stmt]) -> Vec<Stmt>
    {
        self.loops.push(self.scopes.len());
        let block = self.block(stmts);
        self.loops.pop();
        block
    }

    /// Lower a statement list into its own list, inside a new scope.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    ///
    /// # Returns
    ///
    /// The lowered statements.
    fn block(&mut self, stmts: &[st::Stmt]) -> Vec<Stmt>
    {
        let mut out = Vec::new();
        self.scoped_stmts(stmts, &mut out);
        out
    }

    /// Move to a file: keep the top-level places of the files that ended,
    /// and start with a blank scope and the functions of the new file.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit of the new file.
    fn enter_unit(&mut self, unit: u32)
    {
        ret_if!(unit == self.unit);
        while self.saved.len() < unit as usize
        {
            let top = std::mem::take(&mut self.scopes).into_iter().next();
            self.saved.push(top.unwrap_or_default());
            self.scopes.push(HashMap::new());
            self.drops = vec![Vec::new()];
        }
        self.sigs = self.unit_sigs[unit as usize].clone();
        self.unit = unit;
    }

    /// Bring the names of an import into scope: the functions and the
    /// places of the imported file that the import names.
    ///
    /// # Arguments
    ///
    /// * `n` - The import.
    fn import(&mut self, n: &st::ImportStmt)
    {
        let Some(target) = n.unit.map(|u| u as usize)
        else
        {
            return;
        };
        for name in n.names.iter().map(|n| n.name)
        {
            let sig = self.unit_sigs[target].get(&name).cloned();
            do_if!(let Some(sig) = sig, _ = self.sigs.insert(name, sig));
            let place = self.saved.get(target).and_then(|s| s.get(&name)).copied();
            let scope = self.scopes.last_mut().expect("a scope is open");
            do_if!(let Some(place) = place, _ = scope.insert(name, place));
        }
    }

    /// Lower a top-level statement: a function goes to the module, the rest
    /// to the main list.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    /// * `out` - The main list.
    fn top_stmt(&mut self, s: &st::Stmt, out: &mut Vec<Stmt>)
    {
        let st::Stmt::Fn(f) = s
        else
        {
            return self.stmt(s, out);
        };
        // A generic function runs through its instances, which the check added.
        ret_if!(is_generic(f));
        let params = f.params.iter().map(|p| param_kind(p, self.structs)).collect();
        let ret = f.ret.as_ref().map(|k| DataKind::from_ast(k, self.structs).repr());
        let index = self.funcs.len() as u32;
        do_if!(f.start, self.start = Some((index, params, ret, f.name.span)));
        let func = self.func(f, None, &[]);
        self.funcs.push(func);
    }

    /// Call the start block at the end of the top-level statements. Its
    /// parameters get duplicates of hidden globals that the VM fills with the
    /// arguments and the environment of the run, and its result goes to a
    /// hidden global that the VM reads as the exit code.
    ///
    /// The call and the hidden globals take the span of the header word
    /// `ပရိုဂရမ်`, so a run that stops at the call points at the header.
    ///
    /// # Arguments
    ///
    /// * `out` - The main list.
    ///
    /// # Returns
    ///
    /// The hidden globals; none when the program has no start block.
    fn start_call(&mut self, out: &mut Vec<Stmt>) -> Host
    {
        let Some((func, params, ret, span)) = self.start.take()
        else
        {
            return Host::default();
        };
        let names = params.iter().zip(["%args", "%env"]);
        let inputs: Vec<u32> =
            names.map(|(&k, n)| self.host_global(n, k, span)).collect();
        let read = |(&g, &kind): (&u32, &DataKind)| {
            let node = ExprNode::Read(Place::Global(g));
            dup(Expr::builder().kind(kind).node(node).span(span).build())
        };
        let args = inputs.iter().zip(&params).map(read).collect();
        let name = START_NAME.to_string();
        let exit = ret.map(|kind| (self.host_global("%exit", kind, span), kind));
        let stmt = match exit
        {
            Some((g, kind)) =>
            {
                let call = ExprNode::Call { func, name, args };
                let value = Expr::builder().kind(kind).node(call).span(span).build();
                Stmt::Set { place: Place::Global(g), value, span }
            }
            None => Stmt::Call { func, name, args, span },
        };
        out.push(stmt);
        #[rustfmt::skip]
        return Host::builder().maybe_args(inputs.first().copied())
            .maybe_env(inputs.get(1).copied()).maybe_exit(exit.map(|(g, _)| g)).build();
    }

    /// Make a hidden global that links the start block to the host.
    ///
    /// # Arguments
    ///
    /// * `name` - Its name, which starts with `%`.
    /// * `kind` - Its kind.
    /// * `span` - The source region of the program.
    ///
    /// # Returns
    ///
    /// The index of the global.
    fn host_global(&mut self, name: &str, kind: DataKind, span: Span) -> u32
    {
        #[rustfmt::skip]
        let slot = Slot::builder().name(name).kind(kind).access(Access::Var).span(span).build();
        self.globals.push(slot);
        self.globals.len() as u32 - 1
    }

    /// Lower a statement and append it to `out`, after the statements its
    /// calls that pass a failure on run first.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    /// * `out` - The list it goes to.
    fn stmt(&mut self, s: &st::Stmt, out: &mut Vec<Stmt>)
    {
        let pre = std::mem::take(&mut self.pre);
        let init = std::mem::take(&mut self.pre_init);
        let pending = std::mem::replace(&mut self.pending, stmt_passes(s));
        let mut parts = Vec::new();
        self.stmt_parts(s, &mut parts);
        out.append(&mut self.pre_init);
        out.append(&mut self.pre);
        out.append(&mut parts);
        (self.pre, self.pre_init, self.pending) = (pre, init, pending);
    }

    /// Lower the statements of a value into their own list, which the
    /// caller places where the value runs: a condition that runs when an
    /// earlier test lets it, and at no other time.
    ///
    /// # Arguments
    ///
    /// * `f` - The work, which lowers the value.
    ///
    /// # Returns
    ///
    /// The statements, and what the work gives.
    fn with_pre<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> (Vec<Stmt>, T)
    {
        let outer = std::mem::take(&mut self.pre);
        let value = f(self);
        (std::mem::replace(&mut self.pre, outer), value)
    }

    /// Lower a statement and append its parts to `out`.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    /// * `out` - The list it goes to.
    fn stmt_parts(&mut self, s: &st::Stmt, out: &mut Vec<Stmt>)
    {
        match s
        {
            st::Stmt::Decl(n) => out.push(self.decl(n)),
            st::Stmt::Assign(n) => self.assign(n, out),
            st::Stmt::Print(n) => out.push(self.print(n)),
            st::Stmt::Block(n) => self.scoped_stmts(&n.stmts, out),
            st::Stmt::If(n) => out.push(self.if_stmt(n)),
            st::Stmt::While(n) => out.push(self.while_stmt(n)),
            st::Stmt::Counted(n) => out.push(self.counted(n)),
            st::Stmt::Generic(n) => self.generic(n, out),
            st::Stmt::Each(n) => self.each(n, out),
            st::Stmt::Break(n) =>
            {
                out.extend(self.loop_drops(n.span));
                out.push(Stmt::Break { span: n.span });
            }
            st::Stmt::Continue(n) =>
            {
                out.extend(self.loop_drops(n.span));
                out.push(Stmt::Continue { span: n.span });
            }
            st::Stmt::Fn(f) => self.nested_fn(f, out),
            st::Stmt::Return(n) if n.fault => self.fault_return(n, out),
            st::Stmt::Return(n) => self.return_stmt(n, out),
            st::Stmt::Call(n) if n.pass =>
            {
                self.pending = 0;
                self.checked_call(&n.callee, n.place.as_deref(), &n.args, n.span);
            }
            st::Stmt::Call(n) =>
            {
                let (target, sig) = self.call_target(&n.callee, n.place.as_deref());
                let args = self.lower_args(&n.args, &sig);
                out.push(target.stmt(args, n.span));
                out.append(&mut self.post);
            }
            st::Stmt::Alloc(n) => out.push(self.alloc(n)),
            st::Stmt::Resize(n) => out.push(self.resize(n)),
            st::Stmt::Free(n) =>
            {
                out.push(Stmt::Free { dest: self.dest(&n.target), span: n.span })
            }
            st::Stmt::Leave(n) =>
            {
                let value = self.expr(&n.value);
                let place =
                    matches!(value.node, ExprNode::Read(_) | ExprNode::Load { .. });
                let value = tern!(place, value, self.spill(value));
                out.push(Stmt::Leave { value, grant: n.grant, span: n.span });
            }
            st::Stmt::Blit(n) => out.push(self.blit(n)),
            st::Stmt::Fill(n) => out.push(self.fill(n)),
            st::Stmt::Add(n) => out.push(self.add_cell(n)),
            st::Stmt::Remove(n) if n.row => out.push(self.remove_row(n)),
            st::Stmt::Remove(n) =>
            {
                let kind = self.kind_of(&n.list);
                let dest = self.dest(&n.list);
                out.push(Stmt::RemoveCell { dest, kind, span: n.span })
            }
            st::Stmt::Replace(n) => out.push(self.replace_text(n)),
            st::Stmt::Struct(_)
            | st::Stmt::Enum(_)
            | st::Stmt::Alias(_)
            | st::Stmt::Trait(_)
            | st::Stmt::TraitBlock(_) => (),
            st::Stmt::Import(n) => self.import(n),
            st::Stmt::Switch(n) => self.switch_stmt(n, out),
        }
    }

    /// Lower a declaration to a store of its initial value or its kind's
    /// default. The name comes into scope after the value.
    ///
    /// # Arguments
    ///
    /// * `n` - The declaration.
    ///
    /// # Returns
    ///
    /// The store.
    fn decl(&mut self, n: &st::DeclStmt) -> Stmt
    {
        let kind = self.declared_kind(n.kind.as_ref(), n.init.as_ref());
        let value = self.initial(n.init.as_ref(), kind, n.name.span);
        let access = tern!(n.is_const, Access::Const, Access::Var);
        // A local a closure shares lives in a box, so it outlives this call.
        if self.is_boxed_name(&n.name.name)
        {
            let place = self.declare(&n.name, DataKind::box_of(kind), access);
            self.mark_box(place);
            do_if!(n.init.is_none(), self.slot(place).unset = Some(n.span));
            return Stmt::Set {
                place,
                value: self.make_box(value, n.span),
                span: n.span,
            };
        }
        let place = self.declare(&n.name, kind, access);
        do_if!(n.init.is_none(), self.slot(place).unset = Some(n.span));
        Stmt::Set { place, value, span: n.span }
    }

    /// The kind of a declared name: the written kind, or the kind of its
    /// value when no kind is written.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind phrase, when written.
    /// * `init` - The value, when one is written.
    ///
    /// # Returns
    ///
    /// The kind.
    ///
    /// # Panics
    ///
    /// When neither is written, which the parser rules out.
    fn declared_kind(
        &self, written: Option<&Kind>, init: Option<&ast::Expr>,
    ) -> DataKind
    {
        let Some(written) = written
        else
        {
            return concrete(self.kind_of(init.expect("a value gives the kind")));
        };
        DataKind::from_ast(written, self.structs)
    }

    /// The initial value of a declaration.
    ///
    /// # Arguments
    ///
    /// * `init` - The written value, when there is one.
    /// * `kind` - The declared kind.
    /// * `span` - Where to place a default value.
    ///
    /// # Returns
    ///
    /// The value, of the declared kind.
    fn initial(&mut self, init: Option<&ast::Expr>, kind: DataKind, span: Span)
    -> Expr
    {
        ret_if!(let Some(init) = init, self.owned(init, kind));
        default_constant(kind, span)
    }

    /// The initial value of a kind, `ကနဦးတန်ဖိုး`: zero, `'\0'`, false,
    /// a pointer that points at nothing, text and plain lists with no cells,
    /// the blank optional, the marked member of an enumeration, each cell and
    /// field at its own, and a new object for a shared struct. The check made
    /// sure the kind has one.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind, an enumeration kept as it is.
    /// * `span` - The value.
    ///
    /// # Returns
    ///
    /// The value.
    fn initial_value(&self, kind: DataKind, span: Span) -> Expr
    {
        if kind.is_optional()
        {
            let none = Fault::of_kind(kind).expect("an optional is a fault kind");
            return none.value(true, None, None, self.platform, span);
        }
        if let Some(e) = kind.as_enum()
        {
            let at = e.initial().expect("the check found the initial member");
            return member_expr(&e.members()[at].1, e.under(), span);
        }
        if let Some(shape) = kind.as_object()
        {
            let fields = self.initial_value(DataKind::Struct(shape), span);
            return self.make_box(fields, span);
        }
        // A blank of zero octets needs no value for each part, and a long list
        // would make one for each of its cells.
        ret_if!(zero_initial(kind), default_constant(kind, span));
        let parts: Option<Vec<DataKind>> = match (kind.as_struct(), kind.as_list())
        {
            (Some(s), _) => Some(s.fields().iter().map(|(_, k)| *k).collect()),
            (_, Some(l)) => Some(vec![l.cell; l.len as usize]),
            _ => None,
        };
        let Some(parts) = parts
        else
        {
            return default_constant(kind, span);
        };
        let cells = parts.into_iter().map(|k| self.initial_value(k, span)).collect();
        node_expr(kind.repr(), ExprNode::List(cells), span)
    }

    /// Lower an assignment: a store to a slot, or a store at an address
    /// for a dereference target. A place whose kind holds text gets the new
    /// value in a hidden slot first, which can read the old value; then the
    /// old value is dropped and the new one moves in.
    ///
    /// # Arguments
    ///
    /// * `n` - The assignment.
    /// * `out` - The list the statements go to.
    fn assign(&mut self, n: &st::AssignStmt, out: &mut Vec<Stmt>)
    {
        let span = n.span;
        if let Some((text, index, first)) = char_place(&n.target)
        {
            let (kind, index, value) =
                (self.kind_of(text), self.expr(index), self.expr(&n.value));
            let dest = self.dest(text);
            return out.push(Stmt::SetChar { dest, kind, index, first, value, span });
        }
        if let ast::Expr::Field(f) = &n.target
            && self.kind_of(&f.base).as_external().is_some()
        {
            return self.c_field_store(f, &n.value, out);
        }
        if let Some(row) = row_place(&n.target)
        {
            return out.push(self.set_row(row, &n.value, span));
        }
        let dest = self.dest(&n.target);
        let want = match &dest
        {
            Dest::Slot(place) => self.slot_kind(*place),
            Dest::Mem(_) => self.kind_of(&n.target),
        };
        let value = self.owned(&n.value, want);
        let value = tern!(want.holds_text(), self.replace(&dest, value, out), value);
        // A call can move the block an address points into, so the value runs
        // first and the address after it.
        let held = matches!(dest, Dest::Mem(_)) && !want.holds_text();
        let value = tern!(held && has_call(&n.value), self.hold(value, out), value);
        out.push(match dest
        {
            Dest::Slot(place) => Stmt::Set { place, value, span },
            Dest::Mem(addr) => Stmt::Store { addr, value, span },
        });
    }

    /// Keep a new value in a hidden slot and drop the old value of a place.
    ///
    /// # Arguments
    ///
    /// * `dest` - The place.
    /// * `value` - The new value, of a kind that holds text.
    /// * `out` - The list the statements go to.
    ///
    /// # Returns
    ///
    /// The new value, moved out of the hidden slot.
    fn replace(&mut self, dest: &Dest, value: Expr, out: &mut Vec<Stmt>) -> Expr
    {
        let (kind, span) = (value.kind, value.span);
        let new = self.hidden(kind, span);
        out.push(Stmt::Set { place: new, value, span });
        out.push(drop_stmt(dest.clone(), kind, span));
        let read =
            Expr::builder().kind(kind).node(ExprNode::Read(new)).span(span).build();
        take(read)
    }

    /// Keep a value with no heap block in a hidden slot, so it is computed
    /// before the statement that reads it.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    /// * `out` - The list the statements go to.
    ///
    /// # Returns
    ///
    /// The read of the hidden slot.
    fn hold(&mut self, value: Expr, out: &mut Vec<Stmt>) -> Expr
    {
        let (kind, span) = (value.kind, value.span);
        let new = self.hidden(kind, span);
        out.push(Stmt::Set { place: new, value, span });
        read(new, kind, span)
    }

    /// Lower a place, with spilling off: a place spilled to a hidden duplicate
    /// would take the store, and the place itself would keep its old value.
    ///
    /// # Arguments
    ///
    /// * `e` - The place.
    ///
    /// # Returns
    ///
    /// The destination.
    fn dest(&mut self, e: &ast::Expr) -> Dest
    {
        let was = std::mem::replace(&mut self.spilling, true);
        let dest = self.place_dest(e);
        self.spilling = was;
        dest
    }

    /// Lower a place: a name to its slot, a dereference to the address it
    /// reads.
    ///
    /// # Arguments
    ///
    /// * `e` - The place.
    ///
    /// # Returns
    ///
    /// The slot or the address.
    ///
    /// # Panics
    ///
    /// When the expression is not a place, which a clean check rules out.
    fn place_dest(&mut self, e: &ast::Expr) -> Dest
    {
        ret_if!(let ast::Expr::Group(n) = e, self.place_dest(&n.inner));
        ret_if!(let Some(ptr) = deref_operand(e), Dest::Mem(self.expr(ptr)));
        ret_if!(let ast::Expr::Index(n) = e, Dest::Mem(owned_path(self.cell_addr(n))));
        ret_if!(let ast::Expr::Field(n) = e, Dest::Mem(owned_path(self.field_addr(n))));
        ret_if!(let Some(row) = row_place(e), Dest::Mem(owned_path(self.row_addr(row))));
        let ast::Expr::Name(n) = e
        else
        {
            panic!("expression {} is not a place", e.id().0);
        };
        let place = self.lookup(&n.name);
        ret_if!(self.is_edit(place), Dest::Mem(self.edit_addr(place, n.span)));
        Dest::Slot(place)
    }

    /// Lower a heap allocation.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    fn alloc(&mut self, n: &st::AllocStmt) -> Stmt
    {
        let (count, unit) = self.size(&n.size);
        let dest = self.dest(&n.target);
        Stmt::Alloc { dest, count, unit, span: n.span }
    }

    /// Lower an add statement: the value of a new cell is owned, as an
    /// assigned one is; text added to text is read where it is.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    fn add_cell(&mut self, n: &st::AddStmt) -> Stmt
    {
        ret_if!(n.before, self.insert(&n.list, &n.value, n.span));
        let kind = self.kind_of(&n.list);
        if kind.is_text()
        {
            let value = self.expr(&n.value);
            let dest = self.dest(&n.list);
            return Stmt::AddText { dest, kind, value, span: n.span };
        }
        let cell = kind.cell_kind().expect("an add statement grows a list");
        let value = self.owned(&n.value, cell);
        let dest = self.dest(&n.list);
        Stmt::AddCell { dest, kind, value, span: n.span }
    }

    /// Lower an add that puts its value in front of a cell of a plain list
    /// or a character of text. A new cell is owned, as for an add at the
    /// end; added text is read where it is.
    ///
    /// # Arguments
    ///
    /// * `at` - The cell or the character.
    /// * `value` - The value added.
    /// * `span` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    fn insert(&mut self, at: &ast::Expr, value: &ast::Expr, span: Span) -> Stmt
    {
        if let Some((text, index, first)) = char_place(at)
        {
            let (kind, index, value) =
                (self.kind_of(text), self.expr(index), self.expr(value));
            let dest = self.dest(text);
            return Stmt::Insert { dest, kind, index, first, value, span };
        }
        extract!(ast::Expr::Index(cell) = ungrouped(at), "an insert names a cell");
        let kind = self.kind_of(&cell.list);
        let cells = kind.cell_kind().expect("an insert grows a list");
        let (index, value) = (self.expr(&cell.index), self.owned(value, cells));
        let dest = self.dest(&cell.list);
        Stmt::Insert { dest, kind, index, first: cell.first, value, span }
    }

    /// Lower a replace of a range of text. The value is read where it is, as
    /// for an add to text.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    fn replace_text(&mut self, n: &st::ReplaceStmt) -> Stmt
    {
        let kind = self.kind_of(&n.text);
        let (start, end) = (self.expr(&n.start), self.expr(&n.end));
        let value = self.expr(&n.value);
        let dest = self.dest(&n.text);
        #[rustfmt::skip]
        return Stmt::Replace { dest, kind, start, end, chars: n.chars, value, span: n.span };
    }

    /// Lower a heap reallocation.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    fn resize(&mut self, n: &st::ResizeStmt) -> Stmt
    {
        let dest = self.dest(&n.target);
        let (count, unit) = self.size(&n.size);
        Stmt::Resize { dest, count, unit, span: n.span }
    }

    /// Lower a blit of cells.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    fn blit(&mut self, n: &st::BlitStmt) -> Stmt
    {
        let src = self.expr(&n.src);
        let (count, unit) = self.size(&n.size);
        let dst = self.expr(&n.dst);
        Stmt::Blit { src, dst, count, unit, overlap: n.overlap, span: n.span }
    }

    /// Lower a fill of cells; a size in octets fills `u8` cells.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    fn fill(&mut self, n: &st::FillStmt) -> Stmt
    {
        let (count, unit) = self.size(&n.size);
        let dst = self.expr(&n.dst);
        let value = self.coerce(&n.value, unit.unwrap_or(DataKind::U8));
        Stmt::Fill { dst, count, value, span: n.span }
    }

    /// Lower the size operand of a heap statement.
    ///
    /// # Arguments
    ///
    /// * `size` - The operand.
    ///
    /// # Returns
    ///
    /// The count, as a `usize`, and the element kind when written.
    fn size(&mut self, size: &st::SizeArg) -> (Expr, Option<DataKind>)
    {
        let count = self.coerce(&size.count, DataKind::USIZE);
        (count, size.unit.as_ref().map(|k| DataKind::from_ast(k, self.structs)))
    }

    /// The kind of a slot.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// Its kind.
    fn slot_kind(&mut self, place: Place) -> DataKind { self.slot(place).kind }

    /// The slot at a place.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    ///
    /// # Returns
    ///
    /// The slot.
    fn slot(&mut self, place: Place) -> &mut Slot
    {
        match place
        {
            Place::Global(n) => &mut self.globals[n as usize],
            Place::Local(n) =>
            {
                let frame = self.frame.as_mut().expect("a local is inside a function");
                &mut frame.locals[n as usize]
            }
        }
    }

    /// Lower a print statement.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    fn print(&mut self, n: &st::PrintStmt) -> Stmt
    {
        let plain = tern!(let ast::Expr::Str(s) = &n.value, s.plain(), None);
        let value = plain.map_or_else(
            || {
                let lowered = counted_literal(self.expr(&n.value));
                PrintValue::Expr(self.shown(&n.value, lowered))
            },
            |text| PrintValue::Text(text.to_string()),
        );
        Stmt::Print { value, span: n.span }
    }

    /// A lowered value labelled with its enumeration kind, so a print writes
    /// the name of the member; each other value as it is. Codegen reads the
    /// value as its value kind.
    ///
    /// # Arguments
    ///
    /// * `e` - The written value.
    /// * `lowered` - The lowered value.
    ///
    /// # Returns
    ///
    /// The value.
    fn shown(&self, e: &ast::Expr, lowered: Expr) -> Expr
    {
        let kind = self.shown_kind(e);
        ret_if!(kind.as_enum().is_none(), lowered);
        Expr::builder().kind(kind).node(lowered.node).span(lowered.span).build()
    }

    /// Lower an `if` chain; later arms nest in the `else` of earlier ones.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The outermost `if`.
    fn if_stmt(&mut self, n: &st::IfStmt) -> Stmt
    {
        // The condition of a later arm runs inside the `else` of the arm
        // before it, so the statements of its calls go there too.
        let arms: Vec<ChainArm> = n
            .arms
            .iter()
            .enumerate()
            .map(|(i, arm)| {
                tern!(
                    i == 0,
                    (Vec::new(), self.if_arm(arm)),
                    self.with_pre(|l| l.if_arm(arm))
                )
            })
            .collect();
        let els = n.else_block.as_ref().map_or_else(Vec::new, |b| self.block(&b.stmts));
        if_chain(arms, els, n.span)
    }

    /// Lower a switch to the `if` chain an `if` statement lowers to: each arm
    /// tests the subject with `==` against its case, and the default arm is
    /// the last `else`. A subject that is no read of a place or a constant
    /// is kept in a hidden slot first, so it runs once.
    ///
    /// # Arguments
    ///
    /// * `n` - The switch.
    /// * `out` - The list the statements go to.
    fn switch_stmt(&mut self, n: &st::SwitchStmt, out: &mut Vec<Stmt>)
    {
        let owner = self.shown_kind(&n.subject).as_enum();
        let lowered = self.expr(&n.subject);
        let subject = self.switch_subject(lowered, out);
        let arms: Vec<ChainArm> = n
            .arms
            .iter()
            .enumerate()
            .map(|(i, arm)| {
                let lower = |l: &mut Self| {
                    let cond = l.arm_cond(&arm.cases, &subject, owner);
                    (cond, l.block(&arm.block.stmts), arm.span)
                };
                tern!(i == 0, (Vec::new(), lower(self)), self.with_pre(lower))
            })
            .collect();
        let els =
            n.default.as_ref().map_or_else(Vec::new, |d| self.block(&d.block.stmts));
        out.extend(std::mem::take(&mut self.pre));
        out.push(if_chain(arms, els, n.span));
    }

    /// The subject of a switch as each arm reads it: a read of a place or a
    /// constant as it is, each other value through a hidden slot set once.
    /// The slot of a value that holds text joins the drops of its scope.
    ///
    /// # Arguments
    ///
    /// * `lowered` - The lowered subject.
    /// * `out` - The list the statement that sets the slot goes to.
    ///
    /// # Returns
    ///
    /// The subject.
    fn switch_subject(&mut self, lowered: Expr, out: &mut Vec<Stmt>) -> Expr
    {
        let kept = matches!(lowered.node, ExprNode::Read(_) | ExprNode::Const(_));
        ret_if!(kept, lowered);
        let (kind, span, place) =
            (lowered.kind, lowered.span, self.hidden(lowered.kind, lowered.span));
        if kind.holds_text()
        {
            self.pre_init.push(Stmt::Set {
                place,
                value: default_constant(kind, span),
                span,
            });
            self.drops.last_mut().expect("a scope is open").push((place, kind));
        }
        out.extend(std::mem::take(&mut self.pre));
        out.push(Stmt::Set { place, value: lowered, span });
        read(place, kind, span)
    }

    /// The test of one switch arm: the subject `==` one of its cases.
    ///
    /// # Arguments
    ///
    /// * `cases` - The cases.
    /// * `subject` - The subject as each arm reads it.
    /// * `owner` - The enumeration of the subject, when it has one.
    ///
    /// # Returns
    ///
    /// The test, a boolean.
    fn arm_cond(
        &mut self, cases: &[ast::Expr], subject: &Expr, owner: Option<&EnumKind>,
    ) -> Expr
    {
        let mut tests = cases.iter().map(|c| self.case_cond(c, subject, owner));
        let first = tests.next().expect("an arm has a case");
        let tests: Vec<Expr> = tests.collect();
        tests.into_iter().fold(first, |lhs, rhs| {
            let span = lhs.span.to(&rhs.span);
            let node = ExprNode::Logic {
                op: LogicOp::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
            Expr::builder().kind(DataKind::Bool).node(node).span(span).build()
        })
    }

    /// The test of one case: the subject `==` the case. A bare name that is
    /// a member of the enumeration of the subject is that member.
    ///
    /// # Arguments
    ///
    /// * `case` - The case.
    /// * `subject` - The subject as each arm reads it.
    /// * `owner` - The enumeration of the subject, when it has one.
    ///
    /// # Returns
    ///
    /// The test, a boolean.
    fn case_cond(
        &mut self, case: &ast::Expr, subject: &Expr, owner: Option<&EnumKind>,
    ) -> Expr
    {
        let span = case.span();
        let member = match (case, owner)
        {
            (ast::Expr::Name(n), Some(owner)) =>
            {
                owner.member(&n.name).map(|v| (v, owner))
            }
            _ => None,
        };
        let case = match member
        {
            Some((v, owner)) => member_expr(v, owner.under(), span),
            None => self.expr(case),
        };
        let texts = subject.kind.is_text() && case.kind.is_text();
        let common =
            tern!(texts, None, Some(concrete(self.common(subject.kind, case.kind))));
        let p = self.platform;
        let side = |e: Expr| common.map_or(e.clone(), |k| coerce_lowered(e, k, p));
        let (lhs, rhs) = (Box::new(side(subject.clone())), Box::new(side(case)));
        let node = ExprNode::Compare { op: CmpOp::Eq, lhs, rhs };
        Expr::builder().kind(DataKind::Bool).node(node).span(span).build()
    }

    /// Lower one arm of an `if` chain.
    ///
    /// # Arguments
    ///
    /// * `arm` - The arm.
    ///
    /// # Returns
    ///
    /// The condition, the statements and the arm's source region.
    fn if_arm(&mut self, arm: &st::IfArm) -> (Expr, Vec<Stmt>, Span)
    {
        let cond = self.coerce(&arm.cond, DataKind::Bool);
        (cond, self.block(&arm.block.stmts), arm.span)
    }

    /// Lower a while loop.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    ///
    /// # Returns
    ///
    /// A loop with no start or update statements.
    fn while_stmt(&mut self, n: &st::WhileStmt) -> Stmt
    {
        let (cond, block) = self.loop_parts(&n.cond, &n.block.stmts, n.span);
        Stmt::Loop { init: Vec::new(), cond, update: Vec::new(), block, span: n.span }
    }

    /// The test and the round of a loop. A test that needs statements first,
    /// such as a call that copies values back, runs at the start of each
    /// round, which it leaves when the test fails.
    ///
    /// # Arguments
    ///
    /// * `cond` - The test.
    /// * `stmts` - The statements of the loop.
    /// * `span` - The loop.
    ///
    /// # Returns
    ///
    /// The test of the loop and the statements of a round.
    fn loop_parts(
        &mut self, cond: &ast::Expr, stmts: &[st::Stmt], span: Span,
    ) -> (Expr, Vec<Stmt>)
    {
        ret_if!(
            has_pass(cond),
            (true_expr(span), self.passing_round(cond, stmts, span))
        );
        let (pre, test) = self.with_pre(|l| l.coerce(cond, DataKind::Bool));
        let block = self.loop_block(stmts);
        ret_if!(pre.is_empty(), (test, block));
        let stop = node_expr(DataKind::Bool, prefix_node(PrefixOp::Not, test), span);
        let leave = vec![Stmt::Break { span }];
        let mut round = pre;
        round.push(Stmt::If { cond: stop, then: leave, els: Vec::new(), span });
        round.extend(block);
        (true_expr(span), round)
    }

    /// Lower the round of a loop whose condition passes a failure on: the
    /// condition runs at the start of each round, in the scope of the round,
    /// and leaves the loop when it does not hold.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    /// * `stmts` - The loop statements.
    /// * `span` - The loop.
    ///
    /// # Returns
    ///
    /// The statements of a round.
    fn passing_round(
        &mut self, cond: &ast::Expr, stmts: &[st::Stmt], span: Span,
    ) -> Vec<Stmt>
    {
        self.loops.push(self.scopes.len());
        let mut round = Vec::new();
        self.scoped(|l| {
            let (pre, test) = l.with_pre(|l| l.coerce(cond, DataKind::Bool));
            round.extend(pre);
            let mut leave = l.drops_from(l.drops.len() - 1, span);
            leave.push(Stmt::Break { span });
            let stop =
                node_expr(DataKind::Bool, prefix_node(PrefixOp::Not, test), span);
            round.push(Stmt::If { cond: stop, then: leave, els: Vec::new(), span });
            l.scoped_stmts(stmts, &mut round);
            let leaves = round.last().is_some_and(leaves_scope);
            do_if!(!leaves, round.extend(l.drops_from(l.drops.len() - 1, span)));
        });
        self.loops.pop();
        round
    }

    /// Lower a counted loop. Its variable takes the common kind of the
    /// bounds.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    ///
    /// # Returns
    ///
    /// The loop.
    fn counted(&mut self, n: &st::CountedStmt) -> Stmt
    {
        let kind = self.common(self.kind_of(&n.from), self.kind_of(&n.to));
        let from = self.coerce(&n.from, kind);
        let to = self.coerce(&n.to, kind);
        let step = n.step.as_ref().map(|s| self.coerce(s, kind));
        let mut var = Place::Local(0);
        let mut block = Vec::new();
        self.scoped(|l| {
            var = l.declare(&n.var, kind, Access::LoopVar);
            block = l.counted_block(n, var, kind);
        });
        Stmt::Counted { var, from, to, step, mode: n.mode, block, span: n.span }
    }

    /// Lower the block of a counted loop. When a closure in it names the
    /// counter, each round gives the counter a box of its own first, so each
    /// closure keeps the number of its round.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `var` - The slot of the counter.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The lowered statements of a round.
    fn counted_block(
        &mut self, n: &st::CountedStmt, var: Place, kind: DataKind,
    ) -> Vec<Stmt>
    {
        ret_if!(!self.is_boxed_name(&n.var.name), self.loop_block(&n.block.stmts));
        self.loops.push(self.scopes.len());
        let mut out = Vec::new();
        self.scoped(|l| {
            l.rebox(&n.var, var, kind, &mut out);
            do_for!(s in &n.block.stmts, l.stmt(s, &mut out));
            let leaves = out.last().is_some_and(leaves_scope);
            do_if!(!leaves, out.extend(l.drops_from(l.drops.len() - 1, n.block.span)));
        });
        self.loops.pop();
        out
    }

    /// Lower a generic loop: its declaration becomes the start statement.
    /// A loop variable that holds text is dropped after the loop.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `out` - The list the statements go to.
    fn generic(&mut self, n: &st::GenericStmt, out: &mut Vec<Stmt>)
    {
        let kind = self.declared_kind(n.kind.as_ref(), n.init.as_ref());
        let value = self.initial(n.init.as_ref(), kind, n.var.span);
        let written = n.kind.as_ref().map(|k| k.span);
        let head = written.or_else(|| n.init.as_ref().map(ast::Expr::span));
        let mut parts = None;
        self.scoped(|l| {
            // A closure shares the variable of a generic loop across rounds.
            let boxed = l.is_boxed_name(&n.var.name);
            let place = l.declare(
                &n.var,
                tern!(boxed, DataKind::box_of(kind), kind),
                Access::Var,
            );
            do_if!(boxed, l.mark_box(place));
            let value = tern!(boxed, l.make_box(value, n.var.span), value);
            let set = Stmt::Set {
                place,
                value,
                span: n.var.span.to(&head.unwrap_or(n.var.span)),
            };
            let (cond, block) = l.loop_parts(&n.cond, &n.block.stmts, n.span);
            let mut update = Vec::new();
            do_if!(let Some(u) = &n.update, l.stmt(u, &mut update));
            let after = l.drops_from(l.drops.len() - 1, n.span);
            parts = Some((set, cond, block, update, after));
        });
        let (set, cond, block, update, after) = parts.expect("the scope ran");
        out.push(Stmt::Loop { init: vec![set], cond, update, block, span: n.span });
        out.extend(after);
    }

    /// Lower an each-cell or an each-character loop into a walk over a
    /// duplicate of its source: a hidden number starts at 0 and stops at the
    /// count. A round sets the loop variable to the cell (or the character
    /// that starts at that u8), and drops it at the end of the round.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `out` - The list the statements go to.
    fn each(&mut self, n: &st::EachStmt, out: &mut Vec<Stmt>)
    {
        ret_if!(n.part.is_row(), self.each_rows(n, out));
        let chars = n.part == st::EachPart::Char;
        let given = self.kind_of(&n.source);
        let kind = each_list(given, chars);
        let span = n.source.span();
        let source = tern!(
            kind == given,
            self.owned(&n.source, kind),
            self.through(&n.source, kind)
        );
        let mut parts = None;
        self.scoped(|l| {
            let name = ast::Ident::builder().name("%each").span(span).build();
            let each = l.declare(&name, kind, Access::Var);
            let at = l.hidden(DataKind::USIZE, span);
            let zero = l.usize_const(0, span);
            let mut init = vec![
                Stmt::Set { place: each, value: source, span },
                Stmt::Set { place: at, value: zero.clone(), span },
            ];
            // Characters step over u8s, so their numbers count apart.
            let nth = (chars && n.place.is_some()).then(|| l.hidden(DataKind::USIZE, span));
            do_if!(let Some(nth) = nth, init.push(Stmt::Set { place: nth, value: zero, span }));
            let count = l.each_count(each, kind, span);
            let at_read = read(at, DataKind::USIZE, span);
            let test = compare_node(CmpOp::Lt, at_read.clone(), count);
            let cond = node_expr(DataKind::Bool, test, span);
            let (block, step) = l.each_round(n, each, at, nth);
            let next = node_expr(
                DataKind::USIZE,
                infix_node(InfixOp::Add, at_read, step),
                span,
            );
            let update = vec![Stmt::Set { place: at, value: next, span }];
            let after = l.drops_from(l.drops.len() - 1, n.span);
            parts = Some((init, cond, block, update, after));
        });
        let (init, cond, block, update, after) = parts.expect("the scope ran");
        out.push(Stmt::Loop { init, cond, update, block, span: n.span });
        out.extend(after);
    }

    /// The list a pointer points at, read out as a value of its own: a
    /// duplicate for a plain list.
    ///
    /// # Arguments
    ///
    /// * `ptr` - The pointer.
    /// * `kind` - The list kind.
    ///
    /// # Returns
    ///
    /// The list.
    fn through(&mut self, ptr: &ast::Expr, kind: DataKind) -> Expr
    {
        let addr = Box::new(self.expr(ptr));
        let load = node_expr(kind, ExprNode::Load { addr }, ptr.span());
        tern!(kind.holds_text(), dup(load), load)
    }

    /// The count of cells or u8s an each loop walks.
    ///
    /// # Arguments
    ///
    /// * `each` - The slot of the duplicate of the source.
    /// * `kind` - Its kind.
    /// * `span` - The source region of the source.
    ///
    /// # Returns
    ///
    /// A `usize`: a constant for a list, else the second word.
    fn each_count(&self, each: Place, kind: DataKind, span: Span) -> Expr
    {
        let (_, len) = cells_of(kind);
        ret_if!(let Some(len) = len, self.usize_const(len, span));
        let node = self.count_word(read(each, kind, span), span);
        node_expr(DataKind::USIZE, node, span)
    }

    /// Lower one round of an each loop in its own scope: set the loop
    /// variable, then run the block.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    /// * `each` - The slot of the duplicate of the source.
    /// * `at` - The slot of the number of the cell or the u8.
    ///
    /// # Returns
    ///
    /// The round, and the step to the next cell or character.
    fn each_round(
        &mut self, n: &st::EachStmt, each: Place, at: Place, nth: Option<Place>,
    ) -> (Vec<Stmt>, Expr)
    {
        let kind = self.slot_kind(each);
        let span = n.var.span;
        let mut block = Vec::new();
        let mut step = self.usize_const(1, span);
        let chars = n.part == st::EachPart::Char;
        self.loops.push(self.scopes.len());
        self.scoped(|l| {
            let (value, cell) = tern!(
                chars,
                each_char(each, kind, at, span),
                each_cell(each, kind, at, span)
            );
            if let Some(place) = &n.place
            {
                let number = read(nth.unwrap_or(at), DataKind::USIZE, span);
                l.bind(place, number, &mut block);
            }
            if let Some(nth) = nth
            {
                let one = l.usize_const(1, span);
                let next =
                    infix_node(InfixOp::Add, read(nth, DataKind::USIZE, span), one);
                let next = node_expr(DataKind::USIZE, next, span);
                block.push(Stmt::Set { place: nth, value: next, span });
            }
            let var = l.declare(&n.var, cell, Access::Var);
            block.push(Stmt::Set { place: var, value, span });
            if chars
            {
                let held = l.hidden(DataKind::USIZE, span);
                let width = char_width(read(var, cell, span));
                block.push(Stmt::Set { place: held, value: width, span });
                step = read(held, DataKind::USIZE, span);
            }
            do_if!(
                l.is_boxed_name(&n.var.name),
                l.rebox(&n.var, var, cell, &mut block)
            );
            do_for!(s in &n.block.stmts, l.stmt(s, &mut block));
            let leaves = block.last().is_some_and(leaves_scope);
            let end = l.drops_from(l.drops.len() - 1, n.block.span);
            do_if!(!leaves, block.extend(end));
        });
        self.loops.pop();
        (block, step)
    }

    /// Lower a function. Parameters come first among its slots, then the
    /// result variable, set to its default. A function with a result that
    /// does not end in a return returns the result variable.
    ///
    /// # Arguments
    ///
    /// * `f` - The declaration.
    ///
    /// # Returns
    ///
    /// The function.
    fn func(
        &mut self, f: &st::FnDecl, own: Option<(Name, DataKind)>,
        captures: &[(Name, DataKind)],
    ) -> Func
    {
        let given = f.ret.as_ref().map(|k| DataKind::from_ast(k, self.structs));
        // A function that can fail keeps its good result in `ရလဒ်`; an external
        // one keeps the whole result the host or C gives.
        let fault = given.and_then(Fault::of_kind).filter(|_| !f.external);
        let whole = given.map(DataKind::repr).filter(|_| f.external);
        let ret = whole.or_else(|| fault.map_or(given, |f| f.ok));
        let depth = self.scopes.len();
        #[rustfmt::skip]
        let boxed = nested::boxed_in(f);
        #[rustfmt::skip]
        let outer = self.frame.replace(Frame { locals: Vec::new(), ret, fault, depth, edits: HashSet::new(), boxes: HashSet::new(), boxed, write_backs: Vec::new(), owned_ret: f.owned_ret });
        let mut block = Vec::new();
        let mut result = None;
        self.scoped(|l| {
            do_for!(p in &f.params, l.declare_param(p));
            do_if!(let Some((name, kind)) = own, l.declare_own(name, kind, f.name.span));
            do_for!((name, kind) in captures, l.declare_capture(*name, *kind, f.name.span));
            do_for!(p in &f.params, l.box_param(p, &mut block));
            // An external function keeps what the host or C gives in a result
            // variable; each other function gives each value back at its return.
            result = ret.filter(|_| f.external).map(|k| l.result_slot(f, k, &mut block));
            match (f.external, &f.lib)
            {
                // A call names a C function that takes other values itself.
                (true, Some(_)) if f.variadic => (),
                (true, Some(lib)) =>
                {
                    l.foreign_statement(f, &lib.lit.value, result, &mut block)
                }
                (true, None) => l.host_statement(f, given, result, &mut block),
                _ => (),
            }
            do_for!(s in &f.block.stmts, l.stmt(s, &mut block));
            let ends_in_return = matches!(block.last(), Some(Stmt::Return { .. }));
            do_if!(!ends_in_return, block.extend(l.exit_drops(depth, f.block.span)));
        });
        let ends_in_return = matches!(block.last(), Some(Stmt::Return { .. }));
        let implicit = result.zip(ret).filter(|_| !ends_in_return);
        let ret_stmt = implicit.map(|(slot, kind)| {
            let stmt = read_return(slot, kind, f.block.span);
            // A result marked `အပိုင်အနေဖြင့်` hands its block over.
            tern!(f.owned_ret, hand_back(stmt), stmt)
        });
        block.extend(ret_stmt);
        // The check proves that each path of a function with a result returns,
        // so no path reaches its end; the targets still want a value there.
        let ends_in_return = matches!(block.last(), Some(Stmt::Return { .. }));
        let unreached = ret.filter(|_| result.is_none() && !ends_in_return);
        let span = f.block.span;
        let blank = unreached
            .map(|k| Stmt::Return { value: Some(default_constant(k, span)), span });
        block.extend(blank);
        // A function that can fail and ends without a return gives back a
        // good result.
        let ends_in_return = matches!(block.last(), Some(Stmt::Return { .. }));
        let bare = Stmt::Return { value: None, span: f.block.span };
        do_if!(fault.is_some() && !ends_in_return, block.push(bare));
        do_if!(let Some(fault) = fault, wrap_returns(&mut block, fault, self.platform));
        let given = fault.map_or(ret.map(DataKind::repr), |f| Some(f.wrapper));
        let frame =
            std::mem::replace(&mut self.frame, outer).expect("the frame is set");
        #[rustfmt::skip]
        return Func::builder().name(format!("{}{}", f.name.name, self.suffix))
            .params((f.params.len() + usize::from(own.is_some()) + captures.len()) as u32)
            .maybe_result(result).owned(f.params.iter().map(|p| p.owned).collect())
            .owned_ret(f.owned_ret).maybe_ret(given).locals(frame.locals).block(block).span(f.span).build();
    }

    /// The one statement of an external function: its result variable takes
    /// what the host function gives for the borrowed address of each
    /// parameter. A check with no error leaves the host function and the
    /// result there.
    ///
    /// # Arguments
    ///
    /// * `f` - The declaration.
    /// * `given` - Its result kind.
    /// * `result` - The slot of its result variable.
    /// * `block` - The statements so far.
    fn host_statement(
        &mut self, f: &st::FnDecl, given: Option<DataKind>, result: Option<u32>,
        block: &mut Vec<Stmt>,
    )
    {
        let kinds: Vec<DataKind> =
            f.params.iter().map(|p| param_kind(p, self.structs)).collect();
        let func = host::find(&f.name.name, &kinds, given)
            .expect("a check found the host function");
        let slot = result.expect("each host function gives a result");
        let addr = |(p, kind): (&st::Param, &DataKind)| {
            let place = self.lookup(&p.name.name);
            self.slot(place).addressed = true;
            node_expr(kind.pointer_to().as_borrowed(), ExprNode::AddrOf(place), p.span)
        };
        let args = f.params.iter().zip(&kinds).map(addr).collect();
        let kind =
            given.map(DataKind::repr).expect("each host function gives a result");
        let value = node_expr(kind, ExprNode::Host { func, args }, f.span);
        block.push(Stmt::Set { place: Place::Local(slot), value, span: f.span });
    }

    /// The one statement of a C function: its result variable takes what
    /// the C function gives for the value of each parameter, or, with no
    /// result, the C function runs.
    ///
    /// # Arguments
    ///
    /// * `f` - The declaration.
    /// * `lib` - The lib it comes from.
    /// * `result` - The slot of its result variable.
    /// * `block` - The statements so far.
    fn foreign_statement(
        &mut self, f: &st::FnDecl, lib: &str, result: Option<u32>,
        block: &mut Vec<Stmt>,
    )
    {
        let kinds: Vec<DataKind> =
            f.params.iter().map(|p| param_kind(p, self.structs)).collect();
        let ret = f.ret.as_ref().map(|k| DataKind::from_ast(k, self.structs));
        let platform = self.platform;
        let c_kinds = kinds.iter().map(|k| c_param_kind(*k, platform)).collect();
        // C gives back a handle as its address.
        let handle_ret = ret
            .filter(|k| k.as_external().is_some() || foreign::is_c_handle_result(*k));
        let c_ret = tern!(handle_ret.is_some(), Some(DataKind::handle_pointer()), ret);
        #[rustfmt::skip]
        let def = ForeignFn::builder().lib(lib).name(f.name.name.to_string())
            .params(c_kinds).maybe_ret(c_ret).owned_ret(f.owned_ret).build();
        let func = self.foreign_index(def);
        let params: Vec<(Expr, bool)> = f
            .params
            .iter()
            .zip(&kinds)
            .map(|(p, k)| (read(self.lookup(&p.name.name), *k, p.span), p.owned))
            .collect();
        let handles = params.iter().filter(|(e, _)| e.kind.as_external().is_some());
        // A closed handle stops the run before C sees its address.
        let guards: Vec<Stmt> =
            handles.clone().map(|(e, _)| self.handle_guard(e.clone())).collect();
        block.extend(guards);
        // A handle a C function takes over is closed for each value that shares
        // it.
        let closes = handles.filter(|(_, owned)| *owned);
        let closes: Vec<Stmt> =
            closes.map(|(e, _)| self.handle_close(e.clone())).collect();
        // C keeps no locals alive for a closure.
        let arg = |(e, _): (Expr, bool)| match e.kind.as_external()
        {
            Some(_) => self.handle_ptr(e),
            None => c_list_arg(e, platform),
        };
        let args = params.into_iter().map(arg).collect();
        let span = f.span;
        let Some((slot, kind)) = result.zip(ret)
        else
        {
            block.push(Stmt::Foreign { func, args, span });
            return block.extend(closes);
        };
        let place = Place::Local(slot);
        let called =
            node_expr(c_ret.unwrap_or(kind), ExprNode::Foreign { func, args }, span);
        match handle_ret
        {
            Some(k) if k.as_external().is_some() =>
            {
                self.handle_or_stop(called, k, place, block)
            }
            Some(k) => self.handle_or_none(called, k, place, block),
            None => block.push(Stmt::Set { place, value: called, span }),
        }
        block.extend(closes);
    }

    /// Read a field of the struct C keeps behind a handle. A closed handle
    /// stops the run first; text, which the read copies, waits in a slot the
    /// scope drops.
    ///
    /// # Arguments
    ///
    /// * `n` - The field expression.
    /// * `kind` - The kind the check recorded for it.
    ///
    /// # Returns
    ///
    /// The operation.
    fn c_field_read(&mut self, n: &ast::FieldExpr, kind: DataKind) -> ExprNode
    {
        let shape = self.kind_of(&n.base).as_external().expect("the base is a handle");
        let (_, offset) = shape
            .c_field(&n.field.name, self.platform)
            .expect("the check found the field");
        let handle = self.expr(&n.base);
        let place = matches!(handle.node, ExprNode::Read(_) | ExprNode::Load { .. });
        let handle = tern!(place, handle, self.spill(handle));
        let guard = self.handle_guard(handle.clone());
        self.pre.push(guard);
        let base = Box::new(self.handle_ptr(handle));
        let inner = kind.fallible_parts().and_then(|(ok, _)| ok);
        let inner = inner.filter(|k| k.as_external().is_some());
        ret_if!(inner.is_some(), self.c_handle_field(base, offset, kind, n.span));
        let node = ExprNode::CField { base, offset };
        ret_if!(!kind.holds_text(), node);
        let read = node_expr(kind, node, n.span);
        self.spill(read).node
    }

    /// Write a field of the struct C keeps behind a handle, in unsafe code. A
    /// closed handle stops the run first.
    ///
    /// # Arguments
    ///
    /// * `f` - The field.
    /// * `value` - The value.
    /// * `out` - The statements so far.
    fn c_field_store(
        &mut self, f: &ast::FieldExpr, value: &ast::Expr, out: &mut Vec<Stmt>,
    )
    {
        let shape = self.kind_of(&f.base).as_external().expect("the base is a handle");
        let found = shape.c_field(&f.field.name, self.platform);
        let (kind, offset) = found.expect("the check found the field");
        let handle = self.expr(&f.base);
        let place = matches!(handle.node, ExprNode::Read(_) | ExprNode::Load { .. });
        let handle = tern!(place, handle, self.spill(handle));
        out.push(self.handle_guard(handle.clone()));
        let base = self.handle_ptr(handle);
        let value = self.owned(value, kind);
        out.push(Stmt::CStore { base, offset, value, span: f.span });
    }

    /// Read a field that points at another struct C keeps: a new handle of
    /// the address, or the optional with no data for NULL, in a slot the
    /// scope drops.
    ///
    /// # Arguments
    ///
    /// * `base` - The address of the C struct that holds the field.
    /// * `offset` - The offset of the field.
    /// * `kind` - The kind of the read: an optional of the handle.
    /// * `span` - The field expression.
    ///
    /// # Returns
    ///
    /// The read of the slot.
    fn c_handle_field(
        &mut self, base: Box<Expr>, offset: u32, kind: DataKind, span: Span,
    ) -> ExprNode
    {
        let node = ExprNode::CField { base, offset };
        let called = node_expr(DataKind::handle_pointer(), node, span);
        let out = self.hidden(kind, span);
        let blank = default_constant(kind, span);
        self.pre_init.push(Stmt::Set { place: out, value: blank, span });
        self.drops.last_mut().expect("a scope is open").push((out, kind));
        let mut stmts = Vec::new();
        self.handle_or_none(called, kind, out, &mut stmts);
        self.pre.extend(stmts);
        ExprNode::Read(out)
    }

    /// The address a handle holds: the one field of its box, read checked, so
    /// a handle that is the zero word stops the program.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle.
    ///
    /// # Returns
    ///
    /// The address.
    fn handle_ptr(&self, handle: Expr) -> Expr
    {
        let span = handle.span;
        let addr = self.handle_field(handle);
        node_expr(
            DataKind::handle_pointer(),
            ExprNode::Load { addr: Box::new(addr) },
            span,
        )
    }

    /// The address of the one field of the box of a handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle.
    ///
    /// # Returns
    ///
    /// The address of the field.
    fn handle_field(&self, handle: Expr) -> Expr
    {
        let span = handle.span;
        let base = Box::new(self.object_fields(handle));
        let node = ExprNode::FieldAddr { base, field: 0, offset: 0 };
        node_expr(DataKind::handle_pointer().pointer_to(), node, span)
    }

    /// The stop of a call of C with a handle a C function closed: its box
    /// holds 0.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle.
    ///
    /// # Returns
    ///
    /// The statement.
    fn handle_guard(&self, handle: Expr) -> Stmt
    {
        let span = handle.span;
        let null =
            node_expr(DataKind::handle_pointer(), ExprNode::Const(Value::NULL), span);
        let compare = compare_node(CmpOp::Eq, self.handle_ptr(handle), null);
        let cond = node_expr(DataKind::Bool, compare, span);
        let halt = Stmt::Halt { cause: HaltCause::ClosedHandle, value: None, span };
        Stmt::If { cond, then: vec![halt], els: Vec::new(), span }
    }

    /// Close a handle a C function took over: its box holds 0 from here on,
    /// so each value that shares it reads as closed.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle.
    ///
    /// # Returns
    ///
    /// The statement.
    fn handle_close(&self, handle: Expr) -> Stmt
    {
        let span = handle.span;
        let null =
            node_expr(DataKind::handle_pointer(), ExprNode::Const(Value::NULL), span);
        Stmt::Store { addr: self.handle_field(handle), value: null, span }
    }

    /// A new handle of a kind from outside, around an address C gave.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    /// * `kind` - The kind of the handle.
    ///
    /// # Returns
    ///
    /// The handle: a new box that holds the address.
    fn handle_of(&self, addr: Expr, kind: DataKind) -> Expr
    {
        let span = addr.span;
        let shape = kind.as_external().expect("a handle has a kind from outside");
        let fields =
            node_expr(DataKind::Struct(shape), ExprNode::List(vec![addr]), span);
        self.make_box(fields, span)
    }

    /// Set a handle from C, for a result written with no optional: NULL from
    /// C stops the run, since the declaration promised a handle.
    ///
    /// # Arguments
    ///
    /// * `called` - The call of C, which gives an address.
    /// * `kind` - The handle kind.
    /// * `place` - The result slot.
    /// * `block` - The statements so far.
    fn handle_or_stop(
        &mut self, called: Expr, kind: DataKind, place: Place, block: &mut Vec<Stmt>,
    )
    {
        let span = called.span;
        let held = self.hidden(DataKind::handle_pointer(), span);
        block.push(Stmt::Set { place: held, value: called, span });
        let addr = read(held, DataKind::handle_pointer(), span);
        let null =
            node_expr(DataKind::handle_pointer(), ExprNode::Const(Value::NULL), span);
        let compare = compare_node(CmpOp::Eq, addr.clone(), null);
        let cond = node_expr(DataKind::Bool, compare, span);
        let halt = Stmt::Halt { cause: HaltCause::NullFromC, value: None, span };
        block.push(Stmt::If { cond, then: vec![halt], els: Vec::new(), span });
        block.push(Stmt::Set { place, value: self.handle_of(addr, kind), span });
    }

    /// Set an optional of a handle: NULL from C is the optional with no data.
    ///
    /// # Arguments
    ///
    /// * `called` - The call of C, which gives an address.
    /// * `kind` - The result kind.
    /// * `place` - The result slot.
    /// * `block` - The statements so far.
    fn handle_or_none(
        &mut self, called: Expr, kind: DataKind, place: Place, block: &mut Vec<Stmt>,
    )
    {
        let span = called.span;
        let fault =
            Fault::of_kind(kind).expect("the result is an optional of a handle");
        let ok = fault.ok.expect("the good side is a handle");
        let held = self.hidden(DataKind::handle_pointer(), span);
        block.push(Stmt::Set { place: held, value: called, span });
        let addr = read(held, DataKind::handle_pointer(), span);
        let null =
            node_expr(DataKind::handle_pointer(), ExprNode::Const(Value::NULL), span);
        let cond = node_expr(
            DataKind::Bool,
            compare_node(CmpOp::Eq, addr.clone(), null),
            span,
        );
        let platform = self.platform;
        let failed = fault.value(true, None, None, platform, span);
        let good =
            fault.value(false, Some(self.handle_of(addr, ok)), None, platform, span);
        let then = vec![Stmt::Set { place, value: failed, span }];
        let els = vec![Stmt::Set { place, value: good, span }];
        block.push(Stmt::If { cond, then, els, span });
    }

    /// The index of a C function among those of the module, added when its
    /// C name is new. The check gives each C name one set of kinds.
    ///
    /// # Arguments
    ///
    /// * `def` - The C function.
    ///
    /// # Returns
    ///
    /// The index.
    fn foreign_index(&mut self, def: ForeignFn) -> u32
    {
        let known = self.foreign.iter().position(|d| d.name == def.name);
        let index = known.unwrap_or(self.foreign.len());
        do_if!(known.is_none(), self.foreign.push(def));
        index as u32
    }

    /// Declare a function parameter.
    ///
    /// # Arguments
    ///
    /// * `p` - The parameter.
    fn declare_param(&mut self, p: &st::Param)
    {
        let place = self.declare(&p.name, param_kind(p, self.structs), Access::Var);
        // A parameter marked `ဒေတာ ထည့်ပေးရန်` points at a place the function
        // must set before it returns, and the caller need not set first.
        do_if!(p.fill, self.slot(place).unset = Some(p.span));
        let edit = tern!(let Place::Local(n) = place, Some(n), None).filter(|_| p.edit);
        let frame = self.frame.as_mut();
        do_if!(let (Some(n), Some(f)) = (edit, frame), { f.edits.insert(n); });
    }

    /// Whether a place is reached through an address: a parameter the
    /// function can change, which holds a borrow of the caller's place, or
    /// the box of a variable a closure shares.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    fn is_edit(&self, place: Place) -> bool
    {
        let Place::Local(n) = place
        else
        {
            return false;
        };
        self.frame
            .as_ref()
            .is_some_and(|f| f.edits.contains(&n) || f.boxes.contains(&n))
    }

    /// The address a place is reached through: the borrow a parameter the
    /// function can change holds, or the cell of a box.
    ///
    /// # Arguments
    ///
    /// * `place` - The parameter.
    /// * `span` - The use of its name.
    ///
    /// # Returns
    ///
    /// The address.
    fn edit_addr(&mut self, place: Place, span: Span) -> Expr
    {
        let kind = self.slot_kind(place);
        let held =
            Expr::builder().kind(kind).node(ExprNode::Read(place)).span(span).build();
        ret_if!(!self.is_box(place), held);
        let cell = kind.as_seq().map_or(kind, |s| s.cell);
        let index = Box::new(self.usize_const(0, span));
        // A box has one cell, so cell 0 needs no check.
        #[rustfmt::skip]
        let node = ExprNode::CellAddr { list: Box::new(held), index, len: None, first: 0, checked: true };
        node_expr(cell.pointer_to(), node, span)
    }

    /// Whether a place holds the box of a variable a closure shares.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn is_box(&self, place: Place) -> bool
    {
        let Place::Local(n) = place
        else
        {
            return false;
        };
        self.frame.as_ref().is_some_and(|f| f.boxes.contains(&n))
    }

    /// The kind of the variable a place stands for: the kind of the slot,
    /// the kind behind the borrow of a parameter the function can change, or
    /// the kind of the cell of a box.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    ///
    /// # Returns
    ///
    /// The kind.
    fn reached_kind(&mut self, place: Place) -> DataKind
    {
        let kind = self.slot_kind(place);
        ret_if!(self.is_box(place), kind.as_seq().map_or(kind, |s| s.cell));
        tern!(self.is_edit(place), kind.pointee().unwrap_or(kind), kind)
    }

    /// The read of a name: the slot, or for a parameter the function can
    /// change, the caller's place through the borrow it holds.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `span` - The use.
    ///
    /// # Returns
    ///
    /// The operation.
    fn read_name(&mut self, name: &str, span: Span) -> ExprNode
    {
        let place = self.lookup(name);
        ret_if!(!self.is_edit(place), ExprNode::Read(place));
        ExprNode::Load { addr: Box::new(self.edit_addr(place, span)) }
    }

    /// Declare the result variable of a function and set it to its default.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    /// * `kind` - The result kind.
    /// * `block` - The function statements, which get the store.
    ///
    /// # Returns
    ///
    /// The local slot index.
    fn result_slot(
        &mut self, f: &st::FnDecl, kind: DataKind, block: &mut Vec<Stmt>,
    ) -> u32
    {
        let ident = ast::Ident::builder().name(RESULT_NAME).span(f.name.span).build();
        let place = self.declare(&ident, kind, Access::Var);
        let value = default_constant(kind, f.name.span);
        block.push(Stmt::Set { place, value, span: f.name.span });
        tern!(let Place::Local(n) = place, n, 0)
    }

    /// Lower a return statement. With places that hold text in scope, the
    /// value goes to a hidden slot first, since it can read them; then those
    /// are dropped and the slot is returned.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    /// * `out` - The list the statements go to.
    fn return_stmt(&mut self, n: &st::ReturnStmt, out: &mut Vec<Stmt>)
    {
        let span = n.span;
        // A whole result, such as an error value or `အလွတ်`, leaves as it is.
        let wrapper = self.frame.as_ref().and_then(|f| f.fault).map(|f| f.wrapper);
        let whole =
            n.value.as_ref().zip(wrapper).filter(|(v, w)| self.kind_of(v) == *w);
        if let Some((v, w)) = whole
        {
            let value = self.owned(v, w);
            return self.leave_with(value, span, out);
        }
        let ret = self.frame.as_ref().and_then(|f| f.ret);
        let value = n.value.as_ref().zip(ret).map(|(v, k)| self.owned(v, k));
        let handed = self.frame.as_ref().is_some_and(|f| f.owned_ret);
        let value = value.map(|v| tern!(handed, hand_over(v), v));
        let depth = self.frame.as_ref().map(|f| f.depth);
        let drops = match depth
        {
            Some(d) => self.exit_drops(d, span),
            None => self.global_drops(span),
        };
        ret_if!(drops.is_empty(), out.push(Stmt::Return { value, span }));
        let result = value.map(|value| {
            let kind = value.kind;
            let place = self.hidden(kind, span);
            out.push(Stmt::Set { place, value, span });
            let ret = tern!(let Place::Local(slot) = place, read_return(slot, kind, span), Stmt::Return { value: None, span });
            // A result marked `အပိုင်အနေဖြင့်` leaves its slot for the caller.
            tern!(handed, hand_back(ret), ret)
        });
        out.extend(drops);
        out.push(result.unwrap_or(Stmt::Return { value: None, span }));
    }

    /// Lower `... ကို အမှားအယွင်းအဖြစ် ပြန်ပေးမည်။`: the function gives back the
    /// error value with tag 1. The struct is built in a hidden slot before
    /// the places that hold text are dropped, since the value can read them.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    /// * `out` - The list the statements go to.
    fn fault_return(&mut self, n: &st::ReturnStmt, out: &mut Vec<Stmt>)
    {
        let fault = self.frame.as_ref().and_then(|f| f.fault);
        let fault =
            fault.expect("the check allows an error value in a function that can fail");
        let err = n.value.as_ref().zip(fault.err).map(|(v, k)| self.owned(v, k));
        let value = fault.value(true, None, err, self.platform, n.span);
        self.leave_with(value, n.span, out);
    }

    /// Give back a struct of the kind the function gives back, after the
    /// drops of each place that holds text.
    ///
    /// # Arguments
    ///
    /// * `value` - The struct.
    /// * `span` - The source region.
    /// * `out` - The list the statements go to.
    fn leave_with(&mut self, value: Expr, span: Span, out: &mut Vec<Stmt>)
    {
        let depth = self.frame.as_ref().map_or(0, |f| f.depth);
        let drops = self.exit_drops(depth, span);
        ret_if!(drops.is_empty(), out.push(Stmt::Return { value: Some(value), span }));
        let (kind, place) = (value.kind, self.hidden(value.kind, span));
        out.push(Stmt::Set { place, value, span });
        out.extend(drops);
        out.push(Stmt::Return { value: Some(read(place, kind, span)), span });
    }

    /// Call a function that can fail into a hidden slot, and leave the
    /// function around the call with the same error value when it fails. The
    /// statements go to [`Self::pre`]. A slot whose kind holds text joins the
    /// drops of the scope, so the error value goes out as a duplicate.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name.
    /// * `place` - The place that holds the function value, when written.
    /// * `args` - The arguments.
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The hidden slot and what the function gives back.
    fn checked_call(
        &mut self, callee: &ast::Ident, place: Option<&ast::Expr>, args: &[ast::Expr],
        span: Span,
    ) -> (Place, Fault)
    {
        let (target, sig) = self.call_target(callee, place);
        let fault = sig.ret.and_then(Fault::of_kind);
        let fault = fault.expect("the check found a function that can fail");
        let args = self.lower_args(args, &sig);
        let call = target.node(args);
        let held = self.hidden(fault.wrapper, span);
        if fault.wrapper.holds_text()
        {
            let value = fault.value(false, None, None, self.platform, span);
            self.pre_init.push(Stmt::Set { place: held, value, span });
            let drops = self.drops.last_mut().expect("a scope is open");
            drops.push((held, fault.wrapper));
        }
        let value = node_expr(fault.wrapper, call, span);
        self.pre.push(Stmt::Set { place: held, value, span });
        self.pre.append(&mut self.post);
        let err = fault.err.map(|_| fault.field(held, "err", self.platform, span));
        let err = err.map(|e| tern!(e.kind.holds_text(), dup(e), e));
        let own = self.frame.as_ref().and_then(|f| f.fault);
        let own = own.expect("the check allows a pass-on in a function that can fail");
        let mut leave = Vec::new();
        self.leave_with(
            own.value(true, None, err, self.platform, span),
            span,
            &mut leave,
        );
        let cond = fault.failed(held, self.platform, span);
        self.pre.push(Stmt::If { cond, then: leave, els: Vec::new(), span });
        (held, fault)
    }

    /// Lower a test of what a result or an optional holds, such as
    /// `<r> ထဲတွင် ရလဒ်ကောင်း ရှိလျှင်` or `<o> ထဲတွင် ဒေတာ မရှိလျှင်`: a test of
    /// its tag.
    ///
    /// # Arguments
    ///
    /// * `op` - The verb.
    /// * `value` - The result.
    /// * `span` - The test.
    ///
    /// # Returns
    ///
    /// The boolean test.
    fn result_test(&mut self, op: VerbOp, value: &ast::Expr, span: Span) -> ExprNode
    {
        let result = self.expr(value);
        let fault = Fault::of_kind(result.kind).expect("the check found a result");
        let failed = fault.failed_of(result, self.platform, span);
        ret_if!(matches!(op, VerbOp::IsFault | VerbOp::LacksData), failed.node);
        prefix_node(PrefixOp::Not, failed)
    }

    /// Lower `<e> ဟူသော အမှားအယွင်း`: the result of the expected kind that
    /// holds the error value `<e>`, a duplicate of it when it reads text from a
    /// place.
    ///
    /// # Arguments
    ///
    /// * `value` - The error value.
    /// * `kind` - The result kind.
    /// * `span` - The expression.
    ///
    /// # Returns
    ///
    /// The result.
    fn make_fault(&mut self, value: &ast::Expr, kind: DataKind, span: Span)
    -> ExprNode
    {
        let fault = Fault::of_kind(kind).expect("the check found a result");
        // The result owns its error value: text read from a place is copied.
        let err = fault.err.map(|k| self.owned(value, k));
        fault.value(true, None, err, self.platform, span).node
    }

    /// Lower `<r> ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ်` or its twin with
    /// `အမှားအယွင်း`: the result goes to a hidden slot, a statement in
    /// [`Self::pre`] stops the run when it holds the other part, and the
    /// value is the part asked for. A result that holds an error shows its
    /// error value as the run stops.
    ///
    /// # Arguments
    ///
    /// * `op` - The verb.
    /// * `value` - The result.
    /// * `span` - The conversion.
    ///
    /// # Returns
    ///
    /// The read of the part.
    fn result_part(&mut self, op: VerbOp, value: &ast::Expr, span: Span) -> ExprNode
    {
        self.pending = self.pending.saturating_sub(1);
        let good = op == VerbOp::Unwrap;
        let result = self.expr(value);
        let fault = Fault::of_kind(result.kind).expect("the check found a result");
        let held = self.hidden(fault.wrapper, span);
        if fault.wrapper.holds_text()
        {
            let blank = fault.value(false, None, None, self.platform, span);
            self.pre_init.push(Stmt::Set { place: held, value: blank, span });
            self.drops.last_mut().expect("a scope is open").push((held, fault.wrapper));
        }
        let read_place =
            matches!(result.node, ExprNode::Read(_) | ExprNode::Load { .. });
        let result = tern!(read_place && result.kind.holds_text(), dup(result), result);
        self.pre.push(Stmt::Set { place: held, value: result, span });
        let failed = fault.failed(held, self.platform, span);
        let wrong = tern!(
            good,
            failed.clone(),
            node_expr(DataKind::Bool, prefix_node(PrefixOp::Not, failed), span)
        );
        let shown = fault
            .err
            .filter(|_| good)
            .map(|_| fault.field(held, "err", self.platform, span));
        let lacks =
            tern!(fault.wrapper.is_optional(), HaltCause::NoData, HaltCause::NotGood);
        let cause = tern!(good, lacks, HaltCause::NotFault);
        let halt = Stmt::Halt { cause, value: shown, span };
        self.pre.push(Stmt::If {
            cond: wrong,
            then: vec![halt],
            els: Vec::new(),
            span,
        });
        fault.field(held, tern!(good, "ok", "err"), self.platform, span).node
    }

    /// Lower a call that passes a failure on, as part of a value: the call
    /// and its test go to [`Self::pre`], and the value is the good result.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    ///
    /// # Returns
    ///
    /// The read of the good result.
    fn pass_call(&mut self, n: &ast::CallExpr) -> ExprNode
    {
        self.pending = self.pending.saturating_sub(1);
        let (held, fault) =
            self.checked_call(&n.callee, n.place.as_deref(), &n.args, n.span);
        fault.field(held, "ok", self.platform, n.span).node
    }

    /// The drops of the scopes a `break` or `continue` leaves.
    ///
    /// # Arguments
    ///
    /// * `span` - The source region of the jump.
    ///
    /// # Returns
    ///
    /// The drop statements.
    fn loop_drops(&self, span: Span) -> Vec<Stmt>
    {
        self.loops.last().map_or_else(Vec::new, |&depth| self.drops_from(depth, span))
    }

    /// What a call runs, and the signature its arguments follow: a declared
    /// function of the name, as the check decides, else the function value
    /// the name or the place holds.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name, or the first name of the place.
    /// * `place` - The place that holds the function value, when written.
    ///
    /// # Returns
    ///
    /// The target and its signature.
    fn call_target(
        &mut self, callee: &ast::Ident, place: Option<&ast::Expr>,
    ) -> (Target, Sig)
    {
        let nested =
            self.nested_named(&callee.name).filter(|_| place.is_none()).cloned();
        if let Some(n) = nested
        {
            if self.through_closure(&n)
            {
                let value = self.closure_value(&n, callee.span);
                let f = value.kind.as_fn().expect("a closure has a function kind");
                return (Target::Value(value), value_sig(f));
            }
            let target = Target::Fn { func: n.sig.index, name: n.name.clone() };
            return (target, n.sig.clone());
        }
        let sig = self.sigs.get(&callee.name).filter(|_| place.is_none()).cloned();
        if let Some(def) = sig.as_ref().and_then(|s| s.foreign.clone())
        {
            let func = self.foreign_index(def);
            return (
                Target::Foreign(func),
                sig.expect("the C function has a signature"),
            );
        }
        if let Some(sig) = sig
        {
            return (
                Target::Fn { func: sig.index, name: callee.name.to_string() },
                sig,
            );
        }
        let value = match place
        {
            Some(p) => self.expr(p),
            None => self.read_value(callee),
        };
        let f = value.kind.as_fn().expect("the check found a function value");
        (Target::Value(value), value_sig(f))
    }

    /// The read of a variable or a parameter that holds a function value.
    ///
    /// # Arguments
    ///
    /// * `name` - The name at the call.
    ///
    /// # Returns
    ///
    /// The read, of the function kind.
    fn read_value(&mut self, name: &ast::Ident) -> Expr
    {
        let place = self.lookup(&name.name);
        let kind = self.reached_kind(place);
        let node = self.read_name(&name.name, name.span);
        node_expr(kind, node, name.span)
    }

    /// The arguments of a call, each converted to its parameter's kind.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments.
    /// * `sig` - The signature of the call.
    ///
    /// # Returns
    ///
    /// The lowered arguments.
    fn lower_args(&mut self, args: &[ast::Expr], sig: &Sig) -> Vec<Expr>
    {
        let params = sig.params.iter().zip(&sig.edits).enumerate();
        let lower =
            |(a, (at, (k, edit))): (&ast::Expr, (usize, (&DataKind, &bool)))| {
                let fill = sig.fills.get(at).copied().unwrap_or(false);
                tern!(*edit, self.edit_arg(a, *k, fill), self.owned(a, *k))
            };
        let mut lowered: Vec<Expr> = args.iter().zip(params).map(lower).collect();
        // An argument for a parameter marked `အပိုင်အနေဖြင့်` is handed over:
        // the caller's place holds the zero word after the call.
        let handed = lowered.iter_mut().zip(&sig.owned).filter(|(_, o)| **o);
        handed.for_each(|(e, _)| *e = hand_over(e.clone()));
        // A function value is lent for the call: the callee runs the caller's
        // closure, and its drop of the lent value frees nothing.
        lowered = lowered.into_iter().map(|e| self.hold_fresh_fn(e)).collect();
        lowered.iter_mut().for_each(lend_fn);
        // C keeps no locals alive for a closure, and takes a list of
        // numbers as the address of its first cell; a list made for the call
        // waits in a slot that is dropped after it.
        if sig.foreign.is_some()
        {
            let platform = self.platform;
            let held = |l: &mut Self, e: Expr| {
                let fresh = c_list(e.kind, platform).is_some() && fresh_text(&e);
                tern!(fresh, l.spill(e), e)
            };
            lowered = lowered.into_iter().map(|e| held(self, e)).collect();
            lowered = lowered.into_iter().map(|e| c_list_arg(e, platform)).collect();
        }
        let rest = args.iter().skip(sig.params.len());
        rest.for_each(|a| lowered.push(self.rest_arg(a)));
        let span = args.first().map_or_else(Span::default, |a| a.span());
        // A call of a closure inside itself passes its own value on, lent.
        let own = sig.own.map(|name| {
            let place = self.lookup(&name);
            read(place, self.slot_kind(place), span)
        });
        lowered.extend(own);
        let borrows = sig.captures.iter().map(|c| self.capture_arg(c, span));
        let borrows: Vec<Expr> = borrows.collect();
        lowered.extend(borrows);
        lowered
    }

    /// Keep a function value that a call or another expression makes fresh,
    /// and that goes to a parameter, in a hidden slot its scope drops: the
    /// parameter is lent, so the callee frees nothing of it. A read of a
    /// place, an address of a function and a constant pass unchanged.
    ///
    /// # Arguments
    ///
    /// * `e` - The argument.
    ///
    /// # Returns
    ///
    /// The argument, a duplicate of the hidden slot for a fresh value.
    fn hold_fresh_fn(&mut self, e: Expr) -> Expr
    {
        let kept = matches!(
            e.node,
            ExprNode::Read(_)
                | ExprNode::Load { .. }
                | ExprNode::Dup { .. }
                | ExprNode::Take { .. }
                | ExprNode::FnAddr { .. }
                | ExprNode::Const(_)
        );
        ret_if!(!e.kind.is_fn() || kept, e);
        let held = self.spill(e);
        dup(held)
    }

    /// An argument past the parameters of a C function that takes other
    /// values, as C reads it through `...`: a narrow integer or a boolean
    /// becomes an `i32` and an `f32` an `f64`, as C promotes them. Text the
    /// call builds is held in a hidden slot, freed as the scope ends, since
    /// no function of the program takes it.
    ///
    /// # Arguments
    ///
    /// * `a` - The argument.
    ///
    /// # Returns
    ///
    /// The value C gets.
    fn rest_arg(&mut self, a: &ast::Expr) -> Expr
    {
        let lowered = self.expr(a);
        // A handle goes as the address it holds; a closed one stops the run.
        if lowered.kind.as_external().is_some()
        {
            let place =
                matches!(lowered.node, ExprNode::Read(_) | ExprNode::Load { .. });
            let handle = tern!(place, lowered, self.spill(lowered));
            let guard = self.handle_guard(handle.clone());
            self.pre.push(guard);
            return self.handle_ptr(handle);
        }
        if lowered.kind.is_text()
        {
            let plain = matches!(
                lowered.node,
                ExprNode::Read(_) | ExprNode::Load { .. } | ExprNode::Text(_)
            );
            return tern!(
                plain || self.reads_const(&lowered),
                lowered,
                self.spill(lowered)
            );
        }
        let want = promoted(lowered.kind, self.platform);
        coerce_lowered(lowered, want, self.platform)
    }

    /// Lower a call that gives a value. A call that copies values back to
    /// their places runs before the rest of the statement, with the copies,
    /// so the statement reads the places as the call left them.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    /// * `kind` - The kind of its value.
    ///
    /// # Returns
    ///
    /// The operation.
    fn call_node(&mut self, n: &ast::CallExpr, kind: DataKind) -> ExprNode
    {
        let (target, sig) = self.call_target(&n.callee, n.place.as_deref());
        let args = self.lower_args(&n.args, &sig);
        ret_if!(self.post.is_empty(), target.node(args));
        let held = self.spill(node_expr(kind, target.node(args), n.span));
        self.pre.append(&mut self.post);
        tern!(kind.holds_text(), dup(held), held).node
    }

    /// The argument for a parameter the function can change: the address
    /// of the place. A place the callee can reach through another path too, a
    /// global or a variable a closure shares, is copied in to a hidden slot
    /// whose address goes instead, and copied back after the call
    /// ([`Self::post`]): the callee never holds an address into cells that
    /// move under it, and the place it changes is found again after the call.
    ///
    /// # Arguments
    ///
    /// * `a` - The place.
    /// * `want` - The kind of the parameter, a borrow.
    ///
    /// # Returns
    ///
    /// The borrow.
    fn edit_arg(&mut self, a: &ast::Expr, want: DataKind, fill: bool) -> Expr
    {
        let span = a.span();
        let node =
            tern!(self.reached_elsewhere(a), self.stage_in(a, fill), self.addr_of(a));
        Expr::builder().kind(want).node(node).span(span).build()
    }

    /// Whether the callee can reach a place through another path than the
    /// parameter: its variable is a global or the box a closure shares.
    ///
    /// # Arguments
    ///
    /// * `a` - The place.
    ///
    /// # Returns
    ///
    /// `true` for such a place.
    fn reached_elsewhere(&self, a: &ast::Expr) -> bool
    {
        let Some(name) = place_root(a)
        else
        {
            return false;
        };
        let place = self.lookup(name);
        matches!(place, Place::Global(_)) || self.is_box(place)
    }

    /// Stage a place in a hidden slot for a call and queue the write back.
    ///
    /// # Arguments
    ///
    /// * `a` - The place.
    /// * `fill` - Whether the callee fills the place, so the slot starts blank.
    ///
    /// # Returns
    ///
    /// The address of the hidden slot.
    fn stage_in(&mut self, a: &ast::Expr, fill: bool) -> ExprNode
    {
        let (kind, span) = (self.kind_of(a), a.span());
        // A place the callee fills hands it nothing: the slot starts blank.
        let value = tern!(fill, default_constant(kind, span), self.owned(a, kind));
        let held = self.hidden(kind, span);
        if kind.holds_text()
        {
            let blank = default_constant(kind, span);
            self.pre_init.push(Stmt::Set { place: held, value: blank, span });
            self.drops.last_mut().expect("a scope is open").push((held, kind));
        }
        self.pre.push(Stmt::Set { place: held, value, span });
        let dest = self.dest(a);
        let back = take(read(held, kind, span));
        let mut post = Vec::new();
        let back = tern!(kind.holds_text(), self.replace(&dest, back, &mut post), back);
        post.push(match dest
        {
            Dest::Slot(place) => Stmt::Set { place, value: back, span },
            Dest::Mem(addr) => Stmt::Store { addr, value: back, span },
        });
        self.post.extend(post);
        self.slot(held).addressed = true;
        ExprNode::AddrOf(held)
    }

    /// The common kind of two kinds: the one the other widens to. Two
    /// addresses of one shape keep the first kind, whatever their borrow
    /// marks.
    ///
    /// # Arguments
    ///
    /// * `a` - One kind.
    /// * `b` - The other kind.
    ///
    /// # Returns
    ///
    /// The wider kind.
    fn common(&self, a: DataKind, b: DataKind) -> DataKind
    {
        ret_if!(a.same_shape(b), a);
        tern!(a.widens_to(b, self.platform), b, a)
    }

    /// Lower an expression and convert it to `want` when its kind differs.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `want` - The kind its place expects.
    ///
    /// # Returns
    ///
    /// The expression, of kind `want`.
    fn coerce(&mut self, e: &ast::Expr, want: DataKind) -> Expr
    {
        let lowered = self.expr(e);
        coerce_lowered(lowered, want, self.platform)
    }

    /// Lower a value headed for a place of kind `want`. A value of a move
    /// kind, one that holds an owner pointer, headed for a place of a move
    /// kind is taken out of its own place, which is left at its default. A
    /// place of a kind that holds text is read and duplicated, so the two
    /// places never share u8s. Other values are read. A constant is never
    /// taken or duplicated: its text is static, so a read is a duplicate.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    /// * `want` - The kind of the place.
    ///
    /// # Returns
    ///
    /// The expression, of kind `want`.
    fn owned(&mut self, e: &ast::Expr, want: DataKind) -> Expr
    {
        let lowered = self.expr(e);
        ret_if!(
            self.reads_const(&lowered),
            coerce_lowered(lowered, want, self.platform)
        );
        let read = matches!(lowered.node, ExprNode::Read(_) | ExprNode::Load { .. });
        let copied = read && lowered.kind.holds_text();
        coerce_lowered(tern!(copied, dup(lowered), lowered), want, self.platform)
    }

    /// Whether a value is read out of a constant: the constant itself, or a
    /// field or a cell of one.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    ///
    /// # Returns
    ///
    /// `true` for such a read.
    fn reads_const(&mut self, e: &Expr) -> bool
    {
        match &e.node
        {
            // A constant that holds a closure owns its record, as a variable does.
            ExprNode::Read(place) =>
            {
                self.slot(*place).access == Access::Const && !e.kind.holds_fn()
            }
            ExprNode::Load { addr } => self.reads_const(addr),
            ExprNode::FieldAddr { base, .. }
            | ExprNode::CellAddr { list: base, .. } => self.reads_const(base),
            _ => false,
        }
    }

    /// Lower an expression with the kind the check recorded for it; a
    /// `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` with no pointer kind around it is a `ptr`.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The expression.
    fn expr(&mut self, e: &ast::Expr) -> Expr
    {
        let spill = self.pending > 0 && !self.spilling && !has_pass(e);
        ret_if!(!spill, self.expr_node(e));
        self.spilling = true;
        let lowered = self.expr_node(e);
        self.spilling = false;
        self.spill(lowered)
    }

    /// Keep a value in a hidden slot that [`Self::pre`] sets, so it is
    /// computed before a later call that passes a failure on, in the
    /// written order. A constant is left in place, and so is a read of an
    /// owner: the read moves it, and the call cannot change the place
    /// without a borrow the check refuses. A read of a place whose kind
    /// holds text goes there as a duplicate, since the call can change the
    /// place; such a slot starts blank and joins the drops of its scope.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    ///
    /// # Returns
    ///
    /// The read of the slot, or the constant.
    fn spill(&mut self, e: Expr) -> Expr
    {
        let read_place = matches!(e.node, ExprNode::Read(_) | ExprNode::Load { .. });
        ret_if!(matches!(e.node, ExprNode::Const(_)), e);
        let (kind, span, place) = (e.kind, e.span, self.hidden(e.kind, e.span));
        let value = tern!(read_place && kind.holds_text(), dup(e), e);
        if kind.holds_text()
        {
            let blank = default_constant(kind, span);
            self.pre_init.push(Stmt::Set { place, value: blank, span });
            self.drops.last_mut().expect("a scope is open").push((place, kind));
        }
        self.pre.push(Stmt::Set { place, value, span });
        read(place, kind, span)
    }

    /// Lower an expression with the kind the check recorded for it.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The expression.
    fn expr_node(&mut self, e: &ast::Expr) -> Expr
    {
        let kind = concrete(self.kind_of(e));
        let node = match e
        {
            ast::Expr::Int(n) => ExprNode::Const(self.int_lit(n, kind, false)),
            ast::Expr::Float(n) => ExprNode::Const(float_lit(n, kind)),
            ast::Expr::Char(n) => ExprNode::Const(Value::Char(n.value)),
            ast::Expr::Bool(n) => ExprNode::Const(Value::Bool(n.value)),
            ast::Expr::Name(n) => self.read_name(&n.name, n.span),
            ast::Expr::Group(n) => return self.expr(&n.inner),
            ast::Expr::Prefix(n) => self.prefix(n, kind),
            ast::Expr::Infix(n) => self.infix(n.op, &n.lhs, &n.rhs, kind),
            ast::Expr::Not(n) => prefix_node(PrefixOp::Not, self.expr(&n.inner)),
            ast::Expr::Logic(n) => self.logic(n),
            ast::Expr::Verb(n) => self.verb(n, kind),
            ast::Expr::Call(n) if n.pass => self.pass_call(n),
            ast::Expr::Call(n) => self.call_node(n, kind),
            ast::Expr::FnAddr(n) if self.nested_named(&n.name.name).is_some() =>
            {
                self.nested_addr(&n.name).expect("the function is declared")
            }
            ast::Expr::FnAddr(n) =>
            {
                let sig = self.sigs.get(&n.name.name);
                let sig = sig.unwrap_or_else(|| {
                    panic!("function {} is not declared", n.name.name)
                });
                ExprNode::FnAddr { func: sig.index, name: n.name.name.to_string() }
            }
            ast::Expr::Null(n) if n.initial =>
            {
                self.initial_value(self.shown_kind(e), n.span).node
            }
            ast::Expr::Null(n) if n.blank =>
            {
                let none = Fault::of_kind(kind).expect("the check found an optional");
                none.value(true, None, None, self.platform, n.span).node
            }
            ast::Expr::Null(_) => ExprNode::Const(Value::NULL),
            ast::Expr::SizeOf(n) =>
            {
                ExprNode::SizeOf(DataKind::from_ast(&n.kind, self.structs))
            }
            ast::Expr::Index(n) => ExprNode::Load { addr: Box::new(self.cell_addr(n)) },
            ast::Expr::Field(n) if self.kind_of(&n.base).as_external().is_some() =>
            {
                self.c_field_read(n, kind)
            }
            ast::Expr::Field(n) =>
            {
                ExprNode::Load { addr: Box::new(self.field_addr(n)) }
            }
            ast::Expr::Range(n) => self.range(n, kind),
            ast::Expr::List(n) => self.list(n, kind),
            ast::Expr::Struct(n) => self.struct_lit(n, kind),
            ast::Expr::BlankList(_) => ExprNode::BlankList,
            ast::Expr::Table(n) => self.table_lit(n, kind),
            ast::Expr::Str(n) => self.str_node(n),
            ast::Expr::Member(n) =>
            {
                let owner = self.shown_kind(e).as_enum();
                let owner = owner.expect("the check gives a member its enumeration");
                let v =
                    owner.member(&n.member.name).expect("the check found the member");
                return member_expr(v, owner.under(), n.span);
            }
        };
        Expr::builder().kind(kind).node(node).span(e.span()).build()
    }

    /// The value of an integer literal of the recorded kind.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    /// * `kind` - The recorded kind, an integer or a float.
    /// * `negated` - Whether a negate operator stands before it.
    ///
    /// # Returns
    ///
    /// The value.
    fn int_lit(&self, n: &ast::IntLit, kind: DataKind, negated: bool) -> Value
    {
        let DataKind::Int(k) = kind
        else
        {
            let text = tern!(negated, format!("-{}", n.value), n.value.to_string());
            let bits = kind.bits(self.platform);
            return Value::float(&text, bits).unwrap_or(Value::default_of(kind));
        };
        let pattern = tern!(negated, n.value.wrapping_neg(), n.value);
        Value::int(k, pattern, self.platform)
    }

    /// Lower a prefix operator. `-` before an integer literal becomes a
    /// negative constant of the literal's kind; `&` and `*` become address
    /// and load nodes.
    ///
    /// # Arguments
    ///
    /// * `n` - The expression.
    /// * `kind` - Its recorded kind.
    ///
    /// # Returns
    ///
    /// The operation.
    fn prefix(&mut self, n: &ast::PrefixExpr, kind: DataKind) -> ExprNode
    {
        let lit = match n.rhs.as_ref()
        {
            ast::Expr::Int(lit) if n.op == ast::PrefixOp::Neg => Some(lit),
            _ => None,
        };
        ret_if!(let Some(lit) = lit, ExprNode::Const(self.int_lit(lit, kind, true)));
        let op = match n.op
        {
            ast::PrefixOp::Neg => PrefixOp::Neg,
            ast::PrefixOp::BitNot => PrefixOp::BitNot,
            ast::PrefixOp::Not => PrefixOp::Not,
            ast::PrefixOp::AddrOf => return self.addr_of(&n.rhs),
            ast::PrefixOp::Deref => return self.load(&n.rhs),
        };
        prefix_node(op, self.expr(&n.rhs))
    }

    /// Lower a notation operator.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    /// * `kind` - The recorded kind of the operation.
    ///
    /// # Returns
    ///
    /// The operation.
    fn infix(
        &mut self, op: ast::InfixOp, lhs: &ast::Expr, rhs: &ast::Expr, kind: DataKind,
    ) -> ExprNode
    {
        let cmp = match op
        {
            ast::InfixOp::Eq => Some(CmpOp::Eq),
            ast::InfixOp::Ne => Some(CmpOp::Ne),
            ast::InfixOp::Lt => Some(CmpOp::Lt),
            ast::InfixOp::Gt => Some(CmpOp::Gt),
            ast::InfixOp::Le => Some(CmpOp::Le),
            ast::InfixOp::Ge => Some(CmpOp::Ge),
            _ => None,
        };
        ret_if!(let Some(cmp) = cmp, self.compare(cmp, lhs, rhs));
        let op = match op
        {
            ast::InfixOp::Add => InfixOp::Add,
            ast::InfixOp::Sub => InfixOp::Sub,
            ast::InfixOp::Mul => InfixOp::Mul,
            ast::InfixOp::Div => InfixOp::Div,
            ast::InfixOp::Rem => InfixOp::Rem,
            ast::InfixOp::BitAnd => InfixOp::BitAnd,
            ast::InfixOp::BitOr => InfixOp::BitOr,
            ast::InfixOp::BitXor => InfixOp::BitXor,
            ast::InfixOp::Shl => InfixOp::Shl,
            _ => InfixOp::Shr,
        };
        self.arith(op, lhs, rhs, kind)
    }

    /// Lower an arithmetic, bitwise or shift operation. Both operands take
    /// the result kind, except a shift distance, which keeps its own. With
    /// a pointer operand it is pointer arithmetic.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// The operation.
    fn arith(
        &mut self, op: InfixOp, lhs: &ast::Expr, rhs: &ast::Expr, kind: DataKind,
    ) -> ExprNode
    {
        let ptr = self.kind_of(lhs).is_ptr() || self.kind_of(rhs).is_ptr();
        ret_if!(ptr, self.ptr_arith(op, lhs, rhs));
        let pieces = |l: &mut Self| [lhs, rhs].map(|e| FmtPiece::Value(l.expr(e)));
        ret_if!(kind.is_text(), ExprNode::Format(pieces(self).to_vec()));
        let lhs = self.coerce(lhs, kind);
        let shift = matches!(op, InfixOp::Shl | InfixOp::Shr);
        let rhs = tern!(shift, self.expr(rhs), self.coerce(rhs, kind));
        ExprNode::Infix { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }
    }

    /// Lower pointer arithmetic: a pointer and an integer move the pointer,
    /// two pointers give their distance. The integer keeps its own kind,
    /// except when it is subtracted, where it is negated as an `isize`.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator, `+` or `-`.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The operation.
    fn ptr_arith(&mut self, op: InfixOp, lhs: &ast::Expr, rhs: &ast::Expr) -> ExprNode
    {
        let both = self.kind_of(lhs).is_ptr() && self.kind_of(rhs).is_ptr();
        ret_if!(both, ptr_sub(self.expr(lhs), self.expr(rhs)));
        let (base, offset) = tern!(self.kind_of(lhs).is_ptr(), (lhs, rhs), (rhs, lhs));
        let base = Box::new(self.expr(base));
        let forward = op == InfixOp::Add;
        let offset = tern!(forward, self.expr(offset), self.negated(offset));
        ExprNode::PtrAdd { base, offset: Box::new(offset) }
    }

    /// An integer converted to `isize` and negated.
    ///
    /// # Arguments
    ///
    /// * `e` - The integer.
    ///
    /// # Returns
    ///
    /// The negation.
    fn negated(&mut self, e: &ast::Expr) -> Expr
    {
        let operand = self.coerce(e, DataKind::ISIZE);
        let span = operand.span;
        let node = prefix_node(PrefixOp::Neg, operand);
        Expr::builder().kind(DataKind::ISIZE).node(node).span(span).build()
    }

    /// Lower an address-of. The address of a name marks its slot
    /// `addressed`; the address of a dereference is the pointer itself.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    ///
    /// # Returns
    ///
    /// The operation.
    fn addr_of(&mut self, place: &ast::Expr) -> ExprNode
    {
        match self.dest(place)
        {
            Dest::Slot(at) =>
            {
                self.slot(at).addressed = true;
                ExprNode::AddrOf(at)
            }
            Dest::Mem(addr) => pinned(addr).node,
        }
    }

    /// Lower a dereference read.
    ///
    /// # Arguments
    ///
    /// * `ptr` - The pointer.
    ///
    /// # Returns
    ///
    /// The load.
    fn load(&mut self, ptr: &ast::Expr) -> ExprNode
    {
        ExprNode::Load { addr: Box::new(self.expr(ptr)) }
    }

    /// Lower a list literal; each cell takes the cell kind.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    /// * `kind` - Its recorded list kind.
    ///
    /// # Returns
    ///
    /// The operation.
    fn list(&mut self, n: &ast::ListLit, kind: DataKind) -> ExprNode
    {
        let cell = kind.cell_kind().expect("a list literal has a list kind");
        let cells =
            ExprNode::List(n.cells.iter().map(|c| self.owned(c, cell)).collect());
        ret_if!(!kind.is_seq(), cells);
        // A plain list is built as a counted list, whose cells then sit in
        // static data or move to a heap block.
        let counted = DataKind::list_of(cell, n.cells.len() as u32);
        let operand =
            Box::new(Expr::builder().kind(counted).node(cells).span(n.span).build());
        ExprNode::Convert { operand, implicit: true }
    }

    /// Lower a struct literal to its fields in order; each field it leaves
    /// out holds its default.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    /// * `kind` - Its struct kind.
    ///
    /// # Returns
    ///
    /// The operation.
    fn struct_lit(&mut self, n: &ast::StructLit, kind: DataKind) -> ExprNode
    {
        if let Some(s) = kind.as_object()
        {
            // An object is a new box that holds the struct.
            let fields = self.struct_lit(n, DataKind::Struct(s));
            let value = node_expr(DataKind::Struct(s), fields, n.span);
            return self.make_box(value, n.span).node;
        }
        let s = kind.as_struct().expect("a struct literal has a struct kind");
        let init = |name: &str| n.fields.iter().find(|f| f.field.name == name);
        let fields = s.fields().iter().map(|(name, k)| {
            init(name).map_or_else(
                || default_constant(*k, n.span),
                |f| self.owned(&f.value, *k),
            )
        });
        ExprNode::List(fields.collect())
    }

    /// The address of one cell of a list or a slice, or of the list a
    /// pointer points at.
    ///
    /// # Arguments
    ///
    /// * `n` - The cell expression.
    ///
    /// # Returns
    ///
    /// The address, a pointer to the cell kind.
    fn cell_addr(&mut self, n: &ast::IndexExpr) -> Expr
    {
        let (cell, len) = cells_of(self.kind_of(&n.list));
        let list = Box::new(self.expr(&n.list));
        let index = Box::new(self.expr(&n.index));
        let kind = cell.pointer_to();
        #[rustfmt::skip]
        let node = ExprNode::CellAddr { list, index, len, first: n.first, checked: false };
        Expr::builder().kind(kind).node(node).span(n.span).build()
    }

    /// A range of cells: a list or text of its own, with the cells of a
    /// list, a slice, text or a block copied in. The cells are read through
    /// a view that lends them (a room of 0), which `own` copies.
    ///
    /// # Arguments
    ///
    /// * `n` - The range.
    /// * `kind` - Its kind: text, or a plain list.
    ///
    /// # Returns
    ///
    /// The operation.
    fn range(&mut self, n: &ast::RangeExpr, kind: DataKind) -> ExprNode
    {
        let from = slice_from(self.kind_of(&n.source));
        let value = self.expr(&n.source);
        // A value that is no place, such as a literal, waits in a slot.
        let place = matches!(value.node, ExprNode::Read(_) | ExprNode::Load { .. });
        let value = tern!(place || value.kind.is_ptr(), value, self.spill(value));
        let source = Box::new(through(value));
        let start = Box::new(self.coerce(&n.start, DataKind::USIZE));
        let len = Box::new(self.coerce(&n.len, DataKind::USIZE));
        let view = ExprNode::MakeSlice { source, start, len, from, checked: false };
        ExprNode::Own { value: Box::new(node_expr(kind, view, n.span)), pin: false }
    }

    /// A string literal: static text, or text built from its template slots.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    ///
    /// # Returns
    ///
    /// The node.
    fn str_node(&mut self, n: &ast::StrExpr) -> ExprNode
    {
        ret_if!(let Some(text) = n.plain(), ExprNode::Text(text.to_string()));
        let mut pieces = Vec::with_capacity(n.pieces.len());
        for piece in &n.pieces
        {
            pieces.push(match piece
            {
                ast::StrPiece::Text(text) => FmtPiece::Text(text.clone()),
                ast::StrPiece::TemplateSlot(e) =>
                {
                    let lowered = self.expr(e);
                    FmtPiece::Value(self.shown(e, lowered))
                }
            });
        }
        ExprNode::Format(pieces)
    }

    /// A `usize` constant.
    ///
    /// # Arguments
    ///
    /// * `n` - The value.
    /// * `span` - Its source region.
    ///
    /// # Returns
    ///
    /// The expression.
    fn usize_const(&self, n: u32, span: Span) -> Expr
    {
        let usize = DataKind::USIZE.int_kind().expect("usize is an integer");
        let value = Value::int(usize, u128::from(n), self.platform);
        Expr::builder()
            .kind(DataKind::USIZE)
            .node(ExprNode::Const(value))
            .span(span)
            .build()
    }

    /// The address of one field of a struct.
    ///
    /// # Arguments
    ///
    /// * `n` - The field expression.
    ///
    /// # Returns
    ///
    /// The address, a pointer to the field kind.
    ///
    /// # Panics
    ///
    /// When the struct has no such field, which a clean check rules out.
    fn field_addr(&mut self, n: &ast::FieldExpr) -> Expr
    {
        let of = self.kind_of(&n.base);
        let shape = of.object_struct().expect("a field is read from a struct");
        let (at, kind) = shape.field(&n.field.name).expect("the check found the field");
        let offset = shape.offset(at, self.platform);
        let base = self.expr(&n.base);
        let base =
            Box::new(tern!(of.as_object().is_some(), self.object_fields(base), base));
        let field = u32::try_from(at).expect("a struct has fewer than 2^32 fields");
        let node = ExprNode::FieldAddr { base, field, offset };
        Expr::builder().kind(kind.repr().pointer_to()).node(node).span(n.span).build()
    }

    /// The cell count of a list, of the list a pointer points at, or of a
    /// slice.
    ///
    /// # Arguments
    ///
    /// * `value` - The list or the slice.
    ///
    /// # Returns
    ///
    /// A `usize` constant for a list; for a slice, a read of its second word.
    fn cell_count(&mut self, value: &ast::Expr) -> ExprNode
    {
        let (_, len) = cells_of(self.kind_of(value));
        ret_if!(let Some(len) = len, self.usize_const(len, value.span()).node);
        let base = self.expr(value);
        self.count_word(base, value.span())
    }

    /// The read of the second word of a slice or of text: its count.
    ///
    /// # Arguments
    ///
    /// * `base` - The slice or the text.
    /// * `span` - The source region of the count.
    ///
    /// # Returns
    ///
    /// The load of the count, a `usize`.
    fn count_word(&self, base: Expr, span: Span) -> ExprNode
    {
        let offset = u32::from(self.platform) / 8;
        let node = ExprNode::FieldAddr { base: Box::new(base), field: 1, offset };
        let kind = DataKind::USIZE.pointer_to();
        let addr = Expr::builder().kind(kind).node(node).span(span).build();
        ExprNode::Load { addr: Box::new(addr) }
    }

    /// Lower a comparison; both operands take their common kind, which is
    /// `ptr` for two `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The comparison.
    fn compare(&mut self, op: CmpOp, lhs: &ast::Expr, rhs: &ast::Expr) -> ExprNode
    {
        // Text and a view of text compare their u8s, each in its own kind.
        let texts = self.kind_of(lhs).is_text() && self.kind_of(rhs).is_text();
        ret_if!(
            texts,
            ExprNode::Compare {
                op,
                lhs: Box::new(self.expr(lhs)),
                rhs: Box::new(self.expr(rhs))
            }
        );
        let common = concrete(self.common(self.kind_of(lhs), self.kind_of(rhs)));
        let lhs = Box::new(self.coerce(lhs, common));
        let rhs = Box::new(self.coerce(rhs, common));
        ExprNode::Compare { op, lhs, rhs }
    }

    /// A duplicate of a value. A value is duplicated as it is read; a shallow
    /// duplicate of an object is a new box that holds a duplicate of its
    /// struct; a deep duplicate gives each object inside a block of its own,
    /// each duplicated once.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    /// * `deep` - Whether the duplicate is deep.
    ///
    /// # Returns
    ///
    /// The duplicate.
    fn dup_of(&mut self, value: &ast::Expr, deep: bool) -> ExprNode
    {
        let kind = self.kind_of(value);
        if deep
        {
            let read = Box::new(self.expr(value));
            return ExprNode::Dup { value: read, lent: false, deep: true };
        }
        ret_if!(kind.as_object().is_none(), self.owned(value, kind).node);
        let object = self.expr(value);
        let fields = dup(self.object_fields(object));
        self.make_box(fields, value.span()).node
    }

    /// Lower the sameness verb: two function values compare as words, and two
    /// objects compare the first words of their boxes, the blocks held there.
    ///
    /// # Arguments
    ///
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The comparison.
    fn same(&mut self, lhs: &ast::Expr, rhs: &ast::Expr) -> ExprNode
    {
        let common = concrete(self.common(self.kind_of(lhs), self.kind_of(rhs)));
        let (l, r) = (self.coerce(lhs, common), self.coerce(rhs, common));
        let word = |e: Expr| {
            let span = e.span;
            let node = ExprNode::FieldAddr { base: Box::new(e), field: 0, offset: 0 };
            let addr = node_expr(DataKind::OPAQUE.pointer_to(), node, span);
            node_expr(DataKind::OPAQUE, ExprNode::Load { addr: Box::new(addr) }, span)
        };
        let (l, r) = tern!(common.is_fn(), (l, r), (word(l), word(r)));
        ExprNode::Compare { op: CmpOp::Eq, lhs: Box::new(l), rhs: Box::new(r) }
    }

    /// Lower a logical operator. A right side that needs statements first,
    /// such as a call that passes a failure on or copies values back, goes
    /// through a hidden slot: the left side sets it, and the right side runs
    /// in an `if` when the left side does not settle the value, and at no
    /// other time.
    ///
    /// # Arguments
    ///
    /// * `n` - The expression.
    ///
    /// # Returns
    ///
    /// The operation, or the read of the slot.
    fn logic(&mut self, n: &ast::LogicExpr) -> ExprNode
    {
        let op = tern!(n.op == ast::LogicOp::And, LogicOp::And, LogicOp::Or);
        let span = n.span;
        let lhs = self.coerce(&n.lhs, DataKind::Bool);
        let (mut then, rhs) = self.with_pre(|l| l.coerce(&n.rhs, DataKind::Bool));
        let plain =
            |lhs, rhs| ExprNode::Logic { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        ret_if!(then.is_empty(), plain(lhs, rhs));
        let place = self.hidden(DataKind::Bool, span);
        self.pre.push(Stmt::Set { place, value: lhs, span });
        then.push(Stmt::Set { place, value: rhs, span });
        let held = read(place, DataKind::Bool, span);
        let not_held =
            node_expr(DataKind::Bool, prefix_node(PrefixOp::Not, held.clone()), span);
        let cond = tern!(op == LogicOp::And, held, not_held);
        self.pre.push(Stmt::If { cond, then, els: Vec::new(), span });
        ExprNode::Read(place)
    }

    /// Lower a verbal clause to the same operations as the notation.
    ///
    /// # Arguments
    ///
    /// * `n` - The clause.
    /// * `kind` - Its recorded kind.
    ///
    /// # Returns
    ///
    /// The operation.
    fn verb(&mut self, n: &ast::VerbExpr, kind: DataKind) -> ExprNode
    {
        let args = n.args.as_slice();
        let op = match n.op
        {
            VerbOp::Add => Some(InfixOp::Add),
            VerbOp::Sub => Some(InfixOp::Sub),
            VerbOp::Mul => Some(InfixOp::Mul),
            VerbOp::Div => Some(InfixOp::Div),
            VerbOp::Rem => Some(InfixOp::Rem),
            VerbOp::BitAnd => Some(InfixOp::BitAnd),
            VerbOp::BitOr => Some(InfixOp::BitOr),
            VerbOp::BitXor => Some(InfixOp::BitXor),
            VerbOp::Shift =>
            {
                Some(tern!(n.dir == Some(Dir::Right), InfixOp::Shr, InfixOp::Shl))
            }
            _ => None,
        };
        ret_if!(let (Some(op), [l, r]) = (op, args), self.arith(op, l, r, kind));
        let cmp = match n.op
        {
            VerbOp::Eq => Some(CmpOp::Eq),
            VerbOp::Ne => Some(CmpOp::Ne),
            VerbOp::Gt => Some(CmpOp::Gt),
            VerbOp::Le => Some(CmpOp::Le),
            VerbOp::Lt => Some(CmpOp::Lt),
            VerbOp::Ge => Some(CmpOp::Ge),
            _ => None,
        };
        ret_if!(let (Some(cmp), [l, r]) = (cmp, args), self.compare(cmp, l, r));
        ret_if!(let (VerbOp::Same, [l, r]) = (n.op, args), self.same(l, r));
        if n.op == VerbOp::Join
        {
            let pieces = args.iter().map(|a| FmtPiece::Value(self.expr(a))).collect();
            return ExprNode::Format(pieces);
        }
        ret_if!(n.op == VerbOp::RowValue, self.row_value(n, kind));
        ret_if!(n.op == VerbOp::OrElse, self.or_else(n, kind));
        if let (VerbOp::CharAt | VerbOp::CharNth, [text, index]) = (n.op, args)
        {
            let (text, index) = (self.expr(text), self.expr(index));
            let first = u8::from(n.op == VerbOp::CharNth);
            return ExprNode::Chars {
                text: Box::new(text),
                index: Some(Box::new(index)),
                first,
                octet: false,
            };
        }
        let [value] = args
        else
        {
            panic!("verb {} has {} operands", n.op.as_ref(), args.len());
        };
        match n.op
        {
            VerbOp::AddrOf => self.addr_of(value),
            VerbOp::Deref => self.load(value),
            VerbOp::CellCount => self.cell_count(value),
            VerbOp::RowCount => self.row_count(value),
            VerbOp::CharCount => ExprNode::Chars {
                text: Box::new(self.expr(value)),
                index: None,
                first: 0,
                octet: false,
            },
            VerbOp::BitNot => prefix_node(PrefixOp::BitNot, self.expr(value)),
            VerbOp::DupOf => self.dup_of(value, false),
            VerbOp::DeepDupOf => self.dup_of(value, true),
            VerbOp::Cast =>
            {
                let operand = self.expr(value);
                // a member turned into its value kind is its value
                ret_if!(operand.kind == kind, operand.node);
                ExprNode::Convert { operand: Box::new(operand), implicit: false }
            }
            VerbOp::IsGood | VerbOp::IsFault | VerbOp::HasData | VerbOp::LacksData =>
            {
                self.result_test(n.op, value, n.span)
            }
            VerbOp::MakeFault => self.make_fault(value, kind, n.span),
            VerbOp::Unwrap | VerbOp::UnwrapFault =>
            {
                self.result_part(n.op, value, n.span)
            }
            VerbOp::Add
            | VerbOp::Sub
            | VerbOp::Mul
            | VerbOp::Div
            | VerbOp::Rem
            | VerbOp::BitAnd
            | VerbOp::BitOr
            | VerbOp::BitXor
            | VerbOp::Shift
            | VerbOp::Eq
            | VerbOp::Same
            | VerbOp::Ne
            | VerbOp::Gt
            | VerbOp::Le
            | VerbOp::Lt
            | VerbOp::Ge
            | VerbOp::Print
            | VerbOp::Assign
            | VerbOp::DupInto
            | VerbOp::DeepDupInto
            | VerbOp::Return
            | VerbOp::AddCell
            | VerbOp::Replace
            | VerbOp::Cell
            | VerbOp::CharAt
            | VerbOp::CharNth
            | VerbOp::Join
            | VerbOp::Row
            | VerbOp::RowValue
            | VerbOp::OrElse
            | VerbOp::RemoveRow =>
            {
                panic!("verb {} is no one-operand value", n.op.as_ref())
            }
        }
    }
}

/// Convert a lowered expression to `want` when its kind differs. A `မည်သည်ကိုမှ
/// မညွှန်းသော လိပ်စာ` constant headed for a pointer kind, and an address headed for
/// a kind of its own shape with another borrow mark, take `want` as their label
/// with no conversion; other addresses keep their own kind, which pointer math
/// reads its stride from, under an implicit conversion.
/// A plain value headed for an optional or a result goes in as the one that
/// holds it on its good side.
///
/// # Arguments
///
/// * `lowered` - The expression.
/// * `want` - The kind its place expects.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The expression, of kind `want`.
fn coerce_lowered(lowered: Expr, want: DataKind, platform: u16) -> Expr
{
    let want = want.repr();
    ret_if!(lowered.kind == want, lowered);
    let span = lowered.span;
    let plain = lowered.kind.fallible_parts().is_none();
    if let Some(wrap) = Fault::of_kind(want).filter(|f| plain && f.ok.is_some())
    {
        let held = wrap.ok.map(|k| coerce_lowered(lowered, k, platform));
        return wrap.value(false, held, None, platform, span);
    }
    // `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` headed for an object is the blank box.
    let constant = matches!(lowered.node, ExprNode::Const(_));
    let null_object =
        constant && lowered.kind.is_address() && want.as_object().is_some();
    ret_if!(
        null_object,
        Expr::builder().kind(want).node(ExprNode::BlankList).span(span).build()
    );
    let literal = matches!(lowered.node, ExprNode::Const(_));
    let null = literal && lowered.kind.is_ptr() && want.is_ptr();
    let relabel = null || lowered.kind.same_shape(want);
    ret_if!(relabel, Expr::builder().kind(want).node(lowered.node).span(span).build());
    let node = ExprNode::Convert { operand: Box::new(lowered), implicit: true };
    Expr::builder().kind(want).node(node).span(span).build()
}

/// The kind C gives a value it reads through `...`: an integer narrower than
/// 32 bits and a boolean become an `i32`, an `f32` an `f64`, as C promotes
/// them; each other kind is left as it is.
///
/// # Arguments
///
/// * `kind` - The kind of the value.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The kind C reads.
fn promoted(kind: DataKind, platform: u16) -> DataKind
{
    let i32 = DataKind::Int(IntKind::new(true, Width::Bits(32)));
    match kind
    {
        DataKind::Int(n) if n.bits(platform) < 32 => i32,
        DataKind::Bool => i32,
        DataKind::Float(32) => DataKind::F64,
        _ => kind,
    }
}

/// Move a value out of the place it is read from: a slot through its
/// address, a value in mem through the address it is loaded from. A
/// constant or a computed value has no place and is kept as it is. The
/// slot is not marked `addressed`.
///
/// # Arguments
///
/// * `e` - The value.
///
/// # Returns
///
/// The value, of its own kind.
fn take(e: Expr) -> Expr
{
    let (kind, span) = (e.kind, e.span);
    let node = match e.node
    {
        ExprNode::Read(place) =>
        {
            let at = ExprNode::AddrOf(place);
            let addr =
                Expr::builder().kind(kind.pointer_to()).node(at).span(span).build();
            ExprNode::Take { addr: Box::new(addr) }
        }
        ExprNode::Load { addr } => ExprNode::Take { addr },
        other => other,
    };
    Expr::builder().kind(kind).node(node).span(span).build()
}

/// Duplicate a value read out of a place, so each text and plain list in it
/// whose cells sit in a heap block gets a block of its own.
///
/// # Arguments
///
/// * `e` - The read.
///
/// # Returns
///
/// The duplicate, of the read's kind.
fn dup(e: Expr) -> Expr
{
    let (kind, span) = (e.kind, e.span);
    let node = ExprNode::Dup { value: Box::new(e), lent: false, deep: false };
    Expr::builder().kind(kind).node(node).span(span).build()
}

/// The value of a round of an each-cell loop: a duplicate of the cell.
///
/// # Arguments
///
/// * `each` - The slot of the list or the slice.
/// * `kind` - Its kind.
/// * `at` - The slot of the number of the cell.
/// * `span` - The source region of the loop variable.
///
/// # Returns
///
/// The value and the cell kind.
fn each_cell(each: Place, kind: DataKind, at: Place, span: Span) -> (Expr, DataKind)
{
    let (cell, len) = cells_of(kind);
    let list = Box::new(read(each, kind, span));
    let index = Box::new(read(at, DataKind::USIZE, span));
    let node = ExprNode::CellAddr { list, index, len, first: 0, checked: false };
    let addr = node_expr(cell.pointer_to(), node, span);
    let load = node_expr(cell, ExprNode::Load { addr: Box::new(addr) }, span);
    (tern!(cell.holds_text(), dup(load), load), cell)
}

/// The value of a round of an each-character loop: the character that
/// starts at the u8.
///
/// # Arguments
///
/// * `each` - The slot of the text.
/// * `kind` - Its kind.
/// * `at` - The slot of the number of the u8.
/// * `span` - The source region of the loop variable.
///
/// # Returns
///
/// The value and its kind, a character.
fn each_char(each: Place, kind: DataKind, at: Place, span: Span) -> (Expr, DataKind)
{
    let text = Box::new(read(each, kind, span));
    let index = Some(Box::new(read(at, DataKind::USIZE, span)));
    let node = ExprNode::Chars { text, index, first: 0, octet: true };
    (node_expr(DataKind::Char, node, span), DataKind::Char)
}

/// The address of a place about to be written, with each plain list on the
/// path to it made to own its block first, the outer one first.
///
/// # Arguments
///
/// * `addr` - The address of the place.
///
/// # Returns
///
/// The address.
fn owned_path(addr: Expr) -> Expr
{
    let (kind, span) = (addr.kind, addr.span);
    let node = match addr.node
    {
        ExprNode::CellAddr { list, index, len, first, checked } =>
        {
            let inner = through(owned_path(*list));
            let list = tern!(inner.kind.is_seq(), own(inner), inner);
            ExprNode::CellAddr { list: Box::new(list), index, len, first, checked }
        }
        ExprNode::FieldAddr { base, field, offset } =>
        {
            ExprNode::FieldAddr { base: Box::new(owned_path(*base)), field, offset }
        }
        ExprNode::Load { addr } => ExprNode::Load { addr: Box::new(owned_path(*addr)) },
        other => other,
    };
    node_expr(kind, node, span)
}

/// A list literal read where it is: a printed literal needs no heap block,
/// so it keeps the counted list it is built as.
///
/// # Arguments
///
/// * `e` - The value.
///
/// # Returns
///
/// The counted list for a literal of a plain list, else the value.
fn counted_literal(e: Expr) -> Expr
{
    let literal = |o: &Expr| matches!(o.node, ExprNode::List(_)) && o.kind.is_list();
    match e.node
    {
        ExprNode::Convert { operand, .. } if e.kind.is_seq() && literal(&operand) =>
        {
            *operand
        }
        node => Expr::builder().kind(e.kind).node(node).span(e.span).build(),
    }
}

/// The plain list a pointer points at, read in place; each other value as
/// it is.
///
/// # Arguments
///
/// * `e` - The value.
///
/// # Returns
///
/// The expression.
fn through(e: Expr) -> Expr
{
    let Some(seq) = e.kind.pointee().filter(|k| k.is_seq())
    else
    {
        return e;
    };
    let span = e.span;
    node_expr(seq, ExprNode::Load { addr: Box::new(e) }, span)
}

/// A plain list made to own its block before a write. A box, of an object or
/// of a variable a closure shares, is shared on purpose, so it remains.
///
/// # Arguments
///
/// * `list` - The list, a place.
///
/// # Returns
///
/// The expression, of the list kind.
fn own(list: Expr) -> Expr
{
    ret_if!(list.kind.is_box(), list);
    let (kind, span) = (list.kind, list.span);
    node_expr(kind, ExprNode::Own { value: Box::new(list), pin: false }, span)
}

/// The text, the number and the first number of a character place:
/// `t ၏ စာလုံး n` (from 0) or `t ၏ n ခုမြောက်စာလုံး` (from 1).
///
/// # Arguments
///
/// * `e` - The place.
///
/// # Returns
///
/// The parts, or `None` for each other place.
fn char_place(e: &ast::Expr) -> Option<(&ast::Expr, &ast::Expr, u8)>
{
    let ast::Expr::Verb(v) = ungrouped(e)
    else
    {
        return None;
    };
    let (VerbOp::CharAt | VerbOp::CharNth, [text, index]) = (v.op, v.args.as_slice())
    else
    {
        return None;
    };
    Some((text, index, u8::from(v.op == VerbOp::CharNth)))
}

/// An expression of a kind.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `node` - The operation.
/// * `span` - Its source region.
///
/// # Returns
///
/// The expression.
fn node_expr(kind: DataKind, node: ExprNode, span: Span) -> Expr
{
    Expr::builder().kind(kind).node(node).span(span).build()
}

/// The read of a slot.
///
/// # Arguments
///
/// * `place` - The slot.
/// * `kind` - Its kind.
/// * `span` - The source region.
///
/// # Returns
///
/// The read.
fn read(place: Place, kind: DataKind, span: Span) -> Expr
{
    node_expr(kind, ExprNode::Read(place), span)
}

/// A comparison of two operands of one kind.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `lhs` - The left operand.
/// * `rhs` - The right operand.
///
/// # Returns
///
/// The operation, a boolean.
fn compare_node(op: CmpOp, lhs: Expr, rhs: Expr) -> ExprNode
{
    ExprNode::Compare { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }
}

/// An arithmetic operation on two operands of one kind.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `lhs` - The left operand.
/// * `rhs` - The right operand.
///
/// # Returns
///
/// The operation.
fn infix_node(op: InfixOp, lhs: Expr, rhs: Expr) -> ExprNode
{
    ExprNode::Infix { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }
}

/// The count of u8s that a character takes in UTF-8: one, plus one for
/// each of the bounds U+0080, U+0800 and U+10000 it reaches.
///
/// # Arguments
///
/// * `c` - The character.
///
/// # Returns
///
/// The count, a `usize`.
fn char_width(c: Expr) -> Expr
{
    let span = c.span;
    let reaches = |bound: char| {
        let at = constant(Value::Char(bound), span);
        let test = compare_node(CmpOp::Ge, c.clone(), at);
        let operand = Box::new(node_expr(DataKind::Bool, test, span));
        let node = ExprNode::Convert { operand, implicit: true };
        node_expr(DataKind::USIZE, node, span)
    };
    let usize = DataKind::USIZE.int_kind().expect("usize is an integer");
    let start = constant(Value::Int { kind: usize, raw: 1 }, span);
    let bounds = ['\u{80}', '\u{800}', '\u{10000}'].into_iter();
    bounds.fold(start, |sum, bound| {
        node_expr(DataKind::USIZE, infix_node(InfixOp::Add, sum, reaches(bound)), span)
    })
}

/// A constant expression of the value's kind.
///
/// # Arguments
///
/// * `v` - The value.
/// * `span` - Its source region.
///
/// # Returns
///
/// The expression.
pub fn constant(v: Value, span: Span) -> Expr
{
    Expr::builder().kind(v.kind()).node(ExprNode::Const(v)).span(span).build()
}

/// The default value of a kind as a constant labelled with that kind, so a
/// pointer slot starts at a the zero word of its own pointer kind.
///
/// # Arguments
///
/// * `kind` - The kind of the slot.
/// * `span` - Its source region.
///
/// # Returns
///
/// The expression.
fn default_constant(kind: DataKind, span: Span) -> Expr
{
    ret_if!(let Some(e) = kind.as_enum(), member_expr(e.first(), e.under(), span));
    ret_if!(kind.holds_enum(), defaults_of_parts(kind, span));
    let node = tern!(
        kind.is_aggregate(),
        ExprNode::BlankList,
        ExprNode::Const(Value::default_of(kind))
    );
    Expr::builder().kind(kind).node(node).span(span).build()
}

/// Whether the initial value of a kind is zero octets: a number, a
/// character, true/false, a pointer, text, a plain list, and a counted list
/// or a struct of those alone.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for such a kind.
fn zero_initial(kind: DataKind) -> bool
{
    ret_false!(kind.is_optional() || kind.as_enum().is_some() || kind.is_box());
    ret_if!(let Some(l) = kind.as_list(), zero_initial(l.cell));
    let Some(s) = kind.as_struct()
    else
    {
        return !kind.is_fn();
    };
    s.fields().iter().all(|(_, k)| zero_initial(*k))
}

/// A prefix operation node.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `operand` - The operand.
///
/// # Returns
///
/// The node.
fn prefix_node(op: PrefixOp, operand: Expr) -> ExprNode
{
    ExprNode::Prefix { op, operand: Box::new(operand) }
}

/// The kind an IR expression carries for a recorded kind: `null` becomes
/// `ptr`, the kind of [`Value::NULL`]; other kinds are kept.
///
/// # Arguments
///
/// * `kind` - The recorded kind.
///
/// # Returns
///
/// The kind.
fn concrete(kind: DataKind) -> DataKind
{
    tern!(kind == DataKind::Null, DataKind::OPAQUE, kind)
}

/// The cells a cell operand gives: their kind, and their count for a list.
///
/// # Arguments
///
/// * `kind` - The kind of the operand: a list, a pointer to one, or a slice.
///
/// # Returns
///
/// The cell kind, and the count of a list or `None` for a slice.
///
/// # Panics
///
/// When the kind is none of these, which a clean check rules out.
fn cells_of(kind: DataKind) -> (DataKind, Option<u32>)
{
    ret_if!(kind.is_text(), (DataKind::U8, None));
    let seq = kind.as_seq().or_else(|| kind.pointee().and_then(DataKind::as_seq));
    ret_if!(let Some(s) = seq, (s.cell, None));
    let list = kind.as_list().or_else(|| kind.pointee().and_then(DataKind::as_list));
    let list = list.expect("a cell operand is a list, text or a pointer to a list");
    (list.cell, Some(list.len))
}

/// What the source of a range is.
///
/// # Arguments
///
/// * `kind` - The kind of the source.
///
/// # Returns
///
/// A list of its count for a list or a pointer to one, a slice, or else a
/// block.
fn slice_from(kind: DataKind) -> SliceFrom
{
    let seq = kind.is_seq() || kind.pointee().is_some_and(DataKind::is_seq);
    ret_if!(kind.is_text() || seq, SliceFrom::Slice);
    let list = kind.as_list().or_else(|| kind.pointee().and_then(DataKind::as_list));
    list.map_or(SliceFrom::Block, |l| SliceFrom::List(l.len))
}

/// The distance between two pointers.
///
/// # Arguments
///
/// * `lhs` - The later pointer.
/// * `rhs` - The earlier pointer.
///
/// # Returns
///
/// The node.
fn ptr_sub(lhs: Expr, rhs: Expr) -> ExprNode
{
    ExprNode::PtrSub { lhs: Box::new(lhs), rhs: Box::new(rhs) }
}

/// The pointer operand of a dereference.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The operand, or `None` when the expression is no dereference.
fn deref_operand(e: &ast::Expr) -> Option<&ast::Expr>
{
    match e
    {
        ast::Expr::Prefix(n) if n.op == ast::PrefixOp::Deref => Some(&n.rhs),
        ast::Expr::Verb(n) if n.op == VerbOp::Deref => n.args.first(),
        _ => None,
    }
}

/// The value of a float literal, read at the width of its recorded kind.
///
/// # Arguments
///
/// * `n` - The literal.
/// * `kind` - The recorded kind, a float.
///
/// # Returns
///
/// The value.
fn float_lit(n: &ast::FloatLit, kind: DataKind) -> Value
{
    let bits = kind.bits(64);
    Value::float(&n.text, bits).unwrap_or(Value::default_of(kind))
}

/// `return result` at the end of a function with a result.
///
/// # Arguments
///
/// * `slot` - The local slot of the result variable.
/// * `kind` - The result kind.
/// * `span` - The function block.
///
/// # Returns
///
/// The statement.
fn read_return(slot: u32, kind: DataKind, span: Span) -> Stmt
{
    let read = Expr::builder()
        .kind(kind.repr())
        .node(ExprNode::Read(Place::Local(slot)))
        .span(span)
        .build();
    Stmt::Return { value: Some(read), span }
}

/// Lend a duplicate of a function value that goes to a parameter.
///
/// # Arguments
///
/// * `e` - The argument, changed in place.
fn lend_fn(e: &mut Expr)
{
    let fn_dup = e.kind.is_fn() && matches!(e.node, ExprNode::Dup { .. });
    do_if!(fn_dup, do_if!(let ExprNode::Dup { lent, .. } = &mut e.node, *lent = true));
}

/// The constant `true`.
///
/// # Arguments
///
/// * `span` - The source region.
///
/// # Returns
///
/// The expression.
fn true_expr(span: Span) -> Expr
{
    node_expr(DataKind::Bool, ExprNode::Const(Value::Bool(true)), span)
}

/// A drop of the value at a place.
///
/// # Arguments
///
/// * `dest` - The place.
/// * `kind` - Its kind, which holds text.
/// * `span` - The source region.
///
/// # Returns
///
/// The statement.
fn drop_stmt(dest: Dest, kind: DataKind, span: Span) -> Stmt
{
    Stmt::Drop { dest, kind, span }
}

/// Whether a statement leaves the scope it stands in, so nothing after it
/// in that scope runs.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// `true` for a return, a break or a continue.
fn leaves_scope(s: &Stmt) -> bool
{
    matches!(s, Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. })
}

/// An argument handed over to a parameter marked `အပိုင်အနေဖြင့်`: a read
/// of a place is taken, so the place holds the zero word after it; each other
/// value is handed over as it is.
///
/// # Arguments
///
/// * `e` - The lowered argument.
///
/// # Returns
///
/// The argument.
fn hand_over(e: Expr) -> Expr
{
    let read = matches!(e.node, ExprNode::Read(_) | ExprNode::Load { .. });
    tern!(read, take(e), e)
}

/// A return whose value is handed over, as [`hand_over`] gives it.
///
/// # Arguments
///
/// * `stmt` - The return.
///
/// # Returns
///
/// The return, or the statement as it is when it gives back no value.
fn hand_back(stmt: Stmt) -> Stmt
{
    let Stmt::Return { value: Some(v), span } = stmt
    else
    {
        return stmt;
    };
    Stmt::Return { value: Some(hand_over(v)), span }
}

/// The address of a cell of text or a plain list, with the block pinned: a
/// pointer to the cell can write it, so the block is never shared again and
/// a duplicate of the list copies it. Each other address is as it is.
///
/// # Arguments
///
/// * `addr` - The address.
///
/// # Returns
///
/// The address.
fn pinned(addr: Expr) -> Expr
{
    let ExprNode::CellAddr { list, index, len, first, checked } = addr.node
    else
    {
        return addr;
    };
    let list = match list.node
    {
        ExprNode::Own { value, .. } =>
        {
            node_expr(list.kind, ExprNode::Own { value, pin: true }, list.span)
        }
        _ => *list,
    };
    let node = ExprNode::CellAddr { list: Box::new(list), index, len, first, checked };
    node_expr(addr.kind, node, addr.span)
}

/// The variable at the root of a place, through its fields and cells; a
/// place behind a pointer has none.
///
/// # Arguments
///
/// * `e` - The place.
///
/// # Returns
///
/// The name of the variable, or `None`.
fn place_root(e: &ast::Expr) -> Option<&str>
{
    match ungrouped(e)
    {
        ast::Expr::Name(n) => Some(n.name.as_str()),
        ast::Expr::Index(n) => place_root(&n.list),
        ast::Expr::Field(n) => place_root(&n.base),
        _ => None,
    }
}

/// An expression with each pair of parentheses around it taken off.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The expression inside.
fn ungrouped(e: &ast::Expr) -> &ast::Expr
{
    tern!(let ast::Expr::Group(n) = e, ungrouped(&n.inner), e)
}

/// One arm of an `if` chain: the statements its test needs first, then the
/// test, the statements of the arm and its source region.
type ChainArm = (Vec<Stmt>, (Expr, Vec<Stmt>, Span));

/// Nest the arms of an `if` chain: each later arm, with the statements its
/// test needs first, goes in the `else` of the arm before it.
///
/// # Arguments
///
/// * `arms` - The statements before each test, the test, the statements of the
///   arm and its source region.
/// * `els` - The statements of the last `else`.
/// * `whole` - The source region of the whole chain.
///
/// # Returns
///
/// The outermost `if`.
fn if_chain(arms: Vec<ChainArm>, els: Vec<Stmt>, whole: Span) -> Stmt
{
    let nest = |els, (i, (pre, (cond, then, span))): (usize, ChainArm)| {
        let span: Span = tern!(i == 0, whole, span.to(&whole));
        let mut arm: Vec<Stmt> = pre;
        arm.push(Stmt::If { cond, then, els, span });
        arm
    };
    let mut chain = arms.into_iter().enumerate().rev().fold(els, nest);
    chain.pop().expect("an if has at least one arm")
}

/// The default of a struct or a counted list that holds an enumeration: the
/// default of each field or cell, so each enumeration starts at its first
/// member.
///
/// # Arguments
///
/// * `kind` - The struct or the counted list.
/// * `span` - Its source region.
///
/// # Returns
///
/// The expression.
fn defaults_of_parts(kind: DataKind, span: Span) -> Expr
{
    let parts: Vec<Expr> = match (kind.as_struct(), kind.as_list())
    {
        (Some(s), _) =>
        {
            s.fields().iter().map(|(_, k)| default_constant(*k, span)).collect()
        }
        (_, Some(l)) => (0..l.len).map(|_| default_constant(l.cell, span)).collect(),
        _ => Vec::new(),
    };
    Expr::builder().kind(kind).node(ExprNode::List(parts)).span(span).build()
}

/// The value of an enumeration member, as storage holds it.
///
/// # Arguments
///
/// * `v` - The member value.
/// * `kind` - The value kind.
/// * `span` - Its source region.
///
/// # Returns
///
/// The expression: a constant, static text, or the cells or fields of a
/// list or a struct.
pub(crate) fn member_expr(v: &MemberValue, kind: DataKind, span: Span) -> Expr
{
    let kind = kind.repr();
    let node = match v
    {
        MemberValue::Scalar(x) => ExprNode::Const(*x),
        MemberValue::Text(t) => ExprNode::Text(t.clone()),
        MemberValue::Items(items) => return items_expr(items, kind, span),
    };
    Expr::builder().kind(kind).node(node).span(span).build()
}

/// The cells or fields of a list or a struct member value.
///
/// # Arguments
///
/// * `items` - The cells or fields.
/// * `kind` - The list or struct kind.
/// * `span` - Its source region.
///
/// # Returns
///
/// The expression; a plain list is built as a counted list and converted,
/// as a list literal is.
fn items_expr(items: &[MemberValue], kind: DataKind, span: Span) -> Expr
{
    let build =
        |k: DataKind, node| Expr::builder().kind(k).node(node).span(span).build();
    if let Some(s) = kind.as_struct()
    {
        let fields =
            items.iter().zip(s.fields()).map(|(v, (_, k))| member_expr(v, *k, span));
        return build(kind, ExprNode::List(fields.collect()));
    }
    let cell =
        kind.cell_kind().expect("a member value with items is a list or a struct");
    let cells =
        ExprNode::List(items.iter().map(|v| member_expr(v, cell, span)).collect());
    ret_if!(!kind.is_seq(), build(kind, cells));
    let counted = build(DataKind::list_of(cell, items.len() as u32), cells);
    build(kind, ExprNode::Convert { operand: Box::new(counted), implicit: true })
}
