use zaka_website::app::MOTION;

#[test]
fn motion_libraries_sit_in_the_version_folders_trunk_toml_pins()
{
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/vendor");
    let trunk: toml::Table =
        std::fs::read_to_string(format!("{dir}/Trunk.toml")).unwrap().parse().unwrap();
    let index = std::fs::read_to_string(format!("{dir}/index.html")).unwrap();
    for src in MOTION
    {
        let mut parts = src.trim_start_matches("/vendor/").splitn(3, '/');
        let (name, version) = (parts.next().unwrap(), parts.next().unwrap());
        let pin = trunk["node_packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"].as_str() == Some(name))
            .unwrap();
        assert_eq!(pin["version"].as_str(), Some(version), "{src}");
        assert!(
            index.contains(&format!("data-target-path=\"{name}/{version}\"")),
            "{src}"
        );
    }
}
