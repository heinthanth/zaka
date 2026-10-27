use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_span::span::Span;

use crate::expr::BoolLit;
use crate::expr::CallExpr;
use crate::expr::CharLit;
use crate::expr::Expr;
use crate::expr::FloatLit;
use crate::expr::GroupExpr;
use crate::expr::Ident;
use crate::expr::InfixExpr;
use crate::expr::IntLit;
use crate::expr::LogicExpr;
use crate::expr::NameExpr;
use crate::expr::NotExpr;
use crate::expr::NullLit;
use crate::expr::PrefixExpr;
use crate::expr::SizeOfExpr;
use crate::expr::VerbExpr;
use crate::kind::Kind;
use crate::stmt::AllocStmt;
use crate::stmt::AssignStmt;
use crate::stmt::BlockStmt;
use crate::stmt::BreakStmt;
use crate::stmt::CallStmt;
use crate::stmt::ContinueStmt;
use crate::stmt::CountedStmt;
use crate::stmt::DeclStmt;
use crate::stmt::FnDecl;
use crate::stmt::FreeStmt;
use crate::stmt::GenericStmt;
use crate::stmt::IfArm;
use crate::stmt::IfStmt;
use crate::stmt::Param;
use crate::stmt::PrintArg;
use crate::stmt::PrintStmt;
use crate::stmt::Program;
use crate::stmt::ResizeStmt;
use crate::stmt::ReturnStmt;
use crate::stmt::SizeArg;
use crate::stmt::Stmt;
use crate::stmt::StrArg;
use crate::stmt::WhileStmt;

/// A tree node with a source span.
pub trait Node
{
    /// Source region of the node.
    ///
    /// # Returns
    ///
    /// The span.
    fn span(&self) -> Span;

    /// Short name of the node form, such as `decl` or `verb`.
    ///
    /// # Returns
    ///
    /// The name.
    fn label(&self) -> &'static str;

    /// Call the visitor method for this node.
    ///
    /// # Arguments
    ///
    /// * `v` - The visitor.
    fn accept<V: Visitor + ?Sized>(&self, v: &mut V)
    where
        Self: Sized;
}

/// Implement [`Node`] for structs with a `span` field.
#[rustfmt::skip]
macro_rules! node {
($($t:ty => $label:literal, $visit:ident;)+) => {$(
impl Node for $t
{
    fn span(&self) -> Span { self.span }
    fn label(&self) -> &'static str { $label }
    fn accept<V: Visitor + ?Sized>(&self, v: &mut V) { v.$visit(self) }
}
)+};}

node! {
Program => "program", visit_program;
DeclStmt => "decl", visit_decl;
AssignStmt => "assign", visit_assign;
PrintStmt => "print", visit_print;
StrArg => "str", visit_str;
BlockStmt => "block", visit_block;
IfStmt => "if", visit_if;
IfArm => "if_arm", visit_if_arm;
WhileStmt => "while", visit_while;
CountedStmt => "counted", visit_counted;
GenericStmt => "generic", visit_generic;
BreakStmt => "break", visit_break;
ContinueStmt => "continue", visit_continue;
FnDecl => "fn", visit_fn;
Param => "param", visit_param;
ReturnStmt => "return", visit_return;
CallStmt => "call_stmt", visit_call_stmt;
AllocStmt => "alloc", visit_alloc;
ResizeStmt => "resize", visit_resize;
FreeStmt => "free", visit_free;
IntLit => "int", visit_int;
FloatLit => "float", visit_float;
CharLit => "char", visit_char;
BoolLit => "bool", visit_bool;
NameExpr => "name", visit_name;
GroupExpr => "group", visit_group;
PrefixExpr => "prefix", visit_prefix;
InfixExpr => "infix", visit_infix;
NotExpr => "not", visit_not;
LogicExpr => "logic", visit_logic;
VerbExpr => "verb", visit_verb;
CallExpr => "call", visit_call_expr;
NullLit => "null", visit_null;
SizeOfExpr => "sizeof", visit_size_of;
Ident => "ident", visit_ident;
Kind => "kind", visit_kind; }

