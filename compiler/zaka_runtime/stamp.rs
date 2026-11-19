// The prefix of each name the runtime exports, which the build script of the
// runtime and the build script of the C target both include, so the two agree:
// `zk_rt_<version>_<hash>_`, where the version is the one of the runtime
// crate and the hash covers its sources and those of `zaka_host`. Two builds
// of the runtime from different sources export different names, so the libs
// each one serves link into one program together.

/// The prefix of the names the runtime exports.
///
/// # Arguments
///
/// * `compiler` - The `compiler` folder of the repo.
///
/// # Returns
///
/// The prefix, such as `zk_rt_0_1_0_8f3a91c2_`.
fn runtime_prefix(compiler: &std::path::Path) -> String
{
    let runtime = compiler.join("zaka_runtime");
    let manifest = std::fs::read_to_string(runtime.join("Cargo.toml"))
        .expect("the manifest of the runtime is readable");
    let version = manifest
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))
        .and_then(|v| v.strip_suffix('"'))
        .expect("the runtime names its version");
    let mut files = Vec::new();
    let roots = [runtime.join("src"), compiler.join("zaka_host/src")];
    roots.iter().for_each(|root| source_files(root, &mut files));
    files.sort();
    // Each file counts with its path, written with `/` on each host.
    let each = files.iter().flat_map(|file| {
        let name = file.strip_prefix(compiler).unwrap_or(file);
        let name = name.to_string_lossy().replace('\\', "/").into_bytes();
        [name, std::fs::read(file).expect("a source file is readable")].concat()
    });
    let all: Vec<u8> = manifest.bytes().chain(each).collect();
    let hash = xxhash_rust::xxh3::xxh3_64(&all) as u32;
    format!("zk_rt_{}_{hash:08x}_", version.replace('.', "_"))
}

/// Each `.rs` file under a folder, at each depth.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `out` - Where the paths go.
fn source_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>)
{
    let Ok(entries) = std::fs::read_dir(dir)
    else
    {
        return;
    };
    for path in entries.flatten().map(|e| e.path())
    {
        match path.is_dir()
        {
            true => source_files(&path, out),
            false if path.extension().is_some_and(|e| e == "rs") => out.push(path),
            false => (),
        }
    }
}
