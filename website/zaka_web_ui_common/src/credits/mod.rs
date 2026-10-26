use bon::Builder;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;

use crate::sheet;

mod roll;

pub use roll::credits_roll;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/credits/mod.sass");

/// Rules of the credits roll.
static CREDITS: ComponentStyle =
    ComponentStyle::global_css("zaka-credits", STYLE_SHEET);

/// The open source projects the website, the explorer and the compiler build
/// on, as `tests/credits.rs` writes them: one per line, tabs between the
/// fields.
const LIST: &str = include_str!("projects.tsv");

/// Mount the sheet of the credits roll. A page rendered on a server mounts it
/// outside its islands, as a global sheet renders on the server alone.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn credits_sheet() -> impl IntoView { sheet(&CREDITS) }

/// Where a project comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source
{
    /// A Rust crate from crates.io.
    Crate,
    /// An npm package that Trunk fetches.
    Npm,
}

/// One project of the credits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct Project
{
    /// Where it comes from.
    pub source: Source,
    /// Its name.
    pub name: &'static str,
    /// The version in use.
    pub version: &'static str,
    /// Its license, as an SPDX expression.
    pub license: &'static str,
    /// Its home or source page.
    pub url: &'static str,
}

/// The projects, crates first, each group in name order.
///
/// # Returns
///
/// The projects.
pub fn projects() -> Vec<Project>
{
    let rows = LIST.lines().filter(|l| !l.is_empty() && !l.starts_with('#'));
    rows.filter_map(project).collect()
}

/// Read one line of the list.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// The project, or `None` for a line that is not well formed.
fn project(line: &'static str) -> Option<Project>
{
    let mut fields = line.split('\t');
    let source = match fields.next()?
    {
        "crate" => Source::Crate,
        "npm" => Source::Npm,
        _ => return None,
    };
    let project = Project::builder()
        .source(source)
        .name(fields.next()?)
        .version(fields.next()?)
        .license(fields.next()?)
        .url(fields.next()?)
        .build();
    Some(project)
}
