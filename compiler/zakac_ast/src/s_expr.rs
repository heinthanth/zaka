use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::expr::Expr;
use crate::expr::Ident;
use crate::expr::IntLit;
use crate::expr::StrExpr;
use crate::expr::StrPiece;
use crate::expr::VerbExpr;
use crate::kind::BaseKind;
use crate::kind::Cells;
use crate::kind::Kind;
use crate::kind::Wrap;
use crate::stmt::AliasDecl;
use crate::stmt::BlockStmt;
use crate::stmt::CountedStmt;
use crate::stmt::DeclStmt;
use crate::stmt::EnumDecl;
use crate::stmt::FnDecl;
use crate::stmt::GenericStmt;
use crate::stmt::IfStmt;
use crate::stmt::ImportStmt;
use crate::stmt::Param;
use crate::stmt::Program;
use crate::stmt::SizeArg;
use crate::stmt::Stmt;
use crate::stmt::StructDecl;
use crate::stmt::SwitchStmt;
use crate::verb::Dir;
use crate::verb::VerbOp;

/// Conversion to the S-expression form.
pub trait ToSexpr
{
    /// The S-expression form of the node.
    ///
    /// # Returns
    ///
    /// The text.
    fn to_sexpr(&self) -> String;
}

impl ToSexpr for Program
{
    fn to_sexpr(&self) -> String
    {
        list("program", self.stmts.iter().map(ToSexpr::to_sexpr))
    }
}

impl ToSexpr for BlockStmt
{
    fn to_sexpr(&self) -> String
    {
        list("block", self.stmts.iter().map(ToSexpr::to_sexpr))
    }
}

impl ToSexpr for Kind
{
    fn to_sexpr(&self) -> String
    {
        // The opaque kind's own `လိပ်စာ` is its name, not a star. Each wrap
        // writes its mark after the kind inside it, in the order of the
        // Burmese words.
        let skip = usize::from(self.base == BaseKind::Opaque);
        let marks: Vec<String> = self.wraps.iter().skip(skip).map(wrap_mark).collect();
        // A borrow takes the place of the `*` of the last address level.
        let last = marks.iter().rposition(|m| m == "*").filter(|_| self.borrowed);
        let mark = |(at, m): (usize, &String)| {
            tern!(Some(at) == last, "&".to_string(), m.clone())
        };
        let marks: String = marks.iter().enumerate().map(mark).collect();
        let extra = usize::from(self.extra_borrows)
            + usize::from(self.borrowed && last.is_none());
        format!("{}{marks}{}", base_name(self), "&".repeat(extra))
    }
}

impl ToSexpr for Expr
{
    fn to_sexpr(&self) -> String
    {
        match self
        {
            Expr::Int(n) => int_sexpr(n),
            Expr::Float(n) => format!("{:?}", n.value),
            Expr::Char(n) => format!("'{}'", n.value),
            Expr::Bool(n) => n.value.to_string(),
            Expr::Name(n) => n.name.to_string(),
            Expr::Group(n) => format!("[{}]", n.inner.to_sexpr()),
            Expr::Prefix(n) => list(n.op.as_ref(), [n.rhs.to_sexpr()]),
            Expr::Infix(n) => list(n.op.as_ref(), [n.lhs.to_sexpr(), n.rhs.to_sexpr()]),
            Expr::Not(n) => list("not", [n.inner.to_sexpr()]),
            Expr::Logic(n) => list(n.op.as_ref(), [n.lhs.to_sexpr(), n.rhs.to_sexpr()]),
            Expr::Verb(n) => verb_sexpr(n),
            Expr::Call(n) =>
            {
                let word = tern!(n.pass, "call-pass", "call");
                list(
                    word,
                    call_parts(&callee_text(&n.callee, &n.place), &n.args, &n.edits),
                )
            }
            Expr::Null(_) => "null".into(),
            Expr::SizeOf(n) => list("sizeof", [n.kind.to_sexpr()]),
            Expr::Index(n) =>
            {
                let head = tern!(n.first == 0, "cell", "cell1");
                list(head, [n.list.to_sexpr(), n.index.to_sexpr()])
            }
            Expr::Field(n) =>
            {
                list("field", [n.base.to_sexpr(), n.field.name.to_string()])
            }
            Expr::Range(n) => list(
                "range",
                [n.source.to_sexpr(), n.start.to_sexpr(), n.len.to_sexpr()],
            ),
            Expr::List(n) => list("list", n.cells.iter().map(ToSexpr::to_sexpr)),
            Expr::Struct(n) =>
            {
                let fields =
                    n.fields.iter().map(|f| list(&f.field.name, [f.value.to_sexpr()]));
                list("struct", std::iter::once(n.name.name.to_string()).chain(fields))
            }
            Expr::BlankList(_) => "blank_list".into(),
            Expr::Str(n) => str_sexpr(n),
            Expr::Member(n) => list(
                "member",
                [n.qualifier.name.to_string(), n.member.name.to_string()],
            ),
            Expr::FnAddr(n) => list("fn_addr", [n.name.name.to_string()]),
        }
    }
}