/// The wrapped node of a [`Stmt`] as a trait object.
///
/// # Arguments
///
/// * `n` - The statement.
///
/// # Returns
///
/// The wrapped struct.
fn stmt_node(n: &Stmt) -> &dyn Node
{
    match n
    {
        Stmt::Decl(s) => s,
        Stmt::Assign(s) => s,
        Stmt::Print(s) => s,
        Stmt::Block(s) => s,
        Stmt::If(s) => s,
        Stmt::While(s) => s,
        Stmt::Counted(s) => s,
        Stmt::Generic(s) => s,
        Stmt::Break(s) => s,
        Stmt::Continue(s) => s,
        Stmt::Fn(s) => s,
        Stmt::Return(s) => s,
        Stmt::Call(s) => s,
        Stmt::Alloc(s) => s,
        Stmt::Resize(s) => s,
        Stmt::Free(s) => s,
    }
}

/// The wrapped node of an [`Expr`] as a trait object.
///
/// # Arguments
///
/// * `n` - The expression.
///
/// # Returns
///
/// The wrapped struct.
fn expr_node(n: &Expr) -> &dyn Node
{
    match n
    {
        Expr::Int(e) => e,
        Expr::Float(e) => e,
        Expr::Char(e) => e,
        Expr::Bool(e) => e,
        Expr::Name(e) => e,
        Expr::Group(e) => e,
        Expr::Prefix(e) => e,
        Expr::Infix(e) => e,
        Expr::Not(e) => e,
        Expr::Logic(e) => e,
        Expr::Verb(e) => e,
        Expr::Call(e) => e,
        Expr::Null(e) => e,
        Expr::SizeOf(e) => e,
    }
}

impl Node for Stmt
{
    fn span(&self) -> Span { stmt_node(self).span() }

    fn label(&self) -> &'static str { stmt_node(self).label() }

    fn accept<V: Visitor + ?Sized>(&self, v: &mut V) { v.visit_stmt(self) }
}

impl Node for Expr
{
    fn span(&self) -> Span { expr_node(self).span() }

    fn label(&self) -> &'static str { expr_node(self).label() }

    fn accept<V: Visitor + ?Sized>(&self, v: &mut V) { v.visit_expr(self) }
}

impl Node for PrintArg
{
    fn span(&self) -> Span
    {
        match self
        {
            PrintArg::Expr(e) => e.span(),
            PrintArg::Str(s) => s.span,
        }
    }

    fn label(&self) -> &'static str
    {
        match self
        {
            PrintArg::Expr(e) => e.label(),
            PrintArg::Str(_) => "str",
        }
    }

    fn accept<V: Visitor + ?Sized>(&self, v: &mut V) { walk_print_arg(v, self) }
}

/// Traversal that reads the tree. `enter` and `leave` run around each struct
/// node, so a dumper or profiler need not override each `visit_*`.
#[allow(unused_variables)]
pub trait Visitor
{
    /// Called before the children of a node are visited.
    fn enter(&mut self, n: &dyn Node) {}
    /// Called after the children of a node are visited.
    fn leave(&mut self, n: &dyn Node) {}

    /// Visit a program.
    fn visit_program(&mut self, n: &Program) { walk_program(self, n) }
    /// Visit a statement.
    fn visit_stmt(&mut self, n: &Stmt) { walk_stmt(self, n) }
    /// Visit a declaration.
    fn visit_decl(&mut self, n: &DeclStmt) { walk_decl(self, n) }
    /// Visit an assignment.
    fn visit_assign(&mut self, n: &AssignStmt) { walk_assign(self, n) }
    /// Visit a print statement.
    fn visit_print(&mut self, n: &PrintStmt) { walk_print(self, n) }
    /// Visit a string operand of a print statement.
    fn visit_str(&mut self, n: &StrArg) { walk_str(self, n) }
    /// Visit a block.
    fn visit_block(&mut self, n: &BlockStmt) { walk_block(self, n) }
    /// Visit a conditional.
    fn visit_if(&mut self, n: &IfStmt) { walk_if(self, n) }
    /// Visit an arm of a conditional.
    fn visit_if_arm(&mut self, n: &IfArm) { walk_if_arm(self, n) }
    /// Visit a while loop.
    fn visit_while(&mut self, n: &WhileStmt) { walk_while(self, n) }
    /// Visit a counted loop.
    fn visit_counted(&mut self, n: &CountedStmt) { walk_counted(self, n) }
    /// Visit a generic loop.
    fn visit_generic(&mut self, n: &GenericStmt) { walk_generic(self, n) }
    /// Visit a break statement.
    fn visit_break(&mut self, n: &BreakStmt) { walk_break(self, n) }
    /// Visit a continue statement.
    fn visit_continue(&mut self, n: &ContinueStmt) { walk_continue(self, n) }
    /// Visit a function declaration.
    fn visit_fn(&mut self, n: &FnDecl) { walk_fn(self, n) }
    /// Visit a function parameter.
    fn visit_param(&mut self, n: &Param) { walk_param(self, n) }
    /// Visit a return statement.
    fn visit_return(&mut self, n: &ReturnStmt) { walk_return(self, n) }
    /// Visit a call statement.
    fn visit_call_stmt(&mut self, n: &CallStmt) { walk_call_stmt(self, n) }
    /// Visit a heap allocation.
    fn visit_alloc(&mut self, n: &AllocStmt) { walk_alloc(self, n) }
    /// Visit a heap reallocation.
    fn visit_resize(&mut self, n: &ResizeStmt) { walk_resize(self, n) }
    /// Visit a heap release.
    fn visit_free(&mut self, n: &FreeStmt) { walk_free(self, n) }

