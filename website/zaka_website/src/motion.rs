use std::rc::Rc;

use js_sys::Function;
use js_sys::Object;
use js_sys::Reflect;
use leptos::prelude::document;
use leptos::prelude::window;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Element;
use web_sys::MouseEvent;
use zakac_rust_sugars::ret_if;

use crate::components::bands::ClassName as Bands;
use crate::components::close::ClassName as Close;
use crate::components::header::ClassName as Header;
use crate::components::hero::ClassName as Hero;
use crate::components::name::ClassName as Name;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = gsap, js_name = registerPlugin)]
    fn register_plugin(plugin: &JsValue);
    #[wasm_bindgen(js_namespace = gsap, js_name = to)]
    fn tween_to(targets: &str, vars: &Object);
    #[wasm_bindgen(js_namespace = gsap, js_name = to)]
    fn tween_element(target: &Element, vars: &Object);
    #[wasm_bindgen(js_namespace = gsap, js_name = from)]
    fn tween_from(targets: &str, vars: &Object);
    #[wasm_bindgen(js_namespace = gsap, js_name = fromTo)]
    fn tween_between(targets: &str, start: &Object, end: &Object);
    #[wasm_bindgen(js_namespace = ["gsap", "ticker"], js_name = add)]
    fn ticker_add(tick: &Function);
    #[wasm_bindgen(js_namespace = ["gsap", "ticker"], js_name = lagSmoothing)]
    fn lag_smoothing(threshold: f64);
    #[wasm_bindgen(thread_local_v2, js_name = ScrollTrigger)]
    static SCROLL_TRIGGER: JsValue;
    #[wasm_bindgen(js_namespace = ScrollTrigger, js_name = update)]
    fn trigger_update();
    #[wasm_bindgen(js_namespace = ScrollTrigger, js_name = refresh)]
    fn trigger_refresh();

    /// Lenis, the smooth scroller.
    type Lenis;
    #[wasm_bindgen(constructor)]
    fn new(options: &Object) -> Lenis;
    #[wasm_bindgen(method)]
    fn raf(this: &Lenis, time: f64);
    #[wasm_bindgen(method)]
    fn on(this: &Lenis, event: &str, handler: &Function);
    #[wasm_bindgen(method, js_name = scrollTo)]
    fn scroll_to(this: &Lenis, target: &Element, options: &Object);
}

/// Globals the page loads for the motion code.
const LIBRARIES: [&str; 3] = ["gsap", "ScrollTrigger", "Lenis"];

/// Start the motion code, when its libraries are on the page and the
/// visitor has not asked for less motion.
pub fn start()
{
    ret_if!(!loaded() || prefers("(prefers-reduced-motion: reduce)"));
    SCROLL_TRIGGER.with(register_plugin);
    let _ = document().document_element().map(|root| root.class_list().add_1("motion"));
    let lenis = smooth();
    follow_links(lenis);
    hero();
    name();
    bands();
    close();
    remeasure();
}

/// Whether the libraries of the motion code are on the page.
///
/// # Returns
///
/// `true` when all three are.
fn loaded() -> bool
{
    let global = js_sys::global();
    LIBRARIES.iter().all(|name| Reflect::has(&global, &(*name).into()).unwrap_or(false))
}

/// Whether a media query holds.
///
/// # Arguments
///
/// * `query` - The query.
///
/// # Returns
///
/// `true` when it matches.
fn prefers(query: &str) -> bool
{
    let list = window().match_media(query).ok().flatten();
    list.is_some_and(|list| list.matches())
}

/// A JS object from pairs of names and values.
///
/// # Arguments
///
/// * `pairs` - The names and values.
///
/// # Returns
///
/// The object.
fn vars(pairs: &[(&str, JsValue)]) -> Object
{
    let object = Object::new();
    pairs.iter().for_each(|(name, value)| {
        let _ = Reflect::set(&object, &(*name).into(), value);
    });
    object
}

/// Smooth scrolling, stepped from the GSAP ticker so the scenes keep pace.
///
/// # Returns
///
/// The scroller.
fn smooth() -> Rc<Lenis>
{
    let lenis = Rc::new(Lenis::new(&vars(&[("lerp", 0.1.into())])));
    let update = Closure::<dyn Fn()>::new(trigger_update);
    lenis.on("scroll", update.as_ref().unchecked_ref());
    update.forget();
    let stepped = lenis.clone();
    let tick = Closure::<dyn Fn(f64)>::new(move |time: f64| stepped.raf(time * 1000.0));
    ticker_add(tick.as_ref().unchecked_ref());
    tick.forget();
    lag_smoothing(0.0);
    lenis
}

/// Links within the page glide to their section, under the header.
///
/// # Arguments
///
/// * `lenis` - The scroller.
fn follow_links(lenis: Rc<Lenis>)
{
    let glide = Closure::<dyn Fn(MouseEvent)>::new(move |event: MouseEvent| {
        let Some(target) = link_target(&event)
        else
        {
            return;
        };
        event.prevent_default();
        let header =
            document().query_selector(&format!(".{}", Header::HEADER)).ok().flatten();
        let offset = header.map_or(72, |h| h.client_height());
        lenis.scroll_to(&target, &vars(&[("offset", (-offset).into())]));
    });
    let _ = document()
        .add_event_listener_with_callback("click", glide.as_ref().unchecked_ref());
    glide.forget();
}

