use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;
use zaka_web_ui_common::credits::Project;
use zaka_web_ui_common::credits::Source;
use zaka_web_ui_common::credits::projects;

/// The list of projects the credits roll reads, from the crate root.
const LIST: &str = "src/credits/projects.tsv";

/// The first line of the list.
const HEADER: &str = "# source\tname\tversion\tlicense\turl";

/// The credits page of the repo, from the crate root.
const CREDITS_MD: &str = "../../CREDITS.md";

/// The mark before the part of `CREDITS.md` that the list writes.
const MD_OPEN: &str = "<!-- begin: projects -->";

/// The mark after the part of `CREDITS.md` that the list writes.
const MD_CLOSE: &str = "<!-- end: projects -->";

/// The root of the shared crate.
fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// One line of the list.
fn line(source: &str, name: &str, version: &str, license: &str, url: &str) -> String
{
    format!("{source}\t{name}\t{version}\t{license}\t{url}")
}

/// The text of a JSON field, or `None` for a field that is missing or blank.
fn text<'a>(v: &'a Value, field: &str) -> Option<&'a str>
{
    v[field].as_str().filter(|s| !s.is_empty())
}

/// Whether a dep ships: a normal or a build dep, not a dev one.
fn ships(dep: &Value) -> bool
{
    let kinds = dep["dep_kinds"].as_array().into_iter().flatten();
    kinds.into_iter().any(|k| k["kind"].is_null() || k["kind"] == "build")
}

/// The line of a crate from crates.io.
fn crate_line(p: &Value) -> String
{
    let name = text(p, "name").unwrap();
    let fallback = format!("https://crates.io/crates/{name}");
    let url = text(p, "repository").or(text(p, "homepage")).unwrap_or(&fallback);
    let license = text(p, "license").unwrap_or("see the crate");
    line("crate", name, text(p, "version").unwrap(), license, url)
}