    /// Visit an expression.
    fn visit_expr(&mut self, n: &Expr) { walk_expr(self, n) }
    /// Visit an integer literal.
    fn visit_int(&mut self, n: &IntLit) { walk_int(self, n) }
    /// Visit a floating-point literal.
    fn visit_float(&mut self, n: &FloatLit) { walk_float(self, n) }
    /// Visit a character literal.
    fn visit_char(&mut self, n: &CharLit) { walk_char(self, n) }
    /// Visit a boolean literal.
    fn visit_bool(&mut self, n: &BoolLit) { walk_bool(self, n) }
    /// Visit a variable.
    fn visit_name(&mut self, n: &NameExpr) { walk_name(self, n) }
    /// Visit a parenthesized expression.
    fn visit_group(&mut self, n: &GroupExpr) { walk_group(self, n) }
    /// Visit a prefix operator.
    fn visit_prefix(&mut self, n: &PrefixExpr) { walk_prefix(self, n) }
    /// Visit an infix operator.
    fn visit_infix(&mut self, n: &InfixExpr) { walk_infix(self, n) }
    /// Visit a postfix `မဟုတ်`.
    fn visit_not(&mut self, n: &NotExpr) { walk_not(self, n) }
    /// Visit a logical operator.
    fn visit_logic(&mut self, n: &LogicExpr) { walk_logic(self, n) }
    /// Visit a verb clause.
    fn visit_verb(&mut self, n: &VerbExpr) { walk_verb(self, n) }
    /// Visit a call expression.
    fn visit_call_expr(&mut self, n: &CallExpr) { walk_call_expr(self, n) }
    /// Visit a null address.
    fn visit_null(&mut self, n: &NullLit) { walk_null(self, n) }
    /// Visit a size expression.
    fn visit_size_of(&mut self, n: &SizeOfExpr) { walk_size_of(self, n) }
    /// Visit a name in a declaration, parameter, loop header or call.
    fn visit_ident(&mut self, n: &Ident) { walk_ident(self, n) }
    /// Visit a kind.
    fn visit_kind(&mut self, n: &Kind) { walk_kind(self, n) }
}

/// Visit the children of a program.
pub fn walk_program<V: Visitor + ?Sized>(v: &mut V, n: &Program)
{
    v.enter(n);
    do_for!(s in &n.stmts, v.visit_stmt(s));
    v.leave(n);
}

/// Visit the wrapped statement.
pub fn walk_stmt<V: Visitor + ?Sized>(v: &mut V, n: &Stmt)
{
    match n
    {
        Stmt::Decl(s) => v.visit_decl(s),
        Stmt::Assign(s) => v.visit_assign(s),
        Stmt::Print(s) => v.visit_print(s),
        Stmt::Block(s) => v.visit_block(s),
        Stmt::If(s) => v.visit_if(s),
        Stmt::While(s) => v.visit_while(s),
        Stmt::Counted(s) => v.visit_counted(s),
        Stmt::Generic(s) => v.visit_generic(s),
        Stmt::Break(s) => v.visit_break(s),
        Stmt::Continue(s) => v.visit_continue(s),
        Stmt::Fn(s) => v.visit_fn(s),
        Stmt::Return(s) => v.visit_return(s),
        Stmt::Call(s) => v.visit_call_stmt(s),
        Stmt::Alloc(s) => v.visit_alloc(s),
        Stmt::Resize(s) => v.visit_resize(s),
        Stmt::Free(s) => v.visit_free(s),
    }
}

/// Visit the children of a declaration.
pub fn walk_decl<V: Visitor + ?Sized>(v: &mut V, n: &DeclStmt)
{
    v.enter(n);
    v.visit_ident(&n.name);
    v.visit_kind(&n.kind);
    do_if!(let Some(init) = &n.init, v.visit_expr(init));
    v.leave(n);
}

