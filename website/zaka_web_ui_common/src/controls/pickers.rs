use std::iter::once;

use leptos::html::span;
use leptos::prelude::*;
use leptos::task::spawn_local;
use strum::IntoEnumIterator;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;
use zakac_i18n::SUPPORTED_LOCALES;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::controls::ClassName;
use crate::controls::dropdown::Choice;
use crate::controls::dropdown::Dropdown;
use crate::controls::dropdown::dropdown;
use crate::controls::settings::picker;
use crate::icon::globe;
use crate::icon::swatches;
use crate::lang::AUTO_LANG;
use crate::lang::lang_name;
use crate::lang::store_lang;
use crate::lang::stored_lang;
use crate::site::HOME_URL;
use crate::site::Release;
use crate::site::SITE_BASE;
use crate::site::VERSIONS_FILE;
use crate::site::moved_path;
use crate::site::read_releases;
use crate::theme::Theme;
use crate::theme::ThemeMode;

/// Picker of the color theme, grouped into plain and Catppuccin themes.
///
/// # Arguments
///
/// * `theme` - The theme of the page.
/// * `autofocus` - Whether it takes the focus when shown, as in a dialog.
///
/// # Returns
///
/// The view.
pub fn theme_picker(theme: ThemeMode, autofocus: bool) -> impl IntoView
{
    let mut spec = theme_spec(theme);
    spec.autofocus = autofocus;
    dropdown(spec)
}

/// The theme picker drawn as a book of swatches, for a bar.
///
/// # Arguments
///
/// * `theme` - The theme of the page.
///
/// # Returns
///
/// The view.
pub fn theme_icon_picker(theme: ThemeMode) -> impl IntoView
{
    let mut spec = theme_spec(theme);
    spec.icon = Some(|| swatches().into_any());
    dropdown(spec)
}

/// What the theme picker offers and where its pick goes.
///
/// # Arguments
///
/// * `theme` - The theme of the page.
///
/// # Returns
///
/// The dropdown.
fn theme_spec(theme: ThemeMode) -> Dropdown
{
    let choice = |t: Theme| {
        let value: &'static str = t.into();
        Choice::builder().value(value).label(t.label()).group(t.group()).build()
    };
    let current = Signal::derive(move || {
        <&'static str>::from(Theme::from_mode(&theme.mode.get())).to_string()
    });
    let on_pick = move |v: String| {
        let picked = v.parse::<Theme>().unwrap_or_default();
        theme.set_mode.set(picked.to_mode());
    };
    picker(
        t!("web_ui.settings.theme"),
        Theme::iter().map(choice).collect(),
        current,
        on_pick,
    )
}

