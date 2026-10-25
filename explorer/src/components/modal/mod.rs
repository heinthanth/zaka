use bon::Builder;
use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::dialog;
use leptos::html::div;
use leptos::html::h2;
use leptos::prelude::*;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::components::icon::cross;

/// What a modal is called and when it shows.
#[derive(Clone, Copy, Builder)]
pub struct Modal
{
    /// Whether the modal shows.
    pub open: RwSignal<bool>,
    /// Class of the dialog, beside `modal`, for the sheet of its owner.
    pub class: &'static str,
    /// Element id of the title, which names the dialog.
    pub title_id: &'static str,
    /// Locale id of the title.
    pub title: &'static str,
    /// Locale id of the accessible name of the close button.
    pub close_label: &'static str,
}

/// A modal: a native dialog that holds a card with a title bar over
/// contents that scroll. The browser centers it, dims the page and keeps the
/// focus inside; it closes on Escape, on the close button and on a click on the
/// backdrop. It takes no sheet of its own, so the rules of its owner reach
/// the contents; the owner includes the `modal` mixin.
///
/// # Arguments
///
/// * `spec` - What the modal is called and when it shows.
/// * `content` - The contents.
///
/// # Returns
///
/// The view.
pub fn modal(spec: Modal, content: impl IntoView + 'static) -> impl IntoView
{
    let open = spec.open;
    let host = NodeRef::<html::Dialog>::new();
    follow_open(open, host);
    // The card fills the dialog, so a click that lands on the dialog itself
    // is a click on the backdrop.
    let on_click = move |e: ev::MouseEvent| {
        do_if!(e.target() == e.current_target(), open.set(false))
    };
    let shut = button()
        .class("icon-button modal-close")
        .attr("aria-label", t!(spec.close_label.to_string()))
        .title(t!("explorer.modal.close"))
        .on(ev::click, move |_| open.set(false))
        .child(cross());
    let name =
        h2().id(spec.title_id).class("modal-name").child(t!(spec.title.to_string()));
    let top = div().class("modal-top").child((name, shut));
    let card = div()
        .class("modal-card")
        .child((top, div().class("modal-scroll").child(content)));
    dialog()
        .class(format!("modal {}", spec.class))
        .node_ref(host)
        .attr("aria-labelledby", spec.title_id)
        .on(ev::close, move |_| open.set(false))
        .on(ev::click, on_click)
        .child(card)
}

/// Show or close the dialog as `open` changes.
///
/// # Arguments
///
/// * `open` - Whether the modal shows.
/// * `host` - The dialog.
pub fn follow_open(open: RwSignal<bool>, host: NodeRef<html::Dialog>)
{
    Effect::new(move |_| {
        let on = open.get();
        let Some(el) = host.get()
        else
        {
            return;
        };
        ret_if!(on == el.open());
        do_if!(on, {
            let _ = el.show_modal();
        });
        do_if!(!on, el.close());
    });
}