/// Visit the children of an assignment.
pub fn walk_assign<V: Visitor + ?Sized>(v: &mut V, n: &AssignStmt)
{
    v.enter(n);
    v.visit_expr(&n.target);
    v.visit_expr(&n.value);
    v.leave(n);
}

/// Visit the operand of a print statement.
pub fn walk_print<V: Visitor + ?Sized>(v: &mut V, n: &PrintStmt)
{
    v.enter(n);
    walk_print_arg(v, &n.value);
    v.leave(n);
}

/// Visit a print operand.
pub fn walk_print_arg<V: Visitor + ?Sized>(v: &mut V, n: &PrintArg)
{
    match n
    {
        PrintArg::Expr(e) => v.visit_expr(e),
        PrintArg::Str(s) => v.visit_str(s),
    }
}

/// Visit a string operand; it has no children.
pub fn walk_str<V: Visitor + ?Sized>(v: &mut V, n: &StrArg)
{
    v.enter(n);
    v.leave(n);
}

/// Visit the statements of a block.
pub fn walk_block<V: Visitor + ?Sized>(v: &mut V, n: &BlockStmt)
{
    v.enter(n);
    do_for!(s in &n.stmts, v.visit_stmt(s));
    v.leave(n);
}

/// Visit the arms and the else block of a conditional.
pub fn walk_if<V: Visitor + ?Sized>(v: &mut V, n: &IfStmt)
{
    v.enter(n);
    do_for!(arm in &n.arms, v.visit_if_arm(arm));
    do_if!(let Some(block) = &n.else_block, v.visit_block(block));
    v.leave(n);
}

/// Visit the condition and block of a conditional arm.
pub fn walk_if_arm<V: Visitor + ?Sized>(v: &mut V, n: &IfArm)
{
    v.enter(n);
    v.visit_expr(&n.cond);
    v.visit_block(&n.block);
    v.leave(n);
}

/// Visit the condition and block of a while loop.
pub fn walk_while<V: Visitor + ?Sized>(v: &mut V, n: &WhileStmt)
{
    v.enter(n);
    v.visit_expr(&n.cond);
    v.visit_block(&n.block);
    v.leave(n);
}

/// Visit the bounds, step, variable and block of a counted loop.
pub fn walk_counted<V: Visitor + ?Sized>(v: &mut V, n: &CountedStmt)
{
    v.enter(n);
    v.visit_expr(&n.from);
    v.visit_expr(&n.to);
    do_if!(let Some(step) = &n.step, v.visit_expr(step));
    v.visit_ident(&n.var);
    v.visit_block(&n.block);
    v.leave(n);
}

/// Visit the header, block and update of a generic loop.
pub fn walk_generic<V: Visitor + ?Sized>(v: &mut V, n: &GenericStmt)
{
    v.enter(n);
    v.visit_ident(&n.var);
    v.visit_kind(&n.kind);
    do_if!(let Some(init) = &n.init, v.visit_expr(init));
    v.visit_expr(&n.cond);
    v.visit_block(&n.block);
    do_if!(let Some(update) = &n.update, v.visit_stmt(update));
    v.leave(n);
}

/// Visit a break statement; it has no children.
pub fn walk_break<V: Visitor + ?Sized>(v: &mut V, n: &BreakStmt)
{
    v.enter(n);
    v.leave(n);
}

/// Visit a continue statement; it has no children.
pub fn walk_continue<V: Visitor + ?Sized>(v: &mut V, n: &ContinueStmt)
{
    v.enter(n);
    v.leave(n);
}

/// Visit the name, parameters, result kind and block of a function.
pub fn walk_fn<V: Visitor + ?Sized>(v: &mut V, n: &FnDecl)
{
    v.enter(n);
    v.visit_ident(&n.name);
    do_for!(p in &n.params, v.visit_param(p));
    do_if!(let Some(ret) = &n.ret, v.visit_kind(ret));
    v.visit_block(&n.block);
    v.leave(n);
}

/// Visit the kind and name of a parameter.
pub fn walk_param<V: Visitor + ?Sized>(v: &mut V, n: &Param)
{
    v.enter(n);
    v.visit_kind(&n.kind);
    v.visit_ident(&n.name);
    v.leave(n);
}

/// Visit the value of a return statement.
pub fn walk_return<V: Visitor + ?Sized>(v: &mut V, n: &ReturnStmt)
{
    v.enter(n);
    do_if!(let Some(value) = &n.value, v.visit_expr(value));
    v.leave(n);
}

