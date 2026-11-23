//! The check of enumerations whose variants hold data: a variant with its
//! data, the test of a variant, the read of its data, and the name the data
//! takes in the block a test guards.

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::MemberExpr;
use zakac_ast::expr::VariantExpr;
use zakac_ast::expr::VerbExpr;
use zakac_ast::name::Name;
use zakac_ast::stmt::StructDecl;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_diagnostics::resolve as rd;
use zakac_lexer::reserved::AMONG;
use zakac_lexer::reserved::PLURAL;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::enums::stem_of;
use crate::generic::infer_kinds;
use crate::generic::instance_name;
use crate::generic::kind_ast;
use crate::kind::DataKind;
use crate::list::KindUse;
use crate::proof::result_var;
use crate::semcheck::Checker;
use crate::trait_value::held_kind;
use crate::variants::TAG_FIELD;
use crate::variants::Variants;
use crate::variants::data_field;

/// One variant of the enumeration of a value.
#[derive(Debug, Clone)]
pub(crate) struct VariantAt
{
    /// The enumeration name.
    pub owner: String,
    /// The stem of the enumeration name.
    pub stem: String,
    /// The position of the variant.
    pub at: usize,
    /// The kind of its data; `None` for a variant with no data.
    pub data: Option<DataKind>,
}

impl VariantAt
{
    /// The variant as a program writes it: `<stem>များထဲမှ <variant>`.
    ///
    /// # Arguments
    ///
    /// * `name` - The variant name.
    ///
    /// # Returns
    ///
    /// The text.
    pub(crate) fn written(&self, name: &str) -> String
    {
        format!("{}{PLURAL}{AMONG} {name}", self.stem)
    }
}

/// The test of a variant a condition is, through parentheses.
///
/// # Arguments
///
/// * `cond` - The condition.
///
/// # Returns
///
/// The test and the variant it names, or `None` for another condition.
pub(crate) fn variant_test(cond: &Expr) -> Option<(&VerbExpr, &VariantExpr)>
{
    match cond
    {
        Expr::Group(n) => variant_test(&n.inner),
        Expr::Verb(v) if v.op == VerbOp::IsVariant =>
        {
            let Some(Expr::Variant(variant)) = v.args.get(1)
            else
            {
                return None;
            };
            Some((v, variant))
        }
        _ => None,
    }
}

impl Checker
{
    /// The variant of a name in the enumeration of a kind. A kind that is no
    /// enumeration and a name it does not have are reported.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value.
    /// * `variant` - The variant name.
    /// * `value` - Where the value stands.
    ///
    /// # Returns
    ///
    /// The variant, or `None` after a report.
    pub(crate) fn variant_at(
        &mut self, kind: DataKind, variant: &Ident, value: zakac_span::span::Span,
    ) -> Option<VariantAt>
    {
        ret_if!(kind.is_trait_value(), self.held_kind_at(kind, variant));
        let found = match (kind.as_enum(), kind.as_variants())
        {
            (Some(e), _) => e
                .position(&variant.name)
                .map(|at| (e.name.clone(), e.stem().to_string(), at, None)),
            (_, Some(v)) => v.position(&variant.name).map(|at| {
                (
                    v.name().to_string(),
                    v.stem().to_string(),
                    at,
                    v.data(at).map(|(_, k)| k),
                )
            }),
            (None, None) =>
            {
                self.report(rd::not_an_enum_value(value, &kind.name()));
                return None;
            }
        };
        let Some((owner, stem, at, data)) = found
        else
        {
            let owner = kind.name();
            self.report(rd::unknown_member(variant.span, &owner, &variant.name));
            return None;
        };
        // A member of an instance refers to the member of its generic.
        let generic = owner.split('<').next().unwrap_or(&owner);
        let decl = self.decls.member(generic, &variant.name);
        do_if!(let Some(d) = decl, self.refer(&variant.name, variant.span, d));
        Some(VariantAt { owner, stem, at, data })
    }

    /// The generic enumeration a qualifier names, `<stem>များ`: the struct
    /// template it makes.
    ///
    /// # Arguments
    ///
    /// * `written` - The qualifier as written.
    ///
    /// # Returns
    ///
    /// The template, or `None` for another qualifier.
    pub(crate) fn generic_enum(&self, written: &str) -> Option<StructDecl>
    {
        let names = [written.strip_suffix(PLURAL), Some(written)];
        let template = names.into_iter().flatten().find_map(|n| {
            let t = self.templates.structs.get(&Name::new(n))?;
            t.fields
                .first()
                .is_some_and(|f| f.name.name.starts_with(TAG_FIELD))
                .then_some(t)
        })?;
        let stem = format!("{}{PLURAL}", stem_of(&template.name.name));
        (stem == written).then(|| template.clone())
    }