/// The crates the members of the workspace name in their own manifests, as
/// lines.
fn crate_lines() -> BTreeSet<String>
{
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let args = ["metadata", "--format-version", "1", "--offline"];
    let out = Command::new(cargo).args(args).current_dir(root()).output().unwrap();
    let hint = "run cargo fetch to download the crates of each platform";
    assert!(out.status.success(), "{hint}\n{}", String::from_utf8_lossy(&out.stderr));
    let meta: Value = serde_json::from_slice(&out.stdout).unwrap();
    let packages = meta["packages"].as_array().unwrap();
    let with_id: BTreeMap<&str, &Value> =
        packages.iter().map(|p| (text(p, "id").unwrap(), p)).collect();
    let members: BTreeSet<&str> = meta["workspace_members"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let nodes = meta["resolve"]["nodes"].as_array().unwrap();
    let ours = nodes.iter().filter(|n| members.contains(text(n, "id").unwrap()));
    let deps = ours.flat_map(|n| n["deps"].as_array().unwrap()).filter(|d| ships(d));
    let found = deps.filter_map(|d| with_id.get(text(d, "pkg").unwrap()));
    let published = found.filter(|p| !p["source"].is_null());
    published.map(|p| crate_line(p)).collect()
}

/// The Trunk projects whose npm packages the pages ship: the explorer, the
/// web fonts both share, and the motion libraries of the website.
fn trunk_dirs() -> [PathBuf; 3]
{
    [
        root().join("../zaka_explorer"),
        root().join("fonts"),
        root().join("../zaka_website/vendor"),
    ]
}

/// The npm packages Trunk fetches, as name and version pairs, each with the
/// folder of the first Trunk project that fetches it; a package two projects
/// fetch counts once.
fn trunk_packages() -> Vec<(PathBuf, String, String)>
{
    let mut seen = BTreeSet::new();
    let all = trunk_dirs().into_iter().flat_map(packages_of);
    all.filter(|(_, n, v)| seen.insert((n.clone(), v.clone()))).collect()
}

/// The npm packages one Trunk project fetches.
///
/// # Arguments
///
/// * `dir` - The folder of the project.
fn packages_of(dir: PathBuf) -> Vec<(PathBuf, String, String)>
{
    let toml = std::fs::read_to_string(dir.join("Trunk.toml")).unwrap();
    let value = |l: &str, field: &str| {
        let rest = l.strip_prefix(field)?.trim_start().strip_prefix('=')?;
        Some(rest.trim().trim_matches('"').to_string())
    };
    let names = toml.lines().filter_map(|l| value(l.trim(), "name"));
    let versions = toml.lines().filter_map(|l| value(l.trim(), "version"));
    names.zip(versions).map(|(n, v)| (dir.clone(), n, v)).collect()
}

/// The line of an npm package: from its `package.json` in the cache of
/// Trunk, else the line the list holds for that version.
fn npm_line(
    dir: &Path, name: &str, version: &str, known: &BTreeMap<String, String>,
) -> String
{
    let file = dir.join(format!("target/node_modules/{name}/{version}/package.json"));
    let Ok(raw) = std::fs::read_to_string(&file)
    else
    {
        let held = known.get(&format!("{name}\t{version}"));
        let hint = "run a trunk build so the npm packages are in target/node_modules";
        return held.cloned().unwrap_or_else(|| panic!("{name} {version}: {hint}"));
    };
    let p: Value = serde_json::from_str(&raw).unwrap();
    let fallback = format!("https://www.npmjs.com/package/{name}");
    let url = text(&p, "homepage").unwrap_or(&fallback);
    let license = text(&p, "license").unwrap_or("see the package");
    line("npm", name, version, license, url)
}

/// The npm lines the list holds, under their name and version.
///
/// # Arguments
///
/// * `list` - The text of the list.
fn known_npm(list: &str) -> BTreeMap<String, String>
{
    let npm = list.lines().filter(|l| l.starts_with("npm\t"));
    let pair = |l: &str| l.split('\t').skip(1).take(2).collect::<Vec<_>>().join("\t");
    npm.map(|l| (pair(l), l.to_string())).collect()
}

/// One group of the list as a Markdown table under its head.
///
/// # Arguments
///
/// * `head` - The head of the group.
/// * `source` - Where the projects of the group come from.
///
/// # Returns
///
/// The Markdown.
fn md_group(head: &str, source: Source) -> String
{
    let picked = projects().into_iter().filter(|p| p.source == source);
    let row = |p: Project| {
        format!("| [{}]({}) | {} | {} |\n", p.name, p.url, p.version, p.license)
    };
    let top =
        format!("## {head}\n\n| Name | Version | License |\n| --- | --- | --- |\n");
    std::iter::once(top).chain(picked.map(row)).collect()
}

#[test]
fn the_credits_list_the_direct_deps()
{
    let path = root().join(LIST);
    let held = std::fs::read_to_string(&path).unwrap_or_default();
    let known = known_npm(&held);
    let npm =
        trunk_packages().into_iter().map(|(d, n, v)| npm_line(&d, &n, &v, &known));
    let lines: Vec<String> =
        std::iter::once(HEADER.to_string()).chain(crate_lines()).chain(npm).collect();
    let text = lines.join("\n") + "\n";
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    assert_eq!(held, text, "the credits are stale; run with ZAKA_BLESS=1");
}

#[test]
fn the_credits_page_holds_the_list()
{
    let path = root().join(CREDITS_MD);
    let held = std::fs::read_to_string(&path).unwrap();
    let (before, rest) =
        held.split_once(MD_OPEN).expect("CREDITS.md lacks its open mark");
    let (_, after) =
        rest.split_once(MD_CLOSE).expect("CREDITS.md lacks its close mark");
    let groups =
        [md_group("Web packages", Source::Npm), md_group("Rust crates", Source::Crate)];
    let text = format!("{before}{MD_OPEN}\n\n{}\n{MD_CLOSE}{after}", groups.join("\n"));
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    assert_eq!(held, text, "CREDITS.md is stale; run with ZAKA_BLESS=1");
}

#[test]
fn each_line_of_the_list_reads_back()
{
    let all = projects();
    let held = std::fs::read_to_string(root().join(LIST)).unwrap();
    assert_eq!(all.len(), held.lines().filter(|l| !l.starts_with('#')).count());
    assert!(all.iter().any(|p| p.source == Source::Crate && p.name == "leptos"));
    assert!(all.iter().any(|p| p.source == Source::Npm && p.name == "monaco-editor"));
    assert!(all.iter().all(|p| p.url.starts_with("http")));
}