/// Visit the callee and arguments of a call statement.
pub fn walk_call_stmt<V: Visitor + ?Sized>(v: &mut V, n: &CallStmt)
{
    v.enter(n);
    v.visit_ident(&n.callee);
    do_for!(a in &n.args, v.visit_expr(a));
    v.leave(n);
}

/// Visit the size, then the target, of a heap allocation.
pub fn walk_alloc<V: Visitor + ?Sized>(v: &mut V, n: &AllocStmt)
{
    v.enter(n);
    walk_size_arg(v, &n.size);
    v.visit_expr(&n.target);
    v.leave(n);
}

/// Visit the target, then the new size, of a heap reallocation.
pub fn walk_resize<V: Visitor + ?Sized>(v: &mut V, n: &ResizeStmt)
{
    v.enter(n);
    v.visit_expr(&n.target);
    walk_size_arg(v, &n.size);
    v.leave(n);
}

/// Visit the target of a heap release.
pub fn walk_free<V: Visitor + ?Sized>(v: &mut V, n: &FreeStmt)
{
    v.enter(n);
    v.visit_expr(&n.target);
    v.leave(n);
}

/// Visit the count, then the unit kind, of a size operand. The operand is
/// no node of its own, so `enter` and `leave` do not run for it.
fn walk_size_arg<V: Visitor + ?Sized>(v: &mut V, n: &SizeArg)
{
    v.visit_expr(&n.count);
    do_if!(let Some(unit) = &n.unit, v.visit_kind(unit));
}

/// Visit the wrapped expression.
pub fn walk_expr<V: Visitor + ?Sized>(v: &mut V, n: &Expr)
{
    match n
    {
        Expr::Int(e) => v.visit_int(e),
        Expr::Float(e) => v.visit_float(e),
        Expr::Char(e) => v.visit_char(e),
        Expr::Bool(e) => v.visit_bool(e),
        Expr::Name(e) => v.visit_name(e),
        Expr::Group(e) => v.visit_group(e),
        Expr::Prefix(e) => v.visit_prefix(e),
        Expr::Infix(e) => v.visit_infix(e),
        Expr::Not(e) => v.visit_not(e),
        Expr::Logic(e) => v.visit_logic(e),
        Expr::Verb(e) => v.visit_verb(e),
        Expr::Call(e) => v.visit_call_expr(e),
        Expr::Null(e) => v.visit_null(e),
        Expr::SizeOf(e) => v.visit_size_of(e),
    }
}

/// Visit the kind prefix of an integer literal.
pub fn walk_int<V: Visitor + ?Sized>(v: &mut V, n: &IntLit)
{
    v.enter(n);
    do_if!(let Some(kind) = &n.kind, v.visit_kind(kind));
    v.leave(n);
}

/// Visit the kind prefix of a floating-point literal.
pub fn walk_float<V: Visitor + ?Sized>(v: &mut V, n: &FloatLit)
{
    v.enter(n);
    do_if!(let Some(kind) = &n.kind, v.visit_kind(kind));
    v.leave(n);
}

/// Visit a character literal; it has no children.
pub fn walk_char<V: Visitor + ?Sized>(v: &mut V, n: &CharLit)
{
    v.enter(n);
    v.leave(n);
}

/// Visit a boolean literal; it has no children.
pub fn walk_bool<V: Visitor + ?Sized>(v: &mut V, n: &BoolLit)
{
    v.enter(n);
    v.leave(n);
}

/// Visit a variable; it has no children.
pub fn walk_name<V: Visitor + ?Sized>(v: &mut V, n: &NameExpr)
{
    v.enter(n);
    v.leave(n);
}

/// Visit the inner expression of a group.
pub fn walk_group<V: Visitor + ?Sized>(v: &mut V, n: &GroupExpr)
{
    v.enter(n);
    v.visit_expr(&n.inner);
    v.leave(n);
}

/// Visit the operand of a prefix operator.
pub fn walk_prefix<V: Visitor + ?Sized>(v: &mut V, n: &PrefixExpr)
{
    v.enter(n);
    v.visit_expr(&n.rhs);
    v.leave(n);
}

/// Visit the operands of an infix operator.
pub fn walk_infix<V: Visitor + ?Sized>(v: &mut V, n: &InfixExpr)
{
    v.enter(n);
    v.visit_expr(&n.lhs);
    v.visit_expr(&n.rhs);
    v.leave(n);
}

