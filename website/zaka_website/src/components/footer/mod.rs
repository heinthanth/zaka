use leptos::ev;
use leptos::html::a;
use leptos::html::button;
use leptos::html::div;
use leptos::html::footer as footer_el;
use leptos::html::p;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::controls::modal::Modal;
use zaka_web_ui_common::controls::modal::modal;
use zaka_web_ui_common::credits::credits_roll;
use zaka_web_ui_common::credits::credits_sheet;
use zaka_web_ui_common::sheet;
use zakac_consts::ZAKA_AUTHOR;
use zakac_consts::ZAKA_FIRST_YEAR;
use zakac_consts::ZAKA_LICENSE;
use zakac_i18n::t;

use crate::components::header::EXPLORER_URL;
use crate::manual::LEARN_URL;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/footer/mod.sass");

/// Rules of the footer.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-footer", STYLE_SHEET);

/// Element id of the title of the credits modal, which names the dialog.
const CREDITS_TITLE_ID: &str = "credits-modal-title";

/// When the page was built, for the rights line.
const BUILT_IN: &str = "2026";

/// The footer: a gold field with notes at the top and a row of links and
/// rights at the foot.
///
/// # Returns
///
/// The view.
pub fn footer() -> impl IntoView
{
    let notes = div().class(ClassName::FOOTER_NOTES).child((
        p().child(t!("website.footer.what")),
        p().child(t!("website.footer.license", license = ZAKA_LICENSE)),
    ));
    let top = div().class(ClassName::FOOTER_TOP).child((
        notes,
        a().class(ClassName::FOOTER_BACK).href("#top").child(t!("website.footer.top")),
    ));
    let links = div().class(ClassName::FOOTER_LINKS).child((
        a().href(LEARN_URL).child(t!("website.footer.learn")),
        a().href(EXPLORER_URL).child(t!("website.footer.explorer")),
        CreditsLink(),
    ));
    let rights = p().class(ClassName::FOOTER_RIGHTS).child(t!(
        "website.footer.rights",
        author = ZAKA_AUTHOR,
        first = ZAKA_FIRST_YEAR,
        last = BUILT_IN
    ));
    footer_el().class(ClassName::FOOTER).attr("data-field", "gold").child((
        sheet(&SHEET),
        credits_sheet(),
        top,
        div().class(ClassName::FOOTER_META).child((links, rights)),
    ))
}

/// The credits link of the footer and the modal it opens: the fonts, the web
/// packages and the Rust crates Zaka builds on, in the roll the explorer
/// shows. An island: the modal and the roll run in the browser, and the roll
/// starts once the modal opens.
///
/// # Returns
///
/// The view.
#[island]
pub fn CreditsLink() -> impl IntoView
{
    let open = RwSignal::new(false);
    let spec = Modal::builder()
        .open(open)
        .class(ClassName::FOOTER_CREDITS_MODAL)
        .title_id(CREDITS_TITLE_ID)
        .title(t!("website.footer.credits_title"))
        .close_label(t!("website.footer.credits_close"))
        .build();
    let roll = move || open.get().then(credits_roll);
    let link = button()
        .class(ClassName::FOOTER_CREDITS)
        .r#type("button")
        .attr("aria-haspopup", "dialog")
        .attr("aria-expanded", move || open.get().to_string())
        .on(ev::click, move |_| open.set(true))
        .child(t!("website.footer.credits"));
    (link, modal(spec, roll))
}