impl ToSexpr for Stmt
{
    fn to_sexpr(&self) -> String
    {
        match self
        {
            Stmt::Decl(s) => exported(s.exported, decl_sexpr(s)),
            Stmt::Assign(s) => list("set", [s.target.to_sexpr(), s.value.to_sexpr()]),
            Stmt::Print(s) => list("print", [s.value.to_sexpr()]),
            Stmt::Block(s) => s.to_sexpr(),
            Stmt::If(s) => if_sexpr(s),
            Stmt::While(s) => list("while", [s.cond.to_sexpr(), s.block.to_sexpr()]),
            Stmt::Counted(s) => counted_sexpr(s),
            Stmt::Each(s) =>
            {
                let head = tern!(s.chars, "each_char", "each_cell");
                list(
                    head,
                    [s.var.name.to_string(), s.source.to_sexpr(), s.block.to_sexpr()],
                )
            }
            Stmt::Generic(s) => generic_sexpr(s),
            Stmt::Break(_) => "(break)".into(),
            Stmt::Continue(_) => "(continue)".into(),
            Stmt::Fn(s) => exported(s.exported, fn_sexpr(s)),
            Stmt::Return(s) =>
            {
                let word = tern!(
                    s.fault,
                    "return-fault",
                    tern!(s.good, "return-good", "return")
                );
                list(word, s.value.iter().map(ToSexpr::to_sexpr))
            }
            Stmt::Call(s) =>
            {
                let word = tern!(s.pass, "call-pass", "call");
                list(
                    word,
                    call_parts(&callee_text(&s.callee, &s.place), &s.args, &s.edits),
                )
            }
            Stmt::Alloc(s) => list("alloc", heap_parts(&s.target, &s.size)),
            Stmt::Resize(s) => list("resize", heap_parts(&s.target, &s.size)),
            Stmt::Free(s) => list("free", [s.target.to_sexpr()]),
            Stmt::Blit(s) =>
            {
                let head = tern!(s.overlap, "shift", "blit");
                let parts = heap_parts(&s.src, &s.size);
                list(head, parts.into_iter().chain([s.dst.to_sexpr()]))
            }
            Stmt::Fill(s) =>
            {
                let parts = heap_parts(&s.dst, &s.size);
                list("fill", parts.into_iter().chain([s.value.to_sexpr()]))
            }
            Stmt::Add(s) =>
            {
                let word = tern!(s.before, "insert", "add");
                list(word, [s.list.to_sexpr(), s.value.to_sexpr()])
            }
            Stmt::Remove(s) =>
            {
                list(tern!(s.chars, "remove_char", "remove"), [s.list.to_sexpr()])
            }
            Stmt::Replace(s) =>
            {
                let parts = [&s.text, &s.start, &s.end, &s.value].map(|e| e.to_sexpr());
                list(tern!(s.chars, "replace_chars", "replace"), parts)
            }
            Stmt::Struct(s) => exported(s.exported, struct_sexpr(s)),
            Stmt::Import(s) => import_sexpr(s),
            Stmt::Enum(s) => exported(s.exported, enum_sexpr(s)),
            Stmt::Switch(s) => switch_sexpr(s),
            Stmt::Alias(s) => exported(s.exported, alias_sexpr(s)),
        }
    }
}

/// `(export <decl>)` for an exported declaration; the declaration alone
/// otherwise.
///
/// # Arguments
///
/// * `flag` - Whether the declaration is exported.
/// * `decl` - The declaration in S-expression form.
///
/// # Returns
///
/// The text.
fn exported(flag: bool, decl: String) -> String
{
    tern!(flag, list("export", [decl]), decl)
}

/// `(import "file" name name)`.
///
/// # Arguments
///
/// * `s` - The import.
///
/// # Returns
///
/// The text.
fn import_sexpr(s: &ImportStmt) -> String
{
    let path = format!("\"{}\"", s.path.lit.value);
    list(
        "import",
        std::iter::once(path).chain(s.names.iter().map(|n| n.name.to_string())),
    )
}

