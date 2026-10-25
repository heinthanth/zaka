use std::borrow::Cow;

use leptos::html;
use leptos::html::a;
use leptos::html::div;
use leptos::html::h2;
use leptos::html::h3;
use leptos::html::p;
use leptos::html::span;
use leptos::prelude::*;
use leptos_use_interval_fn::use_interval_fn;
use leptos_use_preferences::use_prefers_reduced_motion;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use super::ClassName;
use crate::credits::Project;
use crate::credits::Source;
use crate::credits::projects;

/// The Facebook page of PhoenixDigitalArt, who made PDA10 Burmese Mono.
const PDA_PAGE: &str = "https://www.facebook.com/PhoenixDigitalArt";

/// The Facebook post that released PDA10 Burmese Mono.
const PDA_POST: &str = "https://www.facebook.com/share/p/14td2Bwv4Jh/";

/// The source of Source Code Pro, which Sauce Code Pro (HTMM's Custom Font)
/// is built on.
const SOURCE_REPO: &str = "https://github.com/adobe-fonts/source-code-pro";

/// Fontsource, which serves the other fonts of the pickers.
const FONTSOURCE: &str = "https://fontsource.org";

/// Google Fonts, where most of the other fonts come from.
const GOOGLE_FONTS: &str = "https://fonts.google.com";

/// The SIL Open Font License, under which PDA10 and Sauce Code Pro ship.
const OFL: &str = "https://openfontlicense.org";

/// The Apache License 2.0, under which Myanmar Sabae ships.
const APACHE: &str = "https://www.apache.org/licenses/LICENSE-2.0";

/// Pixels the roll moves each tick.
const STEP_PX: i32 = 1;

/// Time between two ticks of the roll, in ms.
const TICK_MS: u64 = 30;

/// The credits roll: the fonts, the npm packages and the Rust crates. It
/// scrolls on its own and starts over at its end; the reader can scroll it
/// back or on whenever, and it goes on from where they stop. Without
/// motion it is a plain list.
///
/// # Returns
///
/// The view.
pub fn credits_roll() -> impl IntoView
{
    let all = projects();
    let group = |source: Source| {
        let picked = all.iter().filter(move |p| p.source == source).copied();
        picked.map(project_line).collect_view()
    };
    let npm = group(Source::Npm);
    let crates = group(Source::Crate);
    let reel = div().class(ClassName::CREDITS_REEL).child((
        h2().class(ClassName::CREDITS_TITLE).child(t!("explorer.about.credits")),
        fonts(),
        head("explorer.about.web"),
        npm,
        head("explorer.about.crates"),
        crates,
        p().class(ClassName::CREDITS_THANKS).child(t!("explorer.about.thanks")),
    ));
    let host = NodeRef::<html::Div>::new();
    roll_on(host);
    div()
        .class(ClassName::CREDITS_ROLL)
        .node_ref(host)
        .attr("tabindex", "0")
        .attr("aria-label", t!("explorer.about.credits"))
        .child(reel)
}

/// Move the roll on a timer; at its end it starts over. A tick that finds
/// the roll away from where the last tick left it knows the reader is
/// scrolling: it takes up their place and moves on from there at the next
/// still tick. A step of one pixel is not a scroll, so rounding of the
/// browser does not hold the roll.
///
/// # Arguments
///
/// * `host` - The roll.
fn roll_on(host: NodeRef<html::Div>)
{
    let still = use_prefers_reduced_motion();
    let left_at = StoredValue::new(0);
    let tick = move || {
        ret_if!(still.get_untracked());
        let Some(el) = host.get_untracked()
        else
        {
            return;
        };
        let (at, ours) = (el.scroll_top(), left_at.get_value());
        ret_if!((at - ours).abs() > STEP_PX, left_at.set_value(at));
        let end = el.scroll_height() - el.client_height();
        let next = tern!(ours + STEP_PX >= end, 0, ours + STEP_PX);
        el.set_scroll_top(next);
        left_at.set_value(next);
    };
    let _ = use_interval_fn(tick, TICK_MS);
}

/// The head of a group of the roll.
///
/// # Arguments
///
/// * `id` - Locale id of the head.
///
/// # Returns
///
/// The view.
fn head(id: &'static str) -> impl IntoView
{
    h3().class(ClassName::CREDITS_HEAD).child(t!(id.to_string()))
}

/// One project: its name as a link, then its version and license.
///
/// # Arguments
///
/// * `project` - The project.
///
/// # Returns
///
/// The view.
fn project_line(project: Project) -> impl IntoView
{
    let meta = format!("{} · {}", project.version, project.license);
    div().class(ClassName::CREDITS_ITEM).child((
        a().attr("href", project.url)
            .attr("target", "_blank")
            .attr("rel", "noopener noreferrer")
            .child(project.name),
        span().class(ClassName::CREDITS_META).child(meta),
    ))
}

/// The credits of the fonts: PDA10 Burmese Mono, Myanmar Sabae, Sauce Code
/// Pro (HTMM's Custom Font), and one row for the other fonts of the pickers.
///
/// # Returns
///
/// The view.
fn fonts() -> impl IntoView
{
    let pda = credit(
        "PDA10 Burmese Mono".into(),
        "explorer.about.pda_note",
        vec![
            link("explorer.about.pda_page", PDA_PAGE),
            link("explorer.about.pda_post", PDA_POST),
            link("explorer.about.ofl", OFL),
        ],
    );
    let sabae = credit(
        "Myanmar Sabae".into(),
        "explorer.about.sabae_note",
        vec![link("explorer.about.apache", APACHE)],
    );
    let source = credit(
        "Sauce Code Pro (HTMM's Custom Font)".into(),
        "explorer.about.source_note",
        vec![
            link("explorer.about.source_repo", SOURCE_REPO),
            link("explorer.about.ofl", OFL),
        ],
    );
    let others = credit(
        t!("explorer.about.others"),
        "explorer.about.others_note",
        vec![
            link("explorer.about.fontsource", FONTSOURCE),
            link("explorer.about.google_fonts", GOOGLE_FONTS),
        ],
    );
    (head("explorer.about.fonts"), pda, sabae, source, others)
}

/// The credit of one font: its name, a note and its links.
///
/// # Arguments
///
/// * `name` - The font name.
/// * `note` - Locale id of the note.
/// * `links` - The links.
///
/// # Returns
///
/// The view.
fn credit(
    name: Cow<'static, str>, note: &'static str, links: Vec<AnyView>,
) -> impl IntoView
{
    div().class(ClassName::CREDITS_FONT).child((
        span().class(ClassName::CREDITS_FONT_NAME).child(name),
        p().class(ClassName::CREDITS_NOTE).child(t!(note.to_string())),
        div().class(ClassName::CREDITS_LINKS).child(links),
    ))
}

/// A link that opens in a new tab.
///
/// # Arguments
///
/// * `label` - Locale id of the link text.
/// * `href` - The address.
///
/// # Returns
///
/// The view.
fn link(label: &'static str, href: &'static str) -> AnyView
{
    a().attr("href", href)
        .attr("target", "_blank")
        .attr("rel", "noopener noreferrer")
        .child(t!(label.to_string()))
        .into_any()
}
