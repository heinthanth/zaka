use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use headless_chrome::Browser;
use headless_chrome::LaunchOptions;
use headless_chrome::browser::default_executable;
use headless_chrome::protocol::cdp::Emulation::SetDeviceMetricsOverride;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Page::Viewport;
use url::Url;

/// Width of the cover card, in CSS pixels.
const WIDTH: u32 = 1200;

/// Height of the cover card, in CSS pixels.
const HEIGHT: u32 = 630;

/// Chrome flags: one pixel per CSS pixel, no scrollbars, and fonts and
/// pictures that load from `file://` beside the page.
const FLAGS: [&str; 3] = [
    "--force-device-scale-factor=1",
    "--hide-scrollbars",
    "--allow-file-access-from-files",
];

/// Resolves once the fonts the page uses have loaded.
const FONTS_LOADED: &str = "document.fonts.ready.then(() => true)";

/// Draw a page of 1200 x 630 to a PNG with headless Chrome.
///
/// Usage: `zaka_html2png <page.html> <out.png>`.
///
/// Chrome is the one in `CHROME` when set, else the one installed on this
/// machine; when none is found, a Chromium build is fetched and kept for
/// later runs.
fn main() -> Result<(), Box<dyn Error>>
{
    let mut args = std::env::args_os().skip(1);
    let (Some(page), Some(out)) = (args.next(), args.next())
    else
    {
        return Err("usage: zaka_html2png <page.html> <out.png>".into());
    };
    let page = Url::from_file_path(std::path::absolute(&page)?)
        .map_err(|()| "the page path cannot be a file URL")?;
    let png = draw(page.as_str())?;
    std::fs::write(Path::new(&out), png)?;
    Ok(())
}

/// Open the page in headless Chrome and draw it, once its fonts and pictures
/// have loaded.
///
/// # Arguments
///
/// * `page` - The `file://` URL of the page.
///
/// # Returns
///
/// The PNG.
fn draw(page: &str) -> Result<Vec<u8>, Box<dyn Error>>
{
    let options = LaunchOptions::default_builder()
        .path(chrome())
        .window_size(Some((WIDTH, HEIGHT)))
        .args(FLAGS.iter().map(OsStr::new).collect())
        .build()?;
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;
    tab.call_method(card_size())?;
    tab.navigate_to(page)?.wait_until_navigated()?;
    tab.evaluate(FONTS_LOADED, true)?;
    let clip = Viewport {
        x: 0.0,
        y: 0.0,
        width: WIDTH.into(),
        height: HEIGHT.into(),
        scale: 1.0,
    };
    let png = tab.capture_screenshot(
        CaptureScreenshotFormatOption::Png,
        None,
        Some(clip),
        true,
    )?;
    Ok(png)
}

/// Set the page area to the size of the card. The window size alone leaves
/// the page area shorter on some platforms, and Chrome paints nothing past it.
///
/// # Returns
///
/// The call.
fn card_size() -> SetDeviceMetricsOverride
{
    SetDeviceMetricsOverride {
        width: WIDTH,
        height: HEIGHT,
        device_scale_factor: 1.0,
        mobile: false,
        scale: None,
        screen_width: None,
        screen_height: None,
        position_x: None,
        position_y: None,
        dont_set_visible_size: None,
        screen_orientation: None,
        viewport: None,
        display_feature: None,
        device_posture: None,
    }
}

/// The Chrome to run: the one in `CHROME`, else one installed on this
/// machine. `None` lets headless_chrome fetch a Chromium build.
///
/// # Returns
///
/// The path to Chrome, if found.
fn chrome() -> Option<PathBuf>
{
    default_executable()
        .inspect_err(|_| eprintln!("no Chrome found; fetching a Chromium build"))
        .ok()
}