    /// Infer a member of a generic enumeration: its kinds as written, or the
    /// instance its place expects, or the kinds its data gives; then the
    /// member of that instance.
    ///
    /// # Arguments
    ///
    /// * `n` - The member.
    /// * `template` - The struct template of the enumeration.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The instance kind, or `None` while it does not exist or after a
    /// report.
    pub(crate) fn generic_member(
        &mut self, n: &MemberExpr, template: &StructDecl, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let span = n.qualifier.span;
        let (want, got) = (template.kind_params.len(), n.kind_args.len());
        let miscounted = got > 0 && got != want;
        ret_if!(
            miscounted,
            self.report(rd::kind_arg_count(span, &template.name.name, want, got)),
            None
        );
        let written = (!n.kind_args.is_empty()).then(|| {
            n.kind_args.iter().map(|k| self.written_kind(k, KindUse::Other)).collect()
        });
        let kinds = written
            .or_else(|| self.expected_kinds(template, expect))
            .or_else(|| self.member_data_kinds(template, n));
        let named = |ks: &Vec<DataKind>| {
            ks.len() == template.kind_params.len()
                && ks.iter().all(|k| kind_ast(*k, span).is_some())
        };
        let Some(kinds) = kinds.filter(named)
        else
        {
            self.report(rd::kinds_unknown(span, &template.name.name));
            return None;
        };
        let instance = instance_name(template.name.name, &kinds);
        self.struct_wanted.insert(instance, (template.name.name, kinds));
        self.uses.entry(instance).or_insert(span);
        let kind = self.structs.get(&instance).copied();
        do_if!(
            kind.is_none(),
            self.report(rd::instances_endless(span, &template.name.name))
        );
        let owner = kind?.as_variants()?;
        do_if!(let Some(d) = self.decls.name(&template.name.name), self.refer(&n.qualifier.name, span, d));
        self.infer_variant_member(n, owner)
    }

    /// The kinds of a generic enumeration that the data of a member gives,
    /// matched against the data kind the template writes for the member.
    ///
    /// # Arguments
    ///
    /// * `template` - The struct template of the enumeration.
    /// * `n` - The member.
    ///
    /// # Returns
    ///
    /// The kinds, or `None` when one remains unknown.
    fn member_data_kinds(
        &mut self, template: &StructDecl, n: &MemberExpr,
    ) -> Option<Vec<DataKind>>
    {
        let names: Vec<Name> =
            template.kind_params.iter().map(|p| p.name.name).collect();
        let listed = template
            .fields
            .first()?
            .name
            .name
            .split('|')
            .skip(1)
            .map(|v| v.trim_start_matches('!'))
            .position(|v| v == n.member.name.as_str())?;
        let field = template
            .fields
            .iter()
            .find(|p| p.name.name == data_field(listed).as_str())?;
        let found = self.quiet_kinds(std::iter::once(n.data.as_deref()?));
        infer_kinds(&names, [(&field.kind, found[0])], &self.known_structs)
    }

    /// Infer a member of an enumeration whose variants hold data, with the
    /// data of a variant that holds some.
    ///
    /// # Arguments
    ///
    /// * `n` - The member.
    /// * `owner` - The enumeration.
    ///
    /// # Returns
    ///
    /// The kind of the enumeration, or `None` after an error.
    pub(crate) fn infer_variant_member(
        &mut self, n: &MemberExpr, owner: Variants,
    ) -> Option<DataKind>
    {
        let at = self.variant_at(owner.kind(), &n.member, n.qualifier.span)?;
        let written = at.written(&n.member.name);
        match (at.data, n.data.as_deref())
        {
            (Some(kind), Some(value)) => self.expect_kind(value, Some(kind)),
            (Some(kind), None) =>
            {
                let need = rd::variant_needs_data(
                    n.span,
                    &at.owner,
                    &n.member.name,
                    &kind.name(),
                    &written,
                );
                self.report(need);
            }
            (None, Some(value)) =>
            {
                let none = rd::variant_has_no_data(
                    value.span(),
                    &at.owner,
                    &n.member.name,
                    &written,
                );
                self.report(none);
            }
            (None, None) => (),
        }
        Some(owner.kind())
    }