/// Picker of the language of the pages and the compiler messages. A pick
/// reloads the page in the new language.
///
/// # Arguments
///
/// * `before_reload` - Called right before the reload, to keep the state of the
///   page.
///
/// # Returns
///
/// The view.
pub fn lang_picker(before_reload: impl Fn() + Send + Sync + 'static) -> impl IntoView
{
    dropdown(lang_spec(before_reload))
}

/// The language picker drawn as a globe alone, for a bar.
///
/// # Arguments
///
/// * `before_reload` - Called right before the reload, to keep the state of the
///   page.
///
/// # Returns
///
/// The view.
pub fn lang_icon_picker(
    before_reload: impl Fn() + Send + Sync + 'static,
) -> impl IntoView
{
    let mut spec = lang_spec(before_reload);
    spec.icon = Some(|| globe().into_any());
    dropdown(spec)
}

/// What the language picker offers and where its pick goes.
///
/// # Arguments
///
/// * `before_reload` - Called right before the reload.
///
/// # Returns
///
/// The dropdown.
fn lang_spec(before_reload: impl Fn() + Send + Sync + 'static) -> Dropdown
{
    let auto = Choice::builder().value(AUTO_LANG).label(t!("web_ui.lang.auto")).build();
    let named = SUPPORTED_LOCALES
        .iter()
        .map(|tag| Choice::builder().value(*tag).label(lang_name(tag)).build());
    let on_pick = move |v: String| {
        ret_if!(v == stored_lang());
        store_lang(&v);
        before_reload();
        let _ = window().location().reload();
    };
    let choices = once(auto).chain(named).collect();
    // Storage lives in the browser alone, so the stored choice is read there,
    // after the page is drawn; the server draws the Auto choice.
    let current = RwSignal::new(AUTO_LANG.to_string());
    Effect::new(move |_| current.set(stored_lang()));
    picker(t!("web_ui.lang.name"), choices, current.into(), on_pick)
}

/// The releases of [`VERSIONS_FILE`], read once the page runs; none until
/// then, and none on a site with no such file. The pickers of a page share
/// them, so the file is read once.
///
/// # Returns
///
/// The releases, the newest first.
pub fn site_releases() -> RwSignal<Vec<Release>>
{
    let releases = RwSignal::new(Vec::<Release>::new());
    Effect::new(move |_| {
        spawn_local(async move { releases.set(fetch_releases().await) });
    });
    releases
}

/// Picker of the release whose pages show: the latest at the root, and the
/// snapshot of each old release under its base. A pick opens the same page in
/// the other release. Until the releases arrive, the picker names the release
/// of this page alone; the page hides it while [`lists_releases`] is false.
///
/// # Arguments
///
/// * `releases` - The releases, from [`site_releases`].
///
/// # Returns
///
/// The view.
pub fn version_picker(releases: RwSignal<Vec<Release>>) -> impl IntoView
{
    span()
        .class(ClassName::VERSION_PICKER)
        .class((ClassName::VERSION_PICKER_OLD, !SITE_BASE.is_empty()))
        .child(move || dropdown(version_spec(&releases.get())))
}

/// What the version picker offers and where its pick goes.
///
/// # Arguments
///
/// * `releases` - The releases of [`VERSIONS_FILE`], the newest first.
///
/// # Returns
///
/// The dropdown.
fn version_spec(releases: &[Release]) -> Dropdown
{
    let mut choices: Vec<Choice> = releases
        .iter()
        .enumerate()
        .map(|(at, r)| {
            let label = release_label(at, r);
            Choice::builder().value(r.path.clone()).label(label).build()
        })
        .collect();
    let own = Choice::builder().value(HOME_URL).label(own_label()).build();
    do_if!(!choices.iter().any(|c| c.value == HOME_URL), choices.push(own));
    let on_pick = |to: String| {
        let location = window().location();
        let here = location.pathname().unwrap_or_default();
        ret_if!(to == HOME_URL);
        let tail = format!(
            "{}{}",
            location.search().unwrap_or_default(),
            location.hash().unwrap_or_default()
        );
        let _ =
            location.set_href(&format!("{}{tail}", moved_path(&here, HOME_URL, &to)));
    };
    picker(
        t!("web_ui.version.name"),
        choices,
        Signal::stored(HOME_URL.to_string()),
        on_pick,
    )
}

/// The text of a release in the version picker: the latest is marked as the
/// latest, and the release of this page, when old, as old.
///
/// # Arguments
///
/// * `at` - Its place in the list, the newest first.
/// * `release` - The release.
///
/// # Returns
///
/// The text.
fn release_label(at: usize, release: &Release) -> String
{
    let version = format!("v{}", release.version);
    ret_if!(at == 0, t!("web_ui.version.latest_of", version = version).to_string());
    ret_if!(release.path == HOME_URL, own_label());
    version
}

/// The text of the release of this page, from its base alone: the latest
/// at the root, else the old release its base names.
///
/// # Returns
///
/// The text.
fn own_label() -> String
{
    ret_if!(SITE_BASE.is_empty(), t!("web_ui.version.latest").to_string());
    let version = SITE_BASE.trim_start_matches('/');
    t!("web_ui.version.old", version = version).to_string()
}

/// The releases [`VERSIONS_FILE`] lists, read from the site.
///
/// # Returns
///
/// The releases, the newest first; none when the file is missing or does
/// not parse.
async fn fetch_releases() -> Vec<Release>
{
    let text = async {
        let answer =
            JsFuture::from(window().fetch_with_str(VERSIONS_FILE)).await.ok()?;
        let answer: Response = answer.dyn_into().ok()?;
        ret_if!(!answer.ok(), None);
        JsFuture::from(answer.text().ok()?).await.ok()?.as_string()
    };
    text.await.map(|t| read_releases(&t)).unwrap_or_default()
}