/// Visit the clause under a postfix `မဟုတ်`.
pub fn walk_not<V: Visitor + ?Sized>(v: &mut V, n: &NotExpr)
{
    v.enter(n);
    v.visit_expr(&n.inner);
    v.leave(n);
}

/// Visit the clauses of a logical operator.
pub fn walk_logic<V: Visitor + ?Sized>(v: &mut V, n: &LogicExpr)
{
    v.enter(n);
    v.visit_expr(&n.lhs);
    v.visit_expr(&n.rhs);
    v.leave(n);
}

/// Visit the operands and the target kind of a verb clause.
pub fn walk_verb<V: Visitor + ?Sized>(v: &mut V, n: &VerbExpr)
{
    v.enter(n);
    do_for!(a in &n.args, v.visit_expr(a));
    do_if!(let Some(kind) = &n.kind_arg, v.visit_kind(kind));
    v.leave(n);
}

/// Visit the callee and arguments of a call expression.
pub fn walk_call_expr<V: Visitor + ?Sized>(v: &mut V, n: &CallExpr)
{
    v.enter(n);
    v.visit_ident(&n.callee);
    do_for!(a in &n.args, v.visit_expr(a));
    v.leave(n);
}

/// Visit a null address; it has no children.
pub fn walk_null<V: Visitor + ?Sized>(v: &mut V, n: &NullLit)
{
    v.enter(n);
    v.leave(n);
}

/// Visit the kind of a size expression.
pub fn walk_size_of<V: Visitor + ?Sized>(v: &mut V, n: &SizeOfExpr)
{
    v.enter(n);
    v.visit_kind(&n.kind);
    v.leave(n);
}

/// Visit a name; it has no children.
pub fn walk_ident<V: Visitor + ?Sized>(v: &mut V, n: &Ident)
{
    v.enter(n);
    v.leave(n);
}

/// Visit a kind; it has no children.
pub fn walk_kind<V: Visitor + ?Sized>(v: &mut V, n: &Kind)
{
    v.enter(n);
    v.leave(n);
}

/// Traversal that changes the tree in place, such as constant folding
/// (`*expr = Expr::Int(..)`). `visit_span_mut` runs on the span of each node,
/// so a pass that moves spans overrides that one method.
#[allow(unused_variables)]
pub trait VisitorMut
{
    /// Called with the span of each node.
    fn visit_span_mut(&mut self, span: &mut Span) {}

    /// Visit a program.
    fn visit_program_mut(&mut self, n: &mut Program) { walk_program_mut(self, n) }
    /// Visit a statement.
    fn visit_stmt_mut(&mut self, n: &mut Stmt) { walk_stmt_mut(self, n) }
    /// Visit a block.
    fn visit_block_mut(&mut self, n: &mut BlockStmt) { walk_block_mut(self, n) }
    /// Visit an expression.
    fn visit_expr_mut(&mut self, n: &mut Expr) { walk_expr_mut(self, n) }
    /// Visit a name in a declaration, parameter, loop header or call.
    fn visit_ident_mut(&mut self, n: &mut Ident) { self.visit_span_mut(&mut n.span) }
    /// Visit a kind.
    fn visit_kind_mut(&mut self, n: &mut Kind) { self.visit_span_mut(&mut n.span) }
    /// Visit a heap allocation.
    fn visit_alloc_mut(&mut self, n: &mut AllocStmt) { walk_alloc_mut(self, n) }
    /// Visit a heap reallocation.
    fn visit_resize_mut(&mut self, n: &mut ResizeStmt) { walk_resize_mut(self, n) }
    /// Visit a heap release.
    fn visit_free_mut(&mut self, n: &mut FreeStmt) { walk_free_mut(self, n) }
    /// Visit a null address.
    fn visit_null_mut(&mut self, n: &mut NullLit) { self.visit_span_mut(&mut n.span) }
    /// Visit a size expression.
    fn visit_size_of_mut(&mut self, n: &mut SizeOfExpr) { walk_size_of_mut(self, n) }
}

/// Visit the statements of a program.
pub fn walk_program_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut Program)
{
    v.visit_span_mut(&mut n.span);
    do_for!(s in &mut n.stmts, v.visit_stmt_mut(s));
}

/// Visit the statements of a block.
pub fn walk_block_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut BlockStmt)
{
    v.visit_span_mut(&mut n.span);
    do_for!(s in &mut n.stmts, v.visit_stmt_mut(s));
}

/// Visit an optional expression.
fn opt_expr_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut Option<Expr>)
{
    do_if!(let Some(e) = n, v.visit_expr_mut(e));
}

