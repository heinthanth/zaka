//! The check for Ghostty: what a Ghostty build reports about how it draws
//! Burmese text.

pub mod settings;

use crate::terminal::Terminal;

/// Look at the terminal as Ghostty: `TERM_PROGRAM` names it, and its
/// settings tell how it draws Burmese text.
///
/// A build that lists [`settings::FLOW_SETTING`] (saturngod's fork) flows
/// Burmese text. Other builds put it on the cell grid: a grapheme cluster
/// takes its cells under `grapheme-width-method = unicode`, the default, and
/// each code point its own under `legacy`.
///
/// # Returns
///
/// The terminal, or `None` when it is not such a Ghostty build.
pub fn probe() -> Option<Terminal>
{
    #[cfg(unix)]
    {
        unix::probe()
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(unix)]
mod unix
{
    use std::path::PathBuf;
    use std::process::Command;
    use std::process::Stdio;

    use zakac_rust_sugars::ret_if;
    use zakac_rust_sugars::tern;

    use super::settings::BURMESE_FONTS;
    use super::settings::DEFAULT_CELL_EM;
    use super::settings::Settings;
    use super::settings::font_px;
    use super::settings::read_settings;
    use crate::flow::Flow;
    use crate::flow::Grid;
    use crate::fonts::Faces;
    use crate::fonts::cell_em;
    use crate::fonts::installed_fonts;
    use crate::terminal::Drawing;
    use crate::terminal::Terminal;

    /// The check; see [`super::probe`].
    ///
    /// # Returns
    ///
    /// The terminal, or `None`.
    pub(super) fn probe() -> Option<Terminal>
    {
        ret_if!(std::env::var("TERM_PROGRAM").ok()? != "ghostty", None);
        let settings = read_settings(&show_config()?);
        let grid = tern!(settings.clusters, Drawing::Cluster, Drawing::Codepoint);
        let drawing = tern!(settings.flows, Drawing::Flow(flow(&settings)?), grid);
        Some(Terminal::builder().name("Ghostty").drawing(drawing).build())
    }

    /// What a build that flows Burmese text needs for the placement: the
    /// cell grid of the window standard error goes to, and the Burmese font.
    ///
    /// # Arguments
    ///
    /// * `settings` - The settings of the build.
    ///
    /// # Returns
    ///
    /// The flow, or `None` when standard error is not a window or no Burmese
    /// font the build looks for is installed.
    fn flow(settings: &Settings) -> Option<Flow>
    {
        let size = rustix::termios::tcgetwinsize(std::io::stderr()).ok()?;
        ret_if!(size.ws_col == 0 || size.ws_xpixel < size.ws_col, None);
        let db = installed_fonts();
        let cell = f64::from(size.ws_xpixel / size.ws_col);
        let cell_em = settings.font.as_deref().and_then(|name| cell_em(&db, name));
        let px = font_px(cell, settings.points, cell_em.unwrap_or(DEFAULT_CELL_EM));
        let faces = Faces::load(&db, &BURMESE_FONTS, px)?;
        let grid = Grid::builder().cell(cell).cols(usize::from(size.ws_col)).build();
        Some(Flow::builder().grid(grid).shaper(Box::new(faces)).build())
    }

    /// Run `ghostty +show-config` with each setting listed.
    ///
    /// # Returns
    ///
    /// What it printed, or `None` when it can't run.
    fn show_config() -> Option<String>
    {
        let bin = std::env::var_os("GHOSTTY_BIN_DIR")
            .map(|folder| PathBuf::from(folder).join("ghostty"))
            .filter(|path| path.is_file())
            .unwrap_or_else(|| PathBuf::from("ghostty"));
        let out = Command::new(bin)
            .args(["+show-config", "--changes-only=false"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
