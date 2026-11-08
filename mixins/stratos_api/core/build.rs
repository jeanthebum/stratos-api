#[path = "src/patcher.rs"]
mod patcher;

fn main() {
    println!("cargo:rerun-if-changed=data/engine-patches.json");
    println!("cargo:rerun-if-changed=data/engine");
    let dir = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    // A direct SDK/library build never modifies a surrounding checkout. Only the
    // launcher's separate mixin stage has this reserved native-crates location.
    let Some(root) = patcher::stage_root(&dir) else { return };
    let changed = patcher::apply(&root, &dir).unwrap_or_else(|e| panic!("StratosAPI: {e}"));
    for path in changed.watched {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:warning=StratosAPI: verified pinned renderer source ({} files updated)", changed.written);
}
