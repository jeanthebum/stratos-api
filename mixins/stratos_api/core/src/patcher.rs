//! Guarded source overlay, executed before horizon-shader-api and its dependents compile.
//! It writes only the mixin tool's disposable stage, never the player's source checkout.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs, path::{Component, Path, PathBuf}};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Index { schema: u32, files: Vec<Entry> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry { path: String, before_sha256: Option<String>, after_sha256: String }
pub struct Applied { pub written: usize, pub watched: Vec<PathBuf> }

pub fn stage_root(dir: &Path) -> Option<PathBuf> {
    let owner = dir.parent()?;
    let container = owner.parent()?;
    (dir.file_name()? == "core" && owner.file_name()? == "stratos_api"
        && container.file_name()? == "native-crates")
        .then(|| container.parent().unwrap().to_path_buf())
}
fn normalized(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    String::from_utf8(bytes).map(|s| s.replace("\r\n", "\n").into_bytes()).map_err(|e| e.to_string())
}
fn digest(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn safe_source(path: &str) -> bool {
    !path.contains('\\')
        && Path::new(path).components().all(|c| matches!(c, Component::Normal(_)))
        && ((path.ends_with(".rs") && ["crates/horizon-shader-api/", "crates/horizon-client-render/", "crates/horizon-client/"]
            .iter().any(|prefix| path.starts_with(prefix)))
            || ["docs/api/shaders.md", "docs/api/shaders-reference.md", "docs/api/shaders-reference-0.2.md"].contains(&path))
}
fn plain_ancestors(root: &Path, path: &Path) -> Result<(), String> {
    let mut current = root.to_path_buf();
    for component in path.strip_prefix(root).map_err(|_| "path leaves stage")?.components() {
        current.push(component);
        if let Ok(meta) = fs::symlink_metadata(&current) {
            if meta.file_type().is_symlink() { return Err(format!("symlink in {}", current.display())); }
            if !current.canonicalize().map_err(|e| e.to_string())?.starts_with(root) {
                return Err("resolved source path leaves stage".into());
            }
        }
    }
    Ok(())
}
pub fn apply(root: &Path, package: &Path) -> Result<Applied, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    if !root.join(".horizon-mixins-stage.json").is_file() || !root.join("native-post-build.json").is_file() {
        return Err("refusing to modify a workspace without native-mixin stage markers".into());
    }
    let index: Index = serde_json::from_slice(&fs::read(package.join("data/engine-patches.json")).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if index.schema != 1 || index.files.is_empty() { return Err("invalid renderer overlay index".into()); }
    let mut pending = Vec::new();
    let mut watched = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    // Validate every file before writing any of them. Another mod's changes are
    // reported as conflicts, never silently replaced or partially merged.
    for entry in index.files {
        if !safe_source(&entry.path) || !seen.insert(entry.path.clone()) { return Err(format!("unsafe or duplicate source path: {}", entry.path)); }
        let target = root.join(&entry.path);
        plain_ancestors(&root, &target)?;
        let source = package.join("data/engine").join(&entry.path);
        let new = normalized(fs::read(&source).map_err(|e| format!("{}: {e}", source.display()))?)?;
        if digest(&new) != entry.after_sha256 { return Err(format!("overlay data checksum mismatch: {}", entry.path)); }
        let existing = match fs::read(&target) {
            Ok(v) => Some(digest(&normalized(v)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.to_string()),
        };
        watched.push(target.clone());
        if existing.as_ref() == Some(&entry.after_sha256) { continue; }
        if existing != entry.before_sha256 {
            return Err(format!("source conflict in {}: expected the pinned Horizon source or StratosAPI's own output. Use the pinned version and remove mods that edit this file", entry.path));
        }
        pending.push((target, new));
    }
    for (target, bytes) in &pending {
        fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
        let staged = target.with_extension("rs.stratos-new");
        fs::write(&staged, bytes).map_err(|e| e.to_string())?;
        fs::rename(&staged, target).map_err(|e| e.to_string())?;
    }
    Ok(Applied { written: pending.len(), watched })
}