/// `(head part part)`; just `(head)` without parts.
///
/// # Arguments
///
/// * `head` - The first word.
/// * `parts` - The rest, in S-expression form.
///
/// # Returns
///
/// The list.
fn list(head: &str, parts: impl IntoIterator<Item = String>) -> String
{
    let parts: Vec<String> = parts.into_iter().collect();
    ret_if!(parts.is_empty(), format!("({head})"));
    format!("({head} {})", parts.join(" "))
}

/// The mark a wrap puts after the kind inside it: `*`, `^`, `[3]`, `[]`,
/// `[][..]` or `[..]`.
///
/// # Arguments
///
/// * `w` - The wrap.
///
/// # Returns
///
/// The mark.
fn wrap_mark(w: &Wrap) -> String
{
    match w
    {
        Wrap::Ptr => "*".into(),
        Wrap::RawPtr => "^".into(),
        Wrap::List(Cells::Fixed(n)) => format!("[{n}]"),
        Wrap::List(Cells::Expr(e)) => format!("[{}]", e.to_sexpr()),
        Wrap::List(Cells::Inferred) => "[]".into(),
        Wrap::Slice => "[][..]".into(),
        Wrap::View => "[..]".into(),
    }
}

/// The name of a kind without its pointer stars: `u8`, `isize`, `f64`,
/// `bool` or `ptr`.
///
/// # Arguments
///
/// * `k` - The kind.
///
/// # Returns
///
/// The name.
fn base_name(k: &Kind) -> String
{
    let bits = k.bits.map_or("size".to_string(), |b| b.to_string());
    let sign = tern!(k.signed == Some(false), 'u', 'i');
    match k.base
    {
        BaseKind::Int => format!("{sign}{bits}"),
        BaseKind::Float => format!("f{}", k.bits.unwrap_or(64)),
        BaseKind::Opaque if k.wraps.first() == Some(&Wrap::RawPtr) => "rawptr".into(),
        BaseKind::Named =>
        {
            k.name.as_ref().map_or(String::new(), |n| n.name.to_string())
        }
        BaseKind::Result =>
        {
            let side = |k: &Option<Kind>, none: &str| {
                k.as_ref().map_or(none.to_string(), ToSexpr::to_sexpr)
            };
            let parts = k.parts.as_deref();
            let ok = parts.map_or("-".to_string(), |p| side(&p.ok, "-"));
            let err = parts.map_or("-".to_string(), |p| side(&p.err, "-"));
            format!("({ok} ! {err})")
        }
        BaseKind::Fn => fn_kind_sexpr(k),
        BaseKind::Char | BaseKind::Bool | BaseKind::Text | BaseKind::Opaque =>
        {
            k.base.as_ref().to_string()
        }
    }
}

/// A function kind as `fn(<param>, …) -> <result>`, each parameter with its
/// edit flag, its name and its mark when written.
///
/// # Arguments
///
/// * `k` - The kind; its base is [`BaseKind::Fn`].
///
/// # Returns
///
/// The text.
fn fn_kind_sexpr(k: &Kind) -> String
{
    let Some(sig) = k.sig.as_deref()
    else
    {
        return "fn()".into();
    };
    let param = |p: &crate::kind::SigParam| {
        let edit = tern!(p.edit, "mut ", "");
        let name = p.name.as_ref().map_or(String::new(), |n| format!(" {}", n.name));
        let mark = p.mark.as_ref().map_or(String::new(), |m| format!("({})", m.name));
        format!("{edit}{}{name}{mark}", p.kind.to_sexpr())
    };
    let params: Vec<String> = sig.params.iter().map(param).collect();
    let ret =
        sig.ret.as_ref().map_or(String::new(), |r| format!(" -> {}", r.to_sexpr()));
    format!("fn({}){ret}", params.join(", "))
}

/// An integer literal with its kind prefix as a suffix: `18u8`.
///
/// # Arguments
///
/// * `n` - The literal.
///
/// # Returns
///
/// The text.
fn int_sexpr(n: &IntLit) -> String
{
    let kind = n.kind.as_ref().map_or(String::new(), ToSexpr::to_sexpr);
    format!("{}{kind}", n.value)
}

/// `(op arg arg kind)` for a verb clause.
///
/// # Arguments
///
/// * `n` - The clause.
///
/// # Returns
///
/// The text.
fn verb_sexpr(n: &VerbExpr) -> String
{
    let args = n.args.iter().map(ToSexpr::to_sexpr);
    list(verb_name(n), args.chain(n.kind_arg.iter().map(ToSexpr::to_sexpr)))
}

