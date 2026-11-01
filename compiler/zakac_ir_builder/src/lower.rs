use std::collections::HashMap;

use bon::Builder;
use zakac_ast::expr as ast;
use zakac_ast::kind::Kind;
use zakac_ast::stmt as st;
use zakac_ast::verb::Dir;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::semcheck::KindTable;
use zakac_semcheck::semcheck::RESULT_NAME;
use zakac_semcheck::structs::StructTable;
use zakac_span::span::Span;

use crate::ir::Access;
use crate::ir::CmpOp;
use crate::ir::Dest;
use crate::ir::Expr;
use crate::ir::ExprNode;
use crate::ir::Func;
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
use crate::value::Value;

/// Lower a checked program to the IR. Names become slots, blocks are
/// flattened, operators get one spelling, and widening conversions become
/// explicit. A negated integer literal becomes a negative constant.
///
/// # Arguments
///
/// * `program` - The tree; the check must have reported no errors.
/// * `kinds` - The kind of each expression, from the check.
/// * `structs` - The kind of each declared struct, from the check.
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
    let unit_sigs = collect_sigs(program, structs);
    #[rustfmt::skip]
    let mut lowerer = Lowerer::builder().kinds(kinds).structs(structs)
        .platform(options.platform_bits).sigs(unit_sigs[0].clone()).unit_sigs(unit_sigs)
        .build();
    lowerer.scopes.push(HashMap::new());
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
    #[rustfmt::skip]
    return Module::builder().globals(lowerer.globals).funcs(lowerer.funcs).main(main)
        .imported_inits(imported_inits).span(program.span).build();
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
    let name = &program.units[unit as usize].name;
    let stem = std::path::Path::new(name).file_stem().and_then(|s| s.to_str());
    format!("@{}", stem.unwrap_or(name))
}

/// What a call needs to know about a function.
#[derive(Debug, Clone)]
struct Sig
{
    /// Index into the module functions.
    index: u32,
    /// The parameter kinds.
    params: Vec<DataKind>,
}

/// The signatures of the top-level functions of each file, numbered in
/// source order across the files; the first of two with one name in a file
/// wins, as in the check.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `structs` - The kind of each declared struct.
///
/// # Returns
///
/// The signatures of each unit, one per name.
fn collect_sigs(
    program: &st::Program, structs: &StructTable,
) -> Vec<HashMap<String, Sig>>
{
    let mut sigs = vec![HashMap::new(); program.units.len().max(1)];
    let decls = program.stmts.iter().enumerate().filter_map(|(i, s)| {
        tern!(let st::Stmt::Fn(f) = s, Some((program.unit_of(i) as usize, f)), None)
    });
    for (index, (unit, f)) in decls.enumerate()
    {
        let params =
            f.params.iter().map(|p| DataKind::from_ast(&p.kind, structs)).collect();
        let sig = Sig { index: index as u32, params };
        sigs[unit].entry(f.name.name.clone()).or_insert(sig);
    }
    sigs
}

/// The function being lowered.
#[derive(Debug, Default)]
struct Frame
{
    /// Its slots so far.
    locals: Vec<Slot>,
    /// Its result kind.
    ret: Option<DataKind>,
}

/// The lowering walk.
#[derive(Builder)]
struct Lowerer<'k>
{
    /// Kinds from the check.
    kinds: &'k KindTable,
    /// The kind of each declared struct, from the check.
    structs: &'k StructTable,
    /// Bit count of the platform-sized kinds.
    platform: u16,
    /// The signatures of the functions the current file declares or
    /// imports, one per name.
    sigs: HashMap<String, Sig>,
    /// The signatures of the functions each file declares.
    unit_sigs: Vec<HashMap<String, Sig>>,
    /// The top-level places of each file lowered so far.
    #[builder(default)]
    saved: Vec<HashMap<String, Place>>,
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
    /// The function being lowered; `None` at the top level.
    frame: Option<Frame>,
    /// Names in scope, innermost last.
    #[builder(default)]
    scopes: Vec<HashMap<String, Place>>,
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
    fn kind_of(&self, e: &ast::Expr) -> DataKind
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
        let slot = Slot::builder()
            .name(&ident.name)
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
        scope.insert(ident.name.clone(), place);
        place
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
        f(self);
        self.scopes.pop();
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
        self.scoped(|l| do_for!(s in stmts, l.stmt(s, &mut out)));
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
        for name in n.names.iter().map(|n| &n.name)
        {
            let sig = self.unit_sigs[target].get(name).cloned();
            do_if!(let Some(sig) = sig, _ = self.sigs.insert(name.clone(), sig));
            let place = self.saved.get(target).and_then(|s| s.get(name)).copied();
            let scope = self.scopes.last_mut().expect("a scope is open");
            do_if!(let Some(place) = place, _ = scope.insert(name.clone(), place));
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
        let func = self.func(f);
        self.funcs.push(func);
    }

