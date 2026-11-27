// The names the runtime exports, which the build script of the runtime and
// the build script of the C target both include, so the two agree. Each name
// starts `zk_rt_abi<N>_`, where `N` is `ZK_ABI`: libs from two builds of
// `zakac` with the same ABI share one runtime in a program, and libs of two
// ABIs each keep their own. The private names of Rust take a hash of the
// sources through `-C metadata`, so two runtimes never clash on those.

/// The version of the interface between generated code and the runtime: the
/// signature and the meaning of each function generated code calls, the
/// forms of the shared structs, the ids of traps, rights and tags, and the
/// inline code of the C headers. Raise it on each change that breaks a lib
/// built before; an added function keeps it.
const ZK_ABI: u32 = 1;

/// The feature level of the runtime within its ABI: raise it when a function
/// gains a meaning that a lib built from now on relies on. A lib checks it on
/// its first call.
const ZK_LEVEL: u32 = 1;

/// The prefix of the names the runtime exports.
///
/// # Returns
///
/// The prefix, such as `zk_rt_abi1_`; a salt, which a test of two runtimes
/// in one program sets, gives one of its own, such as `zk_rt_abi1s8f3a91c2_`.
fn runtime_prefix() -> String
{
    let salt = std::env::var("ZAKA_RUNTIME_SALT").unwrap_or_default();
    let hash = xxhash_rust::xxh3::xxh3_64(salt.as_bytes()) as u32;
    let mark = tern!(salt.is_empty(), String::new(), format!("s{hash:08x}"));
    format!("zk_rt_abi{ZK_ABI}{mark}_")
}

/// The version of `zakac`, which the runtime gives as text for diagnostics.
///
/// # Arguments
///
/// * `compiler` - The `compiler` folder of the repo.
///
/// # Returns
///
/// The version, such as `0.1.0`.
#[allow(dead_code)]
fn zakac_version(compiler: &std::path::Path) -> String
{
    let manifest = std::fs::read_to_string(compiler.join("zakac/Cargo.toml"))
        .expect("the manifest of zakac is readable");
    let version = manifest.lines().find_map(|l| l.strip_prefix("version = \""));
    let version = version.and_then(|v| v.strip_suffix('"'));
    version.expect("zakac names its version").to_string()
}

/// The metadata of each crate of the runtime lib: the version of the runtime
/// and a hash of its sources, those of `zaka_host` and the salt. Rust derives
/// the private names of each crate from it, so two runtimes of other sources
/// in one program keep theirs apart.
///
/// # Arguments
///
/// * `compiler` - The `compiler` folder of the repo.
///
/// # Returns
///
/// The metadata, such as `zk_rt_0_1_0_8f3a91c2`.
#[allow(dead_code)]
fn runtime_metadata(compiler: &std::path::Path) -> String
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
    let salt = std::env::var("ZAKA_RUNTIME_SALT").unwrap_or_default().into_bytes();
    let all: Vec<u8> = manifest.bytes().chain(each).chain(salt).collect();
    let hash = xxhash_rust::xxh3::xxh3_64(&all) as u32;
    format!("zk_rt_{}_{hash:08x}", version.replace('.', "_"))
}

/// Each `.rs` file under a folder, at each depth.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `out` - Where the paths go.
#[allow(dead_code)]
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