/// The callee of a call as an s-expression: the name, or the place that
/// holds the function value.
///
/// # Arguments
///
/// * `callee` - The function name.
/// * `place` - The place, for a call through a field, a cell or a pointer.
///
/// # Returns
///
/// The text.
fn callee_text(callee: &Ident, place: &Option<Box<Expr>>) -> String
{
    place.as_ref().map_or_else(|| callee.name.to_string(), |p| p.to_sexpr())
}

/// The callee name followed with the arguments.
///
/// # Arguments
///
/// * `callee` - The function name.
/// * `args` - The arguments.
///
/// # Returns
///
/// The parts of a call list.
fn call_parts(callee: &str, args: &[Expr], edits: &[bool]) -> Vec<String>
{
    let marked = |(n, a): (usize, &Expr)| {
        let edit = edits.get(n).copied().unwrap_or(false);
        tern!(edit, list("edit", [a.to_sexpr()]), a.to_sexpr())
    };
    std::iter::once(callee.to_string())
        .chain(args.iter().enumerate().map(marked))
        .collect()
}

/// The target, count and optional unit of a heap statement.
///
/// # Arguments
///
/// * `target` - The place that holds the address.
/// * `size` - The size operand.
///
/// # Returns
///
/// The parts of a heap list.
fn heap_parts(target: &Expr, size: &SizeArg) -> Vec<String>
{
    let head = [target.to_sexpr(), size.count.to_sexpr()];
    head.into_iter().chain(size.unit.iter().map(ToSexpr::to_sexpr)).collect()
}

/// A string literal in quotes, each template slot as `{<expression>}`.
///
/// # Arguments
///
/// * `n` - The literal.
///
/// # Returns
///
/// The text.
fn str_sexpr(n: &StrExpr) -> String
{
    let piece = |p: &StrPiece| match p
    {
        StrPiece::Text(text) => text.clone(),
        StrPiece::TemplateSlot(e) => format!("{{{}}}", e.to_sexpr()),
    };
    format!("\"{}\"", n.pieces.iter().map(piece).collect::<String>())
}

/// `(var name kind init)` or `(const name kind init)`.
///
/// # Arguments
///
/// * `s` - The declaration.
///
/// # Returns
///
/// The text.
fn decl_sexpr(s: &DeclStmt) -> String
{
    let head = tern!(s.is_const, "const", "var");
    let header = [s.name.name.to_string(), kind_or_blank(s.kind.as_ref())];
    list(head, header.into_iter().chain(s.init.iter().map(ToSexpr::to_sexpr)))
}

/// The tree of a written kind, or `_` for a kind the value gives.
///
/// # Arguments
///
/// * `kind` - The kind, when written.
///
/// # Returns
///
/// The text.
fn kind_or_blank(kind: Option<&Kind>) -> String
{
    kind.map_or_else(|| "_".to_string(), ToSexpr::to_sexpr)
}

/// `(if cond {..} cond {..} else {..})`.
///
/// # Arguments
///
/// * `s` - The conditional.
///
/// # Returns
///
/// The text.
fn if_sexpr(s: &IfStmt) -> String
{
    let arms =
        s.arms.iter().flat_map(|arm| [arm.cond.to_sexpr(), arm.block.to_sexpr()]);
    let else_part =
        s.else_block.iter().flat_map(|b| ["else".to_string(), b.to_sexpr()]);
    list("if", arms.chain(else_part))
}

/// `(switch subject case {..} case {..} else kind {..})`.
///
/// # Arguments
///
/// * `s` - The switch.
///
/// # Returns
///
/// The text.
fn switch_sexpr(s: &SwitchStmt) -> String
{
    let arms =
        s.arms.iter().flat_map(|arm| [cases_sexpr(&arm.cases), arm.block.to_sexpr()]);
    let default = s
        .default
        .iter()
        .flat_map(|d| ["else".to_string(), d.kind.to_sexpr(), d.block.to_sexpr()]);
    list("switch", std::iter::once(s.subject.to_sexpr()).chain(arms).chain(default))
}

/// The cases of a switch arm: the case alone, or `(cases a b)` for several.
///
/// # Arguments
///
/// * `cases` - The cases.
///
/// # Returns
///
/// The text.
fn cases_sexpr(cases: &[Expr]) -> String
{
    let [one] = cases
    else
    {
        return list("cases", cases.iter().map(ToSexpr::to_sexpr));
    };
    one.to_sexpr()
}