/// Visit an optional kind.
fn opt_kind_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut Option<Kind>)
{
    do_if!(let Some(k) = n, v.visit_kind_mut(k));
}

/// Visit the children of a statement.
pub fn walk_stmt_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut Stmt)
{
    match n
    {
        Stmt::Decl(s) => walk_decl_mut(v, s),
        Stmt::Assign(s) => walk_assign_mut(v, s),
        Stmt::Print(s) => walk_print_mut(v, s),
        Stmt::Block(s) => v.visit_block_mut(s),
        Stmt::If(s) => walk_if_mut(v, s),
        Stmt::While(s) => walk_while_mut(v, s),
        Stmt::Counted(s) => walk_counted_mut(v, s),
        Stmt::Generic(s) => walk_generic_mut(v, s),
        Stmt::Break(s) => v.visit_span_mut(&mut s.span),
        Stmt::Continue(s) => v.visit_span_mut(&mut s.span),
        Stmt::Fn(s) => walk_fn_mut(v, s),
        Stmt::Return(s) => walk_return_mut(v, s),
        Stmt::Call(s) => walk_call_stmt_mut(v, s),
        Stmt::Alloc(s) => v.visit_alloc_mut(s),
        Stmt::Resize(s) => v.visit_resize_mut(s),
        Stmt::Free(s) => v.visit_free_mut(s),
    }
}

/// Visit the children of a declaration.
pub fn walk_decl_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut DeclStmt)
{
    v.visit_span_mut(&mut n.span);
    v.visit_ident_mut(&mut n.name);
    v.visit_kind_mut(&mut n.kind);
    opt_expr_mut(v, &mut n.init);
}

/// Visit the children of an assignment.
pub fn walk_assign_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut AssignStmt)
{
    v.visit_span_mut(&mut n.span);
    v.visit_expr_mut(&mut n.target);
    v.visit_expr_mut(&mut n.value);
}

/// Visit the operand of a print statement.
pub fn walk_print_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut PrintStmt)
{
    v.visit_span_mut(&mut n.span);
    match &mut n.value
    {
        PrintArg::Expr(e) => v.visit_expr_mut(e),
        PrintArg::Str(s) => v.visit_span_mut(&mut s.span),
    }
}

/// Visit the arms and the else block of a conditional.
pub fn walk_if_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut IfStmt)
{
    v.visit_span_mut(&mut n.span);
    for arm in &mut n.arms
    {
        v.visit_span_mut(&mut arm.span);
        v.visit_expr_mut(&mut arm.cond);
        v.visit_block_mut(&mut arm.block);
    }
    do_if!(let Some(block) = &mut n.else_block, v.visit_block_mut(block));
}

/// Visit the condition and block of a while loop.
pub fn walk_while_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut WhileStmt)
{
    v.visit_span_mut(&mut n.span);
    v.visit_expr_mut(&mut n.cond);
    v.visit_block_mut(&mut n.block);
}

/// Visit the bounds, step, variable and block of a counted loop.
pub fn walk_counted_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut CountedStmt)
{
    v.visit_span_mut(&mut n.span);
    v.visit_expr_mut(&mut n.from);
    v.visit_expr_mut(&mut n.to);
    opt_expr_mut(v, &mut n.step);
    v.visit_ident_mut(&mut n.var);
    v.visit_block_mut(&mut n.block);
}

/// Visit the header, block and update of a generic loop.
pub fn walk_generic_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut GenericStmt)
{
    v.visit_span_mut(&mut n.span);
    v.visit_ident_mut(&mut n.var);
    v.visit_kind_mut(&mut n.kind);
    opt_expr_mut(v, &mut n.init);
    v.visit_expr_mut(&mut n.cond);
    v.visit_block_mut(&mut n.block);
    do_if!(let Some(update) = &mut n.update, v.visit_stmt_mut(update));
}

/// Visit the name, parameters, result kind and block of a function.
pub fn walk_fn_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut FnDecl)
{
    v.visit_span_mut(&mut n.span);
    v.visit_ident_mut(&mut n.name);
    for p in &mut n.params
    {
        v.visit_span_mut(&mut p.span);
        v.visit_kind_mut(&mut p.kind);
        v.visit_ident_mut(&mut p.name);
    }
    opt_kind_mut(v, &mut n.ret);
    v.visit_block_mut(&mut n.block);
}