    /// Infer the test of a variant, `<x> သည် [<name> ဟူသည့်] <variant>
    /// အဖြစ်သို့ ပြောင်းလဲနိုင်သည်`. A name for the data stands in the whole
    /// condition of a conditional alone, and names data that the variant
    /// holds.
    ///
    /// # Arguments
    ///
    /// * `n` - The test.
    /// * `value` - The value tested.
    /// * `variant` - The variant.
    ///
    /// # Returns
    ///
    /// `bool`, or `None` after an error in the value.
    pub(crate) fn infer_is_variant(
        &mut self, n: &VerbExpr, value: &Expr, variant: &VariantExpr,
    ) -> Option<DataKind>
    {
        let kind = self.infer(value, None)?;
        let at = self.variant_at(kind, &variant.variant, value.span());
        let Some(bind) = &variant.bind
        else
        {
            return Some(DataKind::Bool);
        };
        let outside = self.bind_site != Some(n.id);
        ret_if!(
            outside,
            self.report(rd::name_outside_test(bind.span, &bind.name)),
            Some(DataKind::Bool)
        );
        let no_data = at.as_ref().filter(|a| a.data.is_none());
        do_if!(let Some(a) = no_data, {
            let written = a.written(&variant.variant.name);
            self.report(rd::variant_has_no_data(bind.span, &a.owner, &variant.variant.name, &written));
        });
        Some(DataKind::Bool)
    }

    /// Infer the read of the data of a variant, `<x> ကို <variant> အဖြစ်
    /// ပြောင်းလဲသော ရလဒ်`. A read of a variant that a test ruled out for the
    /// variable is reported.
    ///
    /// # Arguments
    ///
    /// * `value` - The value read.
    /// * `variant` - The variant.
    ///
    /// # Returns
    ///
    /// The kind of the data, or `None` after an error.
    pub(crate) fn infer_variant_data(
        &mut self, value: &Expr, variant: &VariantExpr,
    ) -> Option<DataKind>
    {
        let kind = self.infer(value, None)?;
        let at = self.variant_at(kind, &variant.variant, value.span())?;
        let name = &variant.variant.name;
        let Some(data) = at.data
        else
        {
            let written = at.written(name);
            self.report(rd::variant_has_no_data(
                variant.span,
                &at.owner,
                name,
                &written,
            ));
            return None;
        };
        let var = result_var(value);
        let proven = var.and_then(|v| self.variants_proven.get(&v).map(|p| (v, *p)));
        do_if!(let Some((v, p)) = proven.filter(|(_, p)| p != name), {
            self.report(rd::wrong_variant_proven(value.span(), &v, &p, name));
        });
        Some(data)
    }

    /// The name a condition gives the data of a variant, with the kind of
    /// the data, once the condition is checked.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    ///
    /// # Returns
    ///
    /// The name and the kind, or `None` when the condition names no data.
    pub(crate) fn cond_bind(&self, cond: &Expr) -> Option<(Ident, DataKind)>
    {
        let (test, variant) = variant_test(cond)?;
        let bind = variant.bind.clone()?;
        let kind = self.kinds.get(&test.args[0].id()).copied()?;
        Some((bind, self.data_of(kind, &variant.variant.name)?))
    }

    /// The kind of the data of a variant of the enumeration of a kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value.
    /// * `variant` - The variant name.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for no such variant or a variant with no data.
    pub(crate) fn data_of(&self, kind: DataKind, variant: &str) -> Option<DataKind>
    {
        ret_if!(kind.is_trait_value(), held_kind(variant, &self.structs));
        let owner = kind.as_variants()?;
        owner.data(owner.position(variant)?).map(|(_, k)| k)
    }

    /// The variant a condition proves a variable holds, when it holds.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    ///
    /// # Returns
    ///
    /// The variable and the variant, or `None`.
    pub(crate) fn variant_fact(cond: &Expr) -> Option<(Name, Name)>
    {
        let (test, variant) = variant_test(cond)?;
        Some((result_var(&test.args[0])?, variant.variant.name))
    }

    /// Check a block with a name for the data of a variant in a scope of its
    /// own around it.
    ///
    /// # Arguments
    ///
    /// * `bind` - The name and the kind of the data, when one is given.
    /// * `visit` - The check of the block.
    pub(crate) fn with_bind(
        &mut self, bind: Option<(Ident, DataKind)>, visit: impl FnOnce(&mut Checker),
    )
    {
        let Some((name, kind)) = bind
        else
        {
            return visit(self);
        };
        self.scopes.push();
        self.proven.remove(&name.name);
        self.variants_proven.remove(&name.name);
        self.declare(&name, Some(kind), crate::scope::Access::Var);
        visit(self);
        self.scopes.pop();
    }
}