/// `(count var from to step N mode {..})`.
///
/// # Arguments
///
/// * `s` - The counted loop.
///
/// # Returns
///
/// The text.
fn counted_sexpr(s: &CountedStmt) -> String
{
    let bounds = [s.var.name.to_string(), s.from.to_sexpr(), s.to.to_sexpr()];
    let step = s.step.iter().flat_map(|e| ["step".to_string(), e.to_sexpr()]);
    let tail = [s.mode.as_ref().to_string(), s.block.to_sexpr()];
    list("count", bounds.into_iter().chain(step).chain(tail))
}

/// `(for var kind init cond {..} then update)`.
///
/// # Arguments
///
/// * `s` - The generic loop.
///
/// # Returns
///
/// The text.
fn generic_sexpr(s: &GenericStmt) -> String
{
    let header = [s.var.name.to_string(), kind_or_blank(s.kind.as_ref())];
    let init = s.init.iter().map(ToSexpr::to_sexpr);
    let loop_part = [s.cond.to_sexpr(), s.block.to_sexpr()];
    let update = s.update.iter().flat_map(|u| ["then".to_string(), u.to_sexpr()]);
    list("for", header.into_iter().chain(init).chain(loop_part).chain(update))
}

/// `(fn name [kind param, kind param] ret {..})`; `-` for no result. An
/// external function is `(external name ...)`, with its lib first when it
/// has one: `(external "libm" name ...)`.
///
/// # Arguments
///
/// * `s` - The function.
///
/// # Returns
///
/// The text.
fn fn_sexpr(s: &FnDecl) -> String
{
    let mut params: Vec<String> = s.params.iter().map(param_sexpr).collect();
    do_if!(s.variadic, params.push("...".to_string()));
    let ret = s.ret.as_ref().map_or("-".to_string(), ToSexpr::to_sexpr);
    let parts = [s.name.name.to_string(), format!("[{}]", params.join(", ")), ret];
    let lib = s.lib.iter().map(|l| format!("\"{}\"", l.lit.value));
    ret_if!(s.external, list("external", lib.chain(parts)));
    list(tern!(s.start, "start", "fn"), parts.into_iter().chain([s.block.to_sexpr()]))
}

/// `(struct name [kind field, ...] classifier)` for a struct declaration.
///
/// # Arguments
///
/// * `s` - The declaration.
///
/// # Returns
///
/// The text.
fn struct_sexpr(s: &StructDecl) -> String
{
    let fields: Vec<String> = s.fields.iter().map(param_sexpr).collect();
    let parts = [s.name.name.to_string(), format!("[{}]", fields.join(", "))];
    list(
        "struct",
        parts.into_iter().chain(s.classifier.iter().map(|c| c.name.to_string())),
    )
}

/// `(enum name [member, member value, ...])` for an enumeration declaration.
///
/// # Arguments
///
/// * `s` - The declaration.
///
/// # Returns
///
/// The text.
fn enum_sexpr(s: &EnumDecl) -> String
{
    let member = |m: &crate::stmt::EnumMember| {
        let value = m.value.iter().map(|v| format!(" {}", v.to_sexpr()));
        format!("{}{}", m.name.name, value.collect::<String>())
    };
    let members: Vec<String> = s.members.iter().map(member).collect();
    list("enum", [s.name.name.to_string(), format!("[{}]", members.join(", "))])
}

/// `(alias name kind)` for a kind alias.
///
/// # Arguments
///
/// * `s` - The declaration.
///
/// # Returns
///
/// The text.
fn alias_sexpr(s: &AliasDecl) -> String
{
    list("alias", [s.name.name.to_string(), s.kind.to_sexpr()])
}

/// `kind name` for a parameter.
///
/// # Arguments
///
/// * `p` - The parameter.
///
/// # Returns
///
/// The text.
fn param_sexpr(p: &Param) -> String
{
    let edit = tern!(p.edit, "edit ", "");
    let mark = p.mark.as_ref().map(|m| format!("({})", m.name)).unwrap_or_default();
    format!("{edit}{} {}{mark}", p.kind.to_sexpr(), p.name.name)
}

/// Name of a verb operation; a shift takes the name of its direction.
///
/// # Arguments
///
/// * `n` - The verb clause.
///
/// # Returns
///
/// The name.
fn verb_name(n: &VerbExpr) -> &str
{
    ret_if!(n.op != VerbOp::Shift, n.op.as_ref());
    tern!(n.dir == Some(Dir::Left), "shl", "shr")
}