/// Visit the value of a return statement.
pub fn walk_return_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut ReturnStmt)
{
    v.visit_span_mut(&mut n.span);
    opt_expr_mut(v, &mut n.value);
}

/// Visit the callee and arguments of a call statement.
pub fn walk_call_stmt_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut CallStmt)
{
    v.visit_span_mut(&mut n.span);
    v.visit_ident_mut(&mut n.callee);
    do_for!(a in &mut n.args, v.visit_expr_mut(a));
}

/// Visit the size, then the target, of a heap allocation.
pub fn walk_alloc_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut AllocStmt)
{
    v.visit_span_mut(&mut n.span);
    walk_size_arg_mut(v, &mut n.size);
    v.visit_expr_mut(&mut n.target);
}

/// Visit the target, then the new size, of a heap reallocation.
pub fn walk_resize_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut ResizeStmt)
{
    v.visit_span_mut(&mut n.span);
    v.visit_expr_mut(&mut n.target);
    walk_size_arg_mut(v, &mut n.size);
}

/// Visit the target of a heap release.
pub fn walk_free_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut FreeStmt)
{
    v.visit_span_mut(&mut n.span);
    v.visit_expr_mut(&mut n.target);
}

/// Visit the span, the count, then the unit kind of a size operand.
fn walk_size_arg_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut SizeArg)
{
    v.visit_span_mut(&mut n.span);
    v.visit_expr_mut(&mut n.count);
    opt_kind_mut(v, &mut n.unit);
}

/// Visit the children of an expression.
pub fn walk_expr_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut Expr)
{
    match n
    {
        Expr::Int(e) => walk_lit_mut(v, &mut e.span, &mut e.kind),
        Expr::Float(e) => walk_lit_mut(v, &mut e.span, &mut e.kind),
        Expr::Char(e) => v.visit_span_mut(&mut e.span),
        Expr::Bool(e) => v.visit_span_mut(&mut e.span),
        Expr::Name(e) => v.visit_span_mut(&mut e.span),
        Expr::Group(e) => walk_one_mut(v, &mut e.span, &mut e.inner),
        Expr::Prefix(e) => walk_one_mut(v, &mut e.span, &mut e.rhs),
        Expr::Infix(e) => walk_pair_mut(v, &mut e.span, &mut e.lhs, &mut e.rhs),
        Expr::Not(e) => walk_one_mut(v, &mut e.span, &mut e.inner),
        Expr::Logic(e) => walk_pair_mut(v, &mut e.span, &mut e.lhs, &mut e.rhs),
        Expr::Verb(e) => walk_verb_mut(v, e),
        Expr::Call(e) => walk_call_expr_mut(v, e),
        Expr::Null(e) => v.visit_null_mut(e),
        Expr::SizeOf(e) => v.visit_size_of_mut(e),
    }
}

/// Visit a literal span and its kind prefix.
fn walk_lit_mut<V: VisitorMut + ?Sized>(
    v: &mut V, span: &mut Span, kind: &mut Option<Kind>,
)
{
    v.visit_span_mut(span);
    opt_kind_mut(v, kind);
}

/// Visit a node span and its single operand.
fn walk_one_mut<V: VisitorMut + ?Sized>(v: &mut V, span: &mut Span, inner: &mut Expr)
{
    v.visit_span_mut(span);
    v.visit_expr_mut(inner);
}

/// Visit a node span and two operands.
fn walk_pair_mut<V: VisitorMut + ?Sized>(
    v: &mut V, span: &mut Span, lhs: &mut Expr, rhs: &mut Expr,
)
{
    v.visit_span_mut(span);
    v.visit_expr_mut(lhs);
    v.visit_expr_mut(rhs);
}

/// Visit the operands and the target kind of a verb clause.
pub fn walk_verb_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut VerbExpr)
{
    v.visit_span_mut(&mut n.span);
    do_for!(a in &mut n.args, v.visit_expr_mut(a));
    opt_kind_mut(v, &mut n.kind_arg);
}

/// Visit the callee and arguments of a call expression.
pub fn walk_call_expr_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut CallExpr)
{
    v.visit_span_mut(&mut n.span);
    v.visit_ident_mut(&mut n.callee);
    do_for!(a in &mut n.args, v.visit_expr_mut(a));
}

/// Visit the kind of a size expression.
pub fn walk_size_of_mut<V: VisitorMut + ?Sized>(v: &mut V, n: &mut SizeOfExpr)
{
    v.visit_span_mut(&mut n.span);
    v.visit_kind_mut(&mut n.kind);
}
