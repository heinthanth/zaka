use leptos::children::ToChildren;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_scoped_style::ScopeId;
use leptos_scoped_style::Scoped;
use leptos_scoped_style::ScopedChildren;
use leptos_scoped_style::component::ScopedProps;
use leptos_use_element::IntoElementsMaybeSignal;
use leptos_use_element::UseMutationObserverOptions;
use leptos_use_element::use_mutation_observer_with_options;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::MutationRecord;
use web_sys::Node;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;

/// Attribute on the host of a scope; the walk stops at a nested one.
const SCOPE_MARK: &str = "_leptosscope";

/// Attribute on an element whose contents belong to someone else, such as
/// the Monaco host; it is tagged, its contents are not.
pub const SLOT_MARK: &str = "_leptosslot";

/// The scoped sheet of a component: the scope name and the compiled CSS.
pub struct ScopedSheet
{
    /// Unique scope name; the scope attributes derive from it.
    pub name: &'static str,
    /// The CSS, scoped on first mount.
    pub sheet: ComponentStyle,
}

impl ScopedSheet
{
    /// Create a sheet.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique scope name.
    /// * `css` - The CSS, as `scss_file!` gives it.
    ///
    /// # Returns
    ///
    /// The sheet.
    pub const fn new(name: &'static str, css: &'static str) -> Self
    {
        ScopedSheet { name, sheet: ComponentStyle::css(name, css) }
    }
}

/// Wrap the root of a component in [`Scoped`] and keep tagging the nodes that
/// show up later, so the scoped rules match reactive content too. The crate
/// tags the subtree once, on mount.
///
/// # Arguments
///
/// * `sheet` - The sheet of the component.
/// * `root` - Node ref of the root element of `view`.
/// * `view` - The view.
///
/// # Returns
///
/// The scoped view.
pub fn scoped<R, M>(
    sheet: &'static ScopedSheet, root: R, view: impl IntoView + 'static,
) -> impl IntoView
where
    R: IntoElementsMaybeSignal<Element, M>,
{
    let id = ScopeId::from_name(sheet.name);
    let (host, content) = (id.host_attr(), id.content_attr());
    let options = UseMutationObserverOptions::default().child_list(true).subtree(true);
    let adopt_all = move |records: Vec<MutationRecord>, _| {
        do_for!(record in records, adopt_record(&record, &host, &content));
    };
    let _ = use_mutation_observer_with_options(root, adopt_all, options);
    let children = ScopedChildren::to_children(move || view);
    Scoped(ScopedProps::builder().style(&sheet.sheet).children(children).build())
}

/// Tag the nodes a mutation added that belong to this scope.
///
/// # Arguments
///
/// * `record` - The mutation.
/// * `host` - Host attribute of the scope.
/// * `content` - Content attribute of the scope.
fn adopt_record(record: &MutationRecord, host: &str, content: &str)
{
    let added = record.added_nodes();
    let nodes = (0..added.length()).filter_map(|idx| added.item(idx));
    do_for!(node in nodes, adopt(&node, host, content));
}

/// Tag an added element and its subtree when its nearest scope is this one.
///
/// # Arguments
///
/// * `node` - The added node; text nodes are left alone.
/// * `host` - Host attribute of the scope.
/// * `content` - Content attribute of the scope.
fn adopt(node: &Node, host: &str, content: &str)
{
    let Some(el) = node.dyn_ref::<Element>()
    else
    {
        return;
    };
    do_if!(belongs(el, host), tag_tree(el, content));
}

/// Whether the nearest scope host above `el` is the host of this scope, with
/// no slot between them.
///
/// # Arguments
///
/// * `el` - The element.
/// * `host` - Host attribute of the scope.
///
/// # Returns
///
/// `true` when `el` belongs to this scope.
fn belongs(el: &Element, host: &str) -> bool
{
    let mut above = el.parent_element();
    while let Some(node) = above
    {
        ret_true!(node.has_attribute(host));
        ret_false!(is_edge(&node));
        above = node.parent_element();
    }
    false
}

/// Tag `el` and its subtree, stopping at nested scopes and slots.
///
/// # Arguments
///
/// * `el` - The element.
/// * `content` - Content attribute of the scope.
fn tag_tree(el: &Element, content: &str)
{
    let _ = el.set_attribute(content, "");
    ret_if!(is_edge(el));
    let kids = el.children();
    do_for!(idx in 0..kids.length(), do_if!(let Some(kid) = kids.item(idx), tag_tree(&kid, content)));
}

/// Whether the walk stops at `el`: a nested scope host or a slot.
///
/// # Arguments
///
/// * `el` - The element.
///
/// # Returns
///
/// `true` at a nested scope host or a slot.
fn is_edge(el: &Element) -> bool
{
    el.has_attribute(SCOPE_MARK) || el.has_attribute(SLOT_MARK)
}