    /// Lower a statement and append it to `out`.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    /// * `out` - The list it goes to.
    fn stmt(&mut self, s: &st::Stmt, out: &mut Vec<Stmt>)
    {
        match s
        {
            st::Stmt::Decl(n) => out.push(self.decl(n)),
            st::Stmt::Assign(n) => out.push(self.assign(n)),
            st::Stmt::Print(n) => out.push(self.print(n)),
            st::Stmt::Block(n) =>
            {
                self.scoped(|l| do_for!(s in &n.stmts, l.stmt(s, out)))
            }
            st::Stmt::If(n) => out.push(self.if_stmt(n)),
            st::Stmt::While(n) => out.push(self.while_stmt(n)),
            st::Stmt::Counted(n) => out.push(self.counted(n)),
            st::Stmt::Generic(n) => out.push(self.generic(n)),
            st::Stmt::Break(n) => out.push(Stmt::Break { span: n.span }),
            st::Stmt::Continue(n) => out.push(Stmt::Continue { span: n.span }),
            st::Stmt::Fn(f) =>
            {
                let func = self.func(f);
                self.funcs.push(func);
            }
            st::Stmt::Return(n) => out.push(self.return_stmt(n)),
            st::Stmt::Call(n) =>
            {
                let (func, args) = self.call_parts(&n.callee, &n.args);
                let name = n.callee.name.clone();
                out.push(Stmt::Call { func, name, args, span: n.span });
            }
            st::Stmt::Alloc(n) => out.push(self.alloc(n)),
            st::Stmt::Resize(n) => out.push(self.resize(n)),
            st::Stmt::Free(n) =>
            {
                out.push(Stmt::Free { dest: self.dest(&n.target), span: n.span })
            }
            st::Stmt::Struct(_) => (),
            st::Stmt::Import(n) => self.import(n),
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
        let kind = self.declared_kind(&n.kind, n.init.as_ref());
        let value = self.initial(n.init.as_ref(), kind, n.name.span);
        let access = tern!(n.is_const, Access::Const, Access::Var);
        let place = self.declare(&n.name, kind, access);
        Stmt::Set { place, value, span: n.span }
    }

    /// The kind of a declared name: the written kind, or the kind of its
    /// value when the written kind leaves list counts out.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind phrase.
    /// * `init` - The value, when one is written.
    ///
    /// # Returns
    ///
    /// The kind.
    fn declared_kind(&self, written: &Kind, init: Option<&ast::Expr>) -> DataKind
    {
        let kind = DataKind::from_ast(written, self.structs);
        let open = init.filter(|_| kind.has_open_count());
        open.map_or(kind, |e| self.kind_of(e))
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

    /// Lower an assignment: a store to a slot, or a store at an address
    /// for a dereference target.
    ///
    /// # Arguments
    ///
    /// * `n` - The assignment.
    ///
    /// # Returns
    ///
    /// The store.
    fn assign(&mut self, n: &st::AssignStmt) -> Stmt
    {
        let span = n.span;
        match self.dest(&n.target)
        {
            Dest::Slot(place) =>
            {
                let want = self.slot_kind(place);
                let value = self.owned(&n.value, want);
                Stmt::Set { place, value, span }
            }
            Dest::Mem(addr) =>
            {
                let value = self.owned(&n.value, self.kind_of(&n.target));
                Stmt::Store { addr, value, span }
            }
        }
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
    fn dest(&mut self, e: &ast::Expr) -> Dest
    {
        ret_if!(let ast::Expr::Group(n) = e, self.dest(&n.inner));
        ret_if!(let Some(ptr) = deref_operand(e), Dest::Mem(self.expr(ptr)));
        ret_if!(let ast::Expr::Index(n) = e, Dest::Mem(self.cell_addr(n)));
        ret_if!(let ast::Expr::Field(n) = e, Dest::Mem(self.field_addr(n)));
        let ast::Expr::Name(n) = e
        else
        {
            panic!("expression {} is not a place", e.id().0);
        };
        Dest::Slot(self.lookup(&n.name))
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
        let value = match &n.value
        {
            st::PrintArg::Expr(e) => PrintValue::Expr(self.expr(e)),
            st::PrintArg::Str(s) => PrintValue::Text(s.lit.value.clone()),
        };
        Stmt::Print { value, span: n.span }
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
        let arms: Vec<(Expr, Vec<Stmt>, Span)> =
            n.arms.iter().map(|arm| self.if_arm(arm)).collect();
        let els = n.else_block.as_ref().map_or_else(Vec::new, |b| self.block(&b.stmts));
        let nest = |els, (i, (cond, then, span)): (usize, (Expr, Vec<Stmt>, Span))| {
            let span = tern!(i == 0, n.span, span.to(&n.span));
            vec![Stmt::If { cond, then, els, span }]
        };
        let mut chain = arms.into_iter().enumerate().rev().fold(els, nest);
        chain.pop().expect("an if has at least one arm")
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
        let cond = self.coerce(&n.cond, DataKind::Bool);
        let block = self.block(&n.block.stmts);
        Stmt::Loop { init: Vec::new(), cond, update: Vec::new(), block, span: n.span }
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
            do_for!(s in &n.block.stmts, l.stmt(s, &mut block));
        });
        Stmt::Counted { var, from, to, step, mode: n.mode, block, span: n.span }
    }

    /// Lower a generic loop: its declaration becomes the start statement.
    ///
    /// # Arguments
    ///
    /// * `n` - The loop.
    ///
    /// # Returns
    ///
    /// The loop.
    fn generic(&mut self, n: &st::GenericStmt) -> Stmt
    {
        let kind = self.declared_kind(&n.kind, n.init.as_ref());
        let value = self.initial(n.init.as_ref(), kind, n.var.span);
        let mut parts = None;
        self.scoped(|l| {
            let place = l.declare(&n.var, kind, Access::Var);
            let set = Stmt::Set { place, value, span: n.var.span.to(&n.kind.span) };
            let cond = l.coerce(&n.cond, DataKind::Bool);
            let mut block = Vec::new();
            do_for!(s in &n.block.stmts, l.stmt(s, &mut block));
            let mut update = Vec::new();
            do_if!(let Some(u) = &n.update, l.stmt(u, &mut update));
            parts = Some((set, cond, block, update));
        });
        let (set, cond, block, update) = parts.expect("the scope ran");
        Stmt::Loop { init: vec![set], cond, update, block, span: n.span }
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
    fn func(&mut self, f: &st::FnDecl) -> Func
    {
        let ret = f.ret.as_ref().map(|k| DataKind::from_ast(k, self.structs));
        let outer = self.frame.replace(Frame { locals: Vec::new(), ret });
        let mut block = Vec::new();
        let mut result = None;
        self.scoped(|l| {
            do_for!(p in &f.params, l.declare_param(p));
            result = ret.map(|k| l.result_slot(f, k, &mut block));
            do_for!(s in &f.block.stmts, l.stmt(s, &mut block));
        });
        let ends_in_return = matches!(block.last(), Some(Stmt::Return { .. }));
        let implicit = result.zip(ret).filter(|_| !ends_in_return);
        let ret_stmt =
            implicit.map(|(slot, kind)| read_return(slot, kind, f.block.span));
        block.extend(ret_stmt);
        let frame =
            std::mem::replace(&mut self.frame, outer).expect("the frame is set");
        #[rustfmt::skip]
        return Func::builder().name(format!("{}{}", f.name.name, self.suffix))
            .params(f.params.len() as u32)
            .maybe_result(result)
            .maybe_ret(ret).locals(frame.locals).block(block).span(f.span).build();
    }

    /// Declare a function parameter.
    ///
    /// # Arguments
    ///
    /// * `p` - The parameter.
    fn declare_param(&mut self, p: &st::Param)
    {
        self.declare(&p.name, DataKind::from_ast(&p.kind, self.structs), Access::Var);
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

    /// Lower a return statement.
    ///
    /// # Arguments
    ///
    /// * `n` - The statement.
    ///
    /// # Returns
    ///
    /// The statement.
    fn return_stmt(&mut self, n: &st::ReturnStmt) -> Stmt
    {
        let ret = self.frame.as_ref().and_then(|f| f.ret);
        let value = n.value.as_ref().zip(ret).map(|(v, k)| self.owned(v, k));
        Stmt::Return { value, span: n.span }
    }

    /// The function index and the arguments of a call, each converted to
    /// its parameter's kind.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The index and the lowered arguments.
    fn call_parts(
        &mut self, callee: &ast::Ident, args: &[ast::Expr],
    ) -> (u32, Vec<Expr>)
    {
        let sig = self.sigs.get(&callee.name).cloned();
        let sig =
            sig.unwrap_or_else(|| panic!("function {} is not declared", callee.name));
        let args =
            args.iter().zip(&sig.params).map(|(a, k)| self.owned(a, *k)).collect();
        (sig.index, args)
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
        coerce_lowered(lowered, want)
    }

    /// Lower a value headed for a place of kind `want`. A value of a move
    /// kind headed for a place of a move kind is taken out of its own
    /// place, which is left at its default; other values are read.
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
        let moved = lowered.kind.is_move_kind() && want.is_move_kind();
        coerce_lowered(tern!(moved, take(lowered), lowered), want)
    }

    /// Lower an expression with the kind the check recorded for it; a
    /// `ဗလာ` with no pointer kind around it is a `ptr`.
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
        let kind = concrete(self.kind_of(e));
        let node = match e
        {
            ast::Expr::Int(n) => ExprNode::Const(self.int_lit(n, kind, false)),
            ast::Expr::Float(n) => ExprNode::Const(float_lit(n, kind)),
            ast::Expr::Char(n) => ExprNode::Const(Value::Char(n.value)),
            ast::Expr::Bool(n) => ExprNode::Const(Value::Bool(n.value)),
            ast::Expr::Name(n) => ExprNode::Read(self.lookup(&n.name)),
            ast::Expr::Group(n) => return self.expr(&n.inner),
            ast::Expr::Prefix(n) => self.prefix(n, kind),
            ast::Expr::Infix(n) => self.infix(n.op, &n.lhs, &n.rhs, kind),
            ast::Expr::Not(n) => prefix_node(PrefixOp::Not, self.expr(&n.inner)),
            ast::Expr::Logic(n) => self.logic(n),
            ast::Expr::Verb(n) => self.verb(n, kind),
            ast::Expr::Call(n) =>
            {
                let (func, args) = self.call_parts(&n.callee, &n.args);
                ExprNode::Call { func, name: n.callee.name.clone(), args }
            }
            ast::Expr::Null(_) => ExprNode::Const(Value::NULL),
            ast::Expr::SizeOf(n) =>
            {
                ExprNode::SizeOf(DataKind::from_ast(&n.kind, self.structs))
            }
            ast::Expr::Index(n) => ExprNode::Load { addr: Box::new(self.cell_addr(n)) },
            ast::Expr::Field(n) =>
            {
                ExprNode::Load { addr: Box::new(self.field_addr(n)) }
            }
            ast::Expr::Range(n) => self.range(n),
            ast::Expr::List(n) => self.list(n, kind),
            ast::Expr::Struct(n) => self.struct_lit(n, kind),
            ast::Expr::BlankList(_) => ExprNode::BlankList,
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

    /// Lower a prefix operator. `-` before an integer literal without a
    /// kind prefix becomes a negative constant; `&` and `*` become address and
    /// load nodes.
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
            ast::Expr::Int(lit) if lit.kind.is_none() && n.op == ast::PrefixOp::Neg =>
            {
                Some(lit)
            }
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
            Dest::Mem(addr) => addr.node,
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
        let cell = kind.as_list().expect("a list literal has a list kind").cell;
        ExprNode::List(n.cells.iter().map(|c| self.owned(c, cell)).collect())
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

    /// A range of cells: a slice of a list, a slice, or a block.
    ///
    /// # Arguments
    ///
    /// * `n` - The range.
    ///
    /// # Returns
    ///
    /// The slice.
    fn range(&mut self, n: &ast::RangeExpr) -> ExprNode
    {
        let from = slice_from(self.kind_of(&n.source));
        let source = Box::new(self.expr(&n.source));
        let start = Box::new(self.coerce(&n.start, DataKind::USIZE));
        let len = Box::new(self.coerce(&n.len, DataKind::USIZE));
        ExprNode::MakeSlice { source, start, len, from, checked: false }
    }

    /// A slice of each cell of a list: the lend of a list.
    ///
    /// # Arguments
    ///
    /// * `value` - The list.
    ///
    /// # Returns
    ///
    /// The slice.
    fn lend_list(&mut self, value: &ast::Expr) -> ExprNode
    {
        let from = slice_from(self.kind_of(value));
        let SliceFrom::List(count) = from
        else
        {
            panic!("a lent list has a count");
        };
        let span = value.span();
        let source = Box::new(self.expr(value));
        let start = Box::new(self.usize_const(0, span));
        let len = Box::new(self.usize_const(count, span));
        ExprNode::MakeSlice { source, start, len, from, checked: true }
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
        let shape =
            self.kind_of(&n.base).as_struct().expect("a field is read from a struct");
        let (at, kind) = shape.field(&n.field.name).expect("the check found the field");
        let offset = shape.offset(at, self.platform);
        let base = Box::new(self.expr(&n.base));
        let field = u32::try_from(at).expect("a struct has fewer than 2^32 fields");
        let node = ExprNode::FieldAddr { base, field, offset };
        Expr::builder().kind(kind.pointer_to()).node(node).span(n.span).build()
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
        let base = Box::new(self.expr(value));
        let offset = u32::from(self.platform) / 8;
        let node = ExprNode::FieldAddr { base, field: 1, offset };
        let kind = DataKind::USIZE.pointer_to();
        let addr = Expr::builder().kind(kind).node(node).span(value.span()).build();
        ExprNode::Load { addr: Box::new(addr) }
    }

    /// Lower a comparison; both operands take their common kind, which is
    /// `ptr` for two `ဗလာ`.
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
        let common = concrete(self.common(self.kind_of(lhs), self.kind_of(rhs)));
        let lhs = Box::new(self.coerce(lhs, common));
        let rhs = Box::new(self.coerce(rhs, common));
        ExprNode::Compare { op, lhs, rhs }
    }

    /// Lower a logical operator.
    ///
    /// # Arguments
    ///
    /// * `n` - The expression.
    ///
    /// # Returns
    ///
    /// The operation.
    fn logic(&mut self, n: &ast::LogicExpr) -> ExprNode
    {
        let op = tern!(n.op == ast::LogicOp::And, LogicOp::And, LogicOp::Or);
        let lhs = Box::new(self.coerce(&n.lhs, DataKind::Bool));
        let rhs = Box::new(self.coerce(&n.rhs, DataKind::Bool));
        ExprNode::Logic { op, lhs, rhs }
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
            VerbOp::BitNot => prefix_node(PrefixOp::BitNot, self.expr(value)),
            VerbOp::Lend if self.kind_of(value).is_list() => self.lend_list(value),
            VerbOp::Lend => ExprNode::Lend { operand: Box::new(self.expr(value)) },
            VerbOp::Cast => ExprNode::Convert {
                operand: Box::new(self.expr(value)),
                implicit: false,
            },
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
            | VerbOp::Ne
            | VerbOp::Gt
            | VerbOp::Le
            | VerbOp::Lt
            | VerbOp::Ge
            | VerbOp::Print
            | VerbOp::Assign
            | VerbOp::Return
            | VerbOp::Cell =>
            {
                panic!("verb {} is no one-operand value", n.op.as_ref())
            }
        }
    }
}

/// Convert a lowered expression to `want` when its kind differs. A `ဗလာ`
/// constant headed for a pointer kind, and an address headed for a kind of
/// its own shape with another borrow mark, take `want` as their label with
/// no conversion; other addresses keep their own kind, which pointer math
/// reads its stride from, under an implicit conversion.
///
/// # Arguments
///
/// * `lowered` - The expression.
/// * `want` - The kind its place expects.
///
/// # Returns
///
/// The expression, of kind `want`.
fn coerce_lowered(lowered: Expr, want: DataKind) -> Expr
{
    ret_if!(lowered.kind == want, lowered);
    let span = lowered.span;
    let literal = matches!(lowered.node, ExprNode::Const(_));
    let null = literal && lowered.kind.is_ptr() && want.is_ptr();
    let relabel = null || lowered.kind.same_shape(want);
    ret_if!(relabel, Expr::builder().kind(want).node(lowered.node).span(span).build());
    let node = ExprNode::Convert { operand: Box::new(lowered), implicit: true };
    Expr::builder().kind(want).node(node).span(span).build()
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
/// pointer slot starts at a `ဗလာ` of its own pointer kind.
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
    let node = tern!(
        kind.is_aggregate(),
        ExprNode::BlankList,
        ExprNode::Const(Value::default_of(kind))
    );
    Expr::builder().kind(kind).node(node).span(span).build()
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
    ret_if!(let Some(s) = kind.as_slice(), (s.cell, None));
    let list = kind.as_list().or_else(|| kind.pointee().and_then(DataKind::as_list));
    let list = list.expect("a cell operand is a list, a pointer to one, or a slice");
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
    ret_if!(kind.is_slice(), SliceFrom::Slice);
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
        .kind(kind)
        .node(ExprNode::Read(Place::Local(slot)))
        .span(span)
        .build();
    Stmt::Return { value: Some(read), span }
}