/// The section a click on a link within the page goes to.
///
/// # Arguments
///
/// * `event` - The click.
///
/// # Returns
///
/// The section, or `None` for other clicks.
fn link_target(event: &MouseEvent) -> Option<Element>
{
    let clicked = event.target()?.dyn_into::<Element>().ok()?;
    let link = clicked.closest("a[href^=\"#\"]").ok().flatten()?;
    let href = link.get_attribute("href")?;
    document().query_selector(&href).ok().flatten()
}

/// The settings of a trigger that scrubs a tween as the page scrolls.
///
/// # Arguments
///
/// * `trigger` - Selector of the element that drives it.
/// * `start` - Where it starts.
/// * `end` - Where it ends.
///
/// # Returns
///
/// The settings.
fn scrub(trigger: &str, start: &str, end: &str) -> JsValue
{
    vars(&[
        ("trigger", trigger.into()),
        ("start", start.into()),
        ("end", end.into()),
        ("scrub", true.into()),
    ])
    .into()
}

/// Parts of the page that rise into place once, one after the other, as
/// their trigger comes in.
///
/// # Arguments
///
/// * `targets` - Selector of the parts.
/// * `trigger` - Selector of the element that starts them.
/// * `extra` - Settings over the usual ones.
fn rise(targets: &str, trigger: &str, extra: &[(&str, JsValue)])
{
    let on_enter = vars(&[("trigger", trigger.into()), ("start", "top 82%".into())]);
    let base = vars(&[
        ("y", 48.into()),
        ("opacity", 0.into()),
        ("duration", 0.9.into()),
        ("ease", "power3.out".into()),
        ("stagger", 0.08.into()),
        ("scrollTrigger", on_enter.into()),
    ]);
    tween_from(targets, &Object::assign(&base, &vars(extra)));
}

/// The hero: as the page leaves it, the photograph sinks slower than the
/// page and the text rises a little ahead of it.
fn hero()
{
    let leave = || scrub(&format!(".{}", Hero::HERO), "top top", "bottom top");
    let drift = |percent: i32| {
        vars(&[
            ("yPercent", percent.into()),
            ("ease", "none".into()),
            ("scrollTrigger", leave()),
        ])
    };
    tween_to(&format!(".{} img", Hero::HERO_BACKDROP), &drift(18));
    tween_to(&format!(".{}", Hero::HERO_MAIN), &drift(-12));
}

/// The name: the plumeria drifts in its frame; the name comes apart term
/// after term.
fn name()
{
    let drift = vars(&[
        ("yPercent", (-14).into()),
        ("ease", "none".into()),
        (
            "scrollTrigger",
            scrub(&format!(".{}", Name::NAME_PHOTO), "top bottom", "bottom top"),
        ),
    ]);
    tween_to(&format!(".{} img", Name::NAME_FRAME), &drift);
    rise(&format!(".{}", Name::NAME_HEAD), &format!(".{}", Name::NAME), &[]);
    rise(
        &format!(".{} > *", Name::NAME_EQUATION),
        &format!(".{}", Name::NAME_EQUATION),
        &[("stagger", 0.12.into())],
    );
    rise(&format!(".{}", Name::NAME_TEXT), &format!(".{}", Name::NAME_TEXT), &[]);
}

/// The bands slide in from the left, one after the other.
fn bands()
{
    rise(&format!(".{}", Bands::BANDS_HEAD), &format!(".{}", Bands::BANDS), &[]);
    rise(
        &format!(".{}", Bands::BAND),
        &format!(".{}", Bands::BANDS_LIST),
        &[("y", 0.into()), ("xPercent", (-8).into()), ("stagger", 0.1.into())],
    );
}

/// The close: the headline climbs in, and the photograph drifts.
fn close()
{
    rise(
        &format!(".{}", Close::CLOSE_HEAD),
        &format!(".{}", Close::CLOSE),
        &[("y", 90.into()), ("duration", 1.1.into())],
    );
    let drift = vars(&[
        ("yPercent", 6.into()),
        ("ease", "none".into()),
        (
            "scrollTrigger",
            scrub(&format!(".{}", Close::CLOSE), "top bottom", "bottom top"),
        ),
    ]);
    tween_to(&format!(".{}", Close::CLOSE_SHOT), &drift);
    rise(&format!(".{}", Close::CLOSE_FOOT), &format!(".{}", Close::CLOSE_FOOT), &[]);
}

/// Fonts and the islands can move the page once loaded; measure the scenes
/// again then.
fn remeasure()
{
    let again = Closure::<dyn Fn()>::new(trigger_refresh);
    let _ = window()
        .add_event_listener_with_callback("load", again.as_ref().unchecked_ref());
    again.forget();
    let fonts_in = Closure::<dyn FnMut(JsValue)>::new(|_| trigger_refresh());
    let _ = document().fonts().ready().map(|loading| loading.then(&fonts_in));
    fonts_in.forget();
}
